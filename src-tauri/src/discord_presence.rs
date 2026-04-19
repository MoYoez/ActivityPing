use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use crate::{
    artwork_server::{prepare_artwork_publisher, ArtworkPublisher, PublishAssetKind},
    backend_locale::BackendLocale,
    models::{
        ClientConfig, DiscordActivityType, DiscordAppNameMode, DiscordDebugPayload,
        DiscordPresenceSnapshot, DiscordReportMode, DiscordStatusDisplay,
    },
    platform::{
        display_name_for_app_id, get_foreground_app_icon, get_foreground_snapshot_for_reporting,
        get_now_playing, ForegroundSnapshot, MediaArtwork, MediaInfo,
    },
    rules::{
        normalize_client_config, resolve_activity,
        should_capture_foreground_app_icon_for_reporting,
        should_capture_foreground_snapshot_for_reporting, should_capture_media_for_reporting,
        should_capture_process_name_for_reporting, should_capture_window_title_for_reporting,
        ResolvedActivity, ResolvedDiscordAddons,
    },
};

const DEFAULT_SYNC_INTERVAL: Duration = Duration::from_secs(5);
const MAX_ERROR_BACKOFF_MS: u64 = 30_000;
const TIMESTAMP_UPDATE_THRESHOLD_MS: i64 = 100;
const WORKER_JOIN_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const WORKER_JOIN_POLL_STEP: Duration = Duration::from_millis(50);

#[derive(Default)]
struct DiscordPresenceInner {
    running: bool,
    active_run_id: Option<u64>,
    connected: bool,
    last_sync_at: Option<String>,
    last_error: Option<String>,
    current_summary: Option<String>,
    debug_payload: Option<DiscordDebugPayload>,
    stop_flag: Option<Arc<AtomicBool>>,
    worker: Option<JoinHandle<()>>,
}

pub struct DiscordPresenceRuntime {
    inner: Arc<Mutex<DiscordPresenceInner>>,
    sequence: AtomicU64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresencePayload {
    activity_name: Option<String>,
    details: String,
    state: Option<String>,
    status_display_type: Option<DiscordPresenceStatusDisplayType>,
    started_at_millis: Option<i64>,
    ended_at_millis: Option<i64>,
    media_duration_ms: Option<u64>,
    media_position_ms: Option<u64>,
    media_is_playing: bool,
    summary: String,
    signature: String,
    artwork: Option<DiscordPresenceArtwork>,
    icon: Option<DiscordPresenceIcon>,
    buttons: Vec<DiscordPresenceButton>,
    party: Option<DiscordPresenceParty>,
    secrets: Option<DiscordPresenceSecrets>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresenceArtwork {
    bytes: Vec<u8>,
    content_type: String,
    hover_text: String,
    cache_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DiscordPresenceStatusDisplayType {
    Name,
    State,
    Details,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresenceIcon {
    bytes: Vec<u8>,
    content_type: String,
    hover_text: String,
    cache_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresenceButton {
    label: String,
    url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresenceParty {
    id: Option<String>,
    size: Option<[i32; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresenceSecrets {
    join: Option<String>,
    spectate: Option<String>,
    match_secret: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresenceText {
    activity_name: Option<String>,
    details: String,
    state: Option<String>,
    summary: String,
    signature: String,
}

impl DiscordPresencePayload {
    fn publish_signature(&self) -> String {
        let artwork_key = self
            .artwork
            .as_ref()
            .map(|artwork| artwork.cache_key.as_str())
            .unwrap_or("");
        let icon_key = self
            .icon
            .as_ref()
            .map(|icon| icon.cache_key.as_str())
            .unwrap_or("");
        let stable_position_ms = if self.media_is_playing {
            None
        } else {
            self.media_position_ms
        };
        let button_key = self
            .buttons
            .iter()
            .map(|button| format!("{}={}", button.label, button.url))
            .collect::<Vec<_>>()
            .join("|");
        let party_key = self.party.as_ref().map_or_else(String::new, |party| {
            format!("{}:{:?}", party.id.as_deref().unwrap_or(""), party.size)
        });
        let secrets_key = self.secrets.as_ref().map_or_else(String::new, |secrets| {
            format!(
                "{}|{}|{}",
                secrets.join.as_deref().unwrap_or(""),
                secrets.spectate.as_deref().unwrap_or(""),
                secrets.match_secret.as_deref().unwrap_or("")
            )
        });
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{:?}\n{:?}\n{}",
            self.signature,
            self.activity_name.as_deref().unwrap_or(""),
            self.details,
            self.state.as_deref().unwrap_or(""),
            match self.status_display_type {
                Some(DiscordPresenceStatusDisplayType::Name) => "name",
                Some(DiscordPresenceStatusDisplayType::State) => "state",
                Some(DiscordPresenceStatusDisplayType::Details) => "details",
                None => "",
            },
            artwork_key,
            icon_key,
            button_key,
            party_key,
            secrets_key,
            self.media_duration_ms,
            stable_position_ms,
            self.media_is_playing
        )
    }
}

impl DiscordPresenceRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DiscordPresenceInner::default())),
            sequence: AtomicU64::new(1),
        }
    }

    pub fn snapshot(&self) -> DiscordPresenceSnapshot {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        snapshot_from_inner(&inner)
    }

    pub fn start(
        &self,
        mut config: ClientConfig,
        locale: BackendLocale,
    ) -> Result<DiscordPresenceSnapshot, String> {
        normalize_client_config(&mut config);
        validate_discord_presence_config(&config, locale)?;

        let stop_flag = Arc::new(AtomicBool::new(false));
        let run_id = self.sequence.fetch_add(1, Ordering::SeqCst);
        let previous_worker = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if inner.running {
                return Ok(snapshot_from_inner(&inner));
            }

            if let Some(old_flag) = inner.stop_flag.take() {
                old_flag.store(true, Ordering::SeqCst);
            }
            inner.worker.take()
        };

        if let Some(handle) = previous_worker {
            if let Err(handle) = wait_for_worker_exit(handle) {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = Some(handle);
                return Err(discord_worker_stopping(locale));
            }
        }

        let artwork_publisher = prepare_artwork_publisher(&config)?;

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.running = true;
            inner.active_run_id = Some(run_id);
            inner.connected = false;
            inner.last_sync_at = None;
            inner.last_error = None;
            inner.current_summary = None;
            inner.debug_payload = None;
            inner.stop_flag = Some(stop_flag.clone());
        }

        let state = self.inner.clone();
        let worker = thread::spawn(move || {
            run_discord_presence_loop(state, config, stop_flag, run_id, artwork_publisher, locale);
        });

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.worker = Some(worker);
        }

        Ok(self.snapshot())
    }

    pub fn stop(&self) -> DiscordPresenceSnapshot {
        let worker = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(flag) = inner.stop_flag.take() {
                flag.store(true, Ordering::SeqCst);
            }
            inner.running = false;
            inner.active_run_id = None;
            inner.connected = false;
            inner.current_summary = None;
            inner.debug_payload = None;
            inner.worker.take()
        };

        if let Some(handle) = worker {
            if let Err(handle) = wait_for_worker_exit(handle) {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = Some(handle);
            } else {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = None;
            }
        }

        self.snapshot()
    }
}

impl Drop for DiscordPresenceRuntime {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn snapshot_from_inner(inner: &DiscordPresenceInner) -> DiscordPresenceSnapshot {
    DiscordPresenceSnapshot {
        running: inner.running,
        connected: inner.connected,
        last_sync_at: inner.last_sync_at.clone(),
        last_error: inner.last_error.clone(),
        current_summary: inner.current_summary.clone(),
        debug_payload: inner.debug_payload.clone(),
    }
}

fn run_discord_presence_loop(
    state: Arc<Mutex<DiscordPresenceInner>>,
    config: ClientConfig,
    stop_flag: Arc<AtomicBool>,
    run_id: u64,
    artwork_publisher: Option<ArtworkPublisher>,
    locale: BackendLocale,
) {
    let mut discord_client: Option<DiscordIpcClient> = None;
    let sync_interval = Duration::from_millis(
        config
            .poll_interval_ms
            .max(DEFAULT_SYNC_INTERVAL.as_millis() as u64),
    );
    let mut consecutive_errors = 0u32;
    let mut last_signature = String::new();
    let mut last_publish_signature = String::new();
    let mut last_sent_end_timestamp: Option<i64> = None;
    let mut activity_started_at = Some(Utc::now().timestamp_millis());

    while !stop_flag.load(Ordering::SeqCst) {
        match capture_local_presence(&config) {
            Ok(Some(mut payload)) => {
                let signature_changed = payload.signature != last_signature;
                if signature_changed {
                    last_signature = payload.signature.clone();
                    activity_started_at = Some(Utc::now().timestamp_millis());
                }
                if payload.started_at_millis.is_none() && payload.ended_at_millis.is_none() {
                    payload.started_at_millis = activity_started_at;
                }

                let publish_signature = payload.publish_signature();
                if publish_signature != last_publish_signature {
                    last_publish_signature = publish_signature;
                    last_sent_end_timestamp = None;
                }

                if should_skip_timestamp_update(&payload, last_sent_end_timestamp) {
                    update_presence_heartbeat(&state, true, None, run_id);
                    consecutive_errors = 0;
                    sleep_with_stop(sync_interval, &stop_flag);
                    continue;
                }

                match apply_discord_presence(
                    &mut discord_client,
                    &config,
                    &config.discord_application_id,
                    &payload,
                    artwork_publisher.as_ref(),
                    locale,
                ) {
                    Ok(debug_payload) => {
                        update_presence_snapshot(
                            &state,
                            true,
                            None,
                            Some(payload.summary),
                            Some(debug_payload),
                            run_id,
                        );
                        last_sent_end_timestamp =
                            if payload.media_is_playing && payload.ended_at_millis.is_some() {
                                payload.ended_at_millis
                            } else {
                                None
                            };
                        consecutive_errors = 0;
                        sleep_with_stop(sync_interval, &stop_flag);
                    }
                    Err(error) => {
                        discord_client = None;
                        last_sent_end_timestamp = None;
                        consecutive_errors = consecutive_errors.saturating_add(1);
                        set_discord_error(&state, Some(error), false, run_id);
                        sleep_with_stop(error_backoff(consecutive_errors), &stop_flag);
                    }
                }
            }
            Ok(None) => {
                let _ = clear_discord_presence(
                    &mut discord_client,
                    &config.discord_application_id,
                    locale,
                );
                update_presence_snapshot(&state, true, None, None, None, run_id);
                last_signature.clear();
                last_publish_signature.clear();
                last_sent_end_timestamp = None;
                consecutive_errors = 0;
                sleep_with_stop(sync_interval, &stop_flag);
            }
            Err(error) => {
                discord_client = None;
                last_sent_end_timestamp = None;
                consecutive_errors = consecutive_errors.saturating_add(1);
                set_discord_error(&state, Some(error), false, run_id);
                sleep_with_stop(error_backoff(consecutive_errors), &stop_flag);
            }
        }
    }

    let _ = clear_discord_presence(&mut discord_client, &config.discord_application_id, locale);
    mark_stopped(&state, run_id);
}

fn capture_local_presence(config: &ClientConfig) -> Result<Option<DiscordPresencePayload>, String> {
    let snapshot = if should_capture_foreground_snapshot_for_reporting(config) {
        get_foreground_snapshot_for_reporting(
            should_capture_process_name_for_reporting(config),
            should_capture_window_title_for_reporting(config),
        )?
    } else {
        ForegroundSnapshot::default()
    };

    let media = if should_capture_media_for_reporting(config) {
        get_now_playing().unwrap_or_else(|_| MediaInfo::default())
    } else {
        MediaInfo::default()
    };
    let foreground_app_icon = if should_capture_foreground_app_icon_for_reporting(config) {
        get_foreground_app_icon().unwrap_or(None)
    } else {
        None
    };

    let Some(resolved) = resolve_activity(config, &snapshot, &media) else {
        return Ok(None);
    };
    let Some(text) = build_presence_text(config, &resolved, &media) else {
        return Ok(None);
    };
    let active_addons = select_presence_addons(config, &resolved);
    let (started_at_millis, ended_at_millis) = if should_use_media_timestamps(config, &resolved) {
        build_media_timestamps(config, &media)
    } else {
        (None, None)
    };
    let media_reportable = media.is_reportable(config.report_stopped_media);

    Ok(Some(DiscordPresencePayload {
        activity_name: text.activity_name,
        details: text.details,
        state: text.state,
        status_display_type: build_status_display_type(config),
        started_at_millis,
        ended_at_millis,
        media_duration_ms: if media_reportable {
            media.duration_ms
        } else {
            None
        },
        media_position_ms: if media_reportable {
            media.position_ms
        } else {
            None
        },
        media_is_playing: media.is_playing,
        summary: text.summary,
        signature: text.signature,
        artwork: build_presence_artwork(config, &media),
        icon: build_presence_icon(
            config,
            snapshot.process_name.as_str(),
            foreground_app_icon.as_ref(),
            &media,
        ),
        buttons: build_presence_buttons(&active_addons),
        party: build_presence_party(&active_addons),
        secrets: build_presence_secrets(&active_addons),
    }))
}

fn apply_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    config: &ClientConfig,
    application_id: &str,
    payload: &DiscordPresencePayload,
    artwork_publisher: Option<&ArtworkPublisher>,
    locale: BackendLocale,
) -> Result<DiscordDebugPayload, String> {
    with_discord_client(client_slot, application_id, locale, |client| {
        let mut activity_payload =
            activity::Activity::new().activity_type(effective_activity_type(config));
        if let Some(name) = payload.activity_name.as_deref() {
            activity_payload = activity_payload.name(name.to_string());
        }
        if !payload.details.trim().is_empty() {
            activity_payload = activity_payload.details(payload.details.clone());
        }
        if let Some(status_display_type) = payload.status_display_type.as_ref() {
            activity_payload =
                activity_payload.status_display_type(map_status_display_type(status_display_type));
        }
        let mut artwork_url = None;
        let mut artwork_hover_text = None;
        let mut artwork_content_type = None;
        let mut artwork_upload_error = None;
        let mut app_icon_url = None;
        let mut app_icon_text = None;
        let mut app_icon_error = None;
        if config.discord_use_app_artwork || config.discord_use_music_artwork {
            if let (Some(artwork), Some(artwork_publisher)) =
                (payload.artwork.as_ref(), artwork_publisher)
            {
                match artwork_publisher.publish(
                    artwork.bytes.clone(),
                    artwork.cache_key.clone(),
                    PublishAssetKind::MusicArtwork,
                ) {
                    Ok(image_url) => {
                        artwork_url = Some(image_url.clone());
                        artwork_hover_text = Some(artwork.hover_text.clone());
                        artwork_content_type =
                            Some(PublishAssetKind::MusicArtwork.content_type().to_string());
                        activity_payload = activity_payload.assets(
                            activity::Assets::new()
                                .large_image(image_url)
                                .large_text(artwork.hover_text.clone()),
                        );
                    }
                    Err(error) => {
                        artwork_upload_error = Some(error);
                    }
                }
            }

            if let (Some(icon), Some(artwork_publisher)) =
                (payload.icon.as_ref(), artwork_publisher)
            {
                match artwork_publisher.publish(
                    icon.bytes.clone(),
                    icon.cache_key.clone(),
                    PublishAssetKind::AppIcon,
                ) {
                    Ok(image_url) => {
                        app_icon_url = Some(image_url);
                        app_icon_text = Some(icon.hover_text.clone());
                    }
                    Err(error) => {
                        app_icon_error = Some(error);
                    }
                }
            }
        }
        let mut assets = activity::Assets::new();
        let mut has_assets = false;
        if let Some(image_url) = artwork_url.as_ref() {
            assets = assets.large_image(image_url.clone());
            has_assets = true;
            if let Some(hover_text) = artwork_hover_text.as_ref() {
                assets = assets.large_text(hover_text.clone());
            }
        }
        if let Some(image_url) = app_icon_url.as_ref() {
            if has_assets {
                assets = assets.small_image(image_url.clone());
                if let Some(icon_text) = app_icon_text.as_ref() {
                    assets = assets.small_text(icon_text.clone());
                }
            } else {
                assets = assets.large_image(image_url.clone());
                has_assets = true;
                if let Some(icon_text) = app_icon_text.as_ref() {
                    assets = assets.large_text(icon_text.clone());
                }
            }
        }
        if has_assets {
            activity_payload = activity_payload.assets(assets);
        }
        if let Some(state) = payload.state.as_deref() {
            activity_payload = activity_payload.state(state.to_string());
        }
        if !payload.buttons.is_empty() {
            activity_payload = activity_payload.buttons(
                payload
                    .buttons
                    .iter()
                    .map(|button| activity::Button::new(button.label.clone(), button.url.clone()))
                    .collect(),
            );
        }
        if let Some(party) = payload.party.as_ref() {
            let mut activity_party = activity::Party::new();
            if let Some(id) = party.id.as_deref() {
                activity_party = activity_party.id(id.to_string());
            }
            if let Some(size) = party.size {
                activity_party = activity_party.size(size);
            }
            activity_payload = activity_payload.party(activity_party);
        }
        if let Some(secrets) = payload.secrets.as_ref() {
            let mut activity_secrets = activity::Secrets::new();
            if let Some(join) = secrets.join.as_deref() {
                activity_secrets = activity_secrets.join(join.to_string());
            }
            if let Some(spectate) = secrets.spectate.as_deref() {
                activity_secrets = activity_secrets.spectate(spectate.to_string());
            }
            if let Some(match_secret) = secrets.match_secret.as_deref() {
                activity_secrets = activity_secrets.r#match(match_secret.to_string());
            }
            activity_payload = activity_payload.secrets(activity_secrets);
        }
        let mut timestamps = activity::Timestamps::new();
        let mut has_timestamps = false;
        if let Some(started_at) = payload.started_at_millis {
            timestamps = timestamps.start(started_at);
            has_timestamps = true;
        }
        if let Some(ended_at) = payload.ended_at_millis {
            timestamps = timestamps.end(ended_at);
            has_timestamps = true;
        }
        if has_timestamps {
            activity_payload = activity_payload.timestamps(timestamps);
        }
        client
            .set_activity(activity_payload)
            .map_err(|error| format_error(locale, "Failed to update Discord presence", error))?;

        Ok(DiscordDebugPayload {
            activity_name: payload.activity_name.clone(),
            details: payload.details.clone(),
            state: payload.state.clone(),
            summary: payload.summary.clone(),
            signature: payload.signature.clone(),
            report_mode_applied: report_mode_key(config).to_string(),
            activity_type: activity_type_key(config).to_string(),
            status_display_type: payload
                .status_display_type
                .as_ref()
                .map(|value| match value {
                    DiscordPresenceStatusDisplayType::Name => "name".to_string(),
                    DiscordPresenceStatusDisplayType::State => "state".to_string(),
                    DiscordPresenceStatusDisplayType::Details => "details".to_string(),
                }),
            started_at_millis: payload.started_at_millis,
            ended_at_millis: payload.ended_at_millis,
            media_duration_ms: payload.media_duration_ms,
            media_position_ms: payload.media_position_ms,
            app_icon_url,
            app_icon_text,
            app_icon_error,
            artwork_url,
            artwork_hover_text,
            artwork_content_type,
            artwork_upload_error,
        })
    })
}

fn map_activity_type(value: &DiscordActivityType) -> activity::ActivityType {
    match value {
        DiscordActivityType::Listening => activity::ActivityType::Listening,
        DiscordActivityType::Watching => activity::ActivityType::Watching,
        DiscordActivityType::Competing => activity::ActivityType::Competing,
        DiscordActivityType::Playing => activity::ActivityType::Playing,
    }
}

fn map_status_display_type(
    value: &DiscordPresenceStatusDisplayType,
) -> activity::StatusDisplayType {
    match value {
        DiscordPresenceStatusDisplayType::Name => activity::StatusDisplayType::Name,
        DiscordPresenceStatusDisplayType::State => activity::StatusDisplayType::State,
        DiscordPresenceStatusDisplayType::Details => activity::StatusDisplayType::Details,
    }
}

fn effective_activity_type(config: &ClientConfig) -> activity::ActivityType {
    match config.discord_report_mode {
        DiscordReportMode::Music => activity::ActivityType::Listening,
        DiscordReportMode::Custom => map_activity_type(&config.discord_activity_type),
        DiscordReportMode::App | DiscordReportMode::Mixed => activity::ActivityType::Playing,
    }
}

fn report_mode_key(config: &ClientConfig) -> &'static str {
    match config.discord_report_mode {
        DiscordReportMode::Music => "music",
        DiscordReportMode::App => "app",
        DiscordReportMode::Mixed => "mixed",
        DiscordReportMode::Custom => "custom",
    }
}

fn activity_type_key(config: &ClientConfig) -> &'static str {
    match config.discord_report_mode {
        DiscordReportMode::Music => "listening",
        DiscordReportMode::App | DiscordReportMode::Mixed => "playing",
        DiscordReportMode::Custom => match &config.discord_activity_type {
            DiscordActivityType::Listening => "listening",
            DiscordActivityType::Watching => "watching",
            DiscordActivityType::Competing => "competing",
            DiscordActivityType::Playing => "playing",
        },
    }
}

fn should_use_media_timestamps(
    config: &ClientConfig,
    resolved: &crate::rules::ResolvedActivity,
) -> bool {
    if resolved.media_summary.is_none() {
        return false;
    }

    match config.discord_report_mode {
        DiscordReportMode::Music => true,
        DiscordReportMode::Mixed => config.discord_smart_enable_music_countdown,
        DiscordReportMode::Custom => resolved.status_text.is_none(),
        DiscordReportMode::App => false,
    }
}

fn build_presence_text(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
    media: &MediaInfo,
) -> Option<DiscordPresenceText> {
    match config.discord_report_mode {
        DiscordReportMode::Music => build_music_presence_text(config, media),
        DiscordReportMode::App => build_app_presence_text(config, resolved),
        DiscordReportMode::Mixed => build_smart_presence_text(config, resolved, media),
        DiscordReportMode::Custom => build_custom_presence_text(config, resolved, media),
    }
}

fn build_music_presence_text(
    config: &ClientConfig,
    media: &MediaInfo,
) -> Option<DiscordPresenceText> {
    if !is_music_visible(config, media) {
        return None;
    }

    let details =
        first_non_empty_presence_value(&[media.title.as_str(), media.summary().as_str()])?;
    let state = non_empty_presence_value(media.artist.as_str());
    Some(build_presence_text_from_parts(
        build_music_only_activity_name(config, media),
        details,
        state,
    ))
}

fn build_app_presence_text(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
) -> Option<DiscordPresenceText> {
    let activity_name = primary_app_activity_name(config, resolved)?;
    let details = secondary_app_details(config, resolved, Some(activity_name.as_str()));

    Some(build_presence_text_from_parts(
        Some(activity_name),
        details,
        None,
    ))
}

fn build_smart_presence_text(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
    media: &MediaInfo,
) -> Option<DiscordPresenceText> {
    if let Some(activity_name) = primary_app_activity_name(config, resolved) {
        let details = secondary_app_details(config, resolved, Some(activity_name.as_str()));
        let state = smart_music_state(config, media);
        return Some(build_presence_text_from_parts(
            Some(activity_name),
            details,
            state,
        ));
    }

    build_music_presence_text(config, media)
}

fn build_custom_presence_text(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
    media: &MediaInfo,
) -> Option<DiscordPresenceText> {
    Some(build_presence_text_from_parts(
        build_custom_mode_activity_name(config, resolved, media),
        resolved.discord_details.clone(),
        resolved.discord_state.clone(),
    ))
}

fn build_music_only_activity_name(config: &ClientConfig, media: &MediaInfo) -> Option<String> {
    if !is_music_visible(config, media) {
        return None;
    }

    match current_app_name_mode(config) {
        DiscordAppNameMode::Default => None,
        DiscordAppNameMode::Song => non_empty_presence_value(media.title.as_str()),
        DiscordAppNameMode::Artist => non_empty_presence_value(media.artist.as_str()),
        DiscordAppNameMode::Album => non_empty_presence_value(media.album.as_str()),
        DiscordAppNameMode::Source => current_media_source_name(media),
        DiscordAppNameMode::Custom => non_empty_presence_value(current_custom_app_name(config)),
    }
}

fn build_custom_mode_activity_name(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
    media: &MediaInfo,
) -> Option<String> {
    match current_app_name_mode(config) {
        DiscordAppNameMode::Default => primary_app_activity_name(config, resolved),
        DiscordAppNameMode::Song
        | DiscordAppNameMode::Artist
        | DiscordAppNameMode::Album
        | DiscordAppNameMode::Source
        | DiscordAppNameMode::Custom => build_music_only_activity_name(config, media),
    }
}

fn current_media_source_name(media: &MediaInfo) -> Option<String> {
    non_empty_presence_value(media.source_app_id.as_str()).map(|value| fallback_app_name(&value))
}

fn primary_app_activity_name(config: &ClientConfig, resolved: &ResolvedActivity) -> Option<String> {
    resolved
        .status_text
        .as_deref()
        .and_then(non_empty_presence_value)
        .or_else(|| {
            resolved
                .process_title
                .as_deref()
                .and_then(non_empty_presence_value)
        })
        .or_else(|| {
            if config.report_foreground_app {
                non_empty_presence_value(resolved.process_name.as_str())
            } else {
                None
            }
        })
}

fn secondary_app_details(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
    activity_name: Option<&str>,
) -> String {
    let should_show_app_name = match config.discord_report_mode {
        DiscordReportMode::Mixed => config.discord_smart_show_app_name,
        _ => config.report_foreground_app,
    };

    if !should_show_app_name {
        return String::new();
    }

    let Some(process_name) = non_empty_presence_value(resolved.process_name.as_str()) else {
        return String::new();
    };

    if activity_name == Some(process_name.as_str()) {
        String::new()
    } else {
        process_name
    }
}

fn smart_music_state(config: &ClientConfig, media: &MediaInfo) -> Option<String> {
    if !is_music_visible(config, media) {
        return None;
    }

    let summary =
        first_non_empty_presence_value(&[media.summary().as_str(), media.title.as_str()])?;
    Some(format!("🎵 {summary}"))
}

fn build_presence_text_from_parts(
    activity_name: Option<String>,
    details: String,
    state: Option<String>,
) -> DiscordPresenceText {
    let details_ref = if details.trim().is_empty() {
        None
    } else {
        Some(details.as_str())
    };
    let summary =
        join_non_empty_presence(&[activity_name.as_deref(), details_ref, state.as_deref()]);
    let signature = if summary.is_empty() {
        activity_name.clone().unwrap_or_default()
    } else {
        summary.clone()
    };

    DiscordPresenceText {
        activity_name,
        details,
        state,
        summary,
        signature,
    }
}

fn build_status_display_type(config: &ClientConfig) -> Option<DiscordPresenceStatusDisplayType> {
    Some(match current_status_display(config) {
        DiscordStatusDisplay::Name => DiscordPresenceStatusDisplayType::Name,
        DiscordStatusDisplay::State => DiscordPresenceStatusDisplayType::State,
        DiscordStatusDisplay::Details => DiscordPresenceStatusDisplayType::Details,
    })
}

fn current_status_display(config: &ClientConfig) -> &DiscordStatusDisplay {
    match config.discord_report_mode {
        DiscordReportMode::Mixed => &config.discord_smart_status_display,
        DiscordReportMode::Music => &config.discord_music_status_display,
        DiscordReportMode::App => &config.discord_app_status_display,
        DiscordReportMode::Custom => &config.discord_custom_mode_status_display,
    }
}

fn current_app_name_mode(config: &ClientConfig) -> &DiscordAppNameMode {
    match config.discord_report_mode {
        DiscordReportMode::Mixed => &config.discord_smart_app_name_mode,
        DiscordReportMode::Music => &config.discord_music_app_name_mode,
        DiscordReportMode::App => &config.discord_app_app_name_mode,
        DiscordReportMode::Custom => &config.discord_custom_mode_app_name_mode,
    }
}

fn current_custom_app_name(config: &ClientConfig) -> &str {
    match config.discord_report_mode {
        DiscordReportMode::Mixed => config.discord_smart_custom_app_name.as_str(),
        DiscordReportMode::Music => config.discord_music_custom_app_name.as_str(),
        DiscordReportMode::App => config.discord_app_custom_app_name.as_str(),
        DiscordReportMode::Custom => config.discord_custom_mode_custom_app_name.as_str(),
    }
}

fn non_empty_presence_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn first_non_empty_presence_value(values: &[&str]) -> Option<String> {
    values
        .iter()
        .find_map(|value| non_empty_presence_value(value))
}

fn join_non_empty_presence(values: &[Option<&str>]) -> String {
    values
        .iter()
        .filter_map(|value| {
            value.and_then(|text| {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn is_music_visible(config: &ClientConfig, media: &MediaInfo) -> bool {
    config.report_media && media.is_reportable(config.report_stopped_media)
}

fn has_custom_addons(config: &ClientConfig) -> bool {
    !config.discord_custom_buttons.is_empty()
        || !config.discord_custom_party_id.trim().is_empty()
        || config.discord_custom_party_size_current.is_some()
        || config.discord_custom_party_size_max.is_some()
        || !config.discord_custom_join_secret.trim().is_empty()
        || !config.discord_custom_spectate_secret.trim().is_empty()
        || !config.discord_custom_match_secret.trim().is_empty()
}

fn build_custom_addons(config: &ClientConfig) -> ResolvedDiscordAddons {
    let buttons = config
        .discord_custom_buttons
        .iter()
        .map(|button| DiscordPresenceButton {
            label: button.label.trim().to_string(),
            url: button.url.trim().to_string(),
        })
        .filter(|button| !button.label.is_empty() && !button.url.is_empty())
        .take(2)
        .map(|button| crate::models::DiscordRichPresenceButtonConfig {
            label: button.label,
            url: button.url,
        })
        .collect();
    let id = non_empty_presence_value(config.discord_custom_party_id.as_str());
    let size = match (
        config.discord_custom_party_size_current,
        config.discord_custom_party_size_max,
    ) {
        (Some(current), Some(maximum)) if current > 0 && maximum > 0 && current <= maximum => {
            Some((current, maximum))
        }
        _ => None,
    };
    let join = non_empty_presence_value(config.discord_custom_join_secret.as_str());
    let spectate = non_empty_presence_value(config.discord_custom_spectate_secret.as_str());
    let match_secret = non_empty_presence_value(config.discord_custom_match_secret.as_str());

    ResolvedDiscordAddons {
        buttons,
        party: if id.is_none() && size.is_none() {
            None
        } else {
            Some(crate::rules::ResolvedDiscordParty { id, size })
        },
        secrets: if join.is_none() && spectate.is_none() && match_secret.is_none() {
            None
        } else {
            Some(crate::rules::ResolvedDiscordSecrets {
                join,
                spectate,
                match_secret,
            })
        },
    }
}

fn select_presence_addons(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
) -> ResolvedDiscordAddons {
    let custom_addons = build_custom_addons(config);
    let custom_addons_configured = has_custom_addons(config) && !custom_addons.is_empty();

    if config.discord_report_mode == DiscordReportMode::Custom {
        if custom_addons_configured {
            return custom_addons;
        }
        return resolved.discord_addons.clone();
    }

    if config.discord_use_custom_addons_override && custom_addons_configured {
        return custom_addons;
    }

    resolved.discord_addons.clone()
}

fn build_presence_buttons(addons: &ResolvedDiscordAddons) -> Vec<DiscordPresenceButton> {
    addons
        .buttons
        .iter()
        .map(|button| DiscordPresenceButton {
            label: button.label.trim().to_string(),
            url: button.url.trim().to_string(),
        })
        .filter(|button| !button.label.is_empty() && !button.url.is_empty())
        .take(2)
        .collect()
}

fn build_presence_party(addons: &ResolvedDiscordAddons) -> Option<DiscordPresenceParty> {
    let party = addons.party.as_ref()?;
    let size = match party.size {
        Some((current, maximum)) if current > 0 && maximum > 0 && current <= maximum => {
            let current = i32::try_from(current).ok()?;
            let maximum = i32::try_from(maximum).ok()?;
            Some([current, maximum])
        }
        _ => None,
    };

    if party.id.is_none() && size.is_none() {
        return None;
    }

    Some(DiscordPresenceParty {
        id: party.id.clone(),
        size,
    })
}

fn build_presence_secrets(addons: &ResolvedDiscordAddons) -> Option<DiscordPresenceSecrets> {
    let secrets = addons.secrets.as_ref()?;
    if secrets.join.is_none() && secrets.spectate.is_none() && secrets.match_secret.is_none() {
        return None;
    }

    Some(DiscordPresenceSecrets {
        join: secrets.join.clone(),
        spectate: secrets.spectate.clone(),
        match_secret: secrets.match_secret.clone(),
    })
}

fn build_media_timestamps(config: &ClientConfig, media: &MediaInfo) -> (Option<i64>, Option<i64>) {
    if !media.is_reportable(config.report_stopped_media) {
        return (None, None);
    }

    let position_ms = media
        .position_ms
        .and_then(|value| i64::try_from(value).ok());
    let duration_ms = media
        .duration_ms
        .and_then(|value| i64::try_from(value).ok());

    let Some(position_ms) = position_ms else {
        return (None, None);
    };

    if !media.is_playing {
        let Some(duration_ms) = duration_ms else {
            return (None, None);
        };
        return calc_paused_timestamps(position_ms, duration_ms)
            .map(|(started_at, ended_at)| (Some(started_at), Some(ended_at)))
            .unwrap_or((None, None));
    }

    let (started_at, ended_at) = match duration_ms {
        Some(duration_ms) => calc_playing_timestamps(position_ms, duration_ms),
        None => (Utc::now().timestamp_millis().checked_sub(position_ms), None),
    };

    (started_at, ended_at)
}

fn calc_playing_timestamps(position_ms: i64, duration_ms: i64) -> (Option<i64>, Option<i64>) {
    let now_ms = Utc::now().timestamp_millis();
    let remaining_ms = duration_ms.saturating_sub(position_ms).max(0);
    let ended_at = now_ms.checked_add(remaining_ms);
    let started_at = ended_at.and_then(|ended_at| ended_at.checked_sub(duration_ms));
    (started_at, ended_at)
}

fn calc_paused_timestamps(position_ms: i64, duration_ms: i64) -> Option<(i64, i64)> {
    // Based on apoint123/inflink-rs and the musicpresence.app future timestamp
    // trick: https://github.com/apoint123/inflink-rs/blob/main/packages/backend/src/discord.rs
    const ONE_YEAR_MS: i64 = 365 * 24 * 60 * 60 * 1000;

    if duration_ms <= 0 {
        return None;
    }

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let current_progress_ms = position_ms.clamp(0, duration_ms);
    let future_start = now_ms
        .checked_sub(current_progress_ms)?
        .checked_add(ONE_YEAR_MS)?;
    let future_end = future_start.checked_add(duration_ms)?;

    Some((future_start, future_end))
}

fn should_skip_timestamp_update(
    payload: &DiscordPresencePayload,
    last_sent_end_timestamp: Option<i64>,
) -> bool {
    if !payload.media_is_playing {
        return false;
    }

    let Some(last_end) = last_sent_end_timestamp else {
        return false;
    };
    let Some(next_end) = payload.ended_at_millis else {
        return false;
    };

    last_end.abs_diff(next_end) < TIMESTAMP_UPDATE_THRESHOLD_MS as u64
}

fn build_presence_artwork(
    config: &ClientConfig,
    media: &MediaInfo,
) -> Option<DiscordPresenceArtwork> {
    if !config.discord_use_music_artwork
        || !media.is_reportable(config.report_stopped_media)
        || config.discord_report_mode == DiscordReportMode::App
    {
        return None;
    }

    let artwork = media.artwork.as_ref()?;
    if artwork.bytes.is_empty() || artwork.content_type.trim().is_empty() {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&artwork.bytes, &mut hasher);
    std::hash::Hash::hash(&artwork.content_type, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
    let hover_text = if media.album.trim().is_empty() {
        media.summary()
    } else {
        media.album.trim().to_string()
    };

    Some(DiscordPresenceArtwork {
        bytes: artwork.bytes.clone(),
        content_type: artwork.content_type.clone(),
        hover_text,
        cache_key,
    })
}

fn build_presence_icon(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceIcon> {
    if !config.discord_use_app_artwork && !config.discord_use_music_artwork {
        return None;
    }

    if config.discord_report_mode == DiscordReportMode::App {
        if config.discord_use_app_artwork {
            return build_foreground_app_icon(process_name, foreground_app_icon);
        }
        return None;
    }

    if media.is_reportable(config.report_stopped_media)
        && matches!(
            config.discord_report_mode,
            DiscordReportMode::Music | DiscordReportMode::Mixed
        )
    {
        if config.discord_use_music_artwork {
            return build_playback_source_icon(media, config.report_stopped_media).or_else(|| {
                if config.discord_use_app_artwork {
                    build_foreground_app_icon(process_name, foreground_app_icon)
                } else {
                    None
                }
            });
        }
    }

    if config.discord_use_app_artwork {
        return build_foreground_app_icon(process_name, foreground_app_icon).or_else(|| {
            if config.discord_use_music_artwork {
                build_playback_source_icon(media, config.report_stopped_media)
            } else {
                None
            }
        });
    }

    if config.discord_use_music_artwork {
        return build_playback_source_icon(media, config.report_stopped_media);
    }

    None
}

fn build_foreground_app_icon(
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
) -> Option<DiscordPresenceIcon> {
    let foreground_app_icon = foreground_app_icon?;
    if foreground_app_icon.bytes.is_empty() || foreground_app_icon.content_type.trim().is_empty() {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&foreground_app_icon.bytes, &mut hasher);
    std::hash::Hash::hash(&foreground_app_icon.content_type, &mut hasher);
    std::hash::Hash::hash(&process_name, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
    let hover_text = if process_name.trim().is_empty() {
        "Current app".to_string()
    } else {
        fallback_app_name(process_name)
    };

    Some(DiscordPresenceIcon {
        bytes: foreground_app_icon.bytes.clone(),
        content_type: foreground_app_icon.content_type.clone(),
        hover_text,
        cache_key: format!("discord-foreground-app-icon-{cache_key}"),
    })
}

fn build_playback_source_icon(
    media: &MediaInfo,
    include_stopped: bool,
) -> Option<DiscordPresenceIcon> {
    if !media.is_reportable(include_stopped) {
        return None;
    }

    let source_icon = media.source_icon.as_ref()?;
    if source_icon.bytes.is_empty() || source_icon.content_type.trim().is_empty() {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&source_icon.bytes, &mut hasher);
    std::hash::Hash::hash(&source_icon.content_type, &mut hasher);
    std::hash::Hash::hash(&media.source_app_id, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
    let hover_text = fallback_app_name(&media.source_app_id);

    Some(DiscordPresenceIcon {
        bytes: source_icon.bytes.clone(),
        content_type: source_icon.content_type.clone(),
        hover_text,
        cache_key: format!("discord-source-icon-{cache_key}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artwork_config() -> ClientConfig {
        ClientConfig {
            discord_use_app_artwork: true,
            discord_use_music_artwork: true,
            ..ClientConfig::default()
        }
    }

    fn sample_media() -> MediaInfo {
        MediaInfo {
            title: "Track Name".into(),
            artist: "Artist Name".into(),
            album: "Album Name".into(),
            is_playing: true,
            artwork: Some(MediaArtwork {
                bytes: vec![1, 2, 3],
                content_type: "image/png".into(),
            }),
            ..MediaInfo::default()
        }
    }

    fn sample_resolved() -> ResolvedActivity {
        ResolvedActivity {
            process_name: "Code.exe".into(),
            process_title: Some("repo".into()),
            media_summary: None,
            play_source: None,
            status_text: Some("Matched Title".into()),
            discord_addons: ResolvedDiscordAddons::default(),
            discord_details: "Matched Title".into(),
            discord_state: Some("Code.exe".into()),
            summary: "Matched Title · Code.exe".into(),
            signature: "Matched Title · Code.exe".into(),
        }
    }

    #[test]
    fn presence_artwork_prefers_album_for_hover_text() {
        let config = artwork_config();
        let media = sample_media();

        let artwork = build_presence_artwork(&config, &media).expect("artwork");

        assert_eq!(artwork.hover_text, "Album Name");
    }

    #[test]
    fn presence_artwork_falls_back_when_album_missing() {
        let config = artwork_config();
        let mut media = sample_media();
        media.album.clear();

        let artwork = build_presence_artwork(&config, &media).expect("artwork");

        assert_eq!(artwork.hover_text, "Track Name / Artist Name");
    }

    #[test]
    fn app_mode_skips_music_artwork_even_when_media_is_active() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::App,
            discord_use_music_artwork: true,
            ..artwork_config()
        };
        let media = sample_media();

        let artwork = build_presence_artwork(&config, &media);

        assert!(artwork.is_none());
    }

    #[test]
    fn mixed_mode_prefers_source_icon_when_music_artwork_is_enabled() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            discord_use_music_artwork: true,
            discord_use_app_artwork: true,
            ..ClientConfig::default()
        };
        let foreground_icon = MediaArtwork {
            bytes: vec![9, 9, 9],
            content_type: "image/png".into(),
        };
        let mut media = sample_media();
        media.source_app_id = "spotify.exe".into();
        media.source_icon = Some(MediaArtwork {
            bytes: vec![7, 7, 7],
            content_type: "image/png".into(),
        });

        let icon =
            build_presence_icon(&config, "code.exe", Some(&foreground_icon), &media).expect("icon");

        assert_eq!(icon.hover_text, "Spotify");
    }

    #[test]
    fn app_artwork_becomes_main_icon_when_music_is_unavailable() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            discord_use_app_artwork: true,
            ..ClientConfig::default()
        };
        let foreground_icon = MediaArtwork {
            bytes: vec![9, 9, 9],
            content_type: "image/png".into(),
        };
        let media = MediaInfo::default();

        let icon =
            build_presence_icon(&config, "code.exe", Some(&foreground_icon), &media).expect("icon");

        assert_eq!(icon.hover_text, "Code");
    }

    #[test]
    fn app_mode_does_not_fall_back_to_music_source_icon() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::App,
            discord_use_app_artwork: false,
            discord_use_music_artwork: true,
            ..ClientConfig::default()
        };
        let mut media = sample_media();
        media.source_app_id = "spotify.exe".into();
        media.source_icon = Some(MediaArtwork {
            bytes: vec![7, 7, 7],
            content_type: "image/png".into(),
        });

        let icon = build_presence_icon(&config, "code.exe", None, &media);

        assert!(icon.is_none());
    }

    #[test]
    fn music_mode_can_override_activity_name_with_custom_text() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            discord_music_app_name_mode: crate::models::DiscordAppNameMode::Custom,
            discord_music_custom_app_name: "My Custom App".into(),
            ..ClientConfig::default()
        };
        let media = sample_media();

        let value = build_music_only_activity_name(&config, &media);

        assert_eq!(value, Some("My Custom App".to_string()));
    }

    #[test]
    fn music_mode_can_use_media_source_for_activity_name() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            discord_music_app_name_mode: crate::models::DiscordAppNameMode::Source,
            ..ClientConfig::default()
        };
        let mut media = sample_media();
        media.source_app_id = "spotify.exe".into();

        let value = build_music_only_activity_name(&config, &media);

        assert_eq!(value, Some("Spotify".to_string()));
    }

    #[test]
    fn smart_mode_uses_rule_hit_as_name_and_app_as_details() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            discord_smart_show_app_name: true,
            ..ClientConfig::default()
        };
        let resolved = sample_resolved();
        let media = MediaInfo::default();

        let text = build_smart_presence_text(&config, &resolved, &media).expect("text");

        assert_eq!(text.activity_name, Some("Matched Title".to_string()));
        assert_eq!(text.details, "Code.exe".to_string());
        assert_eq!(text.state, None);
    }

    #[test]
    fn smart_mode_puts_music_on_state_line() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            discord_smart_show_app_name: true,
            ..ClientConfig::default()
        };
        let resolved = sample_resolved();
        let media = sample_media();

        let text = build_smart_presence_text(&config, &resolved, &media).expect("text");

        assert_eq!(text.activity_name, Some("Matched Title".to_string()));
        assert_eq!(text.details, "Code.exe".to_string());
        assert_eq!(
            text.state,
            Some("🎵 Track Name / Artist Name / Album Name".to_string())
        );
    }

    #[test]
    fn competing_mode_can_set_status_display_to_state() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Custom,
            discord_activity_type: crate::models::DiscordActivityType::Competing,
            discord_custom_mode_status_display: crate::models::DiscordStatusDisplay::State,
            ..ClientConfig::default()
        };

        let value = build_status_display_type(&config);

        assert_eq!(value, Some(DiscordPresenceStatusDisplayType::State));
    }

    #[test]
    fn paused_media_uses_future_timestamps_when_visible() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Music,
            report_stopped_media: true,
            ..ClientConfig::default()
        };
        let mut media = sample_media();
        media.is_playing = false;
        media.position_ms = Some(30_000);
        media.duration_ms = Some(180_000);

        let (started_at, ended_at) = build_media_timestamps(&config, &media);
        let started_at = started_at.expect("start timestamp");
        let ended_at = ended_at.expect("end timestamp");

        assert!(started_at > Utc::now().timestamp_millis());
        assert_eq!(ended_at - started_at, 180_000);
    }

    #[test]
    fn playing_timestamp_update_skips_small_end_drift() {
        let payload = DiscordPresencePayload {
            activity_name: None,
            details: "Track Name".into(),
            state: Some("Artist Name".into()),
            status_display_type: None,
            started_at_millis: Some(900_000),
            ended_at_millis: Some(1_000_050),
            media_duration_ms: Some(180_000),
            media_position_ms: Some(30_000),
            media_is_playing: true,
            summary: "Track Name".into(),
            signature: "Track Name".into(),
            artwork: None,
            icon: None,
            buttons: Vec::new(),
            party: None,
            secrets: None,
        };

        assert!(should_skip_timestamp_update(&payload, Some(1_000_000)));
        assert!(!should_skip_timestamp_update(&payload, Some(999_800)));
    }

    #[test]
    fn mixed_mode_can_publish_rule_buttons() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            ..ClientConfig::default()
        };
        let mut resolved = sample_resolved();
        resolved.discord_addons = ResolvedDiscordAddons {
            buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
                label: "Open".into(),
                url: "https://example.com".into(),
            }],
            ..ResolvedDiscordAddons::default()
        };

        let addons = select_presence_addons(&config, &resolved);
        let buttons = build_presence_buttons(&addons);

        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0].label, "Open");
    }

    #[test]
    fn custom_addons_override_rule_buttons_when_enabled() {
        let config = ClientConfig {
            discord_report_mode: DiscordReportMode::Mixed,
            discord_use_custom_addons_override: true,
            discord_custom_buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
                label: "Profile".into(),
                url: "https://example.com/profile".into(),
            }],
            ..ClientConfig::default()
        };
        let mut resolved = sample_resolved();
        resolved.discord_addons = ResolvedDiscordAddons {
            buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
                label: "Rule".into(),
                url: "https://example.com/rule".into(),
            }],
            ..ResolvedDiscordAddons::default()
        };

        let addons = select_presence_addons(&config, &resolved);
        let buttons = build_presence_buttons(&addons);

        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0].label, "Profile");
    }
}

fn fallback_app_name(source: &str) -> String {
    if let Some(display_name) = display_name_for_app_id(source) {
        return display_name;
    }

    let trimmed = source.trim();
    if trimmed.is_empty() {
        return "Playback app".to_string();
    }

    let tail = trimmed.rsplit(['\\', '/', '!']).next().unwrap_or(trimmed);
    let tail = tail
        .strip_suffix(".exe")
        .or_else(|| tail.strip_suffix(".app"))
        .or_else(|| tail.strip_suffix(".desktop"))
        .unwrap_or(tail);
    let tail = tail.split('_').next().unwrap_or(tail);

    if tail.contains('.') && !tail.contains(' ') {
        let bundle_tail = tail.rsplit('.').next().unwrap_or(tail);
        return title_case_words(bundle_tail);
    }

    title_case_words(tail)
}

fn title_case_words(value: &str) -> String {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.as_str().to_ascii_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn clear_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    locale: BackendLocale,
) -> Result<(), String> {
    with_discord_client(client_slot, application_id, locale, |client| {
        client
            .clear_activity()
            .map_err(|error| format_error(locale, "Failed to clear Discord presence", error))
    })
}

fn with_discord_client<T, F>(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    locale: BackendLocale,
    mut action: F,
) -> Result<T, String>
where
    F: FnMut(&mut DiscordIpcClient) -> Result<T, String>,
{
    for _ in 0..2 {
        if client_slot.is_none() {
            let mut client = DiscordIpcClient::new(application_id);
            client
                .connect()
                .map_err(|error| format_error(locale, "Failed to connect to Discord IPC", error))?;
            *client_slot = Some(client);
        }

        let Some(client) = client_slot.as_mut() else {
            continue;
        };

        match action(client) {
            Ok(value) => return Ok(value),
            Err(_) => {
                *client_slot = None;
            }
        }
    }

    Err(discord_ipc_unavailable(locale))
}

fn validate_discord_presence_config(
    config: &ClientConfig,
    locale: BackendLocale,
) -> Result<(), String> {
    if config.discord_application_id.trim().is_empty() {
        return Err(discord_config_app_id_missing(locale));
    }
    if (config.discord_use_app_artwork || config.discord_use_music_artwork)
        && config.discord_artwork_worker_upload_url.trim().is_empty()
    {
        return Err("Discord artwork uploader service URL is required.".into());
    }
    Ok(())
}

fn update_presence_snapshot(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    connected: bool,
    last_error: Option<String>,
    current_summary: Option<String>,
    debug_payload: Option<DiscordDebugPayload>,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.connected = connected;
    inner.last_error = last_error;
    inner.last_sync_at = Some(Utc::now().to_rfc3339());
    inner.current_summary = current_summary;
    inner.debug_payload = debug_payload;
}

fn update_presence_heartbeat(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    connected: bool,
    last_error: Option<String>,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.connected = connected;
    inner.last_error = last_error;
    inner.last_sync_at = Some(Utc::now().to_rfc3339());
}

fn set_discord_error(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    error: Option<String>,
    connected: bool,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.connected = connected;
    inner.last_error = error;
    inner.debug_payload = None;
}

fn mark_stopped(state: &Arc<Mutex<DiscordPresenceInner>>, run_id: u64) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.running = false;
    inner.connected = false;
    inner.active_run_id = None;
    inner.current_summary = None;
    inner.debug_payload = None;
    inner.stop_flag = None;
}

fn format_error(locale: BackendLocale, prefix: &str, error: impl std::fmt::Display) -> String {
    if locale.is_en() {
        format!("{prefix}: {error}")
    } else {
        format!("{prefix}: {error}")
    }
}

fn discord_worker_stopping(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord RPC is still stopping. Try again shortly.".into()
    } else {
        "Discord RPC is still stopping. Try again shortly.".into()
    }
}

fn discord_ipc_unavailable(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord IPC is unavailable. Make sure Discord Desktop is running.".into()
    } else {
        "Discord IPC is unavailable. Make sure Discord Desktop is running.".into()
    }
}

fn discord_config_app_id_missing(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord application ID is required before Discord RPC can start.".into()
    } else {
        "Discord application ID is required before Discord RPC can start.".into()
    }
}

fn sleep_with_stop(duration: Duration, stop_flag: &Arc<AtomicBool>) {
    let mut remaining = duration.as_millis() as u64;
    while remaining > 0 {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        let step = remaining.min(200);
        thread::sleep(Duration::from_millis(step));
        remaining = remaining.saturating_sub(step);
    }
}

fn error_backoff(consecutive_errors: u32) -> Duration {
    let multiplier = 2u64.saturating_pow(consecutive_errors.saturating_sub(1));
    Duration::from_millis(
        (DEFAULT_SYNC_INTERVAL.as_millis() as u64)
            .saturating_mul(multiplier.max(1))
            .min(MAX_ERROR_BACKOFF_MS),
    )
}

fn wait_for_worker_exit(handle: JoinHandle<()>) -> Result<(), JoinHandle<()>> {
    let deadline = Instant::now() + WORKER_JOIN_WAIT_TIMEOUT;
    let handle = handle;

    while Instant::now() < deadline {
        if handle.is_finished() {
            let _ = handle.join();
            return Ok(());
        }
        thread::sleep(WORKER_JOIN_POLL_STEP);
    }

    Err(handle)
}

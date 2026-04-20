use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use chrono::Utc;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

mod addons;
mod assets;
mod payload;
mod text;
mod timestamps;

use addons::{
    build_presence_buttons, build_presence_party, build_presence_secrets, select_presence_addons,
};
use assets::{build_presence_artwork, build_presence_icon};
#[cfg(test)]
use text::{build_music_only_activity_name, build_smart_presence_text};
use text::{build_presence_text, build_status_display_type};
use timestamps::{build_media_timestamps, should_skip_timestamp_update};

#[cfg(test)]
use crate::rules::{ResolvedActivity, ResolvedDiscordAddons};
use crate::{
    artwork_server::{prepare_artwork_publisher, ArtworkPublisher, PublishAssetKind},
    backend_locale::BackendLocale,
    models::{
        ClientConfig, DiscordActivityType, DiscordDebugParty, DiscordDebugPayload,
        DiscordDebugSecrets, DiscordPresenceSnapshot, DiscordReportMode,
    },
    platform::{
        get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing,
        ForegroundSnapshot, MediaInfo,
    },
    rules::{
        normalize_client_config, resolve_activity,
        should_capture_foreground_app_icon_for_reporting,
        should_capture_foreground_snapshot_for_reporting, should_capture_media_for_reporting,
        should_capture_process_name_for_reporting, should_capture_window_title_for_reporting,
    },
};

const DEFAULT_SYNC_INTERVAL: Duration = Duration::from_secs(5);
const MAX_ERROR_BACKOFF_MS: u64 = 30_000;
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

use payload::{DiscordPresencePayload, DiscordPresenceStatusDisplayType};

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
            buttons: payload
                .buttons
                .iter()
                .map(|button| crate::models::DiscordRichPresenceButtonConfig {
                    label: button.label.clone(),
                    url: button.url.clone(),
                })
                .collect(),
            party: payload.party.as_ref().map(|party| DiscordDebugParty {
                id: party.id.clone(),
                size: party.size,
            }),
            secrets: payload.secrets.as_ref().map(|secrets| DiscordDebugSecrets {
                join: secrets.join.clone(),
                spectate: secrets.spectate.clone(),
                match_secret: secrets.match_secret.clone(),
            }),
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

#[cfg(test)]
mod tests;

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

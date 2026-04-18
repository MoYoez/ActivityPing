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

use crate::{
    artwork_server::{prepare_artwork_publisher, ArtworkPublisher},
    backend_locale::BackendLocale,
    models::{
        ClientConfig, DiscordActivityType, DiscordDebugPayload, DiscordPresenceSnapshot,
        DiscordReportMode,
    },
    platform::{
        display_name_for_app_id, get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing,
        MediaArtwork, MediaInfo,
    },
    rules::{normalize_client_config, resolve_activity},
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresencePayload {
    details: String,
    state: Option<String>,
    started_at_millis: Option<i64>,
    ended_at_millis: Option<i64>,
    media_duration_ms: Option<u64>,
    media_position_ms: Option<u64>,
    summary: String,
    signature: String,
    artwork: Option<DiscordPresenceArtwork>,
    icon: Option<DiscordPresenceIcon>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresenceArtwork {
    bytes: Vec<u8>,
    content_type: String,
    hover_text: String,
    cache_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiscordPresenceIcon {
    bytes: Vec<u8>,
    content_type: String,
    hover_text: String,
    cache_key: String,
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
    let mut activity_started_at = Some(Utc::now().timestamp_millis());

    while !stop_flag.load(Ordering::SeqCst) {
        match capture_local_presence(&config) {
            Ok(Some(mut payload)) => {
                if payload.signature != last_signature {
                    last_signature = payload.signature.clone();
                    activity_started_at = Some(Utc::now().timestamp_millis());
                }
                if payload.started_at_millis.is_none() && payload.ended_at_millis.is_none() {
                    payload.started_at_millis = activity_started_at;
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
                        consecutive_errors = 0;
                        sleep_with_stop(sync_interval, &stop_flag);
                    }
                    Err(error) => {
                        discord_client = None;
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
                consecutive_errors = 0;
                sleep_with_stop(sync_interval, &stop_flag);
            }
            Err(error) => {
                discord_client = None;
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
    let snapshot = get_foreground_snapshot_for_reporting(
        should_capture_process_name(config),
        config.report_window_title,
    )?;

    let media = if config.report_media || config.report_play_source {
        get_now_playing().unwrap_or_else(|_| MediaInfo::default())
    } else {
        MediaInfo::default()
    };
    let foreground_app_icon = if config.discord_use_app_artwork {
        get_foreground_app_icon().unwrap_or(None)
    } else {
        None
    };

    let Some(resolved) = resolve_activity(config, &snapshot, &media) else {
        return Ok(None);
    };
    let (started_at_millis, ended_at_millis) = if should_use_media_timestamps(config, &resolved) {
        build_media_timestamps(&media)
    } else {
        (None, None)
    };

    Ok(Some(DiscordPresencePayload {
        details: resolved.discord_details,
        state: resolved.discord_state,
        started_at_millis,
        ended_at_millis,
        media_duration_ms: if media.is_active() {
            media.duration_ms
        } else {
            None
        },
        media_position_ms: if media.is_active() {
            media.position_ms
        } else {
            None
        },
        summary: resolved.summary,
        signature: resolved.signature,
        artwork: build_presence_artwork(config, &media),
        icon: build_presence_icon(
            config,
            snapshot.process_name.as_str(),
            foreground_app_icon.as_ref(),
            &media,
        ),
    }))
}

fn should_capture_process_name(config: &ClientConfig) -> bool {
    config.report_foreground_app
        || config.discord_smart_show_app_name
        || config.app_message_rules_show_process_name
        || !config.app_message_rules.is_empty()
        || !config.app_name_only_list.is_empty()
        || matches!(config.app_filter_mode, crate::models::AppFilterMode::Whitelist)
        || !config.app_blacklist.is_empty()
        || !config.app_whitelist.is_empty()
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
        let mut activity_payload = activity::Activity::new()
            .details(payload.details.clone())
            .activity_type(effective_activity_type(config));
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
                match artwork_publisher.publish(artwork.bytes.clone(), artwork.cache_key.clone()) {
                    Ok(image_url) => {
                        artwork_url = Some(image_url.clone());
                        artwork_hover_text = Some(artwork.hover_text.clone());
                        artwork_content_type = Some("image/jpeg".to_string());
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

            if let (Some(icon), Some(artwork_publisher)) = (payload.icon.as_ref(), artwork_publisher)
            {
                match artwork_publisher.publish(icon.bytes.clone(), icon.cache_key.clone()) {
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
            details: payload.details.clone(),
            state: payload.state.clone(),
            summary: payload.summary.clone(),
            signature: payload.signature.clone(),
            report_mode_applied: report_mode_key(config).to_string(),
            activity_type: activity_type_key(config).to_string(),
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

fn build_media_timestamps(media: &MediaInfo) -> (Option<i64>, Option<i64>) {
    if !media.is_active() {
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

    let now = Utc::now().timestamp_millis();
    let started_at = now.checked_sub(position_ms);
    let ended_at = match (started_at, duration_ms) {
        (Some(started_at), Some(duration_ms)) if duration_ms >= position_ms => {
            started_at.checked_add(duration_ms)
        }
        _ => None,
    };

    (started_at, ended_at)
}

fn build_presence_artwork(
    config: &ClientConfig,
    media: &MediaInfo,
) -> Option<DiscordPresenceArtwork> {
    if !config.discord_use_music_artwork
        || !media.is_active()
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

    if media.is_active()
        && matches!(
            config.discord_report_mode,
            DiscordReportMode::Music | DiscordReportMode::Mixed
        )
    {
        if config.discord_use_music_artwork {
            return build_playback_source_icon(media).or_else(|| {
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
                build_playback_source_icon(media)
            } else {
                None
            }
        });
    }

    if config.discord_use_music_artwork {
        return build_playback_source_icon(media);
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
) -> Option<DiscordPresenceIcon> {
    if !media.is_active() {
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

        let icon = build_presence_icon(&config, "code.exe", Some(&foreground_icon), &media)
            .expect("icon");

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

        let icon = build_presence_icon(&config, "code.exe", Some(&foreground_icon), &media)
            .expect("icon");

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

pub fn config_is_ready(config: &ClientConfig) -> bool {
    validate_discord_presence_config(config, BackendLocale::EnUs).is_ok()
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

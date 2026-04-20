use crate::{
    models::{ClientConfig, DiscordReportMode},
    platform::{
        get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing,
        ForegroundSnapshot, MediaInfo,
    },
    rules::{
        resolve_activity, should_capture_foreground_app_icon_for_reporting,
        should_capture_foreground_snapshot_for_reporting, should_capture_media_for_reporting,
        should_capture_process_name_for_reporting, should_capture_window_title_for_reporting,
        ResolvedActivity,
    },
};

use super::super::{
    addons::{
        build_presence_buttons, build_presence_party, build_presence_secrets,
        select_presence_addons,
    },
    assets::{build_presence_artwork, build_presence_icon},
    payload::DiscordPresencePayload,
    text::{build_presence_text, build_status_display_type},
    timestamps::build_media_timestamps,
};

pub(super) fn capture_local_presence(
    config: &ClientConfig,
) -> Result<Option<DiscordPresencePayload>, String> {
    let snapshot = capture_foreground_snapshot(config)?;
    let media = capture_media(config);
    let foreground_app_icon = capture_foreground_app_icon(config);

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
        artwork: build_presence_artwork(
            config,
            snapshot.process_name.as_str(),
            foreground_app_icon.as_ref(),
            &media,
        ),
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

fn capture_foreground_snapshot(config: &ClientConfig) -> Result<ForegroundSnapshot, String> {
    if !should_capture_foreground_snapshot_for_reporting(config) {
        return Ok(ForegroundSnapshot::default());
    }

    get_foreground_snapshot_for_reporting(
        should_capture_process_name_for_reporting(config),
        should_capture_window_title_for_reporting(config),
    )
}

fn capture_media(config: &ClientConfig) -> MediaInfo {
    if should_capture_media_for_reporting(config) {
        get_now_playing().unwrap_or_else(|_| MediaInfo::default())
    } else {
        MediaInfo::default()
    }
}

fn capture_foreground_app_icon(config: &ClientConfig) -> Option<crate::platform::MediaArtwork> {
    if should_capture_foreground_app_icon_for_reporting(config) {
        get_foreground_app_icon().unwrap_or(None)
    } else {
        None
    }
}

fn should_use_media_timestamps(config: &ClientConfig, resolved: &ResolvedActivity) -> bool {
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

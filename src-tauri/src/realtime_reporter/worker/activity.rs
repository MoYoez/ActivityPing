use serde_json::{json, Value};

use crate::{
    models::{ClientConfig, ReporterActivity},
    platform::{ForegroundSnapshot, MediaInfo},
    rules::ResolvedActivity,
};

use super::super::logging::now_iso_string;

pub(super) fn build_reporter_activity(
    config: &ClientConfig,
    snapshot: &ForegroundSnapshot,
    resolved: &ResolvedActivity,
    media: &MediaInfo,
) -> ReporterActivity {
    let media_reportable = media.is_reportable(config.report_stopped_media);
    ReporterActivity {
        process_name: resolved.process_name.clone(),
        process_title: resolved.process_title.clone(),
        raw_process_title: non_empty_string(&snapshot.process_title),
        media_title: if media_reportable {
            non_empty_string(&media.title)
        } else {
            None
        },
        media_artist: if media_reportable {
            non_empty_string(&media.artist)
        } else {
            None
        },
        media_album: if media_reportable {
            non_empty_string(&media.album)
        } else {
            None
        },
        media_summary: resolved.media_summary.clone(),
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
        play_source: resolved.play_source.clone(),
        status_text: resolved.status_text.clone(),
        updated_at: Some(now_iso_string()),
    }
}

pub(super) fn build_report_log_payload(
    resolved: &ResolvedActivity,
    activity: &ReporterActivity,
) -> Value {
    json!({
        "summary": resolved.summary.clone(),
        "signature": resolved.signature.clone(),
        "activity": activity,
        "discord": {
            "details": resolved.discord_details.clone(),
            "state": resolved.discord_state.clone(),
        },
    })
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

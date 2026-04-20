use std::{
    sync::atomic::AtomicBool,
    sync::{Arc, Mutex},
};

use crate::{
    models::ClientConfig,
    platform::{
        get_foreground_snapshot_for_reporting, get_now_playing, ForegroundSnapshot, MediaInfo,
    },
    rules::{
        should_capture_foreground_snapshot_for_reporting, should_capture_media_for_reporting,
        should_capture_process_name_for_reporting, should_capture_window_title_for_reporting,
    },
};

use super::super::{
    logging::{fallback_text, push_background_log},
    state::ReporterInner,
};

pub(super) fn capture_foreground_snapshot(
    config: &ClientConfig,
) -> Result<ForegroundSnapshot, String> {
    if !should_capture_foreground_snapshot_for_reporting(config) {
        return Ok(ForegroundSnapshot::default());
    }

    get_foreground_snapshot_for_reporting(
        should_capture_process_name_for_reporting(config),
        should_capture_window_title_for_reporting(config),
    )
}

pub(super) fn capture_media(
    state: &Arc<Mutex<ReporterInner>>,
    config: &ClientConfig,
    sequence_seed: &mut u64,
    last_media_error: &mut Option<String>,
    _stop_flag: &Arc<AtomicBool>,
) -> MediaInfo {
    if !should_capture_media_for_reporting(config) {
        *last_media_error = None;
        return MediaInfo::default();
    }

    match get_now_playing() {
        Ok(media) => {
            *last_media_error = None;
            media
        }
        Err(error) => {
            let should_log = last_media_error
                .as_ref()
                .map(|last| last != &error)
                .unwrap_or(true);
            *last_media_error = Some(error.clone());
            if should_log {
                push_background_log(
                    state,
                    sequence_seed,
                    "warn",
                    fallback_text("Media capture failed"),
                    fallback_text(error),
                    None,
                );
            }
            MediaInfo::default()
        }
    }
}

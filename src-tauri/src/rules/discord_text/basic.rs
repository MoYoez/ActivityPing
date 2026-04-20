use crate::{models::ClientConfig, platform::MediaInfo};

use super::{first_non_empty, join_non_empty, non_empty};

pub(super) fn build_mixed_music_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<(String, Option<String>)> {
    if !config.report_media || media_hidden || !media.is_reportable(config.report_stopped_media) {
        return None;
    }

    let details = first_non_empty(&[media.title.as_str(), media.summary().as_str()])?;
    let state = join_non_empty(&[
        media.artist.as_str(),
        if config.report_window_title {
            process_title.unwrap_or("")
        } else {
            ""
        },
        if config.report_foreground_app {
            process_name
        } else {
            ""
        },
    ]);
    Some((details, state))
}

pub(super) fn build_music_discord_text(
    config: &ClientConfig,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<(String, Option<String>)> {
    if !config.report_media || media_hidden || !media.is_reportable(config.report_stopped_media) {
        return None;
    }

    let details = first_non_empty(&[media.title.as_str(), media.summary().as_str()])?;
    let state = non_empty(media.artist.as_str());
    Some((details, state))
}

pub(super) fn build_app_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    status_text: Option<&str>,
) -> Option<(String, Option<String>)> {
    if let Some(status_text) = status_text {
        let details = status_text.trim().to_string();
        if details.is_empty() {
            return None;
        }
        let state = if config.app_message_rules_show_process_name {
            non_empty(process_name)
        } else {
            None
        };
        return Some((details, state));
    }

    if let Some(process_title) = process_title {
        let details = process_title.trim().to_string();
        if details.is_empty() {
            return None;
        }
        let state = if config.report_foreground_app && !process_name.trim().is_empty() {
            Some(process_name.trim().to_string())
        } else {
            None
        };
        return Some((details, state));
    }

    if config.report_foreground_app && !process_name.trim().is_empty() {
        return Some((process_name.trim().to_string(), None));
    }

    None
}

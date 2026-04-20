use crate::{models::ClientConfig, platform::MediaInfo};

use super::{basic, non_empty};

pub(super) fn build_smart_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
    status_text: Option<&str>,
) -> Option<(String, Option<String>)> {
    if build_smart_media_text(config, media, media_hidden).is_some()
        && !has_reported_app_name(config, process_name, status_text)
    {
        return basic::build_music_discord_text(config, media, media_hidden);
    }

    if let Some(title_text) =
        build_smart_primary_title_text(config, process_name, process_title, status_text)
    {
        let media_text = build_smart_media_text(config, media, media_hidden);
        let app_text =
            build_smart_rule_hit_app_text(config, process_name, status_text, media_text.is_none());
        return Some(build_smart_text_layout(title_text, media_text, app_text));
    }

    if build_smart_media_text(config, media, media_hidden).is_some() {
        return basic::build_music_discord_text(config, media, media_hidden);
    }

    basic::build_app_discord_text(config, process_name, process_title, status_text)
}

fn build_smart_primary_title_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    status_text: Option<&str>,
) -> Option<String> {
    if let Some(status_text) = status_text.and_then(non_empty) {
        return Some(status_text);
    }

    if let Some(process_title) = process_title.and_then(non_empty) {
        return Some(process_title);
    }

    if config.report_foreground_app {
        return non_empty(process_name);
    }

    None
}

fn build_smart_rule_hit_app_text(
    config: &ClientConfig,
    process_name: &str,
    status_text: Option<&str>,
    no_media_visible: bool,
) -> Option<String> {
    if !config.discord_smart_show_app_name {
        if !no_media_visible
            || !config.app_message_rules_show_process_name
            || status_text.and_then(non_empty).is_none()
        {
            return None;
        }
    }

    non_empty(process_name)
}

fn build_smart_media_text(
    config: &ClientConfig,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<String> {
    if !config.report_media || media_hidden || !media.is_reportable(config.report_stopped_media) {
        return None;
    }

    non_empty(media.summary().as_str())
}

fn has_reported_app_name(
    config: &ClientConfig,
    process_name: &str,
    status_text: Option<&str>,
) -> bool {
    if non_empty(process_name).is_none() {
        return false;
    }

    config.report_foreground_app
        || config.discord_smart_show_app_name
        || status_text.and_then(non_empty).is_some()
}

fn build_smart_text_layout(
    title_text: String,
    media_text: Option<String>,
    app_text: Option<String>,
) -> (String, Option<String>) {
    match (media_text, app_text) {
        (Some(media_text), Some(app_text)) => {
            (format!("{title_text} | {app_text}"), Some(media_text))
        }
        (Some(media_text), None) => (title_text, Some(media_text)),
        (None, Some(app_text)) => (title_text, Some(app_text)),
        (None, None) => (title_text, None),
    }
}

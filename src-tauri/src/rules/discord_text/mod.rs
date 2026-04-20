mod basic;
mod smart;

use crate::{
    models::{ClientConfig, DiscordReportMode},
    platform::MediaInfo,
};

use super::{
    helpers::{first_non_empty, join_non_empty, non_empty},
    templates::{render_discord_template, DiscordTemplateValues},
};

pub(super) fn build_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
    status_text: Option<&str>,
) -> Option<(String, Option<String>)> {
    let (base_details, base_state) = match config.discord_report_mode {
        DiscordReportMode::Music => basic::build_music_discord_text(config, media, media_hidden),
        DiscordReportMode::App => {
            basic::build_app_discord_text(config, process_name, process_title, status_text)
        }
        DiscordReportMode::Mixed => smart::build_smart_discord_text(
            config,
            process_name,
            process_title,
            media,
            media_hidden,
            status_text,
        ),
        DiscordReportMode::Custom => build_custom_base_discord_text(
            config,
            process_name,
            process_title,
            media,
            media_hidden,
            status_text,
        ),
    }?;
    if config.discord_report_mode != DiscordReportMode::Custom {
        return Some((base_details, base_state));
    }
    let media_visible = media.is_reportable(config.report_stopped_media) && !media_hidden;
    let visible_source =
        if media_visible && config.report_play_source && !media.source_app_id.trim().is_empty() {
            Some(media.source_app_id.as_str())
        } else {
            None
        };
    let values = DiscordTemplateValues::new(
        &base_details,
        base_state.as_deref(),
        process_name,
        process_title,
        status_text,
        media,
        visible_source,
        media_visible,
    );
    let state = render_discord_template(&config.discord_state_format, &values)
        .filter(|value| !value.is_empty());
    let details = match render_discord_template(&config.discord_details_format, &values) {
        Some(value) if !value.is_empty() => value,
        Some(_) | None if config.discord_details_format.trim().is_empty() => String::new(),
        _ => base_details,
    };

    Some((details, state))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn build_music_discord_text(
    config: &ClientConfig,
    media: &MediaInfo,
    media_hidden: bool,
) -> Option<(String, Option<String>)> {
    basic::build_music_discord_text(config, media, media_hidden)
}

fn build_custom_base_discord_text(
    config: &ClientConfig,
    process_name: &str,
    process_title: Option<&str>,
    media: &MediaInfo,
    media_hidden: bool,
    status_text: Option<&str>,
) -> Option<(String, Option<String>)> {
    if let Some(app_text) =
        basic::build_app_discord_text(config, process_name, process_title, status_text)
    {
        if status_text.is_some() {
            return Some(app_text);
        }
    }

    basic::build_mixed_music_discord_text(config, process_name, process_title, media, media_hidden)
        .or_else(|| basic::build_app_discord_text(config, process_name, process_title, status_text))
}

mod naming;

use crate::{
    models::{ClientConfig, DiscordReportMode, DiscordStatusDisplay},
    platform::MediaInfo,
    rules::ResolvedActivity,
};

use self::naming::{build_custom_mode_activity_name, primary_app_activity_name};
use super::payload::{DiscordPresenceStatusDisplayType, DiscordPresenceText};

pub(super) fn build_presence_text(
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

pub(super) fn build_music_presence_text(
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

pub(super) fn build_music_only_activity_name(
    config: &ClientConfig,
    media: &MediaInfo,
) -> Option<String> {
    naming::build_music_only_activity_name(config, media)
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

pub(super) fn build_smart_presence_text(
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

pub(super) fn build_status_display_type(
    config: &ClientConfig,
) -> Option<DiscordPresenceStatusDisplayType> {
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

pub(super) fn non_empty_presence_value(value: &str) -> Option<String> {
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

use crate::{
    models::{ClientConfig, DiscordAppNameMode, DiscordReportMode, DiscordStatusDisplay},
    platform::MediaInfo,
    rules::ResolvedActivity,
};

use super::{assets::fallback_app_name, payload::{DiscordPresenceStatusDisplayType, DiscordPresenceText}};
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

pub(super) fn build_music_only_activity_name(config: &ClientConfig, media: &MediaInfo) -> Option<String> {
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

pub(super) fn build_status_display_type(config: &ClientConfig) -> Option<DiscordPresenceStatusDisplayType> {
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


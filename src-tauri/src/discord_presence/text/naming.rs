use crate::{
    models::{ClientConfig, DiscordAppNameMode, DiscordReportMode},
    platform::MediaInfo,
    rules::ResolvedActivity,
};

use super::non_empty_presence_value;
use crate::discord_presence::assets::fallback_app_name;

pub(super) fn build_music_only_activity_name(
    config: &ClientConfig,
    media: &MediaInfo,
) -> Option<String> {
    if !super::is_music_visible(config, media) {
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

pub(super) fn build_custom_mode_activity_name(
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

pub(super) fn primary_app_activity_name(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
) -> Option<String> {
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

fn current_media_source_name(media: &MediaInfo) -> Option<String> {
    non_empty_presence_value(media.source_app_id.as_str()).map(|value| fallback_app_name(&value))
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

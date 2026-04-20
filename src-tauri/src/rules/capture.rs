use crate::models::{
    AppFilterMode, ClientConfig, DiscordCustomAppIconSource, DiscordCustomArtworkSource,
    DiscordReportMode,
};

fn requires_process_name_for_filters(config: &ClientConfig) -> bool {
    matches!(config.app_filter_mode, AppFilterMode::Whitelist)
        || !config.app_blacklist.is_empty()
        || !config.app_whitelist.is_empty()
}

pub fn should_capture_process_name_for_reporting(config: &ClientConfig) -> bool {
    let filter_capture = requires_process_name_for_filters(config);

    match config.discord_report_mode {
        DiscordReportMode::Music => filter_capture,
        _ => {
            config.report_foreground_app
                || config.discord_smart_show_app_name
                || config.app_message_rules_show_process_name
                || !config.app_message_rules.is_empty()
                || !config.app_name_only_list.is_empty()
                || filter_capture
        }
    }
}

pub fn should_capture_window_title_for_reporting(config: &ClientConfig) -> bool {
    match config.discord_report_mode {
        DiscordReportMode::Music => false,
        _ => config.report_window_title,
    }
}

pub fn should_capture_media_for_reporting(config: &ClientConfig) -> bool {
    match config.discord_report_mode {
        DiscordReportMode::App => false,
        DiscordReportMode::Custom => {
            config.report_media
                || config.report_play_source
                || config.discord_custom_artwork_source == DiscordCustomArtworkSource::Music
                || config.discord_custom_app_icon_source == DiscordCustomAppIconSource::Source
        }
        _ => config.report_media || config.report_play_source,
    }
}

pub fn should_capture_foreground_app_icon_for_reporting(config: &ClientConfig) -> bool {
    (config.discord_use_app_artwork && config.discord_report_mode != DiscordReportMode::Music)
        || matches!(
            config.discord_report_mode,
            DiscordReportMode::Custom
        ) && matches!(
            config.discord_custom_artwork_source,
            DiscordCustomArtworkSource::App
        )
        || matches!(
            config.discord_report_mode,
            DiscordReportMode::Custom
        ) && matches!(
            config.discord_custom_app_icon_source,
            DiscordCustomAppIconSource::App
        )
}

pub fn should_capture_foreground_snapshot_for_reporting(config: &ClientConfig) -> bool {
    should_capture_process_name_for_reporting(config)
        || should_capture_window_title_for_reporting(config)
        || should_capture_foreground_app_icon_for_reporting(config)
}

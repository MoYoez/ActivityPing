use super::config::{
    DiscordActivityType, DiscordAppNameMode, DiscordAssetTextMode, DiscordCustomAppIconSource,
    DiscordCustomArtworkSource, DiscordSmartArtworkPreference, DiscordStatusDisplay,
};

pub fn default_poll_interval_ms() -> u64 {
    5_000
}

pub fn default_heartbeat_interval_ms() -> u64 {
    60_000
}

pub fn default_report_foreground_app() -> bool {
    true
}

pub fn default_report_window_title() -> bool {
    true
}

pub fn default_report_media() -> bool {
    true
}

pub fn default_report_stopped_media() -> bool {
    false
}

pub fn default_report_play_source() -> bool {
    true
}

pub fn default_discord_application_id() -> String {
    String::new()
}

pub fn default_discord_details_format() -> String {
    "{activity}".into()
}

pub fn default_discord_state_format() -> String {
    "{context}".into()
}

pub fn default_discord_activity_type() -> DiscordActivityType {
    DiscordActivityType::default()
}

pub fn default_discord_status_display() -> DiscordStatusDisplay {
    DiscordStatusDisplay::Name
}

pub fn default_discord_app_name_mode() -> DiscordAppNameMode {
    DiscordAppNameMode::Default
}

pub fn default_discord_music_app_name_mode() -> DiscordAppNameMode {
    DiscordAppNameMode::Source
}

pub fn default_discord_custom_app_name() -> String {
    String::new()
}

pub fn default_discord_smart_enable_music_countdown() -> bool {
    true
}

pub fn default_discord_smart_show_app_name() -> bool {
    false
}

pub fn default_discord_smart_artwork_preference() -> DiscordSmartArtworkPreference {
    DiscordSmartArtworkPreference::default()
}

pub fn default_discord_custom_artwork_source() -> DiscordCustomArtworkSource {
    DiscordCustomArtworkSource::default()
}

pub fn default_discord_custom_app_icon_source() -> DiscordCustomAppIconSource {
    DiscordCustomAppIconSource::default()
}

pub fn default_discord_asset_text_mode() -> DiscordAssetTextMode {
    DiscordAssetTextMode::default()
}

pub fn default_discord_use_app_artwork() -> bool {
    false
}

pub fn default_discord_use_music_artwork() -> bool {
    false
}

pub fn default_discord_artwork_worker_upload_url() -> String {
    String::new()
}

pub fn default_discord_artwork_worker_token() -> String {
    String::new()
}

pub fn default_discord_asset_text() -> String {
    String::new()
}

pub fn default_discord_custom_asset_id() -> String {
    String::new()
}

pub fn default_discord_use_custom_addons_override() -> bool {
    false
}

pub fn default_capture_reported_apps_enabled() -> bool {
    true
}

pub fn default_capture_history_record_limit() -> u32 {
    3
}

pub fn default_capture_history_title_limit() -> u32 {
    3
}

pub fn default_app_message_rules_show_process_name() -> bool {
    false
}

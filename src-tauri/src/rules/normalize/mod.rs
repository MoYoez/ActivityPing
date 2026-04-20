mod discord;
mod message_rules;
mod strings;

use crate::models::ClientConfig;

use self::{
    discord::{normalize_discord_custom_assets, normalize_discord_custom_presets},
    message_rules::normalize_rule_groups,
    strings::normalize_string_list,
};

pub fn normalize_client_config(config: &mut ClientConfig) {
    config.poll_interval_ms = config.poll_interval_ms.max(1_000);
    config.heartbeat_interval_ms = config.heartbeat_interval_ms.max(0);
    config.capture_history_record_limit = config.capture_history_record_limit.clamp(1, 50);
    config.capture_history_title_limit = config.capture_history_title_limit.clamp(1, 50);
    if config.capture_history_title_limit == 5 && config.capture_history_record_limit != 5 {
        config.capture_history_title_limit = config.capture_history_record_limit;
    }
    config.runtime_autostart_enabled = config.runtime_autostart_enabled
        || config.legacy_reporter_enabled
        || config.legacy_discord_enabled;
    config.legacy_reporter_enabled = false;
    config.legacy_discord_enabled = false;
    config.discord_use_app_artwork =
        config.discord_use_app_artwork || config.legacy_discord_use_media_artwork;
    config.discord_use_music_artwork =
        config.discord_use_music_artwork || config.legacy_discord_use_media_artwork;
    config.legacy_discord_use_media_artwork = false;
    config.discord_application_id = config.discord_application_id.trim().to_string();
    config.discord_artwork_worker_upload_url =
        config.discord_artwork_worker_upload_url.trim().to_string();
    config.discord_artwork_worker_token = config.discord_artwork_worker_token.trim().to_string();
    config.discord_custom_artwork_text = config.discord_custom_artwork_text.trim().to_string();
    config.discord_custom_artwork_asset_id =
        config.discord_custom_artwork_asset_id.trim().to_string();
    config.discord_custom_app_icon_text = config.discord_custom_app_icon_text.trim().to_string();
    config.discord_custom_app_icon_asset_id =
        config.discord_custom_app_icon_asset_id.trim().to_string();
    config.discord_custom_assets = normalize_discord_custom_assets(&config.discord_custom_assets);
    config.app_blacklist = normalize_string_list(&config.app_blacklist, false);
    config.app_whitelist = normalize_string_list(&config.app_whitelist, false);
    config.app_name_only_list = normalize_string_list(&config.app_name_only_list, false);
    config.media_play_source_blocklist =
        normalize_string_list(&config.media_play_source_blocklist, true);
    config.app_message_rules = normalize_rule_groups(&config.app_message_rules);
    if let Some(value) = config.legacy_discord_status_display.clone() {
        config.discord_smart_status_display = value.clone();
        config.discord_app_status_display = value.clone();
        config.discord_custom_mode_status_display = value.clone();
        config.discord_music_status_display = value;
    }
    if let Some(value) = config.legacy_discord_app_name_mode.clone() {
        config.discord_smart_app_name_mode = value.clone();
        config.discord_app_app_name_mode = value.clone();
        config.discord_custom_mode_app_name_mode = value.clone();
        config.discord_music_app_name_mode = value;
    }
    if let Some(value) = config.legacy_discord_custom_app_name.as_ref() {
        let trimmed = value.trim().to_string();
        config.discord_smart_custom_app_name = trimmed.clone();
        config.discord_app_custom_app_name = trimmed.clone();
        config.discord_custom_mode_custom_app_name = trimmed.clone();
        config.discord_music_custom_app_name = trimmed;
    }
    config.legacy_discord_status_display = None;
    config.legacy_discord_app_name_mode = None;
    config.legacy_discord_custom_app_name = None;
    config.discord_smart_custom_app_name = config.discord_smart_custom_app_name.trim().to_string();
    config.discord_music_custom_app_name = config.discord_music_custom_app_name.trim().to_string();
    config.discord_app_custom_app_name = config.discord_app_custom_app_name.trim().to_string();
    config.discord_custom_mode_custom_app_name = config
        .discord_custom_mode_custom_app_name
        .trim()
        .to_string();
    config.discord_custom_buttons =
        discord::normalize_discord_buttons(&config.discord_custom_buttons);
    config.discord_custom_party_id = config.discord_custom_party_id.trim().to_string();
    config.discord_custom_party_size_current =
        discord::normalize_party_size(config.discord_custom_party_size_current);
    config.discord_custom_party_size_max =
        discord::normalize_party_size(config.discord_custom_party_size_max);
    config.discord_custom_join_secret = config.discord_custom_join_secret.trim().to_string();
    config.discord_custom_spectate_secret =
        config.discord_custom_spectate_secret.trim().to_string();
    config.discord_custom_match_secret = config.discord_custom_match_secret.trim().to_string();
    config.discord_custom_presets =
        normalize_discord_custom_presets(&config.discord_custom_presets);
    config.discord_details_format =
        discord::normalize_discord_line_format(&config.discord_details_format);
    config.discord_state_format =
        discord::normalize_discord_line_format(&config.discord_state_format);
}

pub(crate) use discord::{normalize_discord_buttons, normalize_party_size};

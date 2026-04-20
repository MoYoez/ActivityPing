use serde::{Deserialize, Serialize};

use super::{
    super::defaults::{
        default_discord_activity_type, default_discord_app_name_mode, default_discord_asset_text,
        default_discord_asset_text_mode, default_discord_custom_app_icon_source,
        default_discord_custom_app_name, default_discord_custom_artwork_source,
        default_discord_custom_asset_id, default_discord_status_display,
    },
    enums::{
        AppTitleRuleMode, DiscordActivityType, DiscordAppNameMode, DiscordAssetTextMode,
        DiscordCustomAppIconSource, DiscordCustomArtworkSource, DiscordStatusDisplay,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscordCustomAsset {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub file_name: String,
    #[serde(default)]
    pub stored_path: String,
    #[serde(default)]
    pub content_type: String,
    #[serde(default)]
    pub byte_size: u64,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppMessageTitleRule {
    #[serde(default)]
    pub mode: AppTitleRuleMode,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub buttons: Vec<DiscordRichPresenceButtonConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppMessageRuleGroup {
    #[serde(default)]
    pub process_match: String,
    #[serde(default)]
    pub default_text: String,
    #[serde(default)]
    pub title_rules: Vec<AppMessageTitleRule>,
    #[serde(default)]
    pub buttons: Vec<DiscordRichPresenceButtonConfig>,
    #[serde(default)]
    pub party_id: String,
    #[serde(default)]
    pub party_size_current: Option<u32>,
    #[serde(default)]
    pub party_size_max: Option<u32>,
    #[serde(default)]
    pub join_secret: String,
    #[serde(default)]
    pub spectate_secret: String,
    #[serde(default)]
    pub match_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscordRichPresenceButtonConfig {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscordCustomPreset {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_discord_activity_type")]
    pub activity_type: DiscordActivityType,
    #[serde(default = "default_discord_status_display")]
    pub status_display: DiscordStatusDisplay,
    #[serde(default = "default_discord_app_name_mode")]
    pub app_name_mode: DiscordAppNameMode,
    #[serde(default = "default_discord_custom_app_name")]
    pub custom_app_name: String,
    #[serde(default)]
    pub details_format: String,
    #[serde(default)]
    pub state_format: String,
    #[serde(default = "default_discord_custom_artwork_source")]
    pub custom_artwork_source: DiscordCustomArtworkSource,
    #[serde(default = "default_discord_asset_text_mode")]
    pub custom_artwork_text_mode: DiscordAssetTextMode,
    #[serde(default = "default_discord_asset_text")]
    pub custom_artwork_text: String,
    #[serde(default = "default_discord_custom_asset_id")]
    pub custom_artwork_asset_id: String,
    #[serde(default = "default_discord_custom_app_icon_source")]
    pub custom_app_icon_source: DiscordCustomAppIconSource,
    #[serde(default = "default_discord_asset_text_mode")]
    pub custom_app_icon_text_mode: DiscordAssetTextMode,
    #[serde(default = "default_discord_asset_text")]
    pub custom_app_icon_text: String,
    #[serde(default = "default_discord_custom_asset_id")]
    pub custom_app_icon_asset_id: String,
    #[serde(default)]
    pub buttons: Vec<DiscordRichPresenceButtonConfig>,
    #[serde(default)]
    pub party_id: String,
    #[serde(default)]
    pub party_size_current: Option<u32>,
    #[serde(default)]
    pub party_size_max: Option<u32>,
    #[serde(default)]
    pub join_secret: String,
    #[serde(default)]
    pub spectate_secret: String,
    #[serde(default)]
    pub match_secret: String,
}

impl Default for DiscordCustomPreset {
    fn default() -> Self {
        Self {
            name: String::new(),
            activity_type: default_discord_activity_type(),
            status_display: default_discord_status_display(),
            app_name_mode: default_discord_app_name_mode(),
            custom_app_name: default_discord_custom_app_name(),
            details_format: String::new(),
            state_format: String::new(),
            custom_artwork_source: default_discord_custom_artwork_source(),
            custom_artwork_text_mode: default_discord_asset_text_mode(),
            custom_artwork_text: default_discord_asset_text(),
            custom_artwork_asset_id: default_discord_custom_asset_id(),
            custom_app_icon_source: default_discord_custom_app_icon_source(),
            custom_app_icon_text_mode: default_discord_asset_text_mode(),
            custom_app_icon_text: default_discord_asset_text(),
            custom_app_icon_asset_id: default_discord_custom_asset_id(),
            buttons: Vec::new(),
            party_id: String::new(),
            party_size_current: None,
            party_size_max: None,
            join_secret: String::new(),
            spectate_secret: String::new(),
            match_secret: String::new(),
        }
    }
}

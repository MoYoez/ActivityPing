use crate::{
    backend_locale::BackendLocale,
    models::{
        ClientConfig, DiscordCustomPreset, DiscordReportMode, DiscordRichPresenceButtonConfig,
    },
};

use super::{
    super::{ids::DISCORD_CUSTOM_LINE_CUSTOM_VALUE, notice::AppliedSelection, text::preset_label},
    target::TraySwitchTarget,
};

fn normalize_discord_line(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == DISCORD_CUSTOM_LINE_CUSTOM_VALUE {
        String::new()
    } else {
        trimmed.to_string()
    }
}

fn normalize_discord_buttons(
    buttons: &[DiscordRichPresenceButtonConfig],
) -> Vec<DiscordRichPresenceButtonConfig> {
    buttons
        .iter()
        .map(|button| DiscordRichPresenceButtonConfig {
            label: button.label.trim().to_string(),
            url: button.url.trim().to_string(),
        })
        .filter(|button| !button.label.is_empty() && !button.url.is_empty())
        .take(2)
        .collect()
}

fn normalize_party_size(value: Option<u32>) -> Option<u32> {
    value.filter(|size| *size > 0)
}

fn normalized_custom_preset(preset: &DiscordCustomPreset) -> DiscordCustomPreset {
    DiscordCustomPreset {
        name: preset.name.trim().to_string(),
        activity_type: preset.activity_type.clone(),
        status_display: preset.status_display.clone(),
        app_name_mode: preset.app_name_mode.clone(),
        custom_app_name: preset.custom_app_name.trim().to_string(),
        details_format: normalize_discord_line(&preset.details_format),
        state_format: normalize_discord_line(&preset.state_format),
        custom_artwork_source: preset.custom_artwork_source.clone(),
        custom_artwork_text_mode: preset.custom_artwork_text_mode.clone(),
        custom_artwork_text: preset.custom_artwork_text.trim().to_string(),
        custom_artwork_asset_id: preset.custom_artwork_asset_id.trim().to_string(),
        custom_app_icon_source: preset.custom_app_icon_source.clone(),
        custom_app_icon_text_mode: preset.custom_app_icon_text_mode.clone(),
        custom_app_icon_text: preset.custom_app_icon_text.trim().to_string(),
        custom_app_icon_asset_id: preset.custom_app_icon_asset_id.trim().to_string(),
        buttons: normalize_discord_buttons(&preset.buttons),
        party_id: preset.party_id.trim().to_string(),
        party_size_current: normalize_party_size(preset.party_size_current),
        party_size_max: normalize_party_size(preset.party_size_max),
        join_secret: preset.join_secret.trim().to_string(),
        spectate_secret: preset.spectate_secret.trim().to_string(),
        match_secret: preset.match_secret.trim().to_string(),
    }
}

fn apply_custom_preset_to_config(config: &mut ClientConfig, preset: &DiscordCustomPreset) {
    let preset = normalized_custom_preset(preset);
    config.discord_report_mode = DiscordReportMode::Custom;
    config.discord_activity_type = preset.activity_type;
    config.discord_custom_mode_status_display = preset.status_display;
    config.discord_custom_mode_app_name_mode = preset.app_name_mode;
    config.discord_custom_mode_custom_app_name = preset.custom_app_name;
    config.discord_custom_artwork_source = preset.custom_artwork_source;
    config.discord_custom_artwork_text_mode = preset.custom_artwork_text_mode;
    config.discord_custom_artwork_text = preset.custom_artwork_text;
    config.discord_custom_artwork_asset_id = preset.custom_artwork_asset_id;
    config.discord_custom_app_icon_source = preset.custom_app_icon_source;
    config.discord_custom_app_icon_text_mode = preset.custom_app_icon_text_mode;
    config.discord_custom_app_icon_text = preset.custom_app_icon_text;
    config.discord_custom_app_icon_asset_id = preset.custom_app_icon_asset_id;
    config.discord_details_format = preset.details_format;
    config.discord_state_format = preset.state_format;
    config.discord_custom_buttons = preset.buttons;
    config.discord_custom_party_id = preset.party_id;
    config.discord_custom_party_size_current = preset.party_size_current;
    config.discord_custom_party_size_max = preset.party_size_max;
    config.discord_custom_join_secret = preset.join_secret;
    config.discord_custom_spectate_secret = preset.spectate_secret;
    config.discord_custom_match_secret = preset.match_secret;
}

fn config_matches_custom_preset(config: &ClientConfig, preset: &DiscordCustomPreset) -> bool {
    let preset = normalized_custom_preset(preset);

    config.discord_activity_type == preset.activity_type
        && config.discord_custom_mode_status_display == preset.status_display
        && config.discord_custom_mode_app_name_mode == preset.app_name_mode
        && config.discord_custom_mode_custom_app_name.trim() == preset.custom_app_name
        && config.discord_custom_artwork_source == preset.custom_artwork_source
        && config.discord_custom_artwork_text_mode == preset.custom_artwork_text_mode
        && config.discord_custom_artwork_text.trim() == preset.custom_artwork_text
        && config.discord_custom_artwork_asset_id.trim() == preset.custom_artwork_asset_id
        && config.discord_custom_app_icon_source == preset.custom_app_icon_source
        && config.discord_custom_app_icon_text_mode == preset.custom_app_icon_text_mode
        && config.discord_custom_app_icon_text.trim() == preset.custom_app_icon_text
        && config.discord_custom_app_icon_asset_id.trim() == preset.custom_app_icon_asset_id
        && normalize_discord_line(&config.discord_details_format) == preset.details_format
        && normalize_discord_line(&config.discord_state_format) == preset.state_format
        && normalize_discord_buttons(&config.discord_custom_buttons) == preset.buttons
        && config.discord_custom_party_id.trim() == preset.party_id
        && normalize_party_size(config.discord_custom_party_size_current)
            == preset.party_size_current
        && normalize_party_size(config.discord_custom_party_size_max) == preset.party_size_max
        && config.discord_custom_join_secret.trim() == preset.join_secret
        && config.discord_custom_spectate_secret.trim() == preset.spectate_secret
        && config.discord_custom_match_secret.trim() == preset.match_secret
}

pub(super) fn active_custom_preset_index(config: &ClientConfig) -> Option<usize> {
    if config.discord_report_mode != DiscordReportMode::Custom {
        return None;
    }

    config
        .discord_custom_presets
        .iter()
        .position(|preset| config_matches_custom_preset(config, preset))
}

pub(super) fn apply_switch_target(
    config: &mut ClientConfig,
    target: &TraySwitchTarget,
    locale: BackendLocale,
) -> Result<AppliedSelection, String> {
    match target {
        TraySwitchTarget::Mode(mode) => {
            config.discord_report_mode = mode.clone();
            Ok(AppliedSelection::Mode(mode.clone()))
        }
        TraySwitchTarget::CustomCurrent => {
            config.discord_report_mode = DiscordReportMode::Custom;
            Ok(AppliedSelection::CustomCurrent)
        }
        TraySwitchTarget::CustomPreset(index) => {
            let preset = config
                .discord_custom_presets
                .get(*index)
                .cloned()
                .ok_or_else(|| {
                    if locale.is_en() {
                        format!("Custom preset {} was not found.", index + 1)
                    } else {
                        format!("找不到自定义预设 {}", index + 1)
                    }
                })?;
            let label = preset_label(locale, &preset, *index);
            apply_custom_preset_to_config(config, &preset);
            Ok(AppliedSelection::CustomPreset(label))
        }
    }
}

pub(super) fn configs_equal(left: &ClientConfig, right: &ClientConfig) -> bool {
    match (serde_json::to_value(left), serde_json::to_value(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

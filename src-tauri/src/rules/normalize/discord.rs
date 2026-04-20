use crate::models::{DiscordCustomAsset, DiscordCustomPreset, DiscordRichPresenceButtonConfig};

const DISCORD_CUSTOM_LINE_CUSTOM_VALUE: &str = "__custom__";

pub(crate) fn normalize_discord_buttons(
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

pub(super) fn normalize_discord_custom_assets(
    assets: &[DiscordCustomAsset],
) -> Vec<DiscordCustomAsset> {
    let mut normalized = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for asset in assets {
        let id = asset.id.trim().to_string();
        let name = asset.name.trim().to_string();
        let file_name = asset.file_name.trim().to_string();
        let stored_path = asset.stored_path.trim().to_string();
        let content_type = asset.content_type.trim().to_ascii_lowercase();
        let created_at = asset.created_at.trim().to_string();
        if id.is_empty()
            || name.is_empty()
            || file_name.is_empty()
            || stored_path.is_empty()
            || content_type.is_empty()
            || created_at.is_empty()
            || !seen.insert(id.clone())
        {
            continue;
        }

        normalized.push(DiscordCustomAsset {
            id,
            name,
            file_name,
            stored_path,
            content_type,
            byte_size: asset.byte_size,
            created_at,
        });
    }

    normalized
}

pub(super) fn normalize_discord_custom_presets(
    presets: &[DiscordCustomPreset],
) -> Vec<DiscordCustomPreset> {
    presets
        .iter()
        .map(|preset| DiscordCustomPreset {
            name: preset.name.trim().to_string(),
            activity_type: preset.activity_type.clone(),
            status_display: preset.status_display.clone(),
            app_name_mode: preset.app_name_mode.clone(),
            custom_app_name: preset.custom_app_name.trim().to_string(),
            details_format: normalize_discord_line_format(&preset.details_format),
            state_format: normalize_discord_line_format(&preset.state_format),
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
        })
        .collect()
}

pub(crate) fn normalize_party_size(value: Option<u32>) -> Option<u32> {
    value.filter(|size| *size > 0)
}

pub(crate) fn normalize_discord_line_format(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == DISCORD_CUSTOM_LINE_CUSTOM_VALUE {
        String::new()
    } else {
        trimmed.to_string()
    }
}

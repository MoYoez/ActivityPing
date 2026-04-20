use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::models::{ClientConfig, DiscordCustomAsset};

use super::super::payload::{
    DiscordPresenceArtwork, DiscordPresenceAssetKind, DiscordPresenceIcon,
};

pub(super) fn build_library_artwork(
    asset: &DiscordCustomAsset,
    hover_override: Option<String>,
) -> Option<DiscordPresenceArtwork> {
    let bytes = load_custom_asset_bytes(asset)?;
    let content_type = normalize_custom_asset_content_type(&asset.content_type)?;
    let cache_key = hash_cache_key(&[
        &bytes,
        content_type.as_bytes(),
        asset.stored_path.as_bytes(),
    ]);
    let asset_kind = if content_type == "image/png" {
        DiscordPresenceAssetKind::AppIcon
    } else {
        DiscordPresenceAssetKind::MusicArtwork
    };

    Some(DiscordPresenceArtwork {
        bytes,
        content_type,
        hover_text: hover_override.unwrap_or_else(|| asset.name.clone()),
        cache_key: format!("discord-library-artwork-{cache_key}"),
        asset_kind,
    })
}

pub(super) fn build_library_icon(
    asset: &DiscordCustomAsset,
    hover_override: Option<String>,
) -> Option<DiscordPresenceIcon> {
    let bytes = load_custom_asset_bytes(asset)?;
    let content_type = normalize_custom_asset_content_type(&asset.content_type)?;
    let cache_key = hash_cache_key(&[
        &bytes,
        content_type.as_bytes(),
        asset.stored_path.as_bytes(),
    ]);

    Some(DiscordPresenceIcon {
        bytes,
        content_type,
        hover_text: hover_override.unwrap_or_else(|| asset.name.clone()),
        cache_key: format!("discord-library-icon-{cache_key}"),
        asset_kind: DiscordPresenceAssetKind::AppIcon,
    })
}

pub(super) fn find_custom_asset<'a>(
    config: &'a ClientConfig,
    asset_id: &str,
) -> Option<&'a DiscordCustomAsset> {
    let trimmed = asset_id.trim();
    if trimmed.is_empty() {
        return None;
    }

    config
        .discord_custom_assets
        .iter()
        .find(|asset| asset.id == trimmed)
}

fn load_custom_asset_bytes(asset: &DiscordCustomAsset) -> Option<Vec<u8>> {
    let stored_path = asset.stored_path.trim();
    if stored_path.is_empty() {
        return None;
    }

    std::fs::read(stored_path)
        .ok()
        .filter(|bytes| !bytes.is_empty())
}

fn normalize_custom_asset_content_type(value: &str) -> Option<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "image/png" => Some("image/png".to_string()),
        "image/jpeg" | "image/jpg" => Some("image/jpeg".to_string()),
        _ => None,
    }
}

fn hash_cache_key(parts: &[&[u8]]) -> String {
    let mut hasher = DefaultHasher::new();
    for part in parts {
        part.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

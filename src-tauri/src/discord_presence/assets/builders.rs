use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::platform::{MediaArtwork, MediaInfo};

use super::super::payload::{
    DiscordPresenceArtwork, DiscordPresenceAssetKind, DiscordPresenceIcon,
};
use super::labels::fallback_app_name;

pub(super) fn build_music_artwork(
    media: &MediaInfo,
    hover_override: Option<String>,
) -> Option<DiscordPresenceArtwork> {
    let artwork = media.artwork.as_ref()?;
    if artwork.bytes.is_empty() || artwork.content_type.trim().is_empty() {
        return None;
    }

    let cache_key = hash_cache_key(&[
        &artwork.bytes,
        artwork.content_type.as_bytes(),
        media.source_app_id.as_bytes(),
    ]);
    let hover_text = hover_override.unwrap_or_else(|| {
        if media.album.trim().is_empty() {
            media.summary()
        } else {
            media.album.trim().to_string()
        }
    });

    Some(DiscordPresenceArtwork {
        bytes: artwork.bytes.clone(),
        content_type: artwork.content_type.clone(),
        hover_text,
        cache_key,
        asset_kind: DiscordPresenceAssetKind::MusicArtwork,
    })
}

pub(super) fn build_foreground_app_artwork(
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    hover_override: Option<String>,
) -> Option<DiscordPresenceArtwork> {
    let foreground_app_icon = foreground_app_icon?;
    if foreground_app_icon.bytes.is_empty() || foreground_app_icon.content_type.trim().is_empty() {
        return None;
    }

    let cache_key = hash_cache_key(&[
        &foreground_app_icon.bytes,
        foreground_app_icon.content_type.as_bytes(),
        process_name.as_bytes(),
    ]);
    let hover_text = hover_override.unwrap_or_else(|| {
        if process_name.trim().is_empty() {
            "Current app".to_string()
        } else {
            fallback_app_name(process_name)
        }
    });

    Some(DiscordPresenceArtwork {
        bytes: foreground_app_icon.bytes.clone(),
        content_type: foreground_app_icon.content_type.clone(),
        hover_text,
        cache_key: format!("discord-foreground-app-artwork-{cache_key}"),
        asset_kind: DiscordPresenceAssetKind::AppIcon,
    })
}

pub(super) fn build_foreground_app_icon(
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    hover_override: Option<String>,
) -> Option<DiscordPresenceIcon> {
    let foreground_app_icon = foreground_app_icon?;
    if foreground_app_icon.bytes.is_empty() || foreground_app_icon.content_type.trim().is_empty() {
        return None;
    }

    let cache_key = hash_cache_key(&[
        &foreground_app_icon.bytes,
        foreground_app_icon.content_type.as_bytes(),
        process_name.as_bytes(),
    ]);
    let hover_text = hover_override.unwrap_or_else(|| {
        if process_name.trim().is_empty() {
            "Current app".to_string()
        } else {
            fallback_app_name(process_name)
        }
    });

    Some(DiscordPresenceIcon {
        bytes: foreground_app_icon.bytes.clone(),
        content_type: foreground_app_icon.content_type.clone(),
        hover_text,
        cache_key: format!("discord-foreground-app-icon-{cache_key}"),
        asset_kind: DiscordPresenceAssetKind::AppIcon,
    })
}

pub(super) fn build_playback_source_icon(
    media: &MediaInfo,
    include_stopped: bool,
    hover_override: Option<String>,
) -> Option<DiscordPresenceIcon> {
    if !media.is_reportable(include_stopped) {
        return None;
    }

    let source_icon = media.source_icon.as_ref()?;
    if source_icon.bytes.is_empty() || source_icon.content_type.trim().is_empty() {
        return None;
    }

    let cache_key = hash_cache_key(&[
        &source_icon.bytes,
        source_icon.content_type.as_bytes(),
        media.source_app_id.as_bytes(),
    ]);
    let hover_text = hover_override.unwrap_or_else(|| fallback_app_name(&media.source_app_id));

    Some(DiscordPresenceIcon {
        bytes: source_icon.bytes.clone(),
        content_type: source_icon.content_type.clone(),
        hover_text,
        cache_key: format!("discord-source-icon-{cache_key}"),
        asset_kind: DiscordPresenceAssetKind::AppIcon,
    })
}

fn hash_cache_key(parts: &[&[u8]]) -> String {
    let mut hasher = DefaultHasher::new();
    for part in parts {
        part.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
}

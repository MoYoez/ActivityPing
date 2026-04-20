use crate::{
    models::{
        ClientConfig, DiscordAssetTextMode, DiscordCustomAppIconSource,
        DiscordCustomArtworkSource, DiscordCustomAsset, DiscordReportMode,
        DiscordSmartArtworkPreference,
    },
    platform::{display_name_for_app_id, MediaArtwork, MediaInfo},
};

use super::payload::{
    DiscordPresenceArtwork, DiscordPresenceAssetKind, DiscordPresenceIcon,
};

pub(super) fn build_presence_artwork(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceArtwork> {
    if config.discord_report_mode == DiscordReportMode::Custom {
        return build_custom_presence_artwork(
            config,
            process_name,
            foreground_app_icon,
            media,
        );
    }

    if !config.discord_use_music_artwork
        || !media.is_reportable(config.report_stopped_media)
        || config.discord_report_mode == DiscordReportMode::App
        || smart_mode_prefers_app_artwork(config)
    {
        return None;
    }

    build_music_artwork(media, None)
}

pub(super) fn build_presence_icon(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceIcon> {
    if config.discord_report_mode == DiscordReportMode::Custom {
        return build_custom_presence_icon(
            config,
            process_name,
            foreground_app_icon,
            media,
        );
    }

    if !config.discord_use_app_artwork && !config.discord_use_music_artwork {
        return None;
    }

    if config.discord_report_mode == DiscordReportMode::App {
        if config.discord_use_app_artwork {
            return build_foreground_app_icon(process_name, foreground_app_icon, None);
        }
        return None;
    }

    if smart_mode_prefers_app_artwork(config) {
        if config.discord_use_app_artwork {
            return build_foreground_app_icon(process_name, foreground_app_icon, None);
        }
        return None;
    }

    if media.is_reportable(config.report_stopped_media)
        && matches!(
            config.discord_report_mode,
            DiscordReportMode::Music | DiscordReportMode::Mixed
        )
    {
        if config.discord_use_music_artwork {
            return build_playback_source_icon(
                media,
                config.report_stopped_media,
                None,
            )
            .or_else(|| {
                if config.discord_use_app_artwork {
                    build_foreground_app_icon(process_name, foreground_app_icon, None)
                } else {
                    None
                }
            });
        }
    }

    if config.discord_use_app_artwork {
        return build_foreground_app_icon(process_name, foreground_app_icon, None)
            .or_else(|| {
                if config.discord_use_music_artwork {
                    build_playback_source_icon(
                        media,
                        config.report_stopped_media,
                        None,
                    )
                } else {
                    None
                }
            });
    }

    if config.discord_use_music_artwork {
        return build_playback_source_icon(media, config.report_stopped_media, None);
    }

    None
}

fn build_custom_presence_artwork(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceArtwork> {
    let hover_override = custom_hover_text(
        &config.discord_custom_artwork_text_mode,
        &config.discord_custom_artwork_text,
    );

    match config.discord_custom_artwork_source {
        DiscordCustomArtworkSource::Auto => {
            if !config.discord_use_music_artwork
                || !media.is_reportable(config.report_stopped_media)
            {
                return None;
            }
            build_music_artwork(media, hover_override)
        }
        DiscordCustomArtworkSource::None => None,
        DiscordCustomArtworkSource::Music => {
            if !media.is_reportable(config.report_stopped_media) {
                return None;
            }
            build_music_artwork(media, hover_override)
        }
        DiscordCustomArtworkSource::App => {
            build_foreground_app_artwork(
                process_name,
                foreground_app_icon,
                hover_override,
            )
        }
        DiscordCustomArtworkSource::Library => find_custom_asset(
            config,
            &config.discord_custom_artwork_asset_id,
        )
        .and_then(|asset| build_library_artwork(asset, hover_override)),
    }
}

fn build_custom_presence_icon(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceIcon> {
    let hover_override = custom_hover_text(
        &config.discord_custom_app_icon_text_mode,
        &config.discord_custom_app_icon_text,
    );

    match config.discord_custom_app_icon_source {
        DiscordCustomAppIconSource::Auto => {
            if config.discord_use_app_artwork {
                build_foreground_app_icon(
                    process_name,
                    foreground_app_icon,
                    hover_override,
                )
            } else {
                None
            }
        }
        DiscordCustomAppIconSource::None => None,
        DiscordCustomAppIconSource::App => build_foreground_app_icon(
            process_name,
            foreground_app_icon,
            hover_override,
        ),
        DiscordCustomAppIconSource::Source => build_playback_source_icon(
            media,
            config.report_stopped_media,
            hover_override,
        ),
        DiscordCustomAppIconSource::Library => find_custom_asset(
            config,
            &config.discord_custom_app_icon_asset_id,
        )
        .and_then(|asset| build_library_icon(asset, hover_override)),
    }
}

fn custom_hover_text(
    mode: &DiscordAssetTextMode,
    value: &str,
) -> Option<String> {
    match mode {
        DiscordAssetTextMode::Custom => Some(value.trim().to_string()),
        DiscordAssetTextMode::Auto => None,
    }
}

fn build_music_artwork(
    media: &MediaInfo,
    hover_override: Option<String>,
) -> Option<DiscordPresenceArtwork> {
    let artwork = media.artwork.as_ref()?;
    if artwork.bytes.is_empty() || artwork.content_type.trim().is_empty() {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&artwork.bytes, &mut hasher);
    std::hash::Hash::hash(&artwork.content_type, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
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

fn build_foreground_app_artwork(
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    hover_override: Option<String>,
) -> Option<DiscordPresenceArtwork> {
    let foreground_app_icon = foreground_app_icon?;
    if foreground_app_icon.bytes.is_empty()
        || foreground_app_icon.content_type.trim().is_empty()
    {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&foreground_app_icon.bytes, &mut hasher);
    std::hash::Hash::hash(&foreground_app_icon.content_type, &mut hasher);
    std::hash::Hash::hash(&process_name, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
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

fn build_foreground_app_icon(
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    hover_override: Option<String>,
) -> Option<DiscordPresenceIcon> {
    let foreground_app_icon = foreground_app_icon?;
    if foreground_app_icon.bytes.is_empty()
        || foreground_app_icon.content_type.trim().is_empty()
    {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&foreground_app_icon.bytes, &mut hasher);
    std::hash::Hash::hash(&foreground_app_icon.content_type, &mut hasher);
    std::hash::Hash::hash(&process_name, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
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

fn build_playback_source_icon(
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

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&source_icon.bytes, &mut hasher);
    std::hash::Hash::hash(&source_icon.content_type, &mut hasher);
    std::hash::Hash::hash(&media.source_app_id, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
    let hover_text = hover_override
        .unwrap_or_else(|| fallback_app_name(&media.source_app_id));

    Some(DiscordPresenceIcon {
        bytes: source_icon.bytes.clone(),
        content_type: source_icon.content_type.clone(),
        hover_text,
        cache_key: format!("discord-source-icon-{cache_key}"),
        asset_kind: DiscordPresenceAssetKind::AppIcon,
    })
}

fn build_library_artwork(
    asset: &DiscordCustomAsset,
    hover_override: Option<String>,
) -> Option<DiscordPresenceArtwork> {
    let bytes = load_custom_asset_bytes(asset)?;
    let content_type = normalize_custom_asset_content_type(&asset.content_type)?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&bytes, &mut hasher);
    std::hash::Hash::hash(&content_type, &mut hasher);
    std::hash::Hash::hash(&asset.stored_path, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
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

fn build_library_icon(
    asset: &DiscordCustomAsset,
    hover_override: Option<String>,
) -> Option<DiscordPresenceIcon> {
    let bytes = load_custom_asset_bytes(asset)?;
    let content_type = normalize_custom_asset_content_type(&asset.content_type)?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&bytes, &mut hasher);
    std::hash::Hash::hash(&content_type, &mut hasher);
    std::hash::Hash::hash(&asset.stored_path, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));

    Some(DiscordPresenceIcon {
        bytes,
        content_type,
        hover_text: hover_override.unwrap_or_else(|| asset.name.clone()),
        cache_key: format!("discord-library-icon-{cache_key}"),
        asset_kind: DiscordPresenceAssetKind::AppIcon,
    })
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

fn find_custom_asset<'a>(
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

fn smart_mode_prefers_app_artwork(config: &ClientConfig) -> bool {
    config.discord_report_mode == DiscordReportMode::Mixed
        && config.discord_smart_artwork_preference
            == DiscordSmartArtworkPreference::App
}

pub(super) fn fallback_app_name(source: &str) -> String {
    if let Some(display_name) = display_name_for_app_id(source) {
        return display_name;
    }

    let trimmed = source.trim();
    if trimmed.is_empty() {
        return "Playback app".to_string();
    }

    let tail = trimmed.rsplit(['\\', '/', '!']).next().unwrap_or(trimmed);
    let tail = tail
        .strip_suffix(".exe")
        .or_else(|| tail.strip_suffix(".app"))
        .or_else(|| tail.strip_suffix(".desktop"))
        .unwrap_or(tail);
    let tail = tail.split('_').next().unwrap_or(tail);

    if tail.contains('.') && !tail.contains(' ') {
        let bundle_tail = tail.rsplit('.').next().unwrap_or(tail);
        return title_case_words(bundle_tail);
    }

    title_case_words(tail)
}

fn title_case_words(value: &str) -> String {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.as_str().to_ascii_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

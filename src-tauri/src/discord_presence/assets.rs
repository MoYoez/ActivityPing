use crate::{
    models::{ClientConfig, DiscordReportMode},
    platform::{display_name_for_app_id, MediaArtwork, MediaInfo},
};

use super::payload::{DiscordPresenceArtwork, DiscordPresenceIcon};

pub(super) fn build_presence_artwork(
    config: &ClientConfig,
    media: &MediaInfo,
) -> Option<DiscordPresenceArtwork> {
    if !config.discord_use_music_artwork
        || !media.is_reportable(config.report_stopped_media)
        || config.discord_report_mode == DiscordReportMode::App
    {
        return None;
    }

    let artwork = media.artwork.as_ref()?;
    if artwork.bytes.is_empty() || artwork.content_type.trim().is_empty() {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&artwork.bytes, &mut hasher);
    std::hash::Hash::hash(&artwork.content_type, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
    let hover_text = if media.album.trim().is_empty() {
        media.summary()
    } else {
        media.album.trim().to_string()
    };

    Some(DiscordPresenceArtwork {
        bytes: artwork.bytes.clone(),
        content_type: artwork.content_type.clone(),
        hover_text,
        cache_key,
    })
}

pub(super) fn build_presence_icon(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceIcon> {
    if !config.discord_use_app_artwork && !config.discord_use_music_artwork {
        return None;
    }

    if config.discord_report_mode == DiscordReportMode::App {
        if config.discord_use_app_artwork {
            return build_foreground_app_icon(process_name, foreground_app_icon);
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
            return build_playback_source_icon(media, config.report_stopped_media).or_else(|| {
                if config.discord_use_app_artwork {
                    build_foreground_app_icon(process_name, foreground_app_icon)
                } else {
                    None
                }
            });
        }
    }

    if config.discord_use_app_artwork {
        return build_foreground_app_icon(process_name, foreground_app_icon).or_else(|| {
            if config.discord_use_music_artwork {
                build_playback_source_icon(media, config.report_stopped_media)
            } else {
                None
            }
        });
    }

    if config.discord_use_music_artwork {
        return build_playback_source_icon(media, config.report_stopped_media);
    }

    None
}

fn build_foreground_app_icon(
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
) -> Option<DiscordPresenceIcon> {
    let foreground_app_icon = foreground_app_icon?;
    if foreground_app_icon.bytes.is_empty() || foreground_app_icon.content_type.trim().is_empty() {
        return None;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&foreground_app_icon.bytes, &mut hasher);
    std::hash::Hash::hash(&foreground_app_icon.content_type, &mut hasher);
    std::hash::Hash::hash(&process_name, &mut hasher);
    let cache_key = format!("{:x}", std::hash::Hasher::finish(&hasher));
    let hover_text = if process_name.trim().is_empty() {
        "Current app".to_string()
    } else {
        fallback_app_name(process_name)
    };

    Some(DiscordPresenceIcon {
        bytes: foreground_app_icon.bytes.clone(),
        content_type: foreground_app_icon.content_type.clone(),
        hover_text,
        cache_key: format!("discord-foreground-app-icon-{cache_key}"),
    })
}

fn build_playback_source_icon(
    media: &MediaInfo,
    include_stopped: bool,
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
    let hover_text = fallback_app_name(&media.source_app_id);

    Some(DiscordPresenceIcon {
        bytes: source_icon.bytes.clone(),
        content_type: source_icon.content_type.clone(),
        hover_text,
        cache_key: format!("discord-source-icon-{cache_key}"),
    })
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

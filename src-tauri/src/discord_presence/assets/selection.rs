use crate::{
    models::{
        ClientConfig, DiscordCustomAppIconSource, DiscordCustomArtworkSource, DiscordReportMode,
    },
    platform::{MediaArtwork, MediaInfo},
};

use super::super::payload::{DiscordPresenceArtwork, DiscordPresenceIcon};
use super::{
    builders::{
        build_foreground_app_artwork, build_foreground_app_icon, build_music_artwork,
        build_playback_source_icon,
    },
    labels::{custom_hover_text, smart_mode_prefers_app_artwork},
    library::{build_library_artwork, build_library_icon, find_custom_asset},
};

pub(super) fn build_presence_artwork(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceArtwork> {
    if config.discord_report_mode == DiscordReportMode::Custom {
        return build_custom_presence_artwork(config, process_name, foreground_app_icon, media);
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
        return build_custom_presence_icon(config, process_name, foreground_app_icon, media);
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
        && config.discord_use_music_artwork
    {
        return build_playback_source_icon(media, config.report_stopped_media, None).or_else(
            || {
                if config.discord_use_app_artwork {
                    build_foreground_app_icon(process_name, foreground_app_icon, None)
                } else {
                    None
                }
            },
        );
    }

    if config.discord_use_app_artwork {
        return build_foreground_app_icon(process_name, foreground_app_icon, None).or_else(|| {
            if config.discord_use_music_artwork {
                build_playback_source_icon(media, config.report_stopped_media, None)
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
            build_foreground_app_artwork(process_name, foreground_app_icon, hover_override)
        }
        DiscordCustomArtworkSource::Library => {
            find_custom_asset(config, &config.discord_custom_artwork_asset_id)
                .and_then(|asset| build_library_artwork(asset, hover_override))
        }
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
                build_foreground_app_icon(process_name, foreground_app_icon, hover_override)
            } else {
                None
            }
        }
        DiscordCustomAppIconSource::None => None,
        DiscordCustomAppIconSource::App => {
            build_foreground_app_icon(process_name, foreground_app_icon, hover_override)
        }
        DiscordCustomAppIconSource::Source => {
            build_playback_source_icon(media, config.report_stopped_media, hover_override)
        }
        DiscordCustomAppIconSource::Library => {
            find_custom_asset(config, &config.discord_custom_app_icon_asset_id)
                .and_then(|asset| build_library_icon(asset, hover_override))
        }
    }
}

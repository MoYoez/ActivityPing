mod builders;
mod labels;
mod library;
mod selection;

use crate::{
    models::ClientConfig,
    platform::{MediaArtwork, MediaInfo},
};

use super::payload::{DiscordPresenceArtwork, DiscordPresenceIcon};

pub(super) fn build_presence_artwork(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceArtwork> {
    selection::build_presence_artwork(config, process_name, foreground_app_icon, media)
}

pub(super) fn build_presence_icon(
    config: &ClientConfig,
    process_name: &str,
    foreground_app_icon: Option<&MediaArtwork>,
    media: &MediaInfo,
) -> Option<DiscordPresenceIcon> {
    selection::build_presence_icon(config, process_name, foreground_app_icon, media)
}

pub(super) fn fallback_app_name(source: &str) -> String {
    labels::fallback_app_name(source)
}

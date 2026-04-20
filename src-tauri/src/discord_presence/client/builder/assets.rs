use discord_rich_presence::activity;

use crate::{
    artwork_server::{ArtworkPublisher, PublishAssetKind},
    discord_presence::payload::{DiscordPresenceAssetKind, DiscordPresencePayload},
};

#[derive(Default)]
pub(super) struct AssetPublishState {
    pub(super) artwork_url: Option<String>,
    pub(super) artwork_hover_text: Option<String>,
    pub(super) artwork_content_type: Option<String>,
    pub(super) artwork_upload_error: Option<String>,
    pub(super) app_icon_url: Option<String>,
    pub(super) app_icon_text: Option<String>,
    pub(super) app_icon_error: Option<String>,
}

pub(super) fn publish_assets(
    payload: &DiscordPresencePayload,
    artwork_publisher: Option<&ArtworkPublisher>,
) -> AssetPublishState {
    let mut state = AssetPublishState::default();

    if let (Some(artwork), Some(artwork_publisher)) = (payload.artwork.as_ref(), artwork_publisher)
    {
        let publish_kind = match artwork.asset_kind {
            DiscordPresenceAssetKind::AppIcon => PublishAssetKind::AppIcon,
            DiscordPresenceAssetKind::MusicArtwork => PublishAssetKind::MusicArtwork,
        };
        match artwork_publisher.publish(
            artwork.bytes.clone(),
            artwork.cache_key.clone(),
            publish_kind,
        ) {
            Ok(image_url) => {
                state.artwork_url = Some(image_url);
                state.artwork_hover_text =
                    (!artwork.hover_text.trim().is_empty()).then_some(artwork.hover_text.clone());
                state.artwork_content_type = Some(publish_kind.content_type().to_string());
            }
            Err(error) => state.artwork_upload_error = Some(error),
        }
    }

    if let (Some(icon), Some(artwork_publisher)) = (payload.icon.as_ref(), artwork_publisher) {
        let publish_kind = match icon.asset_kind {
            DiscordPresenceAssetKind::AppIcon => PublishAssetKind::AppIcon,
            DiscordPresenceAssetKind::MusicArtwork => PublishAssetKind::MusicArtwork,
        };
        match artwork_publisher.publish(icon.bytes.clone(), icon.cache_key.clone(), publish_kind) {
            Ok(image_url) => {
                state.app_icon_url = Some(image_url);
                state.app_icon_text =
                    (!icon.hover_text.trim().is_empty()).then_some(icon.hover_text.clone());
            }
            Err(error) => state.app_icon_error = Some(error),
        }
    }

    state
}

pub(super) fn build_activity_assets(
    state: &AssetPublishState,
) -> Option<activity::Assets<'static>> {
    let mut assets = activity::Assets::new();
    let mut has_assets = false;

    if let Some(image_url) = state.artwork_url.as_ref() {
        assets = assets.large_image(image_url.clone());
        has_assets = true;
        if let Some(hover_text) = state.artwork_hover_text.as_ref() {
            assets = assets.large_text(hover_text.clone());
        }
    }

    if let Some(image_url) = state.app_icon_url.as_ref() {
        if has_assets {
            assets = assets.small_image(image_url.clone());
            if let Some(icon_text) = state.app_icon_text.as_ref() {
                assets = assets.small_text(icon_text.clone());
            }
        } else {
            assets = assets.large_image(image_url.clone());
            has_assets = true;
            if let Some(icon_text) = state.app_icon_text.as_ref() {
                assets = assets.large_text(icon_text.clone());
            }
        }
    }

    has_assets.then_some(assets)
}

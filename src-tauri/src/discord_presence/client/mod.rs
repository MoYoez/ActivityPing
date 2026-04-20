mod builder;
mod connection;
mod mapping;

use discord_rich_presence::DiscordIpcClient;

use crate::{
    artwork_server::ArtworkPublisher,
    backend_locale::BackendLocale,
    models::{ClientConfig, DiscordDebugPayload},
};

use super::payload::DiscordPresencePayload;

pub(super) fn apply_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    config: &ClientConfig,
    application_id: &str,
    payload: &DiscordPresencePayload,
    artwork_publisher: Option<&ArtworkPublisher>,
    locale: BackendLocale,
) -> Result<DiscordDebugPayload, String> {
    builder::apply_discord_presence(
        client_slot,
        config,
        application_id,
        payload,
        artwork_publisher,
        locale,
    )
}

pub(super) fn clear_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    locale: BackendLocale,
) -> Result<(), String> {
    connection::clear_discord_presence(client_slot, application_id, locale)
}

pub(super) fn discord_config_app_id_missing(locale: BackendLocale) -> String {
    connection::discord_config_app_id_missing(locale)
}

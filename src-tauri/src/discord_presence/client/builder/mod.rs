mod assets;
mod debug_payload;

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

use crate::{
    artwork_server::ArtworkPublisher, backend_locale::BackendLocale, models::ClientConfig,
};

use super::{
    connection::{format_error, with_discord_client},
    mapping::{
        activity_type_key, effective_activity_type, map_status_display_type, report_mode_key,
    },
};
use crate::discord_presence::payload::DiscordPresencePayload;

pub(super) fn apply_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    config: &ClientConfig,
    application_id: &str,
    payload: &DiscordPresencePayload,
    artwork_publisher: Option<&ArtworkPublisher>,
    locale: BackendLocale,
) -> Result<crate::models::DiscordDebugPayload, String> {
    with_discord_client(client_slot, application_id, locale, |client| {
        let mut activity_payload =
            activity::Activity::new().activity_type(effective_activity_type(config));
        if let Some(name) = payload.activity_name.as_deref() {
            activity_payload = activity_payload.name(name.to_string());
        }
        if !payload.details.trim().is_empty() {
            activity_payload = activity_payload.details(payload.details.clone());
        }
        if let Some(status_display_type) = payload.status_display_type.as_ref() {
            activity_payload =
                activity_payload.status_display_type(map_status_display_type(status_display_type));
        }

        let asset_publish = assets::publish_assets(payload, artwork_publisher);
        if let Some(state) = payload.state.as_deref() {
            activity_payload = activity_payload.state(state.to_string());
        }
        if let Some(assets) = assets::build_activity_assets(&asset_publish) {
            activity_payload = activity_payload.assets(assets);
        }
        if !payload.buttons.is_empty() {
            activity_payload = activity_payload.buttons(
                payload
                    .buttons
                    .iter()
                    .map(|button| activity::Button::new(button.label.clone(), button.url.clone()))
                    .collect(),
            );
        }
        if let Some(party) = payload.party.as_ref() {
            let mut activity_party = activity::Party::new();
            if let Some(id) = party.id.as_deref() {
                activity_party = activity_party.id(id.to_string());
            }
            if let Some(size) = party.size {
                activity_party = activity_party.size(size);
            }
            activity_payload = activity_payload.party(activity_party);
        }
        if let Some(secrets) = payload.secrets.as_ref() {
            let mut activity_secrets = activity::Secrets::new();
            if let Some(join) = secrets.join.as_deref() {
                activity_secrets = activity_secrets.join(join.to_string());
            }
            if let Some(spectate) = secrets.spectate.as_deref() {
                activity_secrets = activity_secrets.spectate(spectate.to_string());
            }
            if let Some(match_secret) = secrets.match_secret.as_deref() {
                activity_secrets = activity_secrets.r#match(match_secret.to_string());
            }
            activity_payload = activity_payload.secrets(activity_secrets);
        }
        if let Some(timestamps) = build_activity_timestamps(payload) {
            activity_payload = activity_payload.timestamps(timestamps);
        }

        client
            .set_activity(activity_payload)
            .map_err(|error| format_error(locale, "Failed to update Discord presence", error))?;

        Ok(debug_payload::build_debug_payload(
            config,
            payload,
            asset_publish,
            report_mode_key(config),
            activity_type_key(config),
        ))
    })
}

fn build_activity_timestamps(payload: &DiscordPresencePayload) -> Option<activity::Timestamps> {
    let mut timestamps = activity::Timestamps::new();
    let mut has_timestamps = false;
    if let Some(started_at) = payload.started_at_millis {
        timestamps = timestamps.start(started_at);
        has_timestamps = true;
    }
    if let Some(ended_at) = payload.ended_at_millis {
        timestamps = timestamps.end(ended_at);
        has_timestamps = true;
    }
    has_timestamps.then_some(timestamps)
}

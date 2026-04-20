use crate::{
    discord_presence::payload::{DiscordPresencePayload, DiscordPresenceStatusDisplayType},
    models::{
        ClientConfig, DiscordDebugParty, DiscordDebugPayload, DiscordDebugSecrets,
        DiscordRichPresenceButtonConfig,
    },
};

use super::assets::AssetPublishState;

pub(super) fn build_debug_payload(
    config: &ClientConfig,
    payload: &DiscordPresencePayload,
    asset_publish: AssetPublishState,
    report_mode: &str,
    activity_type: &str,
) -> DiscordDebugPayload {
    let _ = config;

    DiscordDebugPayload {
        activity_name: payload.activity_name.clone(),
        details: payload.details.clone(),
        state: payload.state.clone(),
        summary: payload.summary.clone(),
        signature: payload.signature.clone(),
        report_mode_applied: report_mode.to_string(),
        activity_type: activity_type.to_string(),
        status_display_type: payload
            .status_display_type
            .as_ref()
            .map(|value| match value {
                DiscordPresenceStatusDisplayType::Name => "name".to_string(),
                DiscordPresenceStatusDisplayType::State => "state".to_string(),
                DiscordPresenceStatusDisplayType::Details => "details".to_string(),
            }),
        started_at_millis: payload.started_at_millis,
        ended_at_millis: payload.ended_at_millis,
        media_duration_ms: payload.media_duration_ms,
        media_position_ms: payload.media_position_ms,
        app_icon_url: asset_publish.app_icon_url,
        app_icon_text: asset_publish.app_icon_text,
        app_icon_error: asset_publish.app_icon_error,
        artwork_url: asset_publish.artwork_url,
        artwork_hover_text: asset_publish.artwork_hover_text,
        artwork_content_type: asset_publish.artwork_content_type,
        artwork_upload_error: asset_publish.artwork_upload_error,
        buttons: payload
            .buttons
            .iter()
            .map(|button| DiscordRichPresenceButtonConfig {
                label: button.label.clone(),
                url: button.url.clone(),
            })
            .collect(),
        party: payload.party.as_ref().map(|party| DiscordDebugParty {
            id: party.id.clone(),
            size: party.size,
        }),
        secrets: payload.secrets.as_ref().map(|secrets| DiscordDebugSecrets {
            join: secrets.join.clone(),
            spectate: secrets.spectate.clone(),
            match_secret: secrets.match_secret.clone(),
        }),
    }
}

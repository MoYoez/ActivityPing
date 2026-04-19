use crate::{
    models::{ClientConfig, DiscordReportMode},
    rules::{ResolvedActivity, ResolvedDiscordAddons, ResolvedDiscordParty, ResolvedDiscordSecrets},
};

use super::{
    payload::{DiscordPresenceButton, DiscordPresenceParty, DiscordPresenceSecrets},
    text::non_empty_presence_value,
};
fn has_custom_addons(config: &ClientConfig) -> bool {
    !config.discord_custom_buttons.is_empty()
        || !config.discord_custom_party_id.trim().is_empty()
        || config.discord_custom_party_size_current.is_some()
        || config.discord_custom_party_size_max.is_some()
        || !config.discord_custom_join_secret.trim().is_empty()
        || !config.discord_custom_spectate_secret.trim().is_empty()
        || !config.discord_custom_match_secret.trim().is_empty()
}

fn build_custom_addons(config: &ClientConfig) -> ResolvedDiscordAddons {
    let buttons = config
        .discord_custom_buttons
        .iter()
        .map(|button| DiscordPresenceButton {
            label: button.label.trim().to_string(),
            url: button.url.trim().to_string(),
        })
        .filter(|button| !button.label.is_empty() && !button.url.is_empty())
        .take(2)
        .map(|button| crate::models::DiscordRichPresenceButtonConfig {
            label: button.label,
            url: button.url,
        })
        .collect();
    let id = non_empty_presence_value(config.discord_custom_party_id.as_str());
    let size = match (
        config.discord_custom_party_size_current,
        config.discord_custom_party_size_max,
    ) {
        (Some(current), Some(maximum)) if current > 0 && maximum > 0 && current <= maximum => {
            Some((current, maximum))
        }
        _ => None,
    };
    let join = non_empty_presence_value(config.discord_custom_join_secret.as_str());
    let spectate = non_empty_presence_value(config.discord_custom_spectate_secret.as_str());
    let match_secret = non_empty_presence_value(config.discord_custom_match_secret.as_str());

    ResolvedDiscordAddons {
        buttons,
        party: if id.is_none() && size.is_none() {
            None
        } else {
            Some(ResolvedDiscordParty { id, size })
        },
        secrets: if join.is_none() && spectate.is_none() && match_secret.is_none() {
            None
        } else {
            Some(ResolvedDiscordSecrets {
                join,
                spectate,
                match_secret,
            })
        },
    }
}

pub(super) fn select_presence_addons(
    config: &ClientConfig,
    resolved: &ResolvedActivity,
) -> ResolvedDiscordAddons {
    let custom_addons = build_custom_addons(config);
    let custom_addons_configured = has_custom_addons(config) && !custom_addons.is_empty();

    if config.discord_report_mode == DiscordReportMode::Custom {
        if custom_addons_configured {
            return custom_addons;
        }
        return resolved.discord_addons.clone();
    }

    if config.discord_use_custom_addons_override && custom_addons_configured {
        return custom_addons;
    }

    resolved.discord_addons.clone()
}

pub(super) fn build_presence_buttons(addons: &ResolvedDiscordAddons) -> Vec<DiscordPresenceButton> {
    addons
        .buttons
        .iter()
        .map(|button| DiscordPresenceButton {
            label: button.label.trim().to_string(),
            url: button.url.trim().to_string(),
        })
        .filter(|button| !button.label.is_empty() && !button.url.is_empty())
        .take(2)
        .collect()
}

pub(super) fn build_presence_party(addons: &ResolvedDiscordAddons) -> Option<DiscordPresenceParty> {
    let party = addons.party.as_ref()?;
    let size = match party.size {
        Some((current, maximum)) if current > 0 && maximum > 0 && current <= maximum => {
            let current = i32::try_from(current).ok()?;
            let maximum = i32::try_from(maximum).ok()?;
            Some([current, maximum])
        }
        _ => None,
    };

    if party.id.is_none() && size.is_none() {
        return None;
    }

    Some(DiscordPresenceParty {
        id: party.id.clone(),
        size,
    })
}

pub(super) fn build_presence_secrets(addons: &ResolvedDiscordAddons) -> Option<DiscordPresenceSecrets> {
    let secrets = addons.secrets.as_ref()?;
    if secrets.join.is_none() && secrets.spectate.is_none() && secrets.match_secret.is_none() {
        return None;
    }

    Some(DiscordPresenceSecrets {
        join: secrets.join.clone(),
        spectate: secrets.spectate.clone(),
        match_secret: secrets.match_secret.clone(),
    })
}


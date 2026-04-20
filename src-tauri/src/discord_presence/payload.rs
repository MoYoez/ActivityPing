#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscordPresencePayload {
    pub(super) activity_name: Option<String>,
    pub(super) details: String,
    pub(super) state: Option<String>,
    pub(super) status_display_type: Option<DiscordPresenceStatusDisplayType>,
    pub(super) started_at_millis: Option<i64>,
    pub(super) ended_at_millis: Option<i64>,
    pub(super) media_duration_ms: Option<u64>,
    pub(super) media_position_ms: Option<u64>,
    pub(super) media_is_playing: bool,
    pub(super) summary: String,
    pub(super) signature: String,
    pub(super) artwork: Option<DiscordPresenceArtwork>,
    pub(super) icon: Option<DiscordPresenceIcon>,
    pub(super) buttons: Vec<DiscordPresenceButton>,
    pub(super) party: Option<DiscordPresenceParty>,
    pub(super) secrets: Option<DiscordPresenceSecrets>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscordPresenceArtwork {
    pub(super) bytes: Vec<u8>,
    pub(super) content_type: String,
    pub(super) hover_text: String,
    pub(super) cache_key: String,
    pub(super) asset_kind: DiscordPresenceAssetKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DiscordPresenceStatusDisplayType {
    Name,
    State,
    Details,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscordPresenceIcon {
    pub(super) bytes: Vec<u8>,
    pub(super) content_type: String,
    pub(super) hover_text: String,
    pub(super) cache_key: String,
    pub(super) asset_kind: DiscordPresenceAssetKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DiscordPresenceAssetKind {
    AppIcon,
    MusicArtwork,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscordPresenceButton {
    pub(super) label: String,
    pub(super) url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscordPresenceParty {
    pub(super) id: Option<String>,
    pub(super) size: Option<[i32; 2]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscordPresenceSecrets {
    pub(super) join: Option<String>,
    pub(super) spectate: Option<String>,
    pub(super) match_secret: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DiscordPresenceText {
    pub(super) activity_name: Option<String>,
    pub(super) details: String,
    pub(super) state: Option<String>,
    pub(super) summary: String,
    pub(super) signature: String,
}

impl DiscordPresencePayload {
    pub(super) fn publish_signature(&self) -> String {
        let artwork_key = self
            .artwork
            .as_ref()
            .map(|artwork| artwork.cache_key.as_str())
            .unwrap_or("");
        let icon_key = self
            .icon
            .as_ref()
            .map(|icon| icon.cache_key.as_str())
            .unwrap_or("");
        let stable_position_ms = if self.media_is_playing {
            None
        } else {
            self.media_position_ms
        };
        let button_key = self
            .buttons
            .iter()
            .map(|button| format!("{}={}", button.label, button.url))
            .collect::<Vec<_>>()
            .join("|");
        let party_key = self.party.as_ref().map_or_else(String::new, |party| {
            format!("{}:{:?}", party.id.as_deref().unwrap_or(""), party.size)
        });
        let secrets_key = self.secrets.as_ref().map_or_else(String::new, |secrets| {
            format!(
                "{}|{}|{}",
                secrets.join.as_deref().unwrap_or(""),
                secrets.spectate.as_deref().unwrap_or(""),
                secrets.match_secret.as_deref().unwrap_or("")
            )
        });
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{:?}\n{:?}\n{}",
            self.signature,
            self.activity_name.as_deref().unwrap_or(""),
            self.details,
            self.state.as_deref().unwrap_or(""),
            match self.status_display_type {
                Some(DiscordPresenceStatusDisplayType::Name) => "name",
                Some(DiscordPresenceStatusDisplayType::State) => "state",
                Some(DiscordPresenceStatusDisplayType::Details) => "details",
                None => "",
            },
            artwork_key,
            icon_key,
            button_key,
            party_key,
            secrets_key,
            self.media_duration_ms,
            stable_position_ms,
            self.media_is_playing
        )
    }
}

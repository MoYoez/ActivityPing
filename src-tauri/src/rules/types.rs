use crate::models::DiscordRichPresenceButtonConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedActivity {
    pub process_name: String,
    pub process_title: Option<String>,
    pub media_summary: Option<String>,
    pub play_source: Option<String>,
    pub status_text: Option<String>,
    pub discord_addons: ResolvedDiscordAddons,
    pub discord_details: String,
    pub discord_state: Option<String>,
    pub summary: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ResolvedDiscordAddons {
    pub buttons: Vec<DiscordRichPresenceButtonConfig>,
    pub party: Option<ResolvedDiscordParty>,
    pub secrets: Option<ResolvedDiscordSecrets>,
}

impl ResolvedDiscordAddons {
    pub fn is_empty(&self) -> bool {
        self.buttons.is_empty() && self.party.is_none() && self.secrets.is_none()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDiscordParty {
    pub id: Option<String>,
    pub size: Option<(u32, u32)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDiscordSecrets {
    pub join: Option<String>,
    pub spectate: Option<String>,
    pub match_secret: Option<String>,
}

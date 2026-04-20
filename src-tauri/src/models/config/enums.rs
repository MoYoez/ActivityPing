use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppTitleRuleMode {
    #[default]
    Plain,
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppFilterMode {
    #[default]
    Blacklist,
    Whitelist,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiscordReportMode {
    Music,
    App,
    #[default]
    Mixed,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiscordActivityType {
    #[default]
    Playing,
    Listening,
    Watching,
    Competing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiscordStatusDisplay {
    Name,
    State,
    Details,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiscordAppNameMode {
    Default,
    Song,
    Artist,
    Album,
    #[serde(rename = "source", alias = "media_source")]
    Source,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiscordSmartArtworkPreference {
    #[default]
    Music,
    App,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiscordCustomArtworkSource {
    #[default]
    Auto,
    None,
    Music,
    App,
    Library,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiscordCustomAppIconSource {
    #[default]
    Auto,
    None,
    App,
    Source,
    Library,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiscordAssetTextMode {
    #[default]
    Auto,
    Custom,
}

impl Default for DiscordStatusDisplay {
    fn default() -> Self {
        Self::Name
    }
}

impl Default for DiscordAppNameMode {
    fn default() -> Self {
        Self::Default
    }
}

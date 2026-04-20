mod capabilities;
mod client;
mod discord;
mod enums;

pub use capabilities::{default_client_capabilities, ClientCapabilities};
pub use client::ClientConfig;
pub use discord::{
    AppMessageRuleGroup, AppMessageTitleRule, DiscordCustomAsset, DiscordCustomPreset,
    DiscordRichPresenceButtonConfig,
};
pub use enums::{
    AppFilterMode, AppTitleRuleMode, DiscordActivityType, DiscordAppNameMode, DiscordAssetTextMode,
    DiscordCustomAppIconSource, DiscordCustomArtworkSource, DiscordReportMode,
    DiscordSmartArtworkPreference, DiscordStatusDisplay,
};

mod capture;
mod discord_text;
mod filters;
mod helpers;
mod message_rules;
mod normalize;
mod resolve;
mod templates;
mod types;

pub use capture::{
    should_capture_foreground_app_icon_for_reporting,
    should_capture_foreground_snapshot_for_reporting, should_capture_media_for_reporting,
    should_capture_process_name_for_reporting, should_capture_window_title_for_reporting,
};
pub use normalize::normalize_client_config;
pub use resolve::resolve_activity;
pub use types::{
    ResolvedActivity, ResolvedDiscordAddons, ResolvedDiscordParty, ResolvedDiscordSecrets,
};

#[cfg(test)]
#[allow(unused_imports)]
use self::discord_text::{build_discord_text, build_music_discord_text};
#[cfg(test)]
#[allow(unused_imports)]
use crate::{
    models::{ClientConfig, DiscordReportMode},
    platform::{ForegroundSnapshot, MediaInfo},
};

#[cfg(test)]
mod tests;

use discord_rich_presence::activity;

use crate::{
    discord_presence::payload::DiscordPresenceStatusDisplayType,
    models::{ClientConfig, DiscordActivityType, DiscordReportMode},
};

pub(super) fn map_activity_type(value: &DiscordActivityType) -> activity::ActivityType {
    match value {
        DiscordActivityType::Listening => activity::ActivityType::Listening,
        DiscordActivityType::Watching => activity::ActivityType::Watching,
        DiscordActivityType::Competing => activity::ActivityType::Competing,
        DiscordActivityType::Playing => activity::ActivityType::Playing,
    }
}

pub(super) fn map_status_display_type(
    value: &DiscordPresenceStatusDisplayType,
) -> activity::StatusDisplayType {
    match value {
        DiscordPresenceStatusDisplayType::Name => activity::StatusDisplayType::Name,
        DiscordPresenceStatusDisplayType::State => activity::StatusDisplayType::State,
        DiscordPresenceStatusDisplayType::Details => activity::StatusDisplayType::Details,
    }
}

pub(super) fn effective_activity_type(config: &ClientConfig) -> activity::ActivityType {
    match config.discord_report_mode {
        DiscordReportMode::Music => activity::ActivityType::Listening,
        DiscordReportMode::Custom => map_activity_type(&config.discord_activity_type),
        DiscordReportMode::App | DiscordReportMode::Mixed => activity::ActivityType::Playing,
    }
}

pub(super) fn report_mode_key(config: &ClientConfig) -> &'static str {
    match config.discord_report_mode {
        DiscordReportMode::Music => "music",
        DiscordReportMode::App => "app",
        DiscordReportMode::Mixed => "mixed",
        DiscordReportMode::Custom => "custom",
    }
}

pub(super) fn activity_type_key(config: &ClientConfig) -> &'static str {
    match config.discord_report_mode {
        DiscordReportMode::Music => "listening",
        DiscordReportMode::App | DiscordReportMode::Mixed => "playing",
        DiscordReportMode::Custom => match &config.discord_activity_type {
            DiscordActivityType::Listening => "listening",
            DiscordActivityType::Watching => "watching",
            DiscordActivityType::Competing => "competing",
            DiscordActivityType::Playing => "playing",
        },
    }
}

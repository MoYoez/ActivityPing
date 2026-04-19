use super::super::*;
use crate::{models::DiscordReportMode, platform::MediaInfo};

#[test]
fn custom_mode_applies_global_details_and_state_templates() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Custom,
        discord_details_format: "{app} :: {activity}".into(),
        discord_state_format: "Line 3: {context}".into(),
        ..ClientConfig::default()
    };

    let resolved = build_discord_text(
        &config,
        "Code.exe",
        Some("repo"),
        &MediaInfo::default(),
        false,
        None,
    )
    .expect("activity");

    assert_eq!(resolved.0, "Code.exe :: repo".to_string());
    assert_eq!(resolved.1, Some("Line 3: Code.exe".to_string()));
}

#[test]
fn custom_mode_can_hide_details_line() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Custom,
        discord_details_format: String::new(),
        discord_state_format: "{context}".into(),
        ..ClientConfig::default()
    };

    let resolved = build_discord_text(
        &config,
        "Code.exe",
        Some("repo"),
        &MediaInfo::default(),
        false,
        None,
    )
    .expect("activity");

    assert_eq!(resolved.0, String::new());
    assert_eq!(resolved.1, Some("Code.exe".to_string()));
}

#[test]
fn custom_mode_can_use_literal_custom_text() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Custom,
        discord_details_format: "Coding in {app}".into(),
        discord_state_format: "Working on {title}".into(),
        ..ClientConfig::default()
    };

    let resolved = build_discord_text(
        &config,
        "Code.exe",
        Some("repo"),
        &MediaInfo::default(),
        false,
        None,
    )
    .expect("activity");

    assert_eq!(resolved.0, "Coding in Code.exe".to_string());
    assert_eq!(resolved.1, Some("Working on repo".to_string()));
}

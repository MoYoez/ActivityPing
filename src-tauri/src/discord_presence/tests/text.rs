use super::{
    super::*,
    fixtures::{sample_media, sample_resolved},
};
use crate::platform::MediaInfo;

#[test]
fn music_mode_can_override_activity_name_with_custom_text() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        discord_music_app_name_mode: crate::models::DiscordAppNameMode::Custom,
        discord_music_custom_app_name: "My Custom App".into(),
        ..ClientConfig::default()
    };
    let media = sample_media();

    let value = build_music_only_activity_name(&config, &media);

    assert_eq!(value, Some("My Custom App".to_string()));
}

#[test]
fn music_mode_can_use_media_source_for_activity_name() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        discord_music_app_name_mode: crate::models::DiscordAppNameMode::Source,
        ..ClientConfig::default()
    };
    let mut media = sample_media();
    media.source_app_id = "spotify.exe".into();

    let value = build_music_only_activity_name(&config, &media);

    assert_eq!(value, Some("Spotify".to_string()));
}

#[test]
fn smart_mode_uses_rule_hit_as_name_and_app_as_details() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_smart_show_app_name: true,
        ..ClientConfig::default()
    };
    let resolved = sample_resolved();
    let media = MediaInfo::default();

    let text = build_smart_presence_text(&config, &resolved, &media).expect("text");

    assert_eq!(text.activity_name, Some("Matched Title".to_string()));
    assert_eq!(text.details, "Code.exe".to_string());
    assert_eq!(text.state, None);
}

#[test]
fn smart_mode_puts_music_on_state_line() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_smart_show_app_name: true,
        ..ClientConfig::default()
    };
    let resolved = sample_resolved();
    let media = sample_media();

    let text = build_smart_presence_text(&config, &resolved, &media).expect("text");

    assert_eq!(text.activity_name, Some("Matched Title".to_string()));
    assert_eq!(text.details, "Code.exe".to_string());
    assert_eq!(
        text.state,
        Some("🎵 Track Name / Artist Name / Album Name".to_string())
    );
}

#[test]
fn competing_mode_can_set_status_display_to_state() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Custom,
        discord_activity_type: crate::models::DiscordActivityType::Competing,
        discord_custom_mode_status_display: crate::models::DiscordStatusDisplay::State,
        ..ClientConfig::default()
    };

    let value = build_status_display_type(&config);

    assert_eq!(value, Some(DiscordPresenceStatusDisplayType::State));
}

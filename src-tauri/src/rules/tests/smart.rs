use super::{
    super::*,
    fixtures::{code_snapshot, sample_media},
};
use crate::{models::DiscordReportMode, platform::MediaInfo};

#[test]
fn app_mode_shows_process_name_on_state_when_rule_hits() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::App,
        app_message_rules_show_process_name: true,
        ..ClientConfig::default()
    };

    let resolved = build_discord_text(
        &config,
        "Spotify.exe",
        None,
        &MediaInfo::default(),
        false,
        Some("正在 声破天 听歌中"),
    )
    .expect("activity");

    assert_eq!(resolved.0, "正在 声破天 听歌中".to_string());
    assert_eq!(resolved.1, Some("Spotify.exe".to_string()));
}

#[test]
fn smart_mode_uses_title_then_visible_media_and_keeps_app_in_signature() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        ..ClientConfig::default()
    };
    let snapshot = code_snapshot();
    let media = sample_media();

    let resolved = resolve_activity(&config, &snapshot, &media).expect("activity");

    assert_eq!(resolved.discord_details, "repo");
    assert_eq!(
        resolved.discord_state,
        Some("Track Name / Artist Name / Album Name".to_string())
    );
    assert_eq!(
        resolved.summary,
        "repo · Track Name / Artist Name / Album Name".to_string()
    );
    assert_eq!(
        resolved.signature,
        "repo · Track Name / Artist Name / Album Name · Code.exe".to_string()
    );
}

#[test]
fn smart_mode_appends_app_name_to_last_line_only_on_rule_hit() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_smart_show_app_name: true,
        ..ClientConfig::default()
    };
    let snapshot = code_snapshot();
    let media = sample_media();

    let resolved = build_discord_text(
        &config,
        snapshot.process_name.as_str(),
        None,
        &media,
        false,
        Some("Coding"),
    )
    .expect("activity");

    assert_eq!(resolved.0, "Coding | Code.exe".to_string());
    assert_eq!(
        resolved.1,
        Some("Track Name / Artist Name / Album Name".to_string())
    );
}

#[test]
fn smart_mode_keeps_last_line_empty_without_music_or_rule_app_name() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        ..ClientConfig::default()
    };
    let snapshot = code_snapshot();
    let media = MediaInfo::default();

    let resolved = resolve_activity(&config, &snapshot, &media).expect("activity");

    assert_eq!(resolved.discord_details, "repo");
    assert_eq!(resolved.discord_state, None);
}

#[test]
fn smart_mode_falls_back_to_music_only_when_app_name_is_not_reported() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        report_foreground_app: false,
        ..ClientConfig::default()
    };
    let snapshot = code_snapshot();
    let media = sample_media();

    let resolved = resolve_activity(&config, &snapshot, &media).expect("activity");

    assert_eq!(resolved.discord_details, "Track Name");
    assert_eq!(resolved.discord_state, Some("Artist Name".to_string()));
}

#[test]
fn smart_mode_can_show_rule_hit_app_name_even_when_global_app_reporting_is_off() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        report_foreground_app: false,
        discord_smart_show_app_name: true,
        ..ClientConfig::default()
    };
    let media = sample_media();

    let resolved = build_discord_text(&config, "Code.exe", None, &media, false, Some("Coding"))
        .expect("activity");

    assert_eq!(resolved.0, "Coding | Code.exe".to_string());
    assert_eq!(
        resolved.1,
        Some("Track Name / Artist Name / Album Name".to_string())
    );
}

#[test]
fn smart_mode_can_show_app_name_without_music() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_smart_show_app_name: true,
        ..ClientConfig::default()
    };
    let snapshot = code_snapshot();
    let media = MediaInfo::default();

    let resolved = resolve_activity(&config, &snapshot, &media).expect("activity");

    assert_eq!(resolved.discord_details, "repo");
    assert_eq!(resolved.discord_state, Some("Code.exe".to_string()));
}

#[test]
fn smart_mode_uses_rule_hit_process_name_on_last_line_without_media() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        app_message_rules_show_process_name: true,
        ..ClientConfig::default()
    };

    let resolved = build_discord_text(
        &config,
        "Code.exe",
        None,
        &MediaInfo::default(),
        false,
        Some("Coding"),
    )
    .expect("activity");

    assert_eq!(resolved.0, "Coding".to_string());
    assert_eq!(resolved.1, Some("Code.exe".to_string()));
}

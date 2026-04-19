use super::super::*;
use crate::models::DiscordReportMode;

#[test]
fn app_mode_does_not_capture_media_for_reporting() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::App,
        report_media: true,
        report_play_source: true,
        ..ClientConfig::default()
    };

    assert!(!should_capture_media_for_reporting(&config));
}

#[test]
fn music_mode_does_not_capture_window_title_for_reporting() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        report_window_title: true,
        ..ClientConfig::default()
    };

    assert!(!should_capture_window_title_for_reporting(&config));
}

#[test]
fn music_mode_skips_process_name_without_filters() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        report_foreground_app: true,
        ..ClientConfig::default()
    };

    assert!(!should_capture_process_name_for_reporting(&config));
}

#[test]
fn music_mode_keeps_process_name_for_app_filters() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        app_whitelist: vec!["spotify.exe".into()],
        ..ClientConfig::default()
    };

    assert!(should_capture_process_name_for_reporting(&config));
}

#[test]
fn music_mode_does_not_capture_foreground_app_icon() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        discord_use_app_artwork: true,
        ..ClientConfig::default()
    };

    assert!(!should_capture_foreground_app_icon_for_reporting(&config));
}

#[test]
fn music_mode_can_skip_foreground_snapshot_when_unneeded() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        ..ClientConfig::default()
    };

    assert!(!should_capture_foreground_snapshot_for_reporting(&config));
}

#[test]
fn app_mode_still_captures_foreground_snapshot() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::App,
        report_window_title: true,
        ..ClientConfig::default()
    };

    assert!(should_capture_foreground_snapshot_for_reporting(&config));
}

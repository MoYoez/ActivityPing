use super::*;
use crate::{
    models::DiscordReportMode,
    platform::{ForegroundSnapshot, MediaInfo},
};

fn base_config() -> ClientConfig {
    ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        ..ClientConfig::default()
    }
}

fn sample_media() -> MediaInfo {
    MediaInfo {
        title: "Track Name".into(),
        artist: "Artist Name".into(),
        album: "Album Name".into(),
        is_playing: true,
        ..MediaInfo::default()
    }
}

#[test]
fn music_mode_uses_title_then_artist() {
    let config = base_config();
    let media = sample_media();

    let resolved = build_music_discord_text(&config, &media, false);

    assert_eq!(
        resolved,
        Some(("Track Name".to_string(), Some("Artist Name".to_string())))
    );
}

#[test]
fn music_mode_drops_state_when_artist_missing() {
    let config = base_config();
    let mut media = sample_media();
    media.artist.clear();

    let resolved = build_music_discord_text(&config, &media, false);

    assert_eq!(resolved, Some(("Track Name".to_string(), None)));
}

#[test]
fn music_mode_can_keep_paused_media_visible() {
    let mut config = base_config();
    config.report_stopped_media = true;
    let mut media = sample_media();
    media.is_playing = false;

    let resolved = build_music_discord_text(&config, &media, false);

    assert_eq!(
        resolved,
        Some(("Track Name".to_string(), Some("Artist Name".to_string())))
    );
}

#[test]
fn music_mode_hides_paused_media_by_default() {
    let config = base_config();
    let mut media = sample_media();
    media.is_playing = false;

    let resolved = build_music_discord_text(&config, &media, false);

    assert_eq!(resolved, None);
}

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
    let snapshot = ForegroundSnapshot {
        process_name: "Code.exe".into(),
        process_title: "repo".into(),
    };
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
fn normalize_client_config_migrates_legacy_artwork_toggle() {
    let mut config = ClientConfig {
        legacy_discord_use_media_artwork: true,
        ..ClientConfig::default()
    };

    normalize_client_config(&mut config);

    assert!(config.discord_use_app_artwork);
    assert!(config.discord_use_music_artwork);
    assert!(!config.legacy_discord_use_media_artwork);
}

#[test]
fn smart_mode_appends_app_name_to_last_line_only_on_rule_hit() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_smart_show_app_name: true,
        ..ClientConfig::default()
    };
    let snapshot = ForegroundSnapshot {
        process_name: "Code.exe".into(),
        process_title: "repo".into(),
    };
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
    let snapshot = ForegroundSnapshot {
        process_name: "Code.exe".into(),
        process_title: "repo".into(),
    };
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
    let snapshot = ForegroundSnapshot {
        process_name: "Code.exe".into(),
        process_title: "repo".into(),
    };
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
    let snapshot = ForegroundSnapshot {
        process_name: "Code.exe".into(),
        process_title: "repo".into(),
    };
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

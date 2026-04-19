use super::*;
use crate::platform::MediaArtwork;

fn artwork_config() -> ClientConfig {
    ClientConfig {
        discord_use_app_artwork: true,
        discord_use_music_artwork: true,
        ..ClientConfig::default()
    }
}

fn sample_media() -> MediaInfo {
    MediaInfo {
        title: "Track Name".into(),
        artist: "Artist Name".into(),
        album: "Album Name".into(),
        is_playing: true,
        artwork: Some(MediaArtwork {
            bytes: vec![1, 2, 3],
            content_type: "image/png".into(),
        }),
        ..MediaInfo::default()
    }
}

fn sample_resolved() -> ResolvedActivity {
    ResolvedActivity {
        process_name: "Code.exe".into(),
        process_title: Some("repo".into()),
        media_summary: None,
        play_source: None,
        status_text: Some("Matched Title".into()),
        discord_addons: ResolvedDiscordAddons::default(),
        discord_details: "Matched Title".into(),
        discord_state: Some("Code.exe".into()),
        summary: "Matched Title · Code.exe".into(),
        signature: "Matched Title · Code.exe".into(),
    }
}

#[test]
fn presence_artwork_prefers_album_for_hover_text() {
    let config = artwork_config();
    let media = sample_media();

    let artwork = build_presence_artwork(&config, &media).expect("artwork");

    assert_eq!(artwork.hover_text, "Album Name");
}

#[test]
fn presence_artwork_falls_back_when_album_missing() {
    let config = artwork_config();
    let mut media = sample_media();
    media.album.clear();

    let artwork = build_presence_artwork(&config, &media).expect("artwork");

    assert_eq!(artwork.hover_text, "Track Name / Artist Name");
}

#[test]
fn app_mode_skips_music_artwork_even_when_media_is_active() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::App,
        discord_use_music_artwork: true,
        ..artwork_config()
    };
    let media = sample_media();

    let artwork = build_presence_artwork(&config, &media);

    assert!(artwork.is_none());
}

#[test]
fn mixed_mode_prefers_source_icon_when_music_artwork_is_enabled() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_use_music_artwork: true,
        discord_use_app_artwork: true,
        ..ClientConfig::default()
    };
    let foreground_icon = MediaArtwork {
        bytes: vec![9, 9, 9],
        content_type: "image/png".into(),
    };
    let mut media = sample_media();
    media.source_app_id = "spotify.exe".into();
    media.source_icon = Some(MediaArtwork {
        bytes: vec![7, 7, 7],
        content_type: "image/png".into(),
    });

    let icon =
        build_presence_icon(&config, "code.exe", Some(&foreground_icon), &media).expect("icon");

    assert_eq!(icon.hover_text, "Spotify");
}

#[test]
fn app_artwork_becomes_main_icon_when_music_is_unavailable() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_use_app_artwork: true,
        ..ClientConfig::default()
    };
    let foreground_icon = MediaArtwork {
        bytes: vec![9, 9, 9],
        content_type: "image/png".into(),
    };
    let media = MediaInfo::default();

    let icon =
        build_presence_icon(&config, "code.exe", Some(&foreground_icon), &media).expect("icon");

    assert_eq!(icon.hover_text, "Code");
}

#[test]
fn app_mode_does_not_fall_back_to_music_source_icon() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::App,
        discord_use_app_artwork: false,
        discord_use_music_artwork: true,
        ..ClientConfig::default()
    };
    let mut media = sample_media();
    media.source_app_id = "spotify.exe".into();
    media.source_icon = Some(MediaArtwork {
        bytes: vec![7, 7, 7],
        content_type: "image/png".into(),
    });

    let icon = build_presence_icon(&config, "code.exe", None, &media);

    assert!(icon.is_none());
}

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

#[test]
fn paused_media_uses_future_timestamps_when_visible() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        report_stopped_media: true,
        ..ClientConfig::default()
    };
    let mut media = sample_media();
    media.is_playing = false;
    media.position_ms = Some(30_000);
    media.duration_ms = Some(180_000);

    let (started_at, ended_at) = build_media_timestamps(&config, &media);
    let started_at = started_at.expect("start timestamp");
    let ended_at = ended_at.expect("end timestamp");

    assert!(started_at > Utc::now().timestamp_millis());
    assert_eq!(ended_at - started_at, 180_000);
}

#[test]
fn playing_timestamp_update_skips_small_end_drift() {
    let payload = DiscordPresencePayload {
        activity_name: None,
        details: "Track Name".into(),
        state: Some("Artist Name".into()),
        status_display_type: None,
        started_at_millis: Some(900_000),
        ended_at_millis: Some(1_000_050),
        media_duration_ms: Some(180_000),
        media_position_ms: Some(30_000),
        media_is_playing: true,
        summary: "Track Name".into(),
        signature: "Track Name".into(),
        artwork: None,
        icon: None,
        buttons: Vec::new(),
        party: None,
        secrets: None,
    };

    assert!(should_skip_timestamp_update(&payload, Some(1_000_000)));
    assert!(!should_skip_timestamp_update(&payload, Some(999_800)));
}

#[test]
fn mixed_mode_can_publish_rule_buttons() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        ..ClientConfig::default()
    };
    let mut resolved = sample_resolved();
    resolved.discord_addons = ResolvedDiscordAddons {
        buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
            label: "Open".into(),
            url: "https://example.com".into(),
        }],
        ..ResolvedDiscordAddons::default()
    };

    let addons = select_presence_addons(&config, &resolved);
    let buttons = build_presence_buttons(&addons);

    assert_eq!(buttons.len(), 1);
    assert_eq!(buttons[0].label, "Open");
}

#[test]
fn custom_addons_override_rule_buttons_when_enabled() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_use_custom_addons_override: true,
        discord_custom_buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
            label: "Profile".into(),
            url: "https://example.com/profile".into(),
        }],
        ..ClientConfig::default()
    };
    let mut resolved = sample_resolved();
    resolved.discord_addons = ResolvedDiscordAddons {
        buttons: vec![crate::models::DiscordRichPresenceButtonConfig {
            label: "Rule".into(),
            url: "https://example.com/rule".into(),
        }],
        ..ResolvedDiscordAddons::default()
    };

    let addons = select_presence_addons(&config, &resolved);
    let buttons = build_presence_buttons(&addons);

    assert_eq!(buttons.len(), 1);
    assert_eq!(buttons[0].label, "Profile");
}

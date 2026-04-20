use super::{
    super::*,
    fixtures::{artwork_config, sample_media},
};
use super::super::payload::DiscordPresenceAssetKind;
use crate::{
    models::{DiscordCustomAppIconSource, DiscordCustomArtworkSource},
    platform::{MediaArtwork, MediaInfo},
};

#[test]
fn presence_artwork_prefers_album_for_hover_text() {
    let config = artwork_config();
    let media = sample_media();

    let artwork =
        build_presence_artwork(&config, "code.exe", None, &media).expect("artwork");

    assert_eq!(artwork.hover_text, "Album Name");
}

#[test]
fn presence_artwork_falls_back_when_album_missing() {
    let config = artwork_config();
    let mut media = sample_media();
    media.album.clear();

    let artwork =
        build_presence_artwork(&config, "code.exe", None, &media).expect("artwork");

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

    let artwork = build_presence_artwork(&config, "code.exe", None, &media);

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
fn mixed_mode_app_prefer_skips_music_artwork_even_when_media_is_active() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_use_music_artwork: true,
        discord_smart_artwork_preference: crate::models::DiscordSmartArtworkPreference::App,
        ..artwork_config()
    };
    let media = sample_media();

    let artwork = build_presence_artwork(&config, "code.exe", None, &media);

    assert!(artwork.is_none());
}

#[test]
fn mixed_mode_app_prefer_uses_foreground_app_icon_as_main_artwork() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Mixed,
        discord_use_music_artwork: true,
        discord_use_app_artwork: true,
        discord_smart_artwork_preference: crate::models::DiscordSmartArtworkPreference::App,
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

    assert_eq!(icon.hover_text, "Code");
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
fn custom_mode_can_use_foreground_app_icon_as_large_artwork() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Custom,
        discord_custom_artwork_source: DiscordCustomArtworkSource::App,
        ..ClientConfig::default()
    };
    let foreground_icon = MediaArtwork {
        bytes: vec![9, 9, 9],
        content_type: "image/png".into(),
    };

    let artwork = build_presence_artwork(
        &config,
        "code.exe",
        Some(&foreground_icon),
        &MediaInfo::default(),
    )
    .expect("artwork");

    assert_eq!(artwork.hover_text, "Code");
    assert_eq!(artwork.asset_kind, DiscordPresenceAssetKind::AppIcon);
}

#[test]
fn custom_mode_can_use_playback_source_as_small_icon_without_global_toggles() {
    let config = ClientConfig {
        discord_report_mode: DiscordReportMode::Custom,
        discord_custom_app_icon_source: DiscordCustomAppIconSource::Source,
        ..ClientConfig::default()
    };
    let mut media = sample_media();
    media.source_app_id = "spotify.exe".into();
    media.source_icon = Some(MediaArtwork {
        bytes: vec![7, 7, 7],
        content_type: "image/png".into(),
    });

    let icon = build_presence_icon(&config, "code.exe", None, &media).expect("icon");

    assert_eq!(icon.hover_text, "Spotify");
}

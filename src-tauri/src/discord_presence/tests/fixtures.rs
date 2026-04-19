use super::super::*;
use crate::platform::MediaArtwork;

pub(super) fn artwork_config() -> ClientConfig {
    ClientConfig {
        discord_use_app_artwork: true,
        discord_use_music_artwork: true,
        ..ClientConfig::default()
    }
}

pub(super) fn sample_media() -> MediaInfo {
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

pub(super) fn sample_resolved() -> ResolvedActivity {
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

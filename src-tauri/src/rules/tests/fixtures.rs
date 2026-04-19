use crate::{
    models::{ClientConfig, DiscordReportMode},
    platform::{ForegroundSnapshot, MediaInfo},
};

pub(super) fn base_config() -> ClientConfig {
    ClientConfig {
        discord_report_mode: DiscordReportMode::Music,
        ..ClientConfig::default()
    }
}

pub(super) fn sample_media() -> MediaInfo {
    MediaInfo {
        title: "Track Name".into(),
        artist: "Artist Name".into(),
        album: "Album Name".into(),
        is_playing: true,
        ..MediaInfo::default()
    }
}

pub(super) fn code_snapshot() -> ForegroundSnapshot {
    ForegroundSnapshot {
        process_name: "Code.exe".into(),
        process_title: "repo".into(),
    }
}

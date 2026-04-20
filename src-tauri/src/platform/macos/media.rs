use serde::Deserialize;

use crate::platform::{MediaArtwork, MediaInfo};

use super::{
    command::{
        command_output_with_timeout, CommandError, NOWPLAYING_CLI, NOWPLAYING_CLI_FALLBACK_PATHS,
    },
    icons::read_source_app_icon,
    images::{decode_base64_image_payload, detect_image_content_type},
};

enum NowPlayingCliError {
    NotFound {
        path: String,
        attempted: Vec<String>,
    },
    TimedOut,
    Failed(String),
}

impl NowPlayingCliError {
    fn into_user_message(self) -> String {
        match self {
            Self::NotFound { path, attempted } => {
                format!(
                    "Failed to run nowplaying-cli: executable not found in PATH or common Homebrew locations. Tried: {}. PATH={path}",
                    attempted.join(", ")
                )
            }
            Self::TimedOut => "nowplaying-cli timed out (>1500ms).".into(),
            Self::Failed(detail) => format!("nowplaying-cli returned an error: {detail}"),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawNowPlayingInfo {
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoTitle")]
    title: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoArtist")]
    artist: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoAlbum")]
    album: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoClientBundleIdentifier")]
    client_bundle_identifier: Option<String>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoArtworkData",
        alias = "artworkData"
    )]
    artwork_data: Option<String>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoArtworkMIMEType",
        alias = "artworkMimeType"
    )]
    artwork_mime_type: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoDuration", alias = "duration")]
    duration: Option<f64>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoElapsedTime",
        alias = "elapsedTime"
    )]
    elapsed_time: Option<f64>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoPlaybackRate",
        alias = "playbackRate"
    )]
    playback_rate: Option<f64>,
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    let media = match get_now_playing_via_nowplaying_cli() {
        Ok(media) => media,
        Err(NowPlayingCliError::TimedOut) => return Ok(MediaInfo::default()),
        Err(error) => return Err(error.into_user_message()),
    };
    if media.is_empty() {
        return Ok(MediaInfo::default());
    }
    Ok(media)
}

fn get_now_playing_via_nowplaying_cli() -> Result<MediaInfo, NowPlayingCliError> {
    let attempted = std::iter::once(NOWPLAYING_CLI)
        .chain(NOWPLAYING_CLI_FALLBACK_PATHS.iter().copied())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let output = {
        let mut resolved = None;
        for candidate in
            std::iter::once(NOWPLAYING_CLI).chain(NOWPLAYING_CLI_FALLBACK_PATHS.iter().copied())
        {
            match command_output_with_timeout(candidate, &["get-raw"]) {
                Ok(output) => {
                    resolved = Some(output);
                    break;
                }
                Err(CommandError::NotFound) => {}
                Err(CommandError::TimedOut) => return Err(NowPlayingCliError::TimedOut),
                Err(CommandError::Other(detail)) => return Err(NowPlayingCliError::Failed(detail)),
            }
        }
        resolved
    }
    .ok_or_else(|| NowPlayingCliError::NotFound {
        path: std::env::var("PATH").unwrap_or_default(),
        attempted,
    })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no media")
            || combined.contains("no now playing")
            || combined.contains("nothing is playing")
            || combined.contains("not playing")
            || combined.contains("no player")
            || combined.contains("null")
        {
            return Ok(MediaInfo::default());
        }

        let detail = stderr
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .or_else(|| stdout.lines().map(str::trim).find(|line| !line.is_empty()))
            .unwrap_or("Unknown error");
        return Err(NowPlayingCliError::Failed(detail.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let normalize = |value: String| {
        if value.eq_ignore_ascii_case("null") {
            String::new()
        } else {
            value.trim().to_string()
        }
    };

    let raw: RawNowPlayingInfo = serde_json::from_str(&stdout).map_err(|error| {
        NowPlayingCliError::Failed(format!("Failed to parse get-raw output: {error}"))
    })?;

    let title = raw.title.map(normalize).unwrap_or_default();
    let artist = raw.artist.map(normalize).unwrap_or_default();
    let album = raw.album.map(normalize).unwrap_or_default();
    let source_app_id = raw
        .client_bundle_identifier
        .map(normalize)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| NOWPLAYING_CLI.to_string());
    let is_playing = raw
        .playback_rate
        .map(|value| value.is_finite() && value > 0.0)
        .unwrap_or_else(|| !title.is_empty() || !artist.is_empty() || !album.is_empty());
    let duration_ms = seconds_to_millis(raw.duration).filter(|value| *value > 0);
    let position_ms = seconds_to_millis(raw.elapsed_time);
    let artwork = raw
        .artwork_data
        .as_deref()
        .and_then(decode_base64_image_payload)
        .and_then(|bytes| {
            let content_type = raw
                .artwork_mime_type
                .as_deref()
                .map(str::trim)
                .filter(|value| value.starts_with("image/"))
                .map(str::to_string)
                .or_else(|| detect_image_content_type(&bytes).map(str::to_string))?;
            Some(MediaArtwork {
                bytes,
                content_type,
            })
        });
    let source_icon = read_source_app_icon(&source_app_id);

    Ok(MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        is_playing,
        duration_ms,
        position_ms,
        artwork,
        source_icon,
    })
}

fn seconds_to_millis(value: Option<f64>) -> Option<u64> {
    let seconds = value?;
    (seconds.is_finite() && seconds >= 0.0).then_some((seconds * 1_000.0).round() as u64)
}

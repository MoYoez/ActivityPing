use std::path::PathBuf;

use serde::Deserialize;

use crate::platform::{
    MediaArtwork, MediaCaptureOptions, MediaInfo, PLAYBACK_STATE_PAUSED, PLAYBACK_STATE_PLAYING,
};

use super::{
    command::{command_output_with_timeout, CommandError},
    icons::read_source_app_icon,
    images::{decode_base64_image_payload, detect_image_content_type},
};

const PERL_BINARY: &str = "/usr/bin/perl";
const ADAPTER_SCRIPT_NAME: &str = "mediaremote-adapter.pl";
const ADAPTER_FRAMEWORK_NAME: &str = "MediaRemoteAdapter.framework";
const ADAPTER_TIMEOUT_ERROR_HINT: &str =
    "Run `pnpm tauri dev` / `pnpm tauri build`, or run `pnpm prepare:mediaremote-adapter` manually.";

enum MediaRemoteAdapterError {
    ResourceMissing(String),
    NotFound {
        path: String,
        attempted: Vec<String>,
    },
    TimedOut,
    Failed(String),
}

impl MediaRemoteAdapterError {
    fn into_user_message(self) -> String {
        match self {
            Self::ResourceMissing(path) => format!(
                "Missing macOS mediaremote-adapter resource: {path}. {ADAPTER_TIMEOUT_ERROR_HINT}"
            ),
            Self::NotFound { path, attempted } => format!(
                "Failed to run mediaremote-adapter: the script or framework was not found in the resource directory. Tried: {}. PATH={path}",
                attempted.join(", ")
            ),
            Self::TimedOut => "mediaremote-adapter timed out (>5000ms).".into(),
            Self::Failed(detail) => format!("mediaremote-adapter returned an error: {detail}"),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawNowPlayingInfo {
    #[serde(rename = "bundleIdentifier")]
    bundle_identifier: Option<String>,
    #[serde(rename = "parentApplicationBundleIdentifier")]
    parent_application_bundle_identifier: Option<String>,
    #[serde(rename = "playing")]
    playing: Option<bool>,
    #[serde(rename = "title")]
    title: Option<String>,
    #[serde(rename = "artist")]
    artist: Option<String>,
    #[serde(rename = "album")]
    album: Option<String>,
    #[serde(rename = "durationMicros")]
    duration_micros: Option<i64>,
    #[serde(rename = "elapsedTimeMicros")]
    elapsed_time_micros: Option<i64>,
    #[serde(rename = "elapsedTimeNowMicros")]
    elapsed_time_now_micros: Option<i64>,
    #[serde(rename = "playbackRate")]
    playback_rate: Option<f64>,
    #[serde(rename = "artworkData")]
    artwork_data: Option<String>,
    #[serde(rename = "artworkMimeType")]
    artwork_mime_type: Option<String>,
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    get_now_playing_with_options(MediaCaptureOptions::with_assets())
}

pub fn get_now_playing_with_options(options: MediaCaptureOptions) -> Result<MediaInfo, String> {
    let media = match read_now_playing_with_adapter(options) {
        Ok(media) => media,
        Err(MediaRemoteAdapterError::TimedOut) => {
            return Err(MediaRemoteAdapterError::TimedOut.into_user_message())
        }
        Err(error) => return Err(error.into_user_message()),
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }
    Ok(media)
}

// PLACEHOLDER_ADAPTER_BODY
fn read_now_playing_with_adapter(
    options: MediaCaptureOptions,
) -> Result<MediaInfo, MediaRemoteAdapterError> {
    let (script_path, framework_path) = resolve_adapter_paths()?;
    let attempted = vec![
        script_path.to_string_lossy().to_string(),
        framework_path.to_string_lossy().to_string(),
    ];

    let mut args = vec![
        script_path.to_string_lossy().to_string(),
        framework_path.to_string_lossy().to_string(),
        "get".to_string(),
        "--now".to_string(),
        "--micros".to_string(),
    ];
    if !options.include_artwork {
        args.push("--no-artwork".to_string());
    }
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();

    let output = match command_output_with_timeout(PERL_BINARY, &arg_refs) {
        Ok(output) => output,
        Err(CommandError::NotFound) => {
            let path = std::env::var("PATH").unwrap_or_default();
            return Err(MediaRemoteAdapterError::NotFound { path, attempted });
        }
        Err(CommandError::TimedOut) => return Err(MediaRemoteAdapterError::TimedOut),
        Err(CommandError::Other(detail)) => return Err(MediaRemoteAdapterError::Failed(detail)),
    };

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}").to_lowercase();
        if combined.contains("null")
            || combined.contains("no media")
            || combined.contains("no now playing")
            || combined.contains("nothing is playing")
            || combined.contains("not playing")
            || combined.contains("no player")
        {
            return Ok(MediaInfo::default());
        }

        let detail = stderr
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .or_else(|| stdout.lines().map(str::trim).find(|line| !line.is_empty()))
            .unwrap_or("Unknown error");
        return Err(MediaRemoteAdapterError::Failed(detail.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(MediaInfo::default());
    }

    let raw: Option<RawNowPlayingInfo> = serde_json::from_str(trimmed).map_err(|error| {
        MediaRemoteAdapterError::Failed(format!("Failed to parse output: {error}"))
    })?;
    let Some(raw) = raw else {
        return Ok(MediaInfo::default());
    };

    Ok(build_media_info(raw, options))
}

fn build_media_info(raw: RawNowPlayingInfo, options: MediaCaptureOptions) -> MediaInfo {
    let title = normalize_text(raw.title);
    let artist = normalize_text(raw.artist);
    let album = normalize_text(raw.album);
    let source_app_id = resolve_source_app_id(&raw);
    let playback_state =
        normalize_playback_state(raw.playing, raw.playback_rate, &title, &artist, &album);
    let duration_ms = micros_to_ms(raw.duration_micros).filter(|value| *value > 0);
    let position_ms = resolve_position_ms(&raw);

    let artwork = if options.include_artwork {
        decode_artwork(
            raw.artwork_data.as_deref(),
            raw.artwork_mime_type.as_deref(),
        )
    } else {
        None
    };
    let source_icon = if options.include_source_icon {
        read_source_icon(&raw)
    } else {
        None
    };

    MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        playback_state,
        duration_ms,
        position_ms,
        artwork,
        source_icon,
    }
}

fn resolve_adapter_paths() -> Result<(PathBuf, PathBuf), MediaRemoteAdapterError> {
    let mut candidates = Vec::new();

    if let Some(root) = crate::platform::macos_mediaremote_adapter_root() {
        candidates.push(root.clone());
    }

    let compiled_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("mediaremote-adapter");
    if !candidates
        .iter()
        .any(|candidate| candidate == &compiled_root)
    {
        candidates.push(compiled_root);
    }

    let root = candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| {
            MediaRemoteAdapterError::ResourceMissing("resources/mediaremote-adapter".to_string())
        })?;

    let script_path = root.join(ADAPTER_SCRIPT_NAME);
    let framework_path = root.join(ADAPTER_FRAMEWORK_NAME);
    let framework_binary = framework_path.join("MediaRemoteAdapter");
    if !script_path.exists() || !framework_binary.exists() {
        return Err(MediaRemoteAdapterError::NotFound {
            path: std::env::var("PATH").unwrap_or_default(),
            attempted: vec![
                script_path.to_string_lossy().to_string(),
                framework_path.to_string_lossy().to_string(),
            ],
        });
    }

    Ok((script_path, framework_path))
}

fn normalize_text(value: Option<String>) -> String {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("null"))
        .unwrap_or_default()
}

fn normalize_optional_str(value: Option<&str>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("null"))
}

fn resolve_source_app_id(raw: &RawNowPlayingInfo) -> String {
    normalize_optional_str(raw.bundle_identifier.as_deref())
        .or_else(|| normalize_optional_str(raw.parent_application_bundle_identifier.as_deref()))
        .unwrap_or_default()
}

/// Maps the adapter's playing flag (with a playback-rate fallback) to the
/// normalized playback-state string. An unknown state with metadata is treated
/// as playing to preserve prior reporting behavior.
fn normalize_playback_state(
    playing: Option<bool>,
    playback_rate: Option<f64>,
    title: &str,
    artist: &str,
    album: &str,
) -> String {
    match playing {
        Some(true) => PLAYBACK_STATE_PLAYING.to_string(),
        Some(false) => PLAYBACK_STATE_PAUSED.to_string(),
        None => match playback_rate {
            Some(rate) if rate.is_finite() && rate > 0.0 => PLAYBACK_STATE_PLAYING.to_string(),
            Some(_) => PLAYBACK_STATE_PAUSED.to_string(),
            None => {
                if !title.is_empty() || !artist.is_empty() || !album.is_empty() {
                    PLAYBACK_STATE_PLAYING.to_string()
                } else {
                    String::new()
                }
            }
        },
    }
}

fn resolve_position_ms(raw: &RawNowPlayingInfo) -> Option<u64> {
    if let Some(value) = micros_to_ms(raw.elapsed_time_now_micros) {
        return Some(value);
    }
    micros_to_ms(raw.elapsed_time_micros)
}

fn micros_to_ms(value: Option<i64>) -> Option<u64> {
    let micros = value?;
    (micros >= 0).then_some((micros / 1000) as u64)
}

fn decode_artwork(artwork_data: Option<&str>, mime_type: Option<&str>) -> Option<MediaArtwork> {
    let bytes = artwork_data.and_then(decode_base64_image_payload)?;
    let content_type = mime_type
        .map(str::trim)
        .filter(|value| value.starts_with("image/"))
        .map(str::to_string)
        .or_else(|| detect_image_content_type(&bytes).map(str::to_string))?;
    Some(MediaArtwork {
        bytes,
        content_type,
    })
}

fn read_source_icon(raw: &RawNowPlayingInfo) -> Option<MediaArtwork> {
    let candidates = [
        raw.bundle_identifier.as_deref(),
        raw.parent_application_bundle_identifier.as_deref(),
    ];

    for candidate in candidates.into_iter().flatten() {
        if let Some(bundle_id) = normalize_optional_str(Some(candidate)) {
            if let Some(icon) = read_source_app_icon(&bundle_id) {
                return Some(icon);
            }
        }
    }

    None
}

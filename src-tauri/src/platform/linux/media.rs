use crate::platform::{
    MediaArtwork, MediaCaptureOptions, MediaInfo, PLAYBACK_STATE_PAUSED, PLAYBACK_STATE_PLAYING,
    PLAYBACK_STATE_STOPPED,
};

use super::{
    command::{command_output_with_timeout, EmptyFallback},
    icons::read_source_app_icon,
};

const ARTWORK_DOWNLOAD_TIMEOUT_MS: u64 = 5000;

pub fn get_now_playing() -> Result<MediaInfo, String> {
    get_now_playing_with_options(MediaCaptureOptions::with_assets())
}

pub fn get_now_playing_with_options(options: MediaCaptureOptions) -> Result<MediaInfo, String> {
    let output = command_output_with_timeout(
        "playerctl",
        &[
            "metadata",
            "--format",
            "{{status}}\n{{title}}\n{{artist}}\n{{album}}\n{{playerName}}\n{{mpris:length}}\n{{mpris:artUrl}}",
        ],
    )
    .map_err(|error| format!("Failed to run playerctl: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no players found")
            || combined.contains("no player could handle this command")
        {
            return Ok(MediaInfo::default());
        }
        return Err(format!(
            "Failed to read media info: {}",
            stderr.trim().if_empty("playerctl returned an error")
        ));
    }

    let mut lines = stdout.lines().map(str::trim);
    let playback_state = normalize_playback_state(lines.next().unwrap_or_default());
    let title = lines.next().unwrap_or_default().to_string();
    let artist = lines.next().unwrap_or_default().to_string();
    let album = lines.next().unwrap_or_default().to_string();
    let source_app_id = lines.next().unwrap_or_default().to_string();
    let duration_ms = parse_playerctl_length_ms(lines.next().unwrap_or_default());
    let position_ms = read_player_position_ms().unwrap_or(None);
    let source_icon = if options.include_source_icon {
        read_source_app_icon(&source_app_id)
    } else {
        None
    };

    // Download artwork from mpris:artUrl if available
    let artwork = if options.include_artwork {
        download_artwork_from_url(lines.next().unwrap_or_default())
    } else {
        None
    };

    let media = MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        playback_state,
        duration_ms,
        position_ms,
        artwork,
        source_icon,
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

/// Download artwork from a URL and return as MediaArtwork with base64 bytes.
/// Handles HTTP URLs, file:// URLs, and already-encoded data URLs.
fn download_artwork_from_url(url: &str) -> Option<MediaArtwork> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }

    // For data: URLs — strip the prefix and decode directly
    if trimmed.starts_with("data:") {
        return decode_inline_data_url(trimmed);
    }

    // Must be a fetchable URL
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return None;
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(
            ARTWORK_DOWNLOAD_TIMEOUT_MS,
        ))
        .build()
        .ok()?;

    let response = client.get(trimmed).send().ok()?;
    if !response.status().is_success() {
        return None;
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().trim_start_matches("data:").split(';').next())
        .filter(|v| v.starts_with("image/"))
        .map(str::to_string);

    let bytes = response.bytes().ok()?;
    if bytes.is_empty() {
        return None;
    }

    let content_type = content_type.unwrap_or_else(|| detect_image_content_type(&bytes));

    Some(MediaArtwork {
        bytes: bytes.to_vec(),
        content_type,
    })
}

/// Decode a data: URL inline without downloading.
fn decode_inline_data_url(data_url: &str) -> Option<MediaArtwork> {
    use base64::Engine as _;
    let trimmed = data_url.trim();
    let m = regex::Regex::new(r"^data:([^;]+);base64,(.+)$").ok()?;
    let caps = m.captures(trimmed)?;
    let mime = caps.get(1)?.as_str().to_lowercase();
    if !mime.starts_with("image/") {
        return None;
    }
    let encoded = caps.get(2)?.as_str();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .ok()?;
    if bytes.is_empty() {
        return None;
    }
    Some(MediaArtwork {
        bytes,
        content_type: mime,
    })
}

fn detect_image_content_type(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg".to_string()
    } else if bytes.len() >= 8 && bytes[..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        "image/png".to_string()
    } else if bytes.len() >= 6 && (bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a")) {
        "image/gif".to_string()
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp".to_string()
    } else {
        "image/jpeg".to_string()
    }
}

fn read_player_position_ms() -> Result<Option<u64>, String> {
    let output = command_output_with_timeout("playerctl", &["position"])
        .map_err(|error| format!("Failed to run playerctl position: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no players found")
            || combined.contains("no player could handle this command")
        {
            return Ok(None);
        }
        return Err(stderr
            .trim()
            .if_empty(stdout.trim())
            .if_empty("playerctl position returned an error")
            .to_string());
    }

    Ok(parse_playerctl_position_ms(stdout.trim()))
}

fn parse_playerctl_length_ms(value: &str) -> Option<u64> {
    let micros = value.trim().parse::<u64>().ok()?;
    (micros > 0).then_some(micros / 1_000)
}

fn parse_playerctl_position_ms(value: &str) -> Option<u64> {
    let seconds = value.trim().parse::<f64>().ok()?;
    (seconds.is_finite() && seconds >= 0.0).then_some((seconds * 1_000.0).round() as u64)
}

/// Normalizes a playerctl `{{status}}` value (e.g. "Playing", "Paused",
/// "Stopped") to the platform-wide playback-state string.
fn normalize_playback_state(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "playing" => PLAYBACK_STATE_PLAYING.to_string(),
        "paused" => PLAYBACK_STATE_PAUSED.to_string(),
        "stopped" => PLAYBACK_STATE_STOPPED.to_string(),
        _ => String::new(),
    }
}

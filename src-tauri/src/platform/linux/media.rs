use crate::platform::MediaInfo;

use super::{
    command::{command_output_with_timeout, EmptyFallback},
    icons::read_source_app_icon,
};

pub fn get_now_playing() -> Result<MediaInfo, String> {
    let output = command_output_with_timeout(
        "playerctl",
        &[
            "metadata",
            "--format",
            "{{title}}\n{{artist}}\n{{album}}\n{{playerName}}\n{{mpris:length}}",
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
    let title = lines.next().unwrap_or_default().to_string();
    let artist = lines.next().unwrap_or_default().to_string();
    let album = lines.next().unwrap_or_default().to_string();
    let source_app_id = lines.next().unwrap_or_default().to_string();
    let is_playing = read_player_status()?;
    let duration_ms = parse_playerctl_length_ms(lines.next().unwrap_or_default());
    let position_ms = read_player_position_ms().unwrap_or(None);
    let source_icon = read_source_app_icon(&source_app_id);

    let media = MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        is_playing,
        duration_ms,
        position_ms,
        artwork: None,
        source_icon,
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
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

fn read_player_status() -> Result<bool, String> {
    let output = command_output_with_timeout("playerctl", &["status"])
        .map_err(|error| format!("Failed to run playerctl status: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no players found")
            || combined.contains("no player could handle this command")
        {
            return Ok(false);
        }
        return Err(stderr
            .trim()
            .if_empty(stdout.trim())
            .if_empty("playerctl status returned an error")
            .to_string());
    }

    Ok(stdout.trim().eq_ignore_ascii_case("playing"))
}

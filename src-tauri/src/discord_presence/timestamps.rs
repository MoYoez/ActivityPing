use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;

use crate::{models::ClientConfig, platform::MediaInfo};

use super::payload::DiscordPresencePayload;

const TIMESTAMP_UPDATE_THRESHOLD_MS: i64 = 100;

pub(super) fn build_media_timestamps(
    config: &ClientConfig,
    media: &MediaInfo,
) -> (Option<i64>, Option<i64>) {
    if !media.is_reportable(config.report_stopped_media) {
        return (None, None);
    }

    let position_ms = media
        .position_ms
        .and_then(|value| i64::try_from(value).ok());
    let duration_ms = media
        .duration_ms
        .and_then(|value| i64::try_from(value).ok());

    let Some(position_ms) = position_ms else {
        return (None, None);
    };

    if !media.is_playing {
        let Some(duration_ms) = duration_ms else {
            return (None, None);
        };
        return calc_paused_timestamps(position_ms, duration_ms)
            .map(|(started_at, ended_at)| (Some(started_at), Some(ended_at)))
            .unwrap_or((None, None));
    }

    let (started_at, ended_at) = match duration_ms {
        Some(duration_ms) => calc_playing_timestamps(position_ms, duration_ms),
        None => (Utc::now().timestamp_millis().checked_sub(position_ms), None),
    };

    (started_at, ended_at)
}

fn calc_playing_timestamps(position_ms: i64, duration_ms: i64) -> (Option<i64>, Option<i64>) {
    let now_ms = Utc::now().timestamp_millis();
    let remaining_ms = duration_ms.saturating_sub(position_ms).max(0);
    let ended_at = now_ms.checked_add(remaining_ms);
    let started_at = ended_at.and_then(|ended_at| ended_at.checked_sub(duration_ms));
    (started_at, ended_at)
}

fn calc_paused_timestamps(position_ms: i64, duration_ms: i64) -> Option<(i64, i64)> {
    // Based on apoint123/inflink-rs and the musicpresence.app future timestamp
    // trick: https://github.com/apoint123/inflink-rs/blob/main/packages/backend/src/discord.rs
    const ONE_YEAR_MS: i64 = 365 * 24 * 60 * 60 * 1000;

    if duration_ms <= 0 {
        return None;
    }

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let current_progress_ms = position_ms.clamp(0, duration_ms);
    let future_start = now_ms
        .checked_sub(current_progress_ms)?
        .checked_add(ONE_YEAR_MS)?;
    let future_end = future_start.checked_add(duration_ms)?;

    Some((future_start, future_end))
}

pub(super) fn should_skip_timestamp_update(
    payload: &DiscordPresencePayload,
    last_sent_end_timestamp: Option<i64>,
) -> bool {
    if !payload.media_is_playing {
        return false;
    }

    let Some(last_end) = last_sent_end_timestamp else {
        return false;
    };
    let Some(next_end) = payload.ended_at_millis else {
        return false;
    };

    last_end.abs_diff(next_end) < TIMESTAMP_UPDATE_THRESHOLD_MS as u64
}

use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;

use crate::{models::ClientConfig, platform::MediaInfo};

use super::payload::DiscordPresencePayload;

const MIN_STALLED_PROGRESS_DELTA_MS: u64 = 500;
const STALLED_PROGRESS_REPEAT_THRESHOLD: u32 = 1;
const TIMESTAMP_UPDATE_THRESHOLD_MS: i64 = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlaybackProgressState {
    signature: String,
    last_position_ms: Option<u64>,
    stalled_repeats: u32,
}

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

pub(super) fn downgrade_stalled_playback_to_paused(
    payload: &mut DiscordPresencePayload,
    state: &mut PlaybackProgressState,
    min_progress_delta_ms: u64,
) {
    let signature = payload.signature.as_str();
    let position_ms = payload.media_position_ms;

    if !payload.media_is_playing {
        state.remember(signature, position_ms, 0);
        return;
    }

    let (Some(position_ms), Some(duration_ms)) = (position_ms, payload.media_duration_ms) else {
        state.remember(signature, position_ms, 0);
        return;
    };

    let min_progress_delta_ms = min_progress_delta_ms.max(MIN_STALLED_PROGRESS_DELTA_MS);
    let stalled = state.signature == signature
        && state
            .last_position_ms
            .map(|last_position_ms| position_ms.abs_diff(last_position_ms) < min_progress_delta_ms)
            .unwrap_or(false);
    let stalled_repeats = if stalled {
        state.stalled_repeats.saturating_add(1)
    } else {
        0
    };
    state.remember(signature, Some(position_ms), stalled_repeats);

    if stalled_repeats < STALLED_PROGRESS_REPEAT_THRESHOLD {
        return;
    }

    let (Ok(position_ms), Ok(duration_ms)) =
        (i64::try_from(position_ms), i64::try_from(duration_ms))
    else {
        return;
    };
    let Some((started_at, ended_at)) = calc_paused_timestamps(position_ms, duration_ms) else {
        return;
    };

    payload.media_is_playing = false;
    payload.started_at_millis = Some(started_at);
    payload.ended_at_millis = Some(ended_at);
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

impl PlaybackProgressState {
    fn remember(&mut self, signature: &str, position_ms: Option<u64>, stalled_repeats: u32) {
        self.signature.clear();
        self.signature.push_str(signature);
        self.last_position_ms = position_ms;
        self.stalled_repeats = stalled_repeats;
    }
}

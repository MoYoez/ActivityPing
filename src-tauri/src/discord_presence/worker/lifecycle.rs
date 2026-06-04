use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crate::backend_locale::BackendLocale;

use super::super::{
    DEFAULT_SYNC_INTERVAL, MAX_ERROR_BACKOFF_MS, WORKER_JOIN_POLL_STEP, WORKER_JOIN_WAIT_TIMEOUT,
};

pub(super) fn sleep_with_stop(duration: Duration, stop_flag: &Arc<AtomicBool>) {
    let mut remaining = duration.as_millis() as u64;
    while remaining > 0 {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }
        let step = remaining.min(200);
        thread::sleep(Duration::from_millis(step));
        remaining = remaining.saturating_sub(step);
    }
}

/// Sleeps up to `duration`, returning early when a foreground-change event
/// arrives so presence can be re-synced immediately. Falls back to plain
/// `sleep_with_stop` when no wakeup channel is available or it disconnects.
pub(super) fn sleep_with_stop_and_wakeup(
    duration: Duration,
    stop_flag: &Arc<AtomicBool>,
    wakeup: Option<&Receiver<()>>,
) {
    let Some(rx) = wakeup else {
        sleep_with_stop(duration, stop_flag);
        return;
    };

    let deadline = Instant::now() + duration;
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        if stop_flag.load(Ordering::SeqCst) {
            return;
        }

        let step = remaining.min(Duration::from_millis(200));
        match rx.recv_timeout(step) {
            Ok(()) => {
                while rx.try_recv().is_ok() {}
                return;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                sleep_with_stop(remaining, stop_flag);
                return;
            }
        }
    }
}

pub(super) fn error_backoff(consecutive_errors: u32) -> Duration {
    let multiplier = 2u64.saturating_pow(consecutive_errors.saturating_sub(1));
    Duration::from_millis(
        (DEFAULT_SYNC_INTERVAL.as_millis() as u64)
            .saturating_mul(multiplier.max(1))
            .min(MAX_ERROR_BACKOFF_MS),
    )
}

pub(super) fn discord_worker_stopping(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord RPC is still stopping. Try again shortly.".into()
    } else {
        "Discord RPC is still stopping. Try again shortly.".into()
    }
}

pub(super) fn wait_for_worker_exit(handle: JoinHandle<()>) -> Result<(), JoinHandle<()>> {
    let deadline = Instant::now() + WORKER_JOIN_WAIT_TIMEOUT;
    let handle = handle;

    while Instant::now() < deadline {
        if handle.is_finished() {
            let _ = handle.join();
            return Ok(());
        }
        thread::sleep(WORKER_JOIN_POLL_STEP);
    }

    Err(handle)
}

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
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

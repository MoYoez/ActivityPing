use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use super::super::{WORKER_JOIN_POLL_STEP, WORKER_JOIN_WAIT_TIMEOUT};

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

/// Sleeps up to `duration`, returning early (`true`) when a foreground-change
/// event arrives so the loop can re-capture immediately. Falls back to plain
/// `sleep_with_stop` when no wakeup channel is available (e.g. Linux) or the
/// channel disconnects. Always returns promptly when the stop flag is set.
pub(super) fn sleep_with_stop_and_wakeup(
    duration: Duration,
    stop_flag: &Arc<AtomicBool>,
    wakeup: Option<&Receiver<()>>,
) -> bool {
    let Some(rx) = wakeup else {
        sleep_with_stop(duration, stop_flag);
        return false;
    };

    let deadline = Instant::now() + duration;
    while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
        if stop_flag.load(Ordering::SeqCst) {
            return false;
        }

        let step = remaining.min(Duration::from_millis(200));
        match rx.recv_timeout(step) {
            Ok(()) => {
                // Coalesce any bursts of events into a single wakeup.
                while rx.try_recv().is_ok() {}
                return true;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                sleep_with_stop(remaining, stop_flag);
                return false;
            }
        }
    }
    false
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

    if handle.is_finished() {
        let _ = handle.join();
        Ok(())
    } else {
        Err(handle)
    }
}

pub(super) fn reporter_worker_stopping() -> String {
    "Local monitor is still stopping. Try again shortly.".into()
}

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime},
};

use crate::{models::ClientConfig, rules::resolve_activity};

use super::{
    super::{
        logging::{fallback_text, now_iso_string, push_background_log},
        state::{mark_stopped, set_last_error, update_snapshot, ReporterInner},
        MAX_ERROR_BACKOFF_MS,
    },
    activity::{build_report_log_payload, build_reporter_activity},
    capture::{capture_foreground_snapshot, capture_media},
    lifecycle::sleep_with_stop,
};

pub(super) fn run_reporter_loop(
    state: Arc<Mutex<ReporterInner>>,
    config: ClientConfig,
    stop_flag: Arc<AtomicBool>,
    mut sequence_seed: u64,
    run_id: u64,
) {
    let poll_interval = Duration::from_millis(config.poll_interval_ms.max(1_000));
    let heartbeat_interval = if config.heartbeat_interval_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(
            config.heartbeat_interval_ms.max(1_000),
        ))
    };

    let mut last_signature: Option<String> = None;
    let mut last_emit_at: Option<SystemTime> = None;
    let mut consecutive_errors: u32 = 0;
    let mut last_media_error: Option<String> = None;

    while !stop_flag.load(Ordering::SeqCst) {
        let mut iteration_had_error = false;

        match capture_foreground_snapshot(&config) {
            Ok(snapshot) => {
                let media = capture_media(
                    &state,
                    &config,
                    &mut sequence_seed,
                    &mut last_media_error,
                    &stop_flag,
                );

                match resolve_activity(&config, &snapshot, &media) {
                    Some(resolved) => {
                        let current_activity =
                            build_reporter_activity(&config, &snapshot, &resolved, &media);
                        update_snapshot(&state, Some(current_activity.clone()), None, None, run_id);

                        let same_as_last = last_signature
                            .as_ref()
                            .map(|last| last == &resolved.signature)
                            .unwrap_or(false);
                        let should_emit =
                            should_emit_activity(same_as_last, heartbeat_interval, last_emit_at);

                        if should_emit {
                            let is_heartbeat = same_as_last;
                            let log_payload =
                                Some(build_report_log_payload(&resolved, &current_activity));
                            push_background_log(
                                &state,
                                &mut sequence_seed,
                                if is_heartbeat { "info" } else { "success" },
                                fallback_text(if is_heartbeat {
                                    "Activity heartbeat"
                                } else {
                                    "Activity updated"
                                }),
                                fallback_text(if is_heartbeat {
                                    format!("Heartbeat kept for {}.", resolved.summary)
                                } else {
                                    format!("Captured {}.", resolved.summary)
                                }),
                                log_payload,
                            );
                            update_snapshot(
                                &state,
                                Some(current_activity),
                                None,
                                Some(now_iso_string()),
                                run_id,
                            );
                            last_signature = Some(resolved.signature);
                            last_emit_at = Some(SystemTime::now());
                        }
                    }
                    None => {
                        if last_signature.is_some() {
                            push_background_log(
                                &state,
                                &mut sequence_seed,
                                "info",
                                fallback_text("Activity cleared"),
                                fallback_text("No local activity passed the current rules."),
                                None,
                            );
                            last_signature = None;
                            last_emit_at = None;
                        }
                        update_snapshot(&state, None, None, None, run_id);
                    }
                }
            }
            Err(error) => {
                push_background_log(
                    &state,
                    &mut sequence_seed,
                    "error",
                    fallback_text("Foreground capture failed"),
                    fallback_text(error.clone()),
                    None,
                );
                set_last_error(&state, Some(error), run_id);
                iteration_had_error = true;
            }
        }

        consecutive_errors = if iteration_had_error {
            consecutive_errors.saturating_add(1)
        } else {
            0
        };

        let effective_sleep = if consecutive_errors > 1 {
            let backoff_ms =
                (poll_interval.as_millis() as u64).saturating_mul(1 << consecutive_errors.min(4));
            Duration::from_millis(backoff_ms.min(MAX_ERROR_BACKOFF_MS))
        } else {
            poll_interval
        };

        sleep_with_stop(effective_sleep, &stop_flag);
    }

    mark_stopped(&state, None, run_id);
}

fn should_emit_activity(
    same_as_last: bool,
    heartbeat_interval: Option<Duration>,
    last_emit_at: Option<SystemTime>,
) -> bool {
    if !same_as_last {
        return true;
    }

    heartbeat_interval
        .map(|interval| {
            last_emit_at
                .and_then(|last| SystemTime::now().duration_since(last).ok())
                .map(|elapsed| elapsed >= interval)
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

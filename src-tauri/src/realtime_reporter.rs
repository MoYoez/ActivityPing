use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use serde_json::Value;

use crate::{
    backend_locale::BackendLocale,
    models::{
        ApiResult, ClientConfig, RealtimeReporterSnapshot, ReporterActivity, ReporterLogEntry,
    },
    platform::{get_foreground_snapshot_for_reporting, get_now_playing, MediaInfo},
    rules::{normalize_client_config, resolve_activity},
};

const MAX_LOGS: usize = 20;
const MAX_ERROR_BACKOFF_MS: u64 = 30_000;
const WORKER_JOIN_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const WORKER_JOIN_POLL_STEP: Duration = Duration::from_millis(50);

struct LogTextSpec {
    key: Option<&'static str>,
    params: Option<Value>,
    fallback: String,
}

fn fallback_text(value: impl Into<String>) -> LogTextSpec {
    LogTextSpec {
        key: None,
        params: None,
        fallback: value.into(),
    }
}

struct ReporterInner {
    running: bool,
    active_run_id: Option<u64>,
    logs: Vec<ReporterLogEntry>,
    current_activity: Option<ReporterActivity>,
    last_heartbeat_at: Option<String>,
    last_error: Option<String>,
    stop_flag: Option<Arc<AtomicBool>>,
    worker: Option<JoinHandle<()>>,
}

impl Default for ReporterInner {
    fn default() -> Self {
        Self {
            running: false,
            active_run_id: None,
            logs: Vec::new(),
            current_activity: None,
            last_heartbeat_at: None,
            last_error: None,
            stop_flag: None,
            worker: None,
        }
    }
}

pub struct ReporterRuntime {
    inner: Arc<Mutex<ReporterInner>>,
    sequence: AtomicU64,
}

impl ReporterRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ReporterInner::default())),
            sequence: AtomicU64::new(1),
        }
    }

    pub fn snapshot(&self) -> RealtimeReporterSnapshot {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        snapshot_from_inner(&inner)
    }

    pub fn start(
        &self,
        mut config: ClientConfig,
        _locale: BackendLocale,
    ) -> Result<RealtimeReporterSnapshot, String> {
        normalize_client_config(&mut config);

        let stop_flag = Arc::new(AtomicBool::new(false));
        let run_id = self.sequence.fetch_add(1, Ordering::SeqCst);
        let previous_worker = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if inner.running {
                return Ok(snapshot_from_inner(&inner));
            }

            if let Some(old_flag) = inner.stop_flag.take() {
                old_flag.store(true, Ordering::SeqCst);
            }

            inner.worker.take()
        };

        if let Some(handle) = previous_worker {
            if let Err(handle) = wait_for_worker_exit(handle) {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = Some(handle);
                return Err(reporter_worker_stopping());
            }
        }

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.running = true;
            inner.active_run_id = Some(run_id);
            inner.stop_flag = Some(stop_flag.clone());
            inner.last_error = None;
        }

        self.push_log(
            "info",
            fallback_text("Local monitor started"),
            fallback_text("Activity capture is now running inside this desktop app."),
            None,
        );

        let state = self.inner.clone();
        let sequence_seed = self.sequence.fetch_add(1, Ordering::SeqCst);
        let worker = thread::spawn(move || {
            run_reporter_loop(state, config, stop_flag, sequence_seed, run_id);
        });

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.worker = Some(worker);
        }

        Ok(self.snapshot())
    }

    pub fn stop(&self) -> RealtimeReporterSnapshot {
        let worker = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(flag) = inner.stop_flag.take() {
                flag.store(true, Ordering::SeqCst);
            }
            inner.running = false;
            inner.active_run_id = None;
            inner.worker.take()
        };

        let stopped_cleanly = if let Some(handle) = worker {
            if let Err(handle) = wait_for_worker_exit(handle) {
                {
                    let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                    inner.worker = Some(handle);
                }
                self.push_log(
                    "warn",
                    fallback_text("Local monitor is stopping"),
                    fallback_text(
                        "The worker will exit after the current capture cycle completes.",
                    ),
                    None,
                );
                false
            } else {
                true
            }
        } else {
            true
        };

        if stopped_cleanly {
            {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = None;
                inner.current_activity = None;
            }
            self.push_log(
                "warn",
                fallback_text("Local monitor stopped"),
                fallback_text("Activity capture has been stopped."),
                None,
            );
        }

        self.snapshot()
    }

    fn push_log(
        &self,
        level: &str,
        title: LogTextSpec,
        detail: LogTextSpec,
        payload: Option<Value>,
    ) {
        let id = format!(
            "{}-{}",
            now_unix_millis(),
            self.sequence.fetch_add(1, Ordering::SeqCst)
        );
        let entry = ReporterLogEntry {
            id,
            timestamp: now_iso_string(),
            level: level.to_string(),
            title: title.fallback,
            detail: detail.fallback,
            title_key: title.key.map(str::to_string),
            title_params: title.params,
            detail_key: detail.key.map(str::to_string),
            detail_params: detail.params,
            payload,
        };
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.logs.insert(0, entry);
        if inner.logs.len() > MAX_LOGS {
            inner.logs.truncate(MAX_LOGS);
        }
    }
}

impl Drop for ReporterRuntime {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn snapshot_from_inner(inner: &ReporterInner) -> RealtimeReporterSnapshot {
    RealtimeReporterSnapshot {
        running: inner.running,
        logs: inner.logs.clone(),
        current_activity: inner.current_activity.clone(),
        last_heartbeat_at: inner.last_heartbeat_at.clone(),
        last_error: inner.last_error.clone(),
    }
}

fn run_reporter_loop(
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

        let foreground = get_foreground_snapshot_for_reporting(
            config.report_foreground_app,
            config.report_window_title,
        );

        match foreground {
            Ok(snapshot) => {
                let media = if config.report_media || config.report_play_source {
                    match get_now_playing() {
                        Ok(media) => {
                            last_media_error = None;
                            media
                        }
                        Err(error) => {
                            let should_log = last_media_error
                                .as_ref()
                                .map(|last| last != &error)
                                .unwrap_or(true);
                            last_media_error = Some(error.clone());
                            if should_log {
                                push_background_log(
                                    &state,
                                    &mut sequence_seed,
                                    "warn",
                                    fallback_text("Media capture failed"),
                                    fallback_text(error),
                                    None,
                                );
                            }
                            MediaInfo::default()
                        }
                    }
                } else {
                    last_media_error = None;
                    MediaInfo::default()
                };

                match resolve_activity(&config, &snapshot, &media) {
                    Some(resolved) => {
                        let current_activity = build_reporter_activity(&resolved, &media);
                        update_snapshot(&state, Some(current_activity), None, None, run_id);

                        let same_as_last = last_signature
                            .as_ref()
                            .map(|last| last == &resolved.signature)
                            .unwrap_or(false);
                        let should_emit = if same_as_last {
                            heartbeat_interval
                                .map(|interval| {
                                    last_emit_at
                                        .and_then(|last| {
                                            SystemTime::now().duration_since(last).ok()
                                        })
                                        .map(|elapsed| elapsed >= interval)
                                        .unwrap_or(false)
                                })
                                .unwrap_or(false)
                        } else {
                            true
                        };

                        if should_emit {
                            let is_heartbeat = same_as_last;
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
                                None,
                            );
                            update_snapshot(
                                &state,
                                Some(build_reporter_activity(&resolved, &media)),
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

        if iteration_had_error {
            consecutive_errors = consecutive_errors.saturating_add(1);
        } else {
            consecutive_errors = 0;
        }

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

pub fn config_is_ready(_config: &ClientConfig) -> bool {
    true
}

fn build_reporter_activity(
    resolved: &crate::rules::ResolvedActivity,
    media: &MediaInfo,
) -> ReporterActivity {
    ReporterActivity {
        process_name: resolved.process_name.clone(),
        process_title: resolved.process_title.clone(),
        media_summary: resolved.media_summary.clone(),
        media_duration_ms: if media.is_active() {
            media.duration_ms
        } else {
            None
        },
        media_position_ms: if media.is_active() {
            media.position_ms
        } else {
            None
        },
        play_source: resolved.play_source.clone(),
        status_text: resolved.status_text.clone(),
        updated_at: Some(now_iso_string()),
    }
}

fn update_snapshot(
    state: &Arc<Mutex<ReporterInner>>,
    current_activity: Option<ReporterActivity>,
    last_error: Option<String>,
    last_heartbeat_at: Option<String>,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.current_activity = current_activity;
    inner.last_error = last_error;
    if let Some(heartbeat_at) = last_heartbeat_at {
        inner.last_heartbeat_at = Some(heartbeat_at);
    }
}

fn set_last_error(state: &Arc<Mutex<ReporterInner>>, error: Option<String>, run_id: u64) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }
    inner.last_error = error;
}

fn mark_stopped(state: &Arc<Mutex<ReporterInner>>, error: Option<String>, run_id: u64) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }
    inner.running = false;
    inner.active_run_id = None;
    inner.stop_flag = None;
    inner.current_activity = None;
    if error.is_some() {
        inner.last_error = error;
    }
}

fn push_background_log(
    state: &Arc<Mutex<ReporterInner>>,
    sequence: &mut u64,
    level: &str,
    title: LogTextSpec,
    detail: LogTextSpec,
    payload: Option<Value>,
) {
    let entry = ReporterLogEntry {
        id: format!("{}-{}", now_unix_millis(), *sequence),
        timestamp: now_iso_string(),
        level: level.to_string(),
        title: title.fallback,
        detail: detail.fallback,
        title_key: title.key.map(str::to_string),
        title_params: title.params,
        detail_key: detail.key.map(str::to_string),
        detail_params: detail.params,
        payload,
    };
    *sequence += 1;

    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    inner.logs.insert(0, entry);
    if inner.logs.len() > MAX_LOGS {
        inner.logs.truncate(MAX_LOGS);
    }
}

fn sleep_with_stop(duration: Duration, stop_flag: &Arc<AtomicBool>) {
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

fn wait_for_worker_exit(handle: JoinHandle<()>) -> Result<(), JoinHandle<()>> {
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

fn now_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn now_iso_string() -> String {
    Utc::now().to_rfc3339()
}

pub fn snapshot_result(runtime: &ReporterRuntime) -> ApiResult<RealtimeReporterSnapshot> {
    ApiResult::success(200, runtime.snapshot())
}

fn reporter_worker_stopping() -> String {
    "Local monitor is still stopping. Try again shortly.".into()
}

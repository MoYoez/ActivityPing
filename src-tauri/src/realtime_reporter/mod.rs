mod logging;
mod state;
mod worker;

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::{
    backend_locale::BackendLocale,
    models::{ApiResult, ClientConfig, RealtimeReporterSnapshot, ReporterLogEntry},
    rules::normalize_client_config,
};

use self::{
    logging::{fallback_text, now_iso_string, now_unix_millis, LogTextSpec},
    state::{snapshot_from_inner, ReporterInner},
    worker::{reporter_worker_stopping, run_reporter_loop, wait_for_worker_exit},
};

pub(super) const MAX_LOGS: usize = 20;
pub(super) const MAX_ERROR_BACKOFF_MS: u64 = 30_000;
pub(super) const WORKER_JOIN_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
pub(super) const WORKER_JOIN_POLL_STEP: Duration = Duration::from_millis(50);

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
        payload: Option<serde_json::Value>,
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

pub fn snapshot_result(runtime: &ReporterRuntime) -> ApiResult<RealtimeReporterSnapshot> {
    ApiResult::success(200, runtime.snapshot())
}

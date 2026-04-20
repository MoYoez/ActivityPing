use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::JoinHandle,
};

use crate::models::{RealtimeReporterSnapshot, ReporterActivity, ReporterLogEntry};

pub(super) struct ReporterInner {
    pub(super) running: bool,
    pub(super) active_run_id: Option<u64>,
    pub(super) logs: Vec<ReporterLogEntry>,
    pub(super) current_activity: Option<ReporterActivity>,
    pub(super) last_heartbeat_at: Option<String>,
    pub(super) last_error: Option<String>,
    pub(super) stop_flag: Option<Arc<AtomicBool>>,
    pub(super) worker: Option<JoinHandle<()>>,
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

pub(super) fn snapshot_from_inner(inner: &ReporterInner) -> RealtimeReporterSnapshot {
    RealtimeReporterSnapshot {
        running: inner.running,
        logs: inner.logs.clone(),
        current_activity: inner.current_activity.clone(),
        last_heartbeat_at: inner.last_heartbeat_at.clone(),
        last_error: inner.last_error.clone(),
    }
}

pub(super) fn update_snapshot(
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

pub(super) fn set_last_error(
    state: &Arc<Mutex<ReporterInner>>,
    error: Option<String>,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }
    inner.last_error = error;
}

pub(super) fn mark_stopped(state: &Arc<Mutex<ReporterInner>>, error: Option<String>, run_id: u64) {
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

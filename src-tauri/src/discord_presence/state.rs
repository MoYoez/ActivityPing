use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::JoinHandle,
};

use chrono::Utc;

use crate::models::{DiscordDebugPayload, DiscordPresenceSnapshot};

pub(super) struct DiscordPresenceInner {
    pub(super) running: bool,
    pub(super) active_run_id: Option<u64>,
    pub(super) connected: bool,
    pub(super) last_sync_at: Option<String>,
    pub(super) last_error: Option<String>,
    pub(super) current_summary: Option<String>,
    pub(super) debug_payload: Option<DiscordDebugPayload>,
    pub(super) stop_flag: Option<Arc<AtomicBool>>,
    pub(super) worker: Option<JoinHandle<()>>,
}

impl Default for DiscordPresenceInner {
    fn default() -> Self {
        Self {
            running: false,
            active_run_id: None,
            connected: false,
            last_sync_at: None,
            last_error: None,
            current_summary: None,
            debug_payload: None,
            stop_flag: None,
            worker: None,
        }
    }
}

pub(super) fn snapshot_from_inner(inner: &DiscordPresenceInner) -> DiscordPresenceSnapshot {
    DiscordPresenceSnapshot {
        running: inner.running,
        connected: inner.connected,
        last_sync_at: inner.last_sync_at.clone(),
        last_error: inner.last_error.clone(),
        current_summary: inner.current_summary.clone(),
        debug_payload: inner.debug_payload.clone(),
    }
}

pub(super) fn update_presence_snapshot(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    connected: bool,
    last_error: Option<String>,
    current_summary: Option<String>,
    debug_payload: Option<DiscordDebugPayload>,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.connected = connected;
    inner.last_error = last_error;
    inner.last_sync_at = Some(Utc::now().to_rfc3339());
    inner.current_summary = current_summary;
    inner.debug_payload = debug_payload;
}

pub(super) fn update_presence_heartbeat(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    connected: bool,
    last_error: Option<String>,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.connected = connected;
    inner.last_error = last_error;
    inner.last_sync_at = Some(Utc::now().to_rfc3339());
}

pub(super) fn set_discord_error(
    state: &Arc<Mutex<DiscordPresenceInner>>,
    error: Option<String>,
    connected: bool,
    run_id: u64,
) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.connected = connected;
    inner.last_error = error;
    inner.debug_payload = None;
}

pub(super) fn mark_stopped(state: &Arc<Mutex<DiscordPresenceInner>>, run_id: u64) {
    let mut inner = state.lock().unwrap_or_else(|e| e.into_inner());
    if inner.active_run_id != Some(run_id) {
        return;
    }

    inner.running = false;
    inner.connected = false;
    inner.active_run_id = None;
    inner.current_summary = None;
    inner.debug_payload = None;
    inner.stop_flag = None;
}

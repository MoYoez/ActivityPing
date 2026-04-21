mod addons;
mod assets;
mod client;
mod payload;
mod state;
mod text;
mod timestamps;
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
    artwork_server::prepare_artwork_publisher,
    backend_locale::BackendLocale,
    models::{ClientConfig, DiscordPresenceSnapshot},
    rules::normalize_client_config,
};

use self::{
    state::{snapshot_from_inner, DiscordPresenceInner},
    worker::{discord_worker_stopping, validate_discord_presence_config, wait_for_worker_exit},
};

pub(super) const DEFAULT_SYNC_INTERVAL: Duration = Duration::from_secs(5);
pub(super) const MAX_ERROR_BACKOFF_MS: u64 = 30_000;
pub(super) const WORKER_JOIN_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
pub(super) const WORKER_JOIN_POLL_STEP: Duration = Duration::from_millis(50);

pub struct DiscordPresenceRuntime {
    inner: Arc<Mutex<DiscordPresenceInner>>,
    sequence: AtomicU64,
}

impl DiscordPresenceRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DiscordPresenceInner::default())),
            sequence: AtomicU64::new(1),
        }
    }

    pub fn snapshot(&self) -> DiscordPresenceSnapshot {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        snapshot_from_inner(&inner)
    }

    pub fn start(
        &self,
        mut config: ClientConfig,
        locale: BackendLocale,
    ) -> Result<DiscordPresenceSnapshot, String> {
        normalize_client_config(&mut config);
        validate_discord_presence_config(&config, locale)?;

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
                return Err(discord_worker_stopping(locale));
            }
        }

        let artwork_publisher = prepare_artwork_publisher(&config)?;

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.running = true;
            inner.active_run_id = Some(run_id);
            inner.connected = false;
            inner.last_sync_at = None;
            inner.last_error = None;
            inner.current_summary = None;
            inner.debug_payload = None;
            inner.stop_flag = Some(stop_flag.clone());
        }

        let state = self.inner.clone();
        let worker = thread::spawn(move || {
            worker::run_discord_presence_loop(
                state,
                config,
                stop_flag,
                run_id,
                artwork_publisher,
                locale,
            );
        });

        {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.worker = Some(worker);
        }

        Ok(self.snapshot())
    }

    pub fn stop(&self) -> DiscordPresenceSnapshot {
        let worker = {
            let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(flag) = inner.stop_flag.take() {
                flag.store(true, Ordering::SeqCst);
            }
            inner.running = false;
            inner.active_run_id = None;
            inner.connected = false;
            inner.current_summary = None;
            inner.debug_payload = None;
            inner.worker.take()
        };

        if let Some(handle) = worker {
            if let Err(handle) = wait_for_worker_exit(handle) {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = Some(handle);
            } else {
                let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.worker = None;
            }
        }

        self.snapshot()
    }
}

impl Drop for DiscordPresenceRuntime {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
#[allow(unused_imports)]
use crate::{
    models::DiscordReportMode,
    platform::MediaInfo,
    rules::{ResolvedActivity, ResolvedDiscordAddons},
};
#[cfg(test)]
#[allow(unused_imports)]
use self::{
    addons::{build_presence_buttons, select_presence_addons},
    assets::{build_presence_artwork, build_presence_icon},
    payload::{DiscordPresencePayload, DiscordPresenceStatusDisplayType},
    text::{build_music_only_activity_name, build_smart_presence_text, build_status_display_type},
    timestamps::{build_media_timestamps, should_skip_timestamp_update},
};

#[cfg(test)]
mod tests;

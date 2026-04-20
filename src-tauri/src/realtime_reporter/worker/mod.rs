mod activity;
mod capture;
mod lifecycle;
mod runner;

use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::JoinHandle,
};

use crate::models::ClientConfig;

use super::state::ReporterInner;

pub(super) fn run_reporter_loop(
    state: Arc<Mutex<ReporterInner>>,
    config: ClientConfig,
    stop_flag: Arc<AtomicBool>,
    sequence_seed: u64,
    run_id: u64,
) {
    runner::run_reporter_loop(state, config, stop_flag, sequence_seed, run_id);
}

pub(super) fn wait_for_worker_exit(handle: JoinHandle<()>) -> Result<(), JoinHandle<()>> {
    lifecycle::wait_for_worker_exit(handle)
}

pub(super) fn reporter_worker_stopping() -> String {
    lifecycle::reporter_worker_stopping()
}

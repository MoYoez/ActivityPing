mod capture;
mod lifecycle;
mod runner;
mod validation;

use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::JoinHandle,
};

use crate::{
    artwork_server::ArtworkPublisher, backend_locale::BackendLocale, models::ClientConfig,
};

use super::state::DiscordPresenceInner;

pub(super) fn run_discord_presence_loop(
    state: Arc<Mutex<DiscordPresenceInner>>,
    config: ClientConfig,
    stop_flag: Arc<AtomicBool>,
    run_id: u64,
    artwork_publisher: Option<ArtworkPublisher>,
    locale: BackendLocale,
) {
    runner::run_discord_presence_loop(state, config, stop_flag, run_id, artwork_publisher, locale);
}

pub(super) fn validate_discord_presence_config(
    config: &ClientConfig,
    locale: BackendLocale,
) -> Result<(), String> {
    validation::validate_discord_presence_config(config, locale)
}

pub(super) fn discord_worker_stopping(locale: BackendLocale) -> String {
    lifecycle::discord_worker_stopping(locale)
}

pub(super) fn wait_for_worker_exit(handle: JoinHandle<()>) -> Result<(), JoinHandle<()>> {
    lifecycle::wait_for_worker_exit(handle)
}

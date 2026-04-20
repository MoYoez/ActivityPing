use crate::models::{AppFilterMode, ClientConfig};

use super::helpers::normalize_process_name;

pub(super) fn passes_app_filter(config: &ClientConfig, process_name: &str) -> bool {
    let key = normalize_process_name(process_name);
    match config.app_filter_mode {
        AppFilterMode::Whitelist => {
            if config.app_whitelist.is_empty() {
                return false;
            }
            config
                .app_whitelist
                .iter()
                .any(|candidate| normalize_process_name(candidate) == key)
        }
        AppFilterMode::Blacklist => !config
            .app_blacklist
            .iter()
            .any(|candidate| normalize_process_name(candidate) == key),
    }
}

pub(super) fn in_process_list(values: &[String], process_name: &str) -> bool {
    let key = normalize_process_name(process_name);
    values
        .iter()
        .any(|candidate| normalize_process_name(candidate) == key)
}

pub(super) fn is_media_source_blocked(config: &ClientConfig, play_source: &str) -> bool {
    let key = play_source.trim().to_lowercase();
    if key.is_empty() {
        return false;
    }
    config
        .media_play_source_blocklist
        .iter()
        .any(|candidate| candidate.trim().eq_ignore_ascii_case(&key))
}

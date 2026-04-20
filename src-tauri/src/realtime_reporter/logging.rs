use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use serde_json::Value;

use crate::models::ReporterLogEntry;

use super::{state::ReporterInner, MAX_LOGS};

pub(super) struct LogTextSpec {
    pub(super) key: Option<&'static str>,
    pub(super) params: Option<Value>,
    pub(super) fallback: String,
}

pub(super) fn fallback_text(value: impl Into<String>) -> LogTextSpec {
    LogTextSpec {
        key: None,
        params: None,
        fallback: value.into(),
    }
}

pub(super) fn push_background_log(
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

pub(super) fn now_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

pub(super) fn now_iso_string() -> String {
    Utc::now().to_rfc3339()
}

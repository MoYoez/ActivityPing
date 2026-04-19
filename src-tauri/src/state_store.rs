use std::{
    collections::HashSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    models::{AppHistoryEntry, AppStatePayload, PlaySourceHistoryEntry},
    rules::normalize_client_config,
};

const APP_STATE_FILE_NAME: &str = "client-state.json";
const DEFAULT_LOCALE: &str = "en-US";
const MAX_HISTORY_RECORDS: usize = 3;

fn app_state_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("Failed to resolve the app config directory: {error}"))?;
    Ok(dir.join(APP_STATE_FILE_NAME))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create the config directory: {error}"))?;
    }
    Ok(())
}

fn normalize_state(payload: &mut AppStatePayload) {
    if payload.locale.trim().is_empty() {
        payload.locale = DEFAULT_LOCALE.to_string();
    }
    normalize_client_config(&mut payload.config);
    payload.app_history = normalize_app_history(&payload.app_history);
    payload.play_source_history = normalize_play_source_history(&payload.play_source_history);
}

fn normalize_optional_string(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalize_app_history(values: &[AppHistoryEntry]) -> Vec<AppHistoryEntry> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for value in values {
        let process_name = value.process_name.trim();
        if process_name.is_empty() {
            continue;
        }
        let key = process_name.to_lowercase();
        if !seen.insert(key) {
            continue;
        }
        result.push(AppHistoryEntry {
            process_name: process_name.to_string(),
            process_title: normalize_optional_string(&value.process_title),
            status_text: normalize_optional_string(&value.status_text),
            updated_at: normalize_optional_string(&value.updated_at),
        });
        if result.len() >= MAX_HISTORY_RECORDS {
            break;
        }
    }

    result
}

fn normalize_play_source_history(values: &[PlaySourceHistoryEntry]) -> Vec<PlaySourceHistoryEntry> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();

    for value in values {
        let source = value.source.trim().to_lowercase();
        if source.is_empty() {
            continue;
        }
        if !seen.insert(source.clone()) {
            continue;
        }
        result.push(PlaySourceHistoryEntry {
            source,
            media_title: normalize_optional_string(&value.media_title),
            media_artist: normalize_optional_string(&value.media_artist),
            media_album: normalize_optional_string(&value.media_album),
            media_summary: normalize_optional_string(&value.media_summary),
            updated_at: normalize_optional_string(&value.updated_at),
        });
        if result.len() >= MAX_HISTORY_RECORDS {
            break;
        }
    }

    result
}

pub fn load_app_state(app: &AppHandle) -> Result<AppStatePayload, String> {
    let path = app_state_path(app)?;
    let mut payload = match fs::read_to_string(&path) {
        Ok(content) if content.trim().is_empty() => AppStatePayload::default(),
        Ok(content) => match serde_json::from_str(&content) {
            Ok(parsed) => parsed,
            Err(error) => {
                let backup = path.with_extension("json.corrupt");
                if !backup.exists() {
                    let _ = fs::copy(&path, &backup);
                }
                eprintln!(
                    "Client state file is corrupt; backed up to {}: {error}",
                    backup.display()
                );
                AppStatePayload::default()
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => AppStatePayload::default(),
        Err(error) => return Err(format!("Failed to read the state file: {error}")),
    };

    normalize_state(&mut payload);
    Ok(payload)
}

pub fn save_app_state(app: &AppHandle, payload: &AppStatePayload) -> Result<(), String> {
    let path = app_state_path(app)?;
    ensure_parent_dir(&path)?;
    let mut payload = payload.clone();
    normalize_state(&mut payload);
    let content = serde_json::to_string_pretty(&payload)
        .map_err(|error| format!("Failed to serialize the state: {error}"))?;
    atomic_write(&path, &content)?;
    Ok(())
}

fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let tmp_path = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    let mut file = fs::File::create(&tmp_path)
        .map_err(|error| format!("Failed to create the temporary state file: {error}"))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("Failed to write the temporary state file: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("Failed to flush the temporary state file: {error}"))?;
    drop(file);

    set_owner_only_permissions_if_supported(&tmp_path)?;

    fs::rename(&tmp_path, path).map_err(|error| {
        let _ = fs::remove_file(&tmp_path);
        format!("Failed to replace the state file: {error}")
    })?;

    Ok(())
}

#[cfg(unix)]
fn set_owner_only_permissions_if_supported(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|error| format!("Failed to set state file permissions: {error}"))
}

#[cfg(not(unix))]
fn set_owner_only_permissions_if_supported(_path: &Path) -> Result<(), String> {
    Ok(())
}

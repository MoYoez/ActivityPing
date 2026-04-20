use crate::platform::ForegroundSnapshot;

use super::bridge::{read_frontmost_app_name, read_frontmost_window_title};

pub(super) fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let process_name = read_frontmost_app_name()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Failed to read the macOS foreground app.".to_string())?;
    let process_title = read_frontmost_window_title().unwrap_or_default();

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

pub fn get_foreground_snapshot_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    if !include_process_name && !include_process_title {
        return Ok(ForegroundSnapshot::default());
    }

    if include_process_name {
        let mut snapshot = get_foreground_snapshot()?;
        if !include_process_title {
            snapshot.process_title.clear();
        }
        return Ok(snapshot);
    }

    let process_title = if include_process_title {
        read_frontmost_window_title().unwrap_or_default()
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name: String::new(),
        process_title,
    })
}

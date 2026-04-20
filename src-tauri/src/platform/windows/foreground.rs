use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

use crate::platform::ForegroundSnapshot;

use super::process::exe_base_name_from_pid;

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Err(
            "Failed to read the foreground window: GetForegroundWindow returned a null handle."
                .into(),
        );
    }

    let title_len = unsafe { GetWindowTextLengthW(hwnd) };
    let process_title = if title_len <= 0 {
        String::new()
    } else {
        let mut buffer = vec![0u16; title_len as usize + 1];
        let written = unsafe { GetWindowTextW(hwnd, &mut buffer) };
        String::from_utf16_lossy(&buffer[..written as usize])
    };

    let mut pid = 0u32;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    if pid == 0 {
        return Err(
            "Failed to read the foreground window: could not resolve the foreground process ID."
                .into(),
        );
    }

    let process_name = exe_base_name_from_pid(pid).unwrap_or_else(|_| "unknown".to_string());

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

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Err(
            "Failed to read the foreground window: GetForegroundWindow returned a null handle."
                .into(),
        );
    }

    let process_title = if include_process_title {
        let title_len = unsafe { GetWindowTextLengthW(hwnd) };
        if title_len <= 0 {
            String::new()
        } else {
            let mut buffer = vec![0u16; title_len as usize + 1];
            let written = unsafe { GetWindowTextW(hwnd, &mut buffer) };
            String::from_utf16_lossy(&buffer[..written as usize])
        }
    } else {
        String::new()
    };

    let process_name = if include_process_name {
        let mut pid = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        if pid == 0 {
            return Err("Failed to read the foreground window: could not resolve the foreground process ID.".into());
        }
        exe_base_name_from_pid(pid).unwrap_or_else(|_| "unknown".to_string())
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

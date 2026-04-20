use std::{mem::size_of, path::Path};

use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, MAX_PATH},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
    },
};

pub(super) fn exe_base_name_from_pid(pid: u32) -> Result<String, String> {
    let full_path = process_image_path_from_pid(pid)?;
    let file_name = Path::new(&full_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Failed to resolve the foreground process filename.".to_string())?;

    Ok(file_name.to_string())
}

pub(super) fn process_image_path_from_pid(pid: u32) -> Result<String, String> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
        .map_err(|error| format!("OpenProcess failed: {error}"))?;

    let mut buffer = vec![0u16; MAX_PATH as usize * 8];
    let mut size = buffer.len() as u32;
    let query_result = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
    };
    let _ = unsafe { CloseHandle(handle) };

    query_result.map_err(|error| format!("QueryFullProcessImageNameW failed: {error}"))?;

    Ok(String::from_utf16_lossy(&buffer[..size as usize]))
}

pub(super) fn resolve_process_image_path_from_source_app_id(source_app_id: &str) -> Option<String> {
    let trimmed = source_app_id.trim();
    if trimmed.is_empty() {
        return None;
    }

    if (trimmed.contains('\\') || trimmed.contains('/')) && Path::new(trimmed).exists() {
        return Some(trimmed.to_string());
    }

    let candidates = process_name_candidates(trimmed);
    if candidates.is_empty() {
        return None;
    }

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.ok()?;
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut path = None;
    if unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok() {
        loop {
            let executable_name = utf16z_to_string(&entry.szExeFile);
            if matches_process_candidate(&executable_name, &candidates) {
                if let Ok(executable_path) = process_image_path_from_pid(entry.th32ProcessID) {
                    path = Some(executable_path);
                    break;
                }
            }

            if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
                break;
            }
        }
    }

    let _ = unsafe { CloseHandle(snapshot) };
    path
}

fn process_name_candidates(source_app_id: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut push_candidate = |value: &str| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return;
        }
        if !candidates
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(trimmed))
        {
            candidates.push(trimmed.to_string());
        }
    };

    let tail = source_app_id
        .trim()
        .rsplit(['\\', '/', '!'])
        .next()
        .unwrap_or(source_app_id.trim());
    let tail = tail.split('_').next().unwrap_or(tail);
    push_candidate(tail);

    let dotted_tail = tail.rsplit('.').next().unwrap_or(tail);
    push_candidate(dotted_tail);

    if !tail.to_ascii_lowercase().ends_with(".exe") {
        push_candidate(&format!("{tail}.exe"));
    }
    if !dotted_tail.to_ascii_lowercase().ends_with(".exe") {
        push_candidate(&format!("{dotted_tail}.exe"));
    }

    candidates
}

fn matches_process_candidate(executable_name: &str, candidates: &[String]) -> bool {
    candidates
        .iter()
        .any(|candidate| executable_name.eq_ignore_ascii_case(candidate))
}

pub(super) fn utf16z_to_string(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

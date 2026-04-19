use std::{
    collections::HashMap,
    ffi::c_void,
    mem::size_of,
    path::Path,
    sync::{Mutex, OnceLock},
};

use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use serde_json::json;
use windows::{
    core::{HRESULT, HSTRING, PCWSTR, PWSTR},
    ApplicationModel::AppInfo,
    Foundation::{Size, TimeSpan},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Storage::Streams::{DataReader, RandomAccessStreamReference},
    Win32::{
        Foundation::{CloseHandle, HWND, MAX_PATH},
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        },
        System::{
            Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::{
            Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
            WindowsAndMessaging::{
                DestroyIcon, DrawIconEx, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
                GetWindowThreadProcessId, DI_NORMAL,
            },
        },
    },
};

use super::{build_self_test_result, localized_text, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

const SOURCE_ICON_SIZE: i32 = 128;

fn source_icon_cache() -> &'static Mutex<HashMap<String, Option<super::MediaArtwork>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<super::MediaArtwork>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn foreground_app_icon_cache() -> &'static Mutex<HashMap<String, Option<super::MediaArtwork>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<super::MediaArtwork>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

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

fn exe_base_name_from_pid(pid: u32) -> Result<String, String> {
    let full_path = process_image_path_from_pid(pid)?;
    let file_name = Path::new(&full_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Failed to resolve the foreground process filename.".to_string())?;

    Ok(file_name.to_string())
}

fn process_image_path_from_pid(pid: u32) -> Result<String, String> {
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

#[cfg(target_os = "windows")]
struct ComInitGuard {
    should_uninitialize: bool,
}

#[cfg(target_os = "windows")]
impl Drop for ComInitGuard {
    fn drop(&mut self) {
        if self.should_uninitialize {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn init_com_for_media() -> Result<ComInitGuard, String> {
    const RPC_E_CHANGED_MODE: HRESULT = HRESULT(0x80010106u32 as i32);

    let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

    if result.is_ok() {
        return Ok(ComInitGuard {
            should_uninitialize: true,
        });
    }

    if result == RPC_E_CHANGED_MODE {
        return Ok(ComInitGuard {
            should_uninitialize: false,
        });
    }

    Err(format!("Failed to initialize WinRT: {result:?}"))
}

#[cfg(target_os = "windows")]
fn get_now_playing_native(include_assets: bool) -> Result<MediaInfo, String> {
    let _com_guard = init_com_for_media()?;

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|error| format!("Failed to request the media session manager: {error}"))?
        .get()
        .map_err(|error| format!("Failed to obtain the media session manager: {error}"))?;

    let session = manager
        .GetCurrentSession()
        .map_err(|error| format!("Failed to read the current media session: {error}"))?;

    let source_app_id = session
        .SourceAppUserModelId()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let is_playing = read_playback_status(&session);

    let properties = session
        .TryGetMediaPropertiesAsync()
        .map_err(|error| format!("Failed to request media properties: {error}"))?
        .get()
        .map_err(|error| format!("Failed to read media properties: {error}"))?;

    let title = properties
        .Title()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let artist = properties
        .Artist()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let album = properties
        .AlbumTitle()
        .ok()
        .map(|value| value.to_string())
        .unwrap_or_default();
    let (duration_ms, position_ms) = read_media_timeline(&session);
    let artwork = if include_assets {
        read_media_artwork(&properties).unwrap_or_default()
    } else {
        None
    };
    let source_icon = if include_assets {
        read_source_app_icon(&source_app_id)
    } else {
        None
    };

    let media = MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        is_playing,
        duration_ms,
        position_ms,
        artwork,
        source_icon,
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    get_now_playing_native(true)
}

fn get_now_playing_for_self_test() -> Result<MediaInfo, String> {
    get_now_playing_native(false)
}

pub fn get_foreground_app_icon() -> Result<Option<super::MediaArtwork>, String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Err(
            "Failed to read the foreground window: GetForegroundWindow returned a null handle."
                .into(),
        );
    }

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

    let executable_path = process_image_path_from_pid(pid)?;
    let cache_key = executable_path.trim();
    if cache_key.is_empty() {
        return Ok(None);
    }

    if let Some(cached) = foreground_app_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(cache_key)
        .cloned()
    {
        return Ok(cached);
    }

    let icon = render_executable_icon_png(cache_key)
        .ok()
        .and_then(|bytes| {
            if bytes.is_empty() {
                None
            } else {
                Some(super::MediaArtwork {
                    bytes,
                    content_type: "image/png".to_string(),
                })
            }
        });

    foreground_app_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key.to_string(), icon.clone());

    Ok(icon)
}

#[cfg(target_os = "windows")]
fn read_playback_status(session: &GlobalSystemMediaTransportControlsSession) -> bool {
    session
        .GetPlaybackInfo()
        .ok()
        .and_then(|info| info.PlaybackStatus().ok())
        .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn read_source_app_icon(source_app_id: &str) -> Option<super::MediaArtwork> {
    let cache_key = source_app_id.trim();
    if cache_key.is_empty() {
        return None;
    }

    if let Some(cached) = source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(cache_key)
        .cloned()
    {
        return cached;
    }

    let icon = read_packaged_app_icon(cache_key)
        .or_else(|| read_process_app_icon(cache_key).ok().flatten());

    source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key.to_string(), icon.clone());

    icon
}

#[cfg(target_os = "windows")]
fn read_packaged_app_icon(source_app_id: &str) -> Option<super::MediaArtwork> {
    let app_info = AppInfo::GetFromAppUserModelId(&HSTRING::from(source_app_id)).ok()?;
    let display_info = app_info.DisplayInfo().ok()?;
    let logo_stream = display_info
        .GetLogo(Size {
            Width: SOURCE_ICON_SIZE as f32,
            Height: SOURCE_ICON_SIZE as f32,
        })
        .ok()?;

    read_stream_reference_artwork(&logo_stream).ok().flatten()
}

#[cfg(target_os = "windows")]
fn read_process_app_icon(source_app_id: &str) -> Result<Option<super::MediaArtwork>, String> {
    let executable_path = match resolve_process_image_path_from_source_app_id(source_app_id) {
        Some(path) => path,
        None => return Ok(None),
    };
    let bytes = render_executable_icon_png(&executable_path)?;
    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(super::MediaArtwork {
        bytes,
        content_type: "image/png".to_string(),
    }))
}

#[cfg(target_os = "windows")]
fn read_stream_reference_artwork(
    reference: &RandomAccessStreamReference,
) -> Result<Option<super::MediaArtwork>, String> {
    let stream = reference
        .OpenReadAsync()
        .map_err(|error| format!("Failed to request the app icon stream: {error}"))?
        .get()
        .map_err(|error| format!("Failed to read the app icon stream: {error}"))?;

    let size = stream
        .Size()
        .map_err(|error| format!("Failed to read the app icon size: {error}"))?
        as u32;
    if size == 0 {
        return Ok(None);
    }

    let input_stream = stream
        .GetInputStreamAt(0)
        .map_err(|error| format!("Failed to read the app icon input stream: {error}"))?;
    let reader = DataReader::CreateDataReader(&input_stream)
        .map_err(|error| format!("Failed to create the app icon reader: {error}"))?;
    reader
        .LoadAsync(size)
        .map_err(|error| format!("Failed to request the app icon buffer: {error}"))?
        .get()
        .map_err(|error| format!("Failed to load the app icon buffer: {error}"))?;

    let mut bytes = vec![0u8; size as usize];
    reader
        .ReadBytes(&mut bytes)
        .map_err(|error| format!("Failed to read the app icon bytes: {error}"))?;

    if bytes.is_empty() {
        return Ok(None);
    }

    let content_type = stream
        .ContentType()
        .ok()
        .map(|value| value.to_string())
        .filter(|value| value.starts_with("image/"))
        .unwrap_or_else(|| "image/png".to_string());

    Ok(Some(super::MediaArtwork {
        bytes,
        content_type,
    }))
}

#[cfg(target_os = "windows")]
fn resolve_process_image_path_from_source_app_id(source_app_id: &str) -> Option<String> {
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

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
fn matches_process_candidate(executable_name: &str, candidates: &[String]) -> bool {
    candidates
        .iter()
        .any(|candidate| executable_name.eq_ignore_ascii_case(candidate))
}

#[cfg(target_os = "windows")]
fn utf16z_to_string(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

#[cfg(target_os = "windows")]
fn render_executable_icon_png(executable_path: &str) -> Result<Vec<u8>, String> {
    let wide_path = encode_wide(executable_path);
    let mut file_info = SHFILEINFOW::default();
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            Default::default(),
            Some(&mut file_info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if result == 0 || file_info.hIcon.is_invalid() {
        return Err("Failed to read the app icon handle.".to_string());
    }

    let hicon = file_info.hIcon;
    let render_result = render_hicon_png(hicon);
    let _ = unsafe { DestroyIcon(hicon) };
    render_result
}

#[cfg(target_os = "windows")]
fn render_hicon_png(
    hicon: windows::Win32::UI::WindowsAndMessaging::HICON,
) -> Result<Vec<u8>, String> {
    let screen_dc = unsafe { GetDC(HWND(std::ptr::null_mut())) };
    if screen_dc.is_invalid() {
        return Err("Failed to create the screen drawing context.".to_string());
    }

    let memory_dc = unsafe { CreateCompatibleDC(screen_dc) };
    if memory_dc.is_invalid() {
        let _ = unsafe { ReleaseDC(HWND(std::ptr::null_mut()), screen_dc) };
        return Err("Failed to create the memory drawing context.".to_string());
    }

    let mut bitmap_info = BITMAPINFO::default();
    bitmap_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bitmap_info.bmiHeader.biWidth = SOURCE_ICON_SIZE;
    bitmap_info.bmiHeader.biHeight = -SOURCE_ICON_SIZE;
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32;
    bitmap_info.bmiHeader.biCompression = BI_RGB.0;

    let mut bits_ptr = std::ptr::null_mut::<c_void>();
    let bitmap = unsafe {
        CreateDIBSection(
            screen_dc,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0,
        )
    }
    .map_err(|error| format!("Failed to create the icon bitmap: {error}"))?;

    let old_object = unsafe { SelectObject(memory_dc, bitmap) };
    let draw_result = unsafe {
        DrawIconEx(
            memory_dc,
            0,
            0,
            hicon,
            SOURCE_ICON_SIZE,
            SOURCE_ICON_SIZE,
            0,
            None,
            DI_NORMAL,
        )
    };

    let _ = unsafe { SelectObject(memory_dc, old_object) };
    let _ = unsafe { DeleteDC(memory_dc) };
    let _ = unsafe { ReleaseDC(HWND(std::ptr::null_mut()), screen_dc) };

    draw_result.map_err(|error| format!("Failed to draw the app icon: {error}"))?;
    if bits_ptr.is_null() {
        let _ = unsafe { DeleteObject(bitmap) };
        return Err("The app icon bitmap buffer is empty.".to_string());
    }

    let pixel_len = (SOURCE_ICON_SIZE as usize)
        .saturating_mul(SOURCE_ICON_SIZE as usize)
        .saturating_mul(4);
    let raw_bgra = unsafe { std::slice::from_raw_parts(bits_ptr as *const u8, pixel_len) };
    let mut rgba = raw_bgra.to_vec();
    let _ = unsafe { DeleteObject(bitmap) };

    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(
            &rgba,
            SOURCE_ICON_SIZE as u32,
            SOURCE_ICON_SIZE as u32,
            ColorType::Rgba8.into(),
        )
        .map_err(|error| format!("Failed to encode the app icon PNG: {error}"))?;

    Ok(png)
}

#[cfg(target_os = "windows")]
fn encode_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
fn read_media_timeline(
    session: &GlobalSystemMediaTransportControlsSession,
) -> (Option<u64>, Option<u64>) {
    let timeline = match session.GetTimelineProperties() {
        Ok(timeline) => timeline,
        Err(_) => return (None, None),
    };

    let start_ms = timeline
        .StartTime()
        .ok()
        .and_then(windows_timespan_to_millis)
        .unwrap_or(0);
    let end_ms = timeline
        .EndTime()
        .ok()
        .and_then(windows_timespan_to_millis)
        .unwrap_or(0);
    let raw_position_ms = timeline
        .Position()
        .ok()
        .and_then(windows_timespan_to_millis)
        .unwrap_or(0);

    let duration_ms = end_ms
        .checked_sub(start_ms)
        .filter(|value| *value > 0)
        .or_else(|| (end_ms > 0).then_some(end_ms));
    let position_base = raw_position_ms
        .checked_sub(start_ms)
        .unwrap_or(raw_position_ms);
    let position_ms = if duration_ms.is_some() || position_base > 0 {
        Some(
            duration_ms
                .map(|duration| position_base.min(duration))
                .unwrap_or(position_base),
        )
    } else {
        None
    };

    (duration_ms, position_ms)
}

#[cfg(target_os = "windows")]
fn windows_timespan_to_millis(value: TimeSpan) -> Option<u64> {
    (value.Duration >= 0).then_some((value.Duration as u64) / 10_000)
}

#[cfg(target_os = "windows")]
fn read_media_artwork(
    properties: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<Option<super::MediaArtwork>, String> {
    let thumbnail = match properties.Thumbnail() {
        Ok(thumbnail) => thumbnail,
        Err(_) => return Ok(None),
    };
    let stream = thumbnail
        .OpenReadAsync()
        .map_err(|error| format!("Failed to request the media thumbnail: {error}"))?
        .get()
        .map_err(|error| format!("Failed to read the media thumbnail: {error}"))?;

    let size = stream
        .Size()
        .map_err(|error| format!("Failed to read the media thumbnail size: {error}"))?
        as u32;
    if size == 0 {
        return Ok(None);
    }

    let input_stream = stream
        .GetInputStreamAt(0)
        .map_err(|error| format!("Failed to read the media thumbnail input stream: {error}"))?;
    let reader = DataReader::CreateDataReader(&input_stream)
        .map_err(|error| format!("Failed to create the media thumbnail reader: {error}"))?;
    reader
        .LoadAsync(size)
        .map_err(|error| format!("Failed to request the media thumbnail buffer: {error}"))?
        .get()
        .map_err(|error| format!("Failed to load the media thumbnail buffer: {error}"))?;

    let mut bytes = vec![0u8; size as usize];
    reader
        .ReadBytes(&mut bytes)
        .map_err(|error| format!("Failed to read the media thumbnail bytes: {error}"))?;

    if bytes.is_empty() {
        return Ok(None);
    }

    let content_type = stream
        .ContentType()
        .ok()
        .map(|value| value.to_string())
        .filter(|value| value.starts_with("image/"))
        .unwrap_or_else(|| "image/jpeg".to_string());

    Ok(Some(super::MediaArtwork {
        bytes,
        content_type,
    }))
}

pub fn run_self_test() -> PlatformSelfTestResult {
    let foreground = match get_foreground_snapshot() {
        Ok(snapshot) => make_probe(
            true,
            localized_text(
                "platformSelfTest.summary.foregroundOk",
                None,
                "Foreground app capture OK",
            ),
            localized_text(
                "platformSelfTest.detail.foregroundCurrent",
                Some(json!({ "processName": snapshot.process_name.clone() })),
                format!("Current foreground app: {}", snapshot.process_name),
            ),
            Vec::new(),
        ),
        Err(error) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.foregroundFailed",
                None,
                "Foreground app capture failed",
            ),
            localized_text("platformSelfTest.detail.foregroundReadFailed", None, error),
            Vec::new(),
        ),
    };

    let window_title = match get_foreground_snapshot() {
        Ok(snapshot) => make_probe(
            !snapshot.process_title.trim().is_empty(),
            if snapshot.process_title.trim().is_empty() {
                localized_text(
                    "platformSelfTest.summary.windowTitleEmpty",
                    None,
                    "Window title is empty",
                )
            } else {
                localized_text(
                    "platformSelfTest.summary.windowTitleOk",
                    None,
                    "Window title capture OK",
                )
            },
            if snapshot.process_title.trim().is_empty() {
                localized_text(
                    "platformSelfTest.detail.windowTitleEmpty",
                    None,
                    "The current foreground window has no usable title.",
                )
            } else {
                localized_text(
                    "platformSelfTest.detail.windowTitleCurrent",
                    Some(json!({ "processTitle": snapshot.process_title.clone() })),
                    snapshot.process_title,
                )
            },
            Vec::new(),
        ),
        Err(error) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.windowTitleFailed",
                None,
                "Window title capture failed",
            ),
            localized_text("platformSelfTest.detail.windowTitleReadFailed", None, error),
            Vec::new(),
        ),
    };

    let media = match get_now_playing_for_self_test() {
        Ok(info) if info.is_active() => make_probe(
            true,
            localized_text("platformSelfTest.summary.mediaOk", None, "Media capture OK"),
            localized_text(
                "platformSelfTest.detail.mediaCurrent",
                Some(json!({ "mediaSummary": info.summary() })),
                info.summary(),
            ),
            Vec::new(),
        ),
        Ok(_) => make_probe(
            true,
            localized_text(
                "platformSelfTest.summary.mediaNone",
                None,
                "No media is currently playing",
            ),
            localized_text(
                "platformSelfTest.detail.mediaNone",
                None,
                "No now-playing media information is currently available.",
            ),
            Vec::new(),
        ),
        Err(error) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.mediaFailed",
                None,
                "Media capture failed",
            ),
            localized_text("platformSelfTest.detail.mediaReadFailed", None, error),
            Vec::new(),
        ),
    };

    build_self_test_result(foreground, window_title, media)
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Err("Accessibility permission requests are not supported on this platform.".into())
}

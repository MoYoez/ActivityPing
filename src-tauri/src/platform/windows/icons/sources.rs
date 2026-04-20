use windows::{
    core::HSTRING,
    ApplicationModel::AppInfo,
    Foundation::Size,
    Storage::Streams::{DataReader, RandomAccessStreamReference},
    Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

use crate::platform::MediaArtwork;

use super::{super::APP_ICON_SIZE, render::render_executable_icon_png};
use crate::platform::windows::process::resolve_process_image_path_from_source_app_id;

pub(super) fn foreground_process_id() -> Result<u32, String> {
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

    Ok(pid)
}

pub(super) fn read_source_app_icon_uncached(source_app_id: &str) -> Option<MediaArtwork> {
    read_packaged_app_icon(source_app_id)
        .or_else(|| read_process_app_icon(source_app_id).ok().flatten())
}

fn read_packaged_app_icon(source_app_id: &str) -> Option<MediaArtwork> {
    let app_info = AppInfo::GetFromAppUserModelId(&HSTRING::from(source_app_id)).ok()?;
    let display_info = app_info.DisplayInfo().ok()?;
    let logo_stream = display_info
        .GetLogo(Size {
            Width: APP_ICON_SIZE as f32,
            Height: APP_ICON_SIZE as f32,
        })
        .ok()?;

    read_stream_reference_artwork(&logo_stream).ok().flatten()
}

fn read_process_app_icon(source_app_id: &str) -> Result<Option<MediaArtwork>, String> {
    let executable_path = match resolve_process_image_path_from_source_app_id(source_app_id) {
        Some(path) => path,
        None => return Ok(None),
    };
    let bytes = render_executable_icon_png(&executable_path, APP_ICON_SIZE)?;
    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(MediaArtwork {
        bytes,
        content_type: "image/png".to_string(),
    }))
}

fn read_stream_reference_artwork(
    reference: &RandomAccessStreamReference,
) -> Result<Option<MediaArtwork>, String> {
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

    Ok(Some(MediaArtwork {
        bytes,
        content_type,
    }))
}

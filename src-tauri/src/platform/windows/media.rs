use std::cell::RefCell;

use windows::{
    core::HRESULT,
    Foundation::TimeSpan,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionMediaProperties,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Storage::Streams::DataReader,
    Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
};

use crate::platform::{MediaArtwork, MediaCaptureOptions, MediaInfo};

use super::icons::read_source_app_icon;

std::thread_local! {
    static MEDIA_SESSION_MANAGER: RefCell<Option<GlobalSystemMediaTransportControlsSessionManager>> =
        RefCell::new(None);
}

struct ComInitGuard {
    should_uninitialize: bool,
}

impl Drop for ComInitGuard {
    fn drop(&mut self) {
        if self.should_uninitialize {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

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

fn get_cached_media_session_manager(
) -> Result<GlobalSystemMediaTransportControlsSessionManager, String> {
    MEDIA_SESSION_MANAGER.with(|cache| {
        if let Some(manager) = cache.borrow().as_ref().cloned() {
            return Ok(manager);
        }

        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .map_err(|error| format!("Failed to request the media session manager: {error}"))?
            .get()
            .map_err(|error| format!("Failed to obtain the media session manager: {error}"))?;

        *cache.borrow_mut() = Some(manager.clone());
        Ok(manager)
    })
}

fn clear_cached_media_session_manager() {
    MEDIA_SESSION_MANAGER.with(|cache| {
        cache.borrow_mut().take();
    });
}

fn get_now_playing_native(options: MediaCaptureOptions) -> Result<MediaInfo, String> {
    let _com_guard = init_com_for_media()?;

    let manager = get_cached_media_session_manager()?;
    let session = match manager.GetCurrentSession() {
        Ok(session) => session,
        Err(_) => {
            clear_cached_media_session_manager();
            get_cached_media_session_manager()?
                .GetCurrentSession()
                .map_err(|error| format!("Failed to read the current media session: {error}"))?
        }
    };

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
    let artwork = if options.include_artwork {
        read_media_artwork(&properties).unwrap_or_default()
    } else {
        None
    };
    let source_icon = if options.include_source_icon {
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

pub fn get_now_playing_with_options(options: MediaCaptureOptions) -> Result<MediaInfo, String> {
    get_now_playing_native(options)
}

pub(super) fn get_now_playing_for_self_test() -> Result<MediaInfo, String> {
    get_now_playing_native(MediaCaptureOptions::default())
}

fn read_playback_status(session: &GlobalSystemMediaTransportControlsSession) -> bool {
    session
        .GetPlaybackInfo()
        .ok()
        .and_then(|info| info.PlaybackStatus().ok())
        .map(|status| status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
        .unwrap_or(false)
}

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

fn windows_timespan_to_millis(value: TimeSpan) -> Option<u64> {
    (value.Duration >= 0).then_some((value.Duration as u64) / 10_000)
}

fn read_media_artwork(
    properties: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Result<Option<MediaArtwork>, String> {
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

    Ok(Some(MediaArtwork {
        bytes,
        content_type,
    }))
}

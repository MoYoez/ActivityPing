mod render;
mod sources;

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use crate::platform::MediaArtwork;

use super::{process::process_image_path_from_pid, APP_ICON_SIZE};

fn source_icon_cache() -> &'static Mutex<HashMap<String, Option<MediaArtwork>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<MediaArtwork>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn foreground_app_icon_cache() -> &'static Mutex<HashMap<String, Option<MediaArtwork>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<MediaArtwork>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_foreground_app_icon() -> Result<Option<MediaArtwork>, String> {
    let executable_path = process_image_path_from_pid(sources::foreground_process_id()?)?;
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

    let icon = render::render_executable_icon_png(cache_key, APP_ICON_SIZE)
        .ok()
        .and_then(|bytes| {
            if bytes.is_empty() {
                None
            } else {
                Some(MediaArtwork {
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

pub(super) fn read_source_app_icon(source_app_id: &str) -> Option<MediaArtwork> {
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

    let icon = sources::read_source_app_icon_uncached(cache_key);

    source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key.to_string(), icon.clone());

    icon
}

use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use crate::platform::MediaArtwork;

use super::{
    desktop_entries::resolve_linux_desktop_entry,
    foreground::get_foreground_snapshot_for_reporting,
    theme_icons::{read_icon_file, resolve_desktop_icon_path},
};

fn source_icon_cache() -> &'static Mutex<HashMap<String, Option<MediaArtwork>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<MediaArtwork>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_foreground_app_icon() -> Result<Option<MediaArtwork>, String> {
    let snapshot = get_foreground_snapshot_for_reporting(true, false)?;
    Ok(read_source_app_icon(&snapshot.process_name))
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

    let icon = resolve_linux_desktop_entry(cache_key)
        .and_then(|entry| resolve_desktop_icon_path(&entry))
        .and_then(|path| read_icon_file(&path));

    source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key.to_string(), icon.clone());

    icon
}

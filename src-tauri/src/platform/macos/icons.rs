use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use crate::platform::MediaArtwork;

use super::{
    bridge::{read_bundle_icon_png_base64, read_frontmost_app_bundle_identifier},
    images::decode_base64_image_payload,
    APP_ICON_RENDER_SIZE,
};

fn source_icon_cache() -> &'static Mutex<HashMap<String, Option<MediaArtwork>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<MediaArtwork>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_foreground_app_icon() -> Result<Option<MediaArtwork>, String> {
    let bundle_identifier = read_frontmost_app_bundle_identifier().unwrap_or_default();
    Ok(read_bundle_icon(&bundle_identifier))
}

pub(super) fn read_source_app_icon(bundle_identifier: &str) -> Option<MediaArtwork> {
    read_bundle_icon(bundle_identifier)
}

fn read_bundle_icon(bundle_identifier: &str) -> Option<MediaArtwork> {
    let cache_key = bundle_identifier.trim();
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

    let icon = read_bundle_icon_png_base64(cache_key, APP_ICON_RENDER_SIZE)
        .and_then(|decoded| decode_base64_image_payload(&decoded))
        .map(|bytes| MediaArtwork {
            bytes,
            content_type: "image/png".to_string(),
        });

    source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key.to_string(), icon.clone());

    icon
}

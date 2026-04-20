use std::{
    fs,
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use chrono::Utc;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::{
    models::{
        DiscordCustomAppIconSource, DiscordCustomAsset, DiscordCustomArtworkSource,
    },
    state_store,
};

const CUSTOM_ASSET_DIR: &str = "discord-custom-assets";

pub fn import_discord_custom_asset(
    app: &AppHandle,
    name: &str,
    file_name: &str,
    content_type: &str,
    base64_data: &str,
) -> Result<Vec<DiscordCustomAsset>, String> {
    let normalized_content_type = normalize_content_type(content_type)?;
    let bytes = BASE64_STANDARD
        .decode(base64_data.trim())
        .map_err(|error| format!("Failed to decode the uploaded image: {error}"))?;
    if bytes.is_empty() {
        return Err("The uploaded image is empty.".into());
    }
    image::load_from_memory(&bytes)
        .map_err(|error| format!("Failed to read the uploaded image: {error}"))?;

    let file_name = normalize_file_name(file_name);
    let name = normalize_asset_name(name, &file_name);
    let asset_id = Uuid::new_v4().to_string();
    let extension = content_type_extension(&normalized_content_type);
    let stored_path = custom_asset_dir(app)?.join(format!("{asset_id}.{extension}"));
    ensure_parent_dir(&stored_path)?;
    fs::write(&stored_path, &bytes)
        .map_err(|error| format!("Failed to store the local resource: {error}"))?;

    let mut state = state_store::load_app_state(app)?;
    state.config.discord_custom_assets.push(DiscordCustomAsset {
        id: asset_id,
        name,
        file_name,
        stored_path: stored_path.to_string_lossy().to_string(),
        content_type: normalized_content_type,
        byte_size: bytes.len() as u64,
        created_at: Utc::now().to_rfc3339(),
    });
    state_store::save_app_state(app, &state)?;

    Ok(state.config.discord_custom_assets)
}

pub fn delete_discord_custom_asset(
    app: &AppHandle,
    asset_id: &str,
) -> Result<Vec<DiscordCustomAsset>, String> {
    let trimmed_id = asset_id.trim();
    if trimmed_id.is_empty() {
        return Err("A local resource id is required.".into());
    }

    let mut state = state_store::load_app_state(app)?;
    let mut removed_path = None;
    let original_len = state.config.discord_custom_assets.len();
    state.config.discord_custom_assets.retain(|asset| {
        if asset.id == trimmed_id {
            removed_path = Some(asset.stored_path.clone());
            false
        } else {
            true
        }
    });

    if state.config.discord_custom_assets.len() == original_len {
        return Err("The selected local resource could not be found.".into());
    }

    clear_asset_references(&mut state.config, trimmed_id);
    state_store::save_app_state(app, &state)?;

    if let Some(path) = removed_path {
        let _ = fs::remove_file(path);
    }

    Ok(state.config.discord_custom_assets)
}

pub fn get_discord_custom_asset_preview(
    app: &AppHandle,
    asset_id: &str,
) -> Result<String, String> {
    let trimmed_id = asset_id.trim();
    if trimmed_id.is_empty() {
        return Err("A local resource id is required.".into());
    }

    let state = state_store::load_app_state(app)?;
    let asset = state
        .config
        .discord_custom_assets
        .iter()
        .find(|asset| asset.id == trimmed_id)
        .ok_or_else(|| "The selected local resource could not be found.".to_string())?;
    let bytes = fs::read(&asset.stored_path)
        .map_err(|error| format!("Failed to read the local resource: {error}"))?;
    let content_type = normalize_content_type(&asset.content_type)?;
    Ok(format!(
        "data:{};base64,{}",
        content_type,
        BASE64_STANDARD.encode(bytes)
    ))
}

fn clear_asset_references(config: &mut crate::models::ClientConfig, asset_id: &str) {
    if config.discord_custom_artwork_source == DiscordCustomArtworkSource::Library
        && config.discord_custom_artwork_asset_id.trim() == asset_id
    {
        config.discord_custom_artwork_source = DiscordCustomArtworkSource::None;
    }
    if config.discord_custom_app_icon_source == DiscordCustomAppIconSource::Library
        && config.discord_custom_app_icon_asset_id.trim() == asset_id
    {
        config.discord_custom_app_icon_source = DiscordCustomAppIconSource::None;
    }
    if config.discord_custom_artwork_asset_id.trim() == asset_id {
        config.discord_custom_artwork_asset_id.clear();
    }
    if config.discord_custom_app_icon_asset_id.trim() == asset_id {
        config.discord_custom_app_icon_asset_id.clear();
    }

    for preset in &mut config.discord_custom_presets {
        if preset.custom_artwork_source == DiscordCustomArtworkSource::Library
            && preset.custom_artwork_asset_id.trim() == asset_id
        {
            preset.custom_artwork_source = DiscordCustomArtworkSource::None;
        }
        if preset.custom_app_icon_source == DiscordCustomAppIconSource::Library
            && preset.custom_app_icon_asset_id.trim() == asset_id
        {
            preset.custom_app_icon_source = DiscordCustomAppIconSource::None;
        }
        if preset.custom_artwork_asset_id.trim() == asset_id {
            preset.custom_artwork_asset_id.clear();
        }
        if preset.custom_app_icon_asset_id.trim() == asset_id {
            preset.custom_app_icon_asset_id.clear();
        }
    }
}

fn custom_asset_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("Failed to resolve the app config directory: {error}"))?;
    Ok(dir.join(CUSTOM_ASSET_DIR))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create the local resource directory: {error}"))?;
    }
    Ok(())
}

fn normalize_content_type(value: &str) -> Result<String, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "image/png" => Ok("image/png".to_string()),
        "image/jpeg" | "image/jpg" => Ok("image/jpeg".to_string()),
        _ => Err("Only PNG and JPEG images are supported.".into()),
    }
}

fn content_type_extension(content_type: &str) -> &'static str {
    if content_type == "image/png" {
        "png"
    } else {
        "jpg"
    }
}

fn normalize_file_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "custom-asset".to_string();
    }

    trimmed
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => ch,
        })
        .collect()
}

fn normalize_asset_name(name: &str, file_name: &str) -> String {
    let trimmed = name.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.trim())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("Custom asset")
        .to_string()
}

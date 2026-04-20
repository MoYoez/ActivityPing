use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::platform::MediaArtwork;

use super::desktop_entries::{normalize_linux_id, LinuxDesktopEntry};

const ICON_SEARCH_DEPTH: usize = 5;
const RASTER_ICON_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg"];
const PREFERRED_ICON_SIZES: [i32; 5] = [256, 192, 128, 96, 64];

pub(super) fn resolve_desktop_icon_path(entry: &LinuxDesktopEntry) -> Option<PathBuf> {
    let icon = entry.icon.trim();
    if icon.is_empty() {
        return None;
    }

    if let Some(path) = resolve_explicit_icon_path(icon, &entry.path) {
        return Some(path);
    }

    let normalized_icon = icon.trim_end_matches(".svg").trim_end_matches(".png");
    find_themed_icon_path(normalized_icon)
}

pub(super) fn read_icon_file(path: &Path) -> Option<MediaArtwork> {
    let content_type = match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => return None,
    };

    let bytes = fs::read(path).ok()?;
    if bytes.is_empty() {
        return None;
    }

    Some(MediaArtwork {
        bytes,
        content_type: content_type.to_string(),
    })
}

fn resolve_explicit_icon_path(icon: &str, desktop_file_path: &Path) -> Option<PathBuf> {
    let icon_path = Path::new(icon);
    if icon_path.is_absolute() {
        return resolve_raster_icon_variant(icon_path);
    }

    let relative_path = desktop_file_path
        .parent()
        .map(|parent| parent.join(icon))
        .unwrap_or_else(|| PathBuf::from(icon));
    resolve_raster_icon_variant(&relative_path)
}

fn resolve_raster_icon_variant(path: &Path) -> Option<PathBuf> {
    if path.is_file() && is_supported_raster_icon(path) {
        return Some(path.to_path_buf());
    }
    if path.extension().is_none() {
        for extension in RASTER_ICON_EXTENSIONS {
            let candidate = path.with_extension(extension);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn find_themed_icon_path(icon_name: &str) -> Option<PathBuf> {
    let normalized_icon = normalize_linux_id(icon_name);
    if normalized_icon.is_empty() {
        return None;
    }

    let mut best_match: Option<(i32, PathBuf)> = None;
    for directory in icon_search_dirs() {
        find_icon_in_directory(&directory, &normalized_icon, 0, &mut best_match);
    }

    best_match.map(|(_, path)| path)
}

fn find_icon_in_directory(
    directory: &Path,
    icon_name: &str,
    depth: usize,
    best_match: &mut Option<(i32, PathBuf)>,
) {
    if depth > ICON_SEARCH_DEPTH {
        return;
    }

    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            find_icon_in_directory(&path, icon_name, depth + 1, best_match);
            continue;
        }
        if !is_supported_raster_icon(&path) {
            continue;
        }

        let file_stem = path
            .file_stem()
            .and_then(|value| value.to_str())
            .map(normalize_linux_id)
            .unwrap_or_default();
        if file_stem != icon_name {
            continue;
        }

        let score = score_icon_path(&path);
        let replace = best_match
            .as_ref()
            .map(|(best_score, _)| score > *best_score)
            .unwrap_or(true);
        if replace {
            *best_match = Some((score, path));
        }
    }
}

fn score_icon_path(path: &Path) -> i32 {
    let extension_score = match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => 30,
        "jpg" | "jpeg" => 20,
        _ => 0,
    };

    let size_score = path
        .ancestors()
        .filter_map(|ancestor| ancestor.file_name().and_then(|value| value.to_str()))
        .filter_map(parse_icon_size_component)
        .map(score_icon_size)
        .max()
        .unwrap_or(0);

    extension_score + size_score
}

fn score_icon_size(size: i32) -> i32 {
    PREFERRED_ICON_SIZES
        .iter()
        .enumerate()
        .map(|(index, preferred)| {
            let preference_bonus = 320 - (index as i32 * 35);
            preference_bonus - (size - preferred).abs()
        })
        .max()
        .unwrap_or(0)
}

fn parse_icon_size_component(value: &str) -> Option<i32> {
    let (width, height) = value.split_once('x')?;
    let width = width.parse::<i32>().ok()?;
    let height = height.parse::<i32>().ok()?;
    Some(width.min(height))
}

fn is_supported_raster_icon(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| {
            let value = value.to_ascii_lowercase();
            RASTER_ICON_EXTENSIONS.contains(&value.as_str())
        })
        .unwrap_or(false)
}

fn icon_search_dirs() -> Vec<PathBuf> {
    let mut directories = Vec::new();

    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        directories.push(home.join(".local/share/icons"));
        directories.push(home.join(".icons"));
        directories.push(home.join(".local/share/flatpak/exports/share/icons"));
    }

    directories.push(PathBuf::from("/usr/local/share/icons"));
    directories.push(PathBuf::from("/usr/share/icons"));
    directories.push(PathBuf::from("/usr/share/pixmaps"));
    directories.push(PathBuf::from("/var/lib/flatpak/exports/share/icons"));
    directories
}

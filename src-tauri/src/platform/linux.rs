use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

use serde_json::json;
use serde_json::Value;

use super::{
    build_self_test_result, localized_text, make_probe, ForegroundSnapshot, MediaInfo,
    ProbeTextSpec,
};
use crate::models::PlatformSelfTestResult;

const COMMAND_TIMEOUT: Duration = Duration::from_millis(1500);
const COMMAND_POLL_STEP: Duration = Duration::from_millis(100);
const ICON_SEARCH_DEPTH: usize = 5;
const RASTER_ICON_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg"];

fn source_icon_cache() -> &'static Mutex<HashMap<String, Option<super::MediaArtwork>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<super::MediaArtwork>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Clone, Debug, Default)]
struct LinuxDesktopEntry {
    path: PathBuf,
    name: String,
    icon: String,
    exec: String,
    startup_wm_class: String,
}

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let wayland = has_env("WAYLAND_DISPLAY");

    if wayland {
        let wayland_error = match get_foreground_snapshot_wayland() {
            Ok(snapshot) => return Ok(snapshot),
            Err(error) => error,
        };

        if has_env("DISPLAY") {
            if let Ok(snapshot) = get_foreground_snapshot_x11() {
                return Ok(snapshot);
            }
        }

        return Err(wayland_error);
    }

    get_foreground_snapshot_x11().or_else(|x11_error| {
        get_foreground_snapshot_wayland().map_err(|wayland_error| {
            format!("Failed to read the Linux foreground window. X11: {x11_error}; Wayland: {wayland_error}")
        })
    })
}

pub fn get_foreground_snapshot_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    if !include_process_name && !include_process_title {
        return Ok(ForegroundSnapshot::default());
    }

    let wayland = has_env("WAYLAND_DISPLAY");

    if wayland {
        let wayland_error = match get_foreground_snapshot_wayland_for_reporting(
            include_process_name,
            include_process_title,
        ) {
            Ok(snapshot) => return Ok(snapshot),
            Err(error) => error,
        };

        if has_env("DISPLAY") {
            if let Ok(snapshot) = get_foreground_snapshot_x11_for_reporting(
                include_process_name,
                include_process_title,
            ) {
                return Ok(snapshot);
            }
        }

        return Err(wayland_error);
    }

    get_foreground_snapshot_x11_for_reporting(include_process_name, include_process_title).or_else(
        |x11_error| {
            get_foreground_snapshot_wayland_for_reporting(
                include_process_name,
                include_process_title,
            )
            .map_err(|wayland_error| {
                format!("Failed to read the Linux foreground window. X11: {x11_error}; Wayland: {wayland_error}")
            })
        },
    )
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    let output = command_output_with_timeout(
        "playerctl",
        &[
            "metadata",
            "--format",
            "{{title}}\n{{artist}}\n{{album}}\n{{playerName}}\n{{mpris:length}}",
        ],
    )
    .map_err(|error| format!("Failed to run playerctl: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no players found")
            || combined.contains("no player could handle this command")
        {
            return Ok(MediaInfo::default());
        }
        return Err(format!(
            "Failed to read media info: {}",
            stderr.trim().if_empty("playerctl returned an error")
        ));
    }

    let mut lines = stdout.lines().map(str::trim);
    let title = lines.next().unwrap_or_default().to_string();
    let artist = lines.next().unwrap_or_default().to_string();
    let album = lines.next().unwrap_or_default().to_string();
    let source_app_id = lines.next().unwrap_or_default().to_string();
    let is_playing = read_player_status()?;
    let duration_ms = parse_playerctl_length_ms(lines.next().unwrap_or_default());
    let position_ms = read_player_position_ms().unwrap_or(None);
    let source_icon = read_source_app_icon(&source_app_id);

    let media = MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        is_playing,
        duration_ms,
        position_ms,
        artwork: None,
        source_icon,
    };

    if media.is_empty() {
        return Ok(MediaInfo::default());
    }

    Ok(media)
}

pub fn get_foreground_app_icon() -> Result<Option<super::MediaArtwork>, String> {
    Ok(None)
}

fn read_player_position_ms() -> Result<Option<u64>, String> {
    let output = command_output_with_timeout("playerctl", &["position"])
        .map_err(|error| format!("Failed to run playerctl position: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no players found")
            || combined.contains("no player could handle this command")
        {
            return Ok(None);
        }
        return Err(stderr
            .trim()
            .if_empty(stdout.trim())
            .if_empty("playerctl position returned an error")
            .to_string());
    }

    Ok(parse_playerctl_position_ms(stdout.trim()))
}

fn parse_playerctl_length_ms(value: &str) -> Option<u64> {
    let micros = value.trim().parse::<u64>().ok()?;
    (micros > 0).then_some(micros / 1_000)
}

fn parse_playerctl_position_ms(value: &str) -> Option<u64> {
    let seconds = value.trim().parse::<f64>().ok()?;
    (seconds.is_finite() && seconds >= 0.0).then_some((seconds * 1_000.0).round() as u64)
}

fn read_player_status() -> Result<bool, String> {
    let output = command_output_with_timeout("playerctl", &["status"])
        .map_err(|error| format!("Failed to run playerctl status: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no players found")
            || combined.contains("no player could handle this command")
        {
            return Ok(false);
        }
        return Err(stderr
            .trim()
            .if_empty(stdout.trim())
            .if_empty("playerctl status returned an error")
            .to_string());
    }

    Ok(stdout.trim().eq_ignore_ascii_case("playing"))
}

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

    let icon = resolve_linux_desktop_entry(cache_key)
        .and_then(|entry| resolve_desktop_icon_path(&entry))
        .and_then(|path| read_icon_file(&path));

    source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key.to_string(), icon.clone());

    icon
}

fn resolve_linux_desktop_entry(source_app_id: &str) -> Option<LinuxDesktopEntry> {
    let keys = build_linux_source_match_keys(source_app_id);
    if keys.is_empty() {
        return None;
    }

    let mut best_match: Option<(i32, LinuxDesktopEntry)> = None;
    let mut desktop_files = Vec::new();
    for directory in desktop_entry_dirs() {
        collect_files_with_extension(&directory, "desktop", ICON_SEARCH_DEPTH, &mut desktop_files);
    }

    for path in desktop_files {
        let Some(entry) = parse_linux_desktop_entry(&path) else {
            continue;
        };
        let score = score_linux_desktop_entry(&entry, &keys);
        if score <= 0 {
            continue;
        }

        let replace = best_match
            .as_ref()
            .map(|(best_score, _)| score > *best_score)
            .unwrap_or(true);
        if replace {
            best_match = Some((score, entry));
        }
    }

    best_match.map(|(_, entry)| entry)
}

fn build_linux_source_match_keys(source_app_id: &str) -> Vec<String> {
    let trimmed = source_app_id.trim().to_lowercase();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let tail = trimmed
        .rsplit(['\\', '/', '!'])
        .next()
        .unwrap_or(trimmed.as_str());

    let mut keys = HashSet::new();
    for candidate in [
        trimmed.as_str(),
        tail,
        tail.split('.').next().unwrap_or(tail),
        tail.rsplit('.').next().unwrap_or(tail),
        tail.split('_').next().unwrap_or(tail),
    ] {
        let normalized = normalize_linux_id(candidate);
        if !normalized.is_empty() {
            keys.insert(normalized);
        }
    }

    for segment in tail.split(['.', '_', '-', ' ']) {
        let normalized = normalize_linux_id(segment);
        if !normalized.is_empty() {
            keys.insert(normalized);
        }
    }

    keys.into_iter().collect()
}

fn normalize_linux_id(value: &str) -> String {
    let trimmed = value.trim().to_lowercase();
    if trimmed.is_empty() {
        return String::new();
    }

    let stripped = trimmed
        .strip_suffix(".desktop")
        .or_else(|| trimmed.strip_suffix(".exe"))
        .or_else(|| trimmed.strip_suffix(".app"))
        .unwrap_or(trimmed.as_str());

    stripped
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn parse_linux_desktop_entry(path: &Path) -> Option<LinuxDesktopEntry> {
    let content = fs::read_to_string(path).ok()?;
    let mut entry = LinuxDesktopEntry {
        path: path.to_path_buf(),
        ..LinuxDesktopEntry::default()
    };
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_desktop_entry = trimmed.eq_ignore_ascii_case("[Desktop Entry]");
            continue;
        }
        if !in_desktop_entry {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "Name" if entry.name.is_empty() => entry.name = value.to_string(),
            "Icon" if entry.icon.is_empty() => entry.icon = value.to_string(),
            "Exec" if entry.exec.is_empty() => entry.exec = value.to_string(),
            "StartupWMClass" if entry.startup_wm_class.is_empty() => {
                entry.startup_wm_class = value.to_string()
            }
            _ => {}
        }
    }

    (!entry.icon.trim().is_empty()).then_some(entry)
}

fn score_linux_desktop_entry(entry: &LinuxDesktopEntry, keys: &[String]) -> i32 {
    let file_stem = entry
        .path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let exec_base = desktop_exec_basename(&entry.exec);

    let candidates = [
        (normalize_linux_id(file_stem), 130, 80),
        (normalize_linux_id(&entry.startup_wm_class), 120, 75),
        (normalize_linux_id(&exec_base), 110, 70),
        (normalize_linux_id(&entry.icon), 100, 65),
        (normalize_linux_id(&entry.name), 80, 50),
    ];

    let mut best_score = 0;
    for key in keys {
        if key.is_empty() {
            continue;
        }
        for (candidate, exact_score, partial_score) in &candidates {
            if candidate.is_empty() {
                continue;
            }
            if candidate == key {
                best_score = best_score.max(*exact_score);
            } else if candidate.contains(key) || key.contains(candidate) {
                best_score = best_score.max(*partial_score);
            }
        }
    }

    best_score
}

fn desktop_exec_basename(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let command = if let Some(rest) = trimmed.strip_prefix('"') {
        rest.split('"').next().unwrap_or(rest)
    } else {
        trimmed.split_whitespace().next().unwrap_or(trimmed)
    };

    Path::new(command)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(command)
        .to_string()
}

fn resolve_desktop_icon_path(entry: &LinuxDesktopEntry) -> Option<PathBuf> {
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
        .map(|size| 200 - (size - 128).abs())
        .max()
        .unwrap_or(0);

    extension_score + size_score
}

fn parse_icon_size_component(value: &str) -> Option<i32> {
    let (width, height) = value.split_once('x')?;
    let width = width.parse::<i32>().ok()?;
    let height = height.parse::<i32>().ok()?;
    Some(width.min(height))
}

fn read_icon_file(path: &Path) -> Option<super::MediaArtwork> {
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

    Some(super::MediaArtwork {
        bytes,
        content_type: content_type.to_string(),
    })
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

fn collect_files_with_extension(
    directory: &Path,
    extension: &str,
    depth: usize,
    output: &mut Vec<PathBuf>,
) {
    if depth == 0 {
        return;
    }

    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, depth - 1, output);
            continue;
        }

        let matches_extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(extension))
            .unwrap_or(false);
        if matches_extension {
            output.push(path);
        }
    }
}

fn desktop_entry_dirs() -> Vec<PathBuf> {
    let mut directories = Vec::new();

    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        directories.push(home.join(".local/share/applications"));
        directories.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    directories.push(PathBuf::from("/usr/local/share/applications"));
    directories.push(PathBuf::from("/usr/share/applications"));
    directories.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    directories
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

fn get_foreground_snapshot_x11() -> Result<ForegroundSnapshot, String> {
    get_foreground_snapshot_x11_for_reporting(true, true)
}

fn get_foreground_snapshot_x11_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    let window_id = get_active_window_id_x11()?;
    let detail_stdout =
        read_x11_window_detail(&window_id, include_process_name, include_process_title)?;

    let process_title = if include_process_title {
        parse_window_title(&detail_stdout).unwrap_or_default()
    } else {
        String::new()
    };

    let process_name = if include_process_name {
        let wm_class = parse_wm_class(&detail_stdout).unwrap_or_default();
        parse_window_pid(&detail_stdout)
            .and_then(read_process_name_from_pid)
            .or_else(|| {
                if wm_class.trim().is_empty() {
                    None
                } else {
                    Some(wm_class)
                }
            })
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

fn get_foreground_snapshot_wayland() -> Result<ForegroundSnapshot, String> {
    let mut errors = Vec::new();

    match get_foreground_snapshot_gnome_focused_window_dbus() {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => errors.push(format!("GNOME Focused Window D-Bus: {error}")),
    }

    match get_foreground_snapshot_kde_kdotool() {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => errors.push(format!("KDE kdotool: {error}")),
    }

    Err(format!(
        "Wayland foreground window capture failed. {}",
        errors.join("; ")
    ))
}

fn get_foreground_snapshot_wayland_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    let mut errors = Vec::new();

    match get_foreground_snapshot_gnome_focused_window_dbus() {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => errors.push(format!("GNOME Focused Window D-Bus: {error}")),
    }

    match get_foreground_snapshot_kde_kdotool_for_reporting(
        include_process_name,
        include_process_title,
    ) {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => errors.push(format!("KDE kdotool: {error}")),
    }

    Err(format!(
        "Wayland foreground window capture failed. {}",
        errors.join("; ")
    ))
}

fn get_foreground_snapshot_gnome_focused_window_dbus() -> Result<ForegroundSnapshot, String> {
    let output = command_output_with_timeout(
        "gdbus",
        &[
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/org/gnome/shell/extensions/FocusedWindow",
            "--method",
            "org.gnome.shell.extensions.FocusedWindow.Get",
        ],
    )
    .map_err(|error| format!("Failed to run gdbus: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Focused Window D-Bus call failed: {}",
            stderr.trim().if_empty("gdbus returned an error")
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_payload = parse_gdbus_string_tuple(&stdout)
        .ok_or_else(|| "Failed to parse the Focused Window D-Bus return value.".to_string())?;
    let payload: Value = serde_json::from_str(&json_payload)
        .map_err(|error| format!("Failed to parse Focused Window D-Bus JSON: {error}"))?;

    let process_name = [
        value_as_trimmed_string(payload.get("wm_class_instance")),
        value_as_trimmed_string(payload.get("wm_class")),
        value_as_trimmed_string(payload.get("app_id")),
    ]
    .into_iter()
    .flatten()
    .next()
    .ok_or_else(|| "Focused Window D-Bus did not return a usable window class name.".to_string())?;

    let process_title = value_as_trimmed_string(payload.get("title")).unwrap_or_default();

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

fn get_foreground_snapshot_kde_kdotool() -> Result<ForegroundSnapshot, String> {
    get_foreground_snapshot_kde_kdotool_for_reporting(true, true)
}

fn get_foreground_snapshot_kde_kdotool_for_reporting(
    include_process_name: bool,
    include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    let window_id = run_command_trimmed("kdotool", ["getactivewindow"])
        .map_err(|error| format!("Failed to read the active window: {error}"))?;
    if window_id == "0" {
        return Err("There is no active window.".into());
    }

    let process_name = if include_process_name {
        let value = run_command_trimmed("kdotool", ["getwindowclassname", &window_id])
            .map_err(|error| format!("Failed to read the window class name: {error}"))?;
        if value.is_empty() {
            return Err("kdotool did not return a window class name.".into());
        }
        value
    } else {
        String::new()
    };

    let process_title = if include_process_title {
        run_command_trimmed("kdotool", ["getwindowname", &window_id]).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name,
        process_title,
    })
}

fn get_active_window_id_x11() -> Result<String, String> {
    let active_output = command_output_with_timeout("xprop", &["-root", "_NET_ACTIVE_WINDOW"])
        .map_err(|error| format!("Failed to run xprop: {error}"))?;

    if !active_output.status.success() {
        let stderr = String::from_utf8_lossy(&active_output.stderr);
        return Err(format!(
            "Failed to read the active window: {}",
            stderr.trim().if_empty("xprop returned an error")
        ));
    }

    let active_stdout = String::from_utf8_lossy(&active_output.stdout);
    let window_id = parse_active_window_id(&active_stdout)
        .ok_or_else(|| "Failed to parse _NET_ACTIVE_WINDOW.".to_string())?;

    if window_id == "0x0" {
        return Err("There is no active window.".into());
    }

    Ok(window_id)
}

fn read_x11_window_detail(
    window_id: &str,
    include_process_name: bool,
    include_process_title: bool,
) -> Result<String, String> {
    let mut args = vec!["-id", window_id];
    if include_process_name {
        args.push("WM_CLASS");
        args.push("_NET_WM_PID");
    }
    if include_process_title {
        args.push("_NET_WM_NAME");
        args.push("WM_NAME");
    }

    let detail_output = command_output_with_timeout("xprop", &args)
        .map_err(|error| format!("Failed to run xprop for window details: {error}"))?;

    if !detail_output.status.success() {
        let stderr = String::from_utf8_lossy(&detail_output.stderr);
        return Err(format!(
            "Failed to read window details: {}",
            stderr.trim().if_empty("xprop returned an error")
        ));
    }

    Ok(String::from_utf8_lossy(&detail_output.stdout).to_string())
}

fn run_command_trimmed<const N: usize>(program: &str, args: [&str; N]) -> Result<String, String> {
    let output = command_output_with_timeout(program, &args)
        .map_err(|error| format!("Failed to run {program}: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(stderr
            .trim()
            .if_empty(stdout.trim())
            .if_empty("Command returned an error")
            .to_string());
    }

    Ok(stdout.lines().next().unwrap_or_default().trim().to_string())
}

fn command_output_with_timeout(program: &str, args: &[&str]) -> Result<Output, String> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().map_err(|error| error.to_string()),
            Ok(None) if start.elapsed() >= COMMAND_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "Command timed out (>{}ms)",
                    COMMAND_TIMEOUT.as_millis()
                ));
            }
            Ok(None) => thread::sleep(COMMAND_POLL_STEP),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
    }
}

fn parse_gdbus_string_tuple(stdout: &str) -> Option<String> {
    let start = stdout.find('\'')?;
    let mut escaped = false;
    let mut value = String::new();

    for ch in stdout[start + 1..].chars() {
        if escaped {
            value.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '\'' => return Some(value),
            other => value.push(other),
        }
    }

    None
}

fn value_as_trimmed_string(value: Option<&Value>) -> Option<String> {
    let raw = value?.as_str()?.trim();
    if raw.is_empty() {
        None
    } else {
        Some(raw.to_string())
    }
}

fn parse_active_window_id(stdout: &str) -> Option<String> {
    stdout
        .split('#')
        .nth(1)
        .map(str::trim)
        .and_then(|value| value.split_whitespace().next())
        .map(str::to_string)
}

fn parse_wm_class(stdout: &str) -> Option<String> {
    for line in stdout.lines() {
        if !line.starts_with("WM_CLASS") {
            continue;
        }
        let values = extract_quoted_values(line);
        if values.len() >= 2 {
            return Some(values[1].clone());
        }
        if let Some(value) = values.first() {
            return Some(value.clone());
        }
    }
    None
}

fn parse_window_title(stdout: &str) -> Option<String> {
    for key in ["_NET_WM_NAME", "WM_NAME"] {
        for line in stdout.lines() {
            if !line.starts_with(key) {
                continue;
            }
            let values = extract_quoted_values(line);
            if let Some(value) = values.first() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn parse_window_pid(stdout: &str) -> Option<u32> {
    for line in stdout.lines() {
        if !line.starts_with("_NET_WM_PID") {
            continue;
        }
        let raw = line.split('=').nth(1)?.trim();
        if let Ok(pid) = raw.parse::<u32>() {
            return Some(pid);
        }
    }
    None
}

fn extract_quoted_values(line: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut start = None;

    for (idx, ch) in line.char_indices() {
        if ch == '"' {
            match start {
                Some(begin) => {
                    values.push(line[begin..idx].to_string());
                    start = None;
                }
                None => start = Some(idx + 1),
            }
        }
    }

    values
}

fn read_process_name_from_pid(pid: u32) -> Option<String> {
    let comm_path = Path::new("/proc").join(pid.to_string()).join("comm");
    let name = fs::read_to_string(comm_path).ok()?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn has_env(key: &str) -> bool {
    env::var(key)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn linux_detail(error: &str, probe: &str) -> ProbeTextSpec {
    let lower = error.to_lowercase();

    if probe == "media" {
        if lower.contains("playerctl") {
            return localized_text(
                "platformSelfTest.detail.linuxMediaPlayerctlMissing",
                None,
                error,
            );
        }

        return localized_text("platformSelfTest.detail.linuxMediaUnavailable", None, error);
    }

    if lower.contains("focused window d-bus") || lower.contains("gdbus") {
        return localized_text(
            "platformSelfTest.detail.linuxForegroundGnomeSupportMissing",
            None,
            error,
        );
    }

    if lower.contains("kdotool") {
        return localized_text(
            "platformSelfTest.detail.linuxForegroundKdeSupportMissing",
            None,
            error,
        );
    }

    if lower.contains("xprop") {
        return localized_text(
            "platformSelfTest.detail.linuxForegroundXpropMissing",
            None,
            error,
        );
    }

    localized_text(
        "platformSelfTest.detail.linuxForegroundUnavailable",
        None,
        error,
    )
}

fn linux_guidance(error: &str, probe: &str) -> Vec<ProbeTextSpec> {
    let lower = error.to_lowercase();
    let mut guidance = Vec::new();

    if probe == "foreground" || lower.contains("wayland") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxX11",
            None,
            "On X11 sessions, the foreground window can be read directly with xprop.",
        ));
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxGnomeFocusedWindow",
            None,
            "On GNOME Wayland, install the Focused Window D-Bus extension so the client can read the foreground window via gdbus.",
        ));
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxKdeKdotool",
            None,
            "On KDE Plasma Wayland, install kdotool so the client can read the active window class and title.",
        ));
    }

    if lower.contains("xprop") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxInstallXprop",
            None,
            "Install xprop first (usually provided by xorg-xprop / x11-utils).",
        ));
    }

    if lower.contains("focused window d-bus") || lower.contains("gdbus") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxGnomeFocusedWindow",
            None,
            "On GNOME, install the Focused Window D-Bus extension and ensure gdbus is available.",
        ));
    }

    if lower.contains("kdotool") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxInstallKdotool",
            None,
            "On KDE Plasma, install kdotool.",
        ));
    }

    if probe == "media" || lower.contains("playerctl") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxInstallPlayerctl",
            None,
            "Install playerctl and ensure the player implements MPRIS.",
        ));
    }

    if guidance.is_empty() {
        guidance.push(localized_text(
            "platformSelfTest.guidance.linuxCheckDesktopPermission",
            None,
            "Confirm that the current desktop environment allows foreground-window and media capture.",
        ));
    }

    guidance
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
            linux_detail(&error, "foreground"),
            linux_guidance(&error, "foreground"),
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
            linux_detail(&error, "window"),
            linux_guidance(&error, "foreground"),
        ),
    };

    let media = match get_now_playing() {
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
            vec![localized_text(
                "platformSelfTest.guidance.playMediaThenRetry",
                None,
                "To validate media capture, start audio or video playback first and retry.",
            )],
        ),
        Err(error) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.mediaFailed",
                None,
                "Media capture failed",
            ),
            linux_detail(&error, "media"),
            linux_guidance(&error, "media"),
        ),
    };

    build_self_test_result(foreground, window_title, media)
}

trait EmptyFallback {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl EmptyFallback for str {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.trim().is_empty() {
            fallback
        } else {
            self
        }
    }
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Err("Accessibility permission requests are not supported on this platform.".into())
}

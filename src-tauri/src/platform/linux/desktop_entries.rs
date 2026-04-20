use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

const ICON_SEARCH_DEPTH: usize = 5;

#[derive(Clone, Debug, Default)]
pub(super) struct LinuxDesktopEntry {
    pub(super) path: PathBuf,
    pub(super) name: String,
    pub(super) icon: String,
    pub(super) exec: String,
    pub(super) startup_wm_class: String,
}

pub(super) fn resolve_linux_desktop_entry(source_app_id: &str) -> Option<LinuxDesktopEntry> {
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

pub(super) fn normalize_linux_id(value: &str) -> String {
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

    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        directories.push(home.join(".local/share/applications"));
        directories.push(home.join(".local/share/flatpak/exports/share/applications"));
    }

    directories.push(PathBuf::from("/usr/local/share/applications"));
    directories.push(PathBuf::from("/usr/share/applications"));
    directories.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
    directories
}

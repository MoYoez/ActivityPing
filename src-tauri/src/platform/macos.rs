use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
    sync::{Mutex, OnceLock},
};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::Deserialize;
use serde_json::json;

use super::{
    build_self_test_result, localized_text, make_probe, ForegroundSnapshot, MediaInfo,
    ProbeTextSpec,
};
use crate::models::PlatformSelfTestResult;

const COMMAND_TIMEOUT: Duration = Duration::from_millis(1500);
const COMMAND_POLL_STEP: Duration = Duration::from_millis(100);
const NOWPLAYING_CLI: &str = "nowplaying-cli";
const NOWPLAYING_CLI_FALLBACK_PATHS: [&str; 2] = [
    "/opt/homebrew/bin/nowplaying-cli",
    "/usr/local/bin/nowplaying-cli",
];

unsafe extern "C" {
    fn waken_frontmost_app_name() -> *mut c_char;
    fn waken_frontmost_window_title() -> *mut c_char;
    fn waken_bundle_icon_png_base64(bundle_identifier: *const c_char) -> *mut c_char;
    fn waken_accessibility_is_trusted() -> bool;
    fn waken_request_accessibility_permission() -> bool;
    fn waken_string_free(value: *mut c_char);
}

fn source_icon_cache() -> &'static Mutex<HashMap<String, Option<super::MediaArtwork>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<super::MediaArtwork>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn read_bridge_string(fetch: unsafe extern "C" fn() -> *mut c_char) -> Option<String> {
    let ptr = unsafe { fetch() };
    if ptr.is_null() {
        return None;
    }

    let value = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
    unsafe { waken_string_free(ptr) };
    Some(value)
}

pub fn accessibility_permission_granted() -> bool {
    unsafe { waken_accessibility_is_trusted() }
}

fn request_accessibility_permission_via_bridge() -> bool {
    unsafe { waken_request_accessibility_permission() }
}

enum NowPlayingCliError {
    NotFound {
        path: String,
        attempted: Vec<String>,
    },
    TimedOut,
    Failed(String),
}

impl NowPlayingCliError {
    fn into_user_message(self) -> String {
        match self {
            Self::NotFound { path, attempted } => {
                format!(
                    "Failed to run nowplaying-cli: executable not found in PATH or common Homebrew locations. Tried: {}. PATH={path}",
                    attempted.join(", ")
                )
            }
            Self::TimedOut => "nowplaying-cli timed out (>1500ms).".into(),
            Self::Failed(detail) => format!("nowplaying-cli returned an error: {detail}"),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawNowPlayingInfo {
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoTitle")]
    title: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoArtist")]
    artist: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoAlbum")]
    album: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoClientBundleIdentifier")]
    client_bundle_identifier: Option<String>,
    #[serde(rename = "kMRMediaRemoteNowPlayingInfoDuration", alias = "duration")]
    duration: Option<f64>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoElapsedTime",
        alias = "elapsedTime"
    )]
    elapsed_time: Option<f64>,
    #[serde(
        rename = "kMRMediaRemoteNowPlayingInfoPlaybackRate",
        alias = "playbackRate"
    )]
    playback_rate: Option<f64>,
}

fn get_now_playing_via_nowplaying_cli() -> Result<MediaInfo, NowPlayingCliError> {
    let attempted = std::iter::once(NOWPLAYING_CLI)
        .chain(NOWPLAYING_CLI_FALLBACK_PATHS.iter().copied())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let output = {
        let mut resolved = None;
        for candidate in
            std::iter::once(NOWPLAYING_CLI).chain(NOWPLAYING_CLI_FALLBACK_PATHS.iter().copied())
        {
            match command_output_with_timeout(candidate, &["get-raw"]) {
                Ok(output) => {
                    resolved = Some(output);
                    break;
                }
                Err(CommandError::NotFound) => {}
                Err(CommandError::TimedOut) => return Err(NowPlayingCliError::TimedOut),
                Err(CommandError::Other(detail)) => return Err(NowPlayingCliError::Failed(detail)),
            }
        }
        resolved
    }
    .ok_or_else(|| NowPlayingCliError::NotFound {
        path: std::env::var("PATH").unwrap_or_default(),
        attempted,
    })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr).to_lowercase();
        if combined.contains("no media")
            || combined.contains("no now playing")
            || combined.contains("nothing is playing")
            || combined.contains("not playing")
            || combined.contains("no player")
            || combined.contains("null")
        {
            return Ok(MediaInfo::default());
        }

        let detail = stderr
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .or_else(|| stdout.lines().map(str::trim).find(|line| !line.is_empty()))
            .unwrap_or("Unknown error");
        return Err(NowPlayingCliError::Failed(detail.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let normalize = |value: String| {
        if value.eq_ignore_ascii_case("null") {
            String::new()
        } else {
            value.trim().to_string()
        }
    };

    let raw: RawNowPlayingInfo = serde_json::from_str(&stdout)
        .map_err(|error| NowPlayingCliError::Failed(format!("Failed to parse get-raw output: {error}")))?;

    let title = raw.title.map(normalize).unwrap_or_default();
    let artist = raw.artist.map(normalize).unwrap_or_default();
    let album = raw.album.map(normalize).unwrap_or_default();
    let source_app_id = raw
        .client_bundle_identifier
        .map(normalize)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| NOWPLAYING_CLI.to_string());
    let is_playing = raw
        .playback_rate
        .map(|value| value.is_finite() && value > 0.0)
        .unwrap_or_else(|| !title.is_empty() || !artist.is_empty() || !album.is_empty());
    let duration_ms = seconds_to_millis(raw.duration).filter(|value| *value > 0);
    let position_ms = seconds_to_millis(raw.elapsed_time);
    let source_icon = read_source_app_icon(&source_app_id);

    Ok(MediaInfo {
        title,
        artist,
        album,
        source_app_id,
        is_playing,
        duration_ms,
        position_ms,
        artwork: None,
        source_icon,
    })
}

fn seconds_to_millis(value: Option<f64>) -> Option<u64> {
    let seconds = value?;
    (seconds.is_finite() && seconds >= 0.0).then_some((seconds * 1_000.0).round() as u64)
}

fn read_source_app_icon(bundle_identifier: &str) -> Option<super::MediaArtwork> {
    let cache_key = bundle_identifier.trim();
    if cache_key.is_empty() || cache_key.eq_ignore_ascii_case(NOWPLAYING_CLI) {
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

    let c_bundle_identifier = CString::new(cache_key).ok()?;
    let base64_png =
        read_bridge_string_with_input(waken_bundle_icon_png_base64, c_bundle_identifier.as_c_str());
    let icon = base64_png
        .and_then(|value| BASE64_STANDARD.decode(value.trim()).ok())
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| super::MediaArtwork {
            bytes,
            content_type: "image/png".to_string(),
        });

    source_icon_cache()
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(cache_key.to_string(), icon.clone());

    icon
}

fn read_bridge_string_with_input(
    fetch: unsafe extern "C" fn(*const c_char) -> *mut c_char,
    value: &std::ffi::CStr,
) -> Option<String> {
    let ptr = unsafe { fetch(value.as_ptr()) };
    if ptr.is_null() {
        return None;
    }

    let decoded = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
    unsafe { waken_string_free(ptr) };
    Some(decoded)
}

enum CommandError {
    NotFound,
    TimedOut,
    Other(String),
}

fn command_output_with_timeout(program: &str, args: &[&str]) -> Result<Output, CommandError> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => CommandError::NotFound,
            _ => CommandError::Other(error.to_string()),
        })?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|error| CommandError::Other(error.to_string()))
            }
            Ok(None) if start.elapsed() >= COMMAND_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CommandError::TimedOut);
            }
            Ok(None) => thread::sleep(COMMAND_POLL_STEP),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CommandError::Other(error.to_string()));
            }
        }
    }
}

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    let process_name = read_bridge_string(waken_frontmost_app_name)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Failed to read the macOS foreground app.".to_string())?;
    let process_title = read_bridge_string(waken_frontmost_window_title).unwrap_or_default();

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

    if include_process_name {
        let mut snapshot = get_foreground_snapshot()?;
        if !include_process_title {
            snapshot.process_title.clear();
        }
        return Ok(snapshot);
    }

    let process_title = if include_process_title {
        read_bridge_string(waken_frontmost_window_title).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(ForegroundSnapshot {
        process_name: String::new(),
        process_title,
    })
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    let media = match get_now_playing_via_nowplaying_cli() {
        Ok(media) => media,
        Err(NowPlayingCliError::TimedOut) => return Ok(MediaInfo::default()),
        Err(error) => return Err(error.into_user_message()),
    };
    if media.is_empty() {
        return Ok(MediaInfo::default());
    }
    Ok(media)
}

pub fn get_foreground_app_icon() -> Result<Option<super::MediaArtwork>, String> {
    Ok(None)
}

fn macos_guidance(error: &str, probe: &str) -> Vec<ProbeTextSpec> {
    let lower = error.to_lowercase();
    let mut guidance = Vec::new();

    if probe == "foreground" {
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosForegroundBridge",
            None,
            "This macOS foreground-app capture path uses the native bridge only and no longer relies on osascript.",
        ));
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosCheckSystemVersion",
            None,
            "If this still fails, check the macOS version and any window-list access restrictions.",
        ));
    }

    if probe == "window" {
        if accessibility_permission_granted() {
            guidance.push(localized_text(
                "platformSelfTest.guidance.macosWindowPermissionGrantedButNoTitle",
                None,
                "Accessibility permission is already granted. If the title is still missing, the app likely does not expose a stable window title.",
            ));
        } else {
            guidance.push(localized_text(
                "platformSelfTest.guidance.macosWindowPermissionRequired",
                None,
                "macOS window title capture requires Accessibility permission.",
            ));
            guidance.push(localized_text(
                "platformSelfTest.guidance.macosWindowPermissionSettings",
                None,
                "Use the settings page to request Accessibility permission, or enable it manually in System Settings > Privacy & Security > Accessibility.",
            ));
        }
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosWindowTitleUnstable",
            None,
            "Some apps may still not return a stable window title even after permission is granted.",
        ));
    }

    if probe == "media" || lower.contains("nowplaying-cli") {
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosInstallNowPlayingCli",
            None,
            "Install nowplaying-cli first: `brew install nowplaying-cli`.",
        ));
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosMediaEmpty",
            None,
            "If no media is currently playing, the client now returns an empty result instead of treating it as a failure.",
        ));
    }

    if guidance.is_empty() {
        guidance.push(localized_text(
            "platformSelfTest.guidance.macosCheckPermissions",
            None,
            "If this looks like a permission issue, check macOS Accessibility and Automation permissions first.",
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
            localized_text(
                "platformSelfTest.detail.foregroundReadFailed",
                None,
                error.clone(),
            ),
            macos_guidance(&error, "foreground"),
        ),
    };

    let window_title = match get_foreground_snapshot_for_reporting(false, true) {
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
                if accessibility_permission_granted() {
                    localized_text(
                        "platformSelfTest.detail.windowTitleEmpty",
                        None,
                        "The current foreground window has no usable title.",
                    )
                } else {
                    localized_text(
                        "platformSelfTest.detail.windowTitleEmptyPermissionMissing",
                        None,
                        "The current foreground window has no usable title, and Accessibility permission is not granted.",
                    )
                }
            } else {
                localized_text(
                    "platformSelfTest.detail.windowTitleCurrent",
                    Some(json!({ "processTitle": snapshot.process_title.clone() })),
                    snapshot.process_title,
                )
            },
            macos_guidance("", "window"),
        ),
        Err(error) => make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.windowTitleFailed",
                None,
                "Window title capture failed",
            ),
            localized_text(
                "platformSelfTest.detail.windowTitleReadFailed",
                None,
                error.clone(),
            ),
            macos_guidance(&error, "window"),
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
                "If you are testing media capture, start playback first and run the self-test again.",
            )],
        ),
        Err(error) => make_probe(
            false,
            localized_text("platformSelfTest.summary.mediaFailed", None, "Media capture failed"),
            localized_text(
                "platformSelfTest.detail.mediaReadFailed",
                None,
                error.clone(),
            ),
            macos_guidance(&error, "media"),
        ),
    };

    build_self_test_result(foreground, window_title, media)
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Ok(request_accessibility_permission_via_bridge())
}

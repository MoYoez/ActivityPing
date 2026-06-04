#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod stub;
#[cfg(target_os = "windows")]
mod windows;

use serde_json::Value;
#[cfg(target_os = "macos")]
use std::{path::PathBuf, sync::OnceLock};

use crate::models::{LocalizedTextEntry, PlatformProbeResult, PlatformSelfTestResult};

#[cfg(target_os = "macos")]
static MACOS_MEDIAREMOTE_ADAPTER_ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Records the resolved on-disk root of the bundled `mediaremote-adapter`
/// resource (script + framework) so the macOS media capture can locate it at
/// runtime. Called once during app setup.
#[cfg(target_os = "macos")]
pub fn set_macos_mediaremote_adapter_root(path: PathBuf) {
    let _ = MACOS_MEDIAREMOTE_ADAPTER_ROOT.set(path);
}

#[cfg(target_os = "macos")]
pub(crate) fn macos_mediaremote_adapter_root() -> Option<&'static PathBuf> {
    MACOS_MEDIAREMOTE_ADAPTER_ROOT.get()
}

#[derive(Clone, Debug, Default)]
pub struct ForegroundSnapshot {
    pub process_name: String,
    pub process_title: String,
}

#[derive(Clone, Debug, Default)]
pub struct MediaArtwork {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

/// Normalized playback states emitted by the platform capture layer.
pub const PLAYBACK_STATE_PLAYING: &str = "playing";
pub const PLAYBACK_STATE_PAUSED: &str = "paused";
pub const PLAYBACK_STATE_STOPPED: &str = "stopped";

#[derive(Clone, Debug, Default)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub source_app_id: String,
    /// Normalized playback state: `"playing"`, `"paused"`, `"stopped"`, or
    /// empty when the platform could not determine it.
    pub playback_state: String,
    pub duration_ms: Option<u64>,
    pub position_ms: Option<u64>,
    pub artwork: Option<MediaArtwork>,
    pub source_icon: Option<MediaArtwork>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MediaCaptureOptions {
    pub include_artwork: bool,
    pub include_source_icon: bool,
}

impl MediaCaptureOptions {
    pub fn with_assets() -> Self {
        Self {
            include_artwork: true,
            include_source_icon: true,
        }
    }
}

impl MediaInfo {
    pub fn is_empty(&self) -> bool {
        self.title.trim().is_empty()
            && self.artist.trim().is_empty()
            && self.album.trim().is_empty()
    }

    pub fn is_active(&self) -> bool {
        self.is_playing() && !self.is_empty()
    }

    pub fn is_reportable(&self, include_stopped: bool) -> bool {
        !self.is_empty() && (self.is_playing() || include_stopped)
    }

    /// Whether playback is currently active. Treats an unknown (empty) state
    /// as playing when there is metadata, matching the previous boolean-based
    /// behavior so downstream consumers stay consistent.
    pub fn is_playing(&self) -> bool {
        match self.playback_state.trim().to_ascii_lowercase().as_str() {
            PLAYBACK_STATE_PLAYING => true,
            PLAYBACK_STATE_PAUSED | PLAYBACK_STATE_STOPPED => false,
            _ => !self.is_empty(),
        }
    }

    /// Whether playback is explicitly paused (not stopped, not playing).
    #[allow(dead_code)]
    pub fn is_paused(&self) -> bool {
        self.playback_state
            .trim()
            .eq_ignore_ascii_case(PLAYBACK_STATE_PAUSED)
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if !self.title.trim().is_empty() {
            parts.push(self.title.trim().to_string());
        }
        if !self.artist.trim().is_empty() {
            parts.push(self.artist.trim().to_string());
        }
        if !self.album.trim().is_empty() {
            parts.push(self.album.trim().to_string());
        }
        parts.join(" / ")
    }
}

#[cfg(target_os = "linux")]
pub use linux::{
    get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing_with_options,
};
#[cfg(target_os = "macos")]
pub use macos::{
    get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing_with_options,
};
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use stub::{
    get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing_with_options,
};
#[cfg(target_os = "windows")]
pub use windows::{
    get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing_with_options,
};

#[cfg(target_os = "linux")]
pub use linux::run_self_test;
#[cfg(target_os = "macos")]
pub use macos::run_self_test;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use stub::run_self_test;
#[cfg(target_os = "windows")]
pub use windows::run_self_test;

#[cfg(target_os = "linux")]
pub use linux::request_accessibility_permission;
#[cfg(target_os = "macos")]
pub use macos::request_accessibility_permission;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use stub::request_accessibility_permission;
#[cfg(target_os = "windows")]
pub use windows::request_accessibility_permission;

pub fn platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "unsupported"
    }
}

#[cfg(target_os = "macos")]
pub fn display_name_for_app_id(app_id: &str) -> Option<String> {
    macos::read_bundle_display_name(app_id)
}

#[cfg(not(target_os = "macos"))]
pub fn display_name_for_app_id(_app_id: &str) -> Option<String> {
    None
}

/// Subscribes to foreground-window change events when the platform supports
/// them, so a capture loop can wake early instead of waiting out its poll
/// interval. Returns `None` on platforms without an event source (e.g. Linux),
/// where callers fall back to plain polling.
pub fn subscribe_foreground_changes() -> Option<std::sync::mpsc::Receiver<()>> {
    #[cfg(target_os = "windows")]
    {
        Some(windows::subscribe_foreground_changes())
    }
    #[cfg(target_os = "macos")]
    {
        Some(macos::subscribe_foreground_changes())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

pub fn make_probe(
    success: bool,
    summary: ProbeTextSpec,
    detail: ProbeTextSpec,
    guidance: Vec<ProbeTextSpec>,
) -> PlatformProbeResult {
    let ProbeTextSpec {
        key: summary_key,
        params: summary_params,
        fallback: summary_fallback,
    } = summary;
    let ProbeTextSpec {
        key: detail_key,
        params: detail_params,
        fallback: detail_fallback,
    } = detail;

    PlatformProbeResult {
        success,
        summary: summary_fallback,
        detail: detail_fallback,
        guidance: guidance
            .iter()
            .map(|entry| entry.fallback.clone())
            .collect(),
        summary_key: summary_key.map(str::to_string),
        summary_params,
        detail_key: detail_key.map(str::to_string),
        detail_params,
        guidance_entries: guidance
            .into_iter()
            .map(|entry| LocalizedTextEntry {
                text: entry.fallback,
                key: entry.key.map(str::to_string),
                params: entry.params,
            })
            .collect(),
    }
}

pub fn build_self_test_result(
    foreground: PlatformProbeResult,
    window_title: PlatformProbeResult,
    media: PlatformProbeResult,
) -> PlatformSelfTestResult {
    PlatformSelfTestResult {
        platform: platform_name().to_string(),
        foreground,
        window_title,
        media,
    }
}

pub struct ProbeTextSpec {
    key: Option<&'static str>,
    params: Option<Value>,
    fallback: String,
}

pub fn localized_text(
    key: &'static str,
    params: Option<Value>,
    fallback: impl Into<String>,
) -> ProbeTextSpec {
    ProbeTextSpec {
        key: Some(key),
        params,
        fallback: fallback.into(),
    }
}

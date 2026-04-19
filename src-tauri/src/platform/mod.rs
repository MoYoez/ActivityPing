#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod stub;
#[cfg(target_os = "windows")]
mod windows;

use serde_json::Value;

use crate::models::{LocalizedTextEntry, PlatformProbeResult, PlatformSelfTestResult};

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

#[derive(Clone, Debug, Default)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub source_app_id: String,
    pub is_playing: bool,
    pub duration_ms: Option<u64>,
    pub position_ms: Option<u64>,
    pub artwork: Option<MediaArtwork>,
    pub source_icon: Option<MediaArtwork>,
}

impl MediaInfo {
    pub fn is_empty(&self) -> bool {
        self.title.trim().is_empty()
            && self.artist.trim().is_empty()
            && self.album.trim().is_empty()
    }

    pub fn is_active(&self) -> bool {
        self.is_playing && !self.is_empty()
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
pub use linux::{get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing};
#[cfg(target_os = "macos")]
pub use macos::{get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing};
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use stub::{get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing};
#[cfg(target_os = "windows")]
pub use windows::{
    get_foreground_app_icon, get_foreground_snapshot_for_reporting, get_now_playing,
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

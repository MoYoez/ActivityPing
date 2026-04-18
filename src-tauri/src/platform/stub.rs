use super::{build_self_test_result, localized_text, make_probe, ForegroundSnapshot, MediaInfo};
use crate::models::PlatformSelfTestResult;

pub fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
    Err("Realtime capture is not supported on this platform.".into())
}

pub fn get_foreground_snapshot_for_reporting(
    _include_process_name: bool,
    _include_process_title: bool,
) -> Result<ForegroundSnapshot, String> {
    Ok(ForegroundSnapshot::default())
}

pub fn get_now_playing() -> Result<MediaInfo, String> {
    Ok(MediaInfo::default())
}

pub fn get_foreground_app_icon() -> Result<Option<super::MediaArtwork>, String> {
    Ok(None)
}

pub fn run_self_test() -> PlatformSelfTestResult {
    build_self_test_result(
        make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.foregroundFailed",
                None,
                "Foreground app capture is unsupported",
            ),
            localized_text(
                "platformSelfTest.detail.unsupportedPlatform",
                None,
                "Realtime capture is not supported on this platform.",
            ),
            Vec::new(),
        ),
        make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.windowTitleFailed",
                None,
                "Window title capture is unsupported",
            ),
            localized_text(
                "platformSelfTest.detail.unsupportedPlatform",
                None,
                "Realtime capture is not supported on this platform.",
            ),
            Vec::new(),
        ),
        make_probe(
            false,
            localized_text(
                "platformSelfTest.summary.mediaFailed",
                None,
                "Media capture is unsupported",
            ),
            localized_text(
                "platformSelfTest.detail.unsupportedPlatform",
                None,
                "Realtime capture is not supported on this platform.",
            ),
            Vec::new(),
        ),
    )
}

pub fn request_accessibility_permission() -> Result<bool, String> {
    Err("Accessibility permission requests are not supported on this platform.".into())
}

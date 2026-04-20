use serde_json::json;

use crate::{
    models::PlatformSelfTestResult,
    platform::{build_self_test_result, localized_text, make_probe, ProbeTextSpec},
};

use super::{
    bridge::{accessibility_permission_granted, request_accessibility_permission_via_bridge},
    foreground::{get_foreground_snapshot, get_foreground_snapshot_for_reporting},
    media::get_now_playing,
};

pub fn request_accessibility_permission() -> Result<bool, String> {
    Ok(request_accessibility_permission_via_bridge())
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

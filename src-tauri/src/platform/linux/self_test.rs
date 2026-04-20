use serde_json::json;

use crate::{
    models::PlatformSelfTestResult,
    platform::{build_self_test_result, localized_text, make_probe, ProbeTextSpec},
};

use super::{foreground::get_foreground_snapshot, media::get_now_playing};

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

pub fn request_accessibility_permission() -> Result<bool, String> {
    Err("Accessibility permission requests are not supported on this platform.".into())
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

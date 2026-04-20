use crate::platform::ForegroundSnapshot;

use super::{
    command::has_env,
    wayland::{get_foreground_snapshot_wayland, get_foreground_snapshot_wayland_for_reporting},
    x11::{get_foreground_snapshot_x11, get_foreground_snapshot_x11_for_reporting},
};

pub(super) fn get_foreground_snapshot() -> Result<ForegroundSnapshot, String> {
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
            format!(
                "Failed to read the Linux foreground window. X11: {x11_error}; Wayland: {wayland_error}"
            )
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
                format!(
                    "Failed to read the Linux foreground window. X11: {x11_error}; Wayland: {wayland_error}"
                )
            })
        },
    )
}

# Linux Foreground Detection Requirements

This document defines what must be available on Linux for foreground app, window title, and media detection to be considered normal.

## Normal detection checklist

Linux should be considered working when all of the following are true:

1. Foreground app detection returns the current app name or window class.
2. Window title detection returns the active window title when the app exposes one.
3. Media detection can read active playback when the current player exposes MPRIS data.
4. The built-in self-test reports `Foreground app capture OK`.
5. When the self-test is run against an app with a visible title bar, it should report `Window title capture OK`.
6. Media self-test should report either `Media capture OK` or `No media is currently playing`.

## X11 requirements

On X11 sessions, ActivityPing reads the active window directly with `xprop`.

The machine must provide:

- a valid `DISPLAY` session
- the `xprop` executable, usually from `xorg-xprop` or `x11-utils`
- an active window that exposes `_NET_ACTIVE_WINDOW`
- window metadata that exposes `WM_CLASS` and, when title capture is needed, `_NET_WM_NAME` or `WM_NAME`

Notes:

- Process name resolution prefers the PID exposed by X11 metadata and falls back to `WM_CLASS`.
- If the active window has no usable title, app detection can still work, but title-based matching will have less data.
- `Window title is empty` means the current app or surface did not expose a usable title. Re-test with a normal titled window before treating it as a platform gap.

## Wayland requirements

On Wayland, there is no generic cross-desktop foreground-window API in this client. Detection is normal only when one of the supported desktop-specific bridges is available.

### GNOME Wayland

The machine must provide:

- a valid `WAYLAND_DISPLAY` session
- the `gdbus` executable
- the GNOME Shell [Focused Window D-Bus](https://extensions.gnome.org/extension/5592/focused-window-d-bus/) extension exposing `org.gnome.shell.extensions.FocusedWindow`

Install source:

- GNOME Extensions: [Focused Window D-Bus](https://extensions.gnome.org/extension/5592/focused-window-d-bus/)
- Project homepage: [flexagoon/focused-window-dbus](https://github.com/flexagoon/focused-window-dbus)

The extension must return usable JSON with at least one of:

- `wm_class_instance`
- `wm_class`
- `app_id`

When title capture is needed, the payload should also include `title`.

### KDE Plasma Wayland

The machine must provide:

- a valid `WAYLAND_DISPLAY` session
- the `kdotool` executable
- an active window that `kdotool getactivewindow` can resolve

For full detection, the active window must also return:

- a class name from `kdotool getwindowclassname`
- a title from `kdotool getwindowname` when title capture is needed

## Mixed X11 and Wayland sessions

If both `WAYLAND_DISPLAY` and `DISPLAY` are present, ActivityPing tries the Wayland bridge first and then falls back to X11. This means XWayland-hosted apps may still be detected even when the Wayland-specific bridge is missing.

## Media detection requirements

Linux media detection is considered normal when:

- `playerctl` is installed
- the current player exposes MPRIS metadata
- the current player returns title, artist, album, player name, and playback state when playback is active

`No media is currently playing` is a normal self-test result when playback is idle. A failure is only expected when `playerctl` is missing, the command errors, or the player does not expose usable MPRIS data.

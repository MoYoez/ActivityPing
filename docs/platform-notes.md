# Platform Notes

Foreground and media detection are platform-specific. Use the detailed requirement docs below when you need to confirm whether a machine should be considered fully supported for local capture.

- [Linux Foreground Detection Requirements](./linux-wayland-foreground-bridge.md)
- [macOS Detection Requirements](./macos-foreground-detection.md)
- [Windows Detection Requirements](./windows-foreground-detection.md)

Quick summary:

- **Windows**: foreground app, window title, and system media data use native Windows APIs and do not require an extra accessibility permission prompt.
- **macOS**: foreground app capture uses the native bridge, but window title capture requires Accessibility permission and media capture requires `nowplaying-cli`.
- **Linux**: foreground detection depends on the active display stack. X11 uses `xprop`; GNOME Wayland needs the Focused Window D-Bus extension; KDE Plasma Wayland needs `kdotool`; media capture needs `playerctl` plus an MPRIS-capable player.

The built-in self-test is the fastest way to verify the current machine. For media checks, `No media is currently playing` is expected when playback is idle and should not be treated as a platform failure.

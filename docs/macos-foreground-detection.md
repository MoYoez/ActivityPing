# macOS Detection Requirements

This document defines what must be available on macOS for foreground app, window title, and media detection to be considered normal.

## Normal detection checklist

macOS should be considered working when all of the following are true:

1. Foreground app detection returns the current frontmost app name.
2. Window title detection returns the focused window title for apps that expose one.
3. Media detection can read active now-playing data through `nowplaying-cli`.
4. The built-in self-test reports `Foreground app capture OK`.
5. When the self-test is run against an app with a visible titled window and Accessibility permission is granted, it should report `Window title capture OK`.
6. Media self-test should report either `Media capture OK` or `No media is currently playing`.

## Foreground app requirements

Foreground app detection uses the native macOS bridge and should work when:

- `NSWorkspace.sharedWorkspace.frontmostApplication` returns the current app
- the frontmost app exposes a localized name

No AppleScript or `osascript` bridge is required for the current implementation.

## Window title requirements

Window title detection uses the Accessibility API and is considered normal only when:

- ActivityPing has macOS Accessibility permission
- the frontmost app has a valid process identifier
- the frontmost app exposes a focused window through `kAXFocusedWindowAttribute`
- that focused window exposes a non-empty title through `kAXTitleAttribute`

Important behavior:

- Accessibility permission is required for window title capture.
- Some apps may still return an empty or unstable title even after permission is granted.
- If the current app does not expose a usable title, foreground app detection can still work.
- `Window title is empty` should be re-checked against a normal titled app before treating it as a platform failure.

The expected manual path is:

- `System Settings > Privacy & Security > Accessibility`

## Foreground app icon requirements

Foreground app icon detection works best when the frontmost app exposes a bundle identifier that can be resolved through AppKit.

## Media detection requirements

Media detection is considered normal when:

- `nowplaying-cli` is installed and executable
- the binary is available in `PATH`, `/opt/homebrew/bin/nowplaying-cli`, or `/usr/local/bin/nowplaying-cli`
- the current media app exposes usable now-playing metadata

Install command:

- `brew install nowplaying-cli`

`No media is currently playing` is a normal self-test result when playback is idle. A failure is only expected when `nowplaying-cli` is missing, times out, or returns invalid data.

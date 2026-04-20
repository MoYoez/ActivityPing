# Windows Detection Requirements

This document defines what must be available on Windows for foreground app, window title, and media detection to be considered normal.

## Normal detection checklist

Windows should be considered working when all of the following are true:

1. Foreground app detection returns the executable name for the current foreground window.
2. Window title detection returns the current window title when the app exposes one.
3. Media detection can read the current Global System Media Transport Controls session when a compatible player is active.
4. The built-in self-test reports `Foreground app capture OK`.
5. When the self-test is run against an app with a visible title bar, it should report `Window title capture OK`.
6. Media self-test should report either `Media capture OK` or `No media is currently playing`.

## Foreground app requirements

Foreground app detection uses native Win32 APIs and is considered normal when:

- `GetForegroundWindow` returns a valid window handle
- `GetWindowThreadProcessId` resolves a valid process ID
- `QueryFullProcessImageNameW` can resolve the foreground process path
- the executable filename can be derived from that path

No extra accessibility permission is required by the current Windows implementation.

## Window title requirements

Window title detection is considered normal when:

- `GetForegroundWindow` returns a valid window handle
- `GetWindowTextLengthW` reports a readable title length or zero
- `GetWindowTextW` returns the current title when the app exposes one

Important behavior:

- A zero-length title is not always a platform failure; some windows simply do not expose a usable title.
- When the current app exposes no title, foreground app detection can still work.
- `Window title is empty` should be re-checked against a normal titled window before treating it as a platform gap.

## Foreground app icon requirements

Foreground app icon detection works best when the process image path can be resolved and the executable exposes a readable icon resource.

## Media detection requirements

Media detection uses the Windows Global System Media Transport Controls APIs and is considered normal when:

- WinRT media session initialization succeeds
- `GlobalSystemMediaTransportControlsSessionManager` returns a current session
- the active session exposes readable media properties
- the active player publishes title, artist, album, or playback state through the system media session APIs

No extra accessibility permission is required for media capture.

`No media is currently playing` is a normal self-test result when playback is idle. A failure is only expected when the current player does not expose a usable system media session or the native API calls fail.

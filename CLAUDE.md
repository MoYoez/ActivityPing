# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

ActivityPing is a Tauri 2 desktop app that captures the foreground app, window title, and media metadata, runs it through local rules/filters, and publishes a cleaned-up status to a built-in monitor and Discord Rich Presence. Frontend is React 19 + TypeScript (`src/`); backend is Rust (`src-tauri/src/`).

## Commands

Package manager is **pnpm** (run `corepack enable` first). Node 20+ and a Rust toolchain are required.

- `pnpm install` — install JS deps
- `pnpm tauri dev` — run the full app in development (Vite dev server on port 1420 + Rust backend)
- `pnpm build` — type-check and build the frontend bundle only (`tsc && vite build`)
- `pnpm tauri build` — build desktop packages
- `pnpm dev` — frontend-only Vite server (no backend; most features won't work)

Rust backend (run from `src-tauri/`):
- `cargo test` — run all backend tests
- `cargo test <name>` — run a single test by substring (e.g. `cargo test smart`)
- `cargo build` / `cargo clippy` — build / lint Rust

There is **no ESLint**. Frontend "linting" is TypeScript strict mode (`tsconfig.json` enables `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`), enforced via `tsc` during `pnpm build`. CI (`.github/workflows/build-desktop.yml`) only runs `pnpm tauri build` across Windows/macOS/Linux — it does not run `cargo test`, so run tests locally.

## Architecture

### Frontend ↔ backend boundary

The frontend calls the backend exclusively through Tauri `invoke` commands wrapped in `src/lib/api.ts`. Each wrapper maps to a `#[tauri::command]` in `src-tauri/src/commands/`, registered in the `invoke_handler!` list in `src-tauri/src/lib.rs` (desktop and mobile have separate command lists). When adding a command you must touch all three: the Rust command, the `lib.rs` registration, and the `api.ts` wrapper.

All types crossing the boundary use serde `camelCase`. The shared config object is one large `ClientConfig` struct mirrored between `src/types.ts` and `src-tauri/src/models/config/`. Backend results are wrapped in `ApiResult<T>` (`success`/`status`/`data`/`error`).

### Two runtime workers

The core runtime is two long-lived background threads, each a Tauri-managed singleton with an identical concurrency pattern (`Arc<Mutex<Inner>>` state + `Arc<AtomicBool>` stop flag + `AtomicU64` run-id sequence; `start()` spawns the thread, `stop()` flips the flag and joins with a timeout):

- `ReporterRuntime` (`src-tauri/src/realtime_reporter/`) — the local capture loop that feeds the in-app monitor.
- `DiscordPresenceRuntime` (`src-tauri/src/discord_presence/`) — pushes presence to the Discord client.

The reporter loop (`realtime_reporter/worker/runner.rs`) is the heart of capture: poll platform → resolve → diff against last signature → emit on change or heartbeat → exponential backoff on errors.

### Capture pipeline

`platform layer → ForegroundSnapshot + MediaInfo → rules::resolve_activity → ResolvedActivity → discord_text builders → Discord payload`

- **Platform layer** (`src-tauri/src/platform/`): `mod.rs` selects an OS impl via `cfg` attributes (`windows/`, `macos/`, `linux/`, `stub/`). Each impl exposes the same surface: `get_foreground_snapshot_for_reporting`, `get_now_playing_with_options`, `get_foreground_app_icon`, `run_self_test`, `request_accessibility_permission`, and (Windows/macOS only) `subscribe_foreground_changes`. `MediaInfo.playback_state` is a normalized string (`"playing"`/`"paused"`/`"stopped"`/empty) — use the `is_playing()`/`is_paused()` accessors, not a raw bool. Artwork/source-icon are carried as raw `MediaArtwork` bytes (not data URLs) because the Discord layer uploads them. macOS media uses a bundled `mediaremote-adapter` (Perl script + framework under `src-tauri/resources/`, prepared by `pnpm prepare:mediaremote-adapter`); Linux needs `xprop`/`playerctl`/etc. See `docs/` for per-OS requirements.
- **Foreground event wakeup**: `subscribe_foreground_changes()` returns `Option<Receiver<()>>` (Windows `SetWinEventHook`, macOS `NSWorkspace` observer; `None` on Linux). The reporter and discord worker loops use `sleep_with_stop_and_wakeup(...)` to re-capture immediately on app switch instead of waiting out the poll interval; Linux falls back to plain polling.
- **Rules engine** (`src-tauri/src/rules/`): `resolve_activity` applies app filters (blacklist/whitelist/name-only mask/media-source blocklist), matches process-based message rules with optional title subrules, and builds the per-mode Discord text. Reporting modes are Smart/Music/App/Custom.

### `normalize_client_config` is load-bearing

`src-tauri/src/rules/normalize/` contains the single source of truth for config normalization (trimming, clamping, list dedup, and migrating legacy `legacy_*` fields onto current fields). It runs on state load, on state save (`state_store.rs`), and at worker start. When adding config fields, decide whether they need normalization here; legacy-field migration also lives here.

### State persistence

`src-tauri/src/state_store.rs` reads/writes `client-state.json` in the Tauri app-config dir using atomic temp-file-then-rename writes, owner-only permissions on Unix, and a `.json.corrupt` backup on parse failure. Holds `ClientConfig` plus app/play-source history.

### Localization

Backend text (log entries, self-test probes, errors) is locale-aware via `BackendLocale` (ZhCn/EnUs, `backend_locale.rs`). The backend returns a localization key + params + a fallback string (e.g. `title_key`/`title_params`/`title`), and the frontend resolves the key, falling back to the provided text. Backend error codes like `backendErrors.*` are mapped to messages in `src/lib/api.ts`.

### Frontend structure

`App.tsx` is a large orchestrator: it pulls UI state from a Zustand store (`src/store/`) and composes "view props" via factory functions in `src/app/` (`createSettingsViewProps`, `createRuntimeViewProps`, `createOverlayProps`, etc.) that are passed down to page components in `src/components/pages/`. Logic for rules/config editing and import/export lives in `src/lib/rules/`. Prefer extending the existing view-props factories over adding state directly in `App.tsx`.

### Artwork uploads

Discord image slots require public URLs, so local images (app icons, album art, gallery images) are uploaded through a user-configured external HTTP uploader to obtain reachable URLs. The publisher logic is in `src-tauri/src/artwork_server/`; the request/response contract and a reference Python server are documented in `docs/configuration-and-runtime.md` and `docs/examples/`.

## Conventions

- Rust modules follow a "split by concern" layout: a domain has a `mod.rs` that wires submodules and re-exports the public surface, with implementation in `worker/`, `client/`, `assets/`, `tests/` subdirs. A recent refactor explicitly split oversized modules — keep new files small and focused rather than growing one file.
- Backend tests live in per-domain `tests/` modules (e.g. `rules/tests/`, `discord_presence/tests/`) gated by `#[cfg(test)]`, with shared `fixtures.rs`.
- `#[cfg(desktop)]` / `#[cfg(mobile)]` gate platform-only commands and the tray/autostart/single-instance plugins.

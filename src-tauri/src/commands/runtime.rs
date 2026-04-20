use tauri::{AppHandle, State};

use crate::{
    backend_locale::load_locale,
    discord_presence::DiscordPresenceRuntime,
    models::{ApiResult, ClientConfig, DiscordPresenceSnapshot, RealtimeReporterSnapshot},
    realtime_reporter::{snapshot_result, ReporterRuntime},
    tray,
};

fn refresh_tray_menu(app: &AppHandle) {
    if let Err(error) = tray::refresh_tray(app) {
        eprintln!("Failed to refresh the tray menu: {error}");
    }
}

pub fn hide_to_tray(app: AppHandle) -> Result<(), String> {
    tray::hide_main_window(&app)
}

pub fn start_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    match reporter.start(config, load_locale(&app)) {
        Ok(snapshot) => {
            refresh_tray_menu(&app);
            Ok(ApiResult::success(200, snapshot))
        }
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            reporter_start_error_code(&error).map(str::to_string),
            error,
            None,
            None,
        )),
    }
}

pub fn stop_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    let snapshot = reporter.stop();
    refresh_tray_menu(&app);
    Ok(ApiResult::success(200, snapshot))
}

pub fn get_realtime_reporter_snapshot(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    Ok(snapshot_result(&reporter))
}

pub fn start_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    match discord_presence_runtime.start(config, load_locale(&app)) {
        Ok(snapshot) => {
            refresh_tray_menu(&app);
            Ok(ApiResult::success(200, snapshot))
        }
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            discord_start_error_code(&error).map(str::to_string),
            error,
            None,
            None,
        )),
    }
}

pub fn stop_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    let snapshot = discord_presence_runtime.stop();
    refresh_tray_menu(&app);
    Ok(ApiResult::success(200, snapshot))
}

pub fn get_discord_presence_snapshot(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    Ok(ApiResult::success(200, discord_presence_runtime.snapshot()))
}

fn reporter_start_error_code(error: &str) -> Option<&'static str> {
    if error.contains("still stopping") {
        Some("backendErrors.reporterWorkerStopping")
    } else {
        None
    }
}

fn discord_start_error_code(error: &str) -> Option<&'static str> {
    if error.contains("Discord application ID is required") {
        Some("backendErrors.discordConfigAppIdMissing")
    } else if error.contains("still stopping") {
        Some("backendErrors.discordWorkerStopping")
    } else {
        None
    }
}

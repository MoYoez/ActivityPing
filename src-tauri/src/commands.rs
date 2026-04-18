use tauri::{AppHandle, State};

use crate::{
    backend_locale::load_locale,
    discord_presence::DiscordPresenceRuntime,
    models::{
        default_client_capabilities, ApiResult, AppStatePayload, ClientCapabilities, ClientConfig,
        DiscordPresenceSnapshot, PlatformSelfTestResult,
    },
    platform, state_store,
};

#[cfg(desktop)]
use crate::realtime_reporter::{snapshot_result, ReporterRuntime};
#[cfg(desktop)]
use crate::tray;

#[tauri::command]
pub fn load_app_state(app: AppHandle) -> Result<AppStatePayload, String> {
    state_store::load_app_state(&app)
}

#[tauri::command]
pub fn save_app_state(app: AppHandle, payload: AppStatePayload) -> Result<(), String> {
    state_store::save_app_state(&app, &payload)
}

#[tauri::command]
pub fn get_client_capabilities() -> Result<ApiResult<ClientCapabilities>, String> {
    Ok(ApiResult::success(200, default_client_capabilities()))
}

#[cfg(desktop)]
#[tauri::command]
pub fn hide_to_tray(app: AppHandle) -> Result<(), String> {
    tray::hide_main_window(&app)
}

#[cfg(desktop)]
#[tauri::command]
pub fn start_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
    match reporter.start(config, load_locale(&app)) {
        Ok(snapshot) => Ok(ApiResult::success(200, snapshot)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            reporter_start_error_code(&error).map(str::to_string),
            error,
            None,
            None,
        )),
    }
}

#[cfg(desktop)]
#[tauri::command]
pub fn stop_realtime_reporter(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
    Ok(ApiResult::success(200, reporter.stop()))
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_realtime_reporter_snapshot(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
    Ok(snapshot_result(&reporter))
}

#[cfg(desktop)]
#[tauri::command]
pub fn start_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    match discord_presence_runtime.start(config, load_locale(&app)) {
        Ok(snapshot) => Ok(ApiResult::success(200, snapshot)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            discord_start_error_code(&error).map(str::to_string),
            error,
            None,
            None,
        )),
    }
}

#[cfg(desktop)]
#[tauri::command]
pub fn stop_discord_presence_sync(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    Ok(ApiResult::success(200, discord_presence_runtime.stop()))
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_discord_presence_snapshot(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    Ok(ApiResult::success(200, discord_presence_runtime.snapshot()))
}

#[tauri::command]
pub fn run_platform_self_test() -> Result<ApiResult<PlatformSelfTestResult>, String> {
    Ok(ApiResult::success(200, platform::run_self_test()))
}

#[tauri::command]
pub fn request_accessibility_permission() -> Result<ApiResult<bool>, String> {
    match platform::request_accessibility_permission() {
        Ok(granted) => Ok(ApiResult::success(200, granted)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            Some("backendErrors.accessibilityPermissionUnsupported".to_string()),
            error,
            None,
            None,
        )),
    }
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

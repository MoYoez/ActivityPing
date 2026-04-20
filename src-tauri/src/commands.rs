use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use serde_json::json;
use tauri::{AppHandle, State};

use crate::{
    custom_assets,
    backend_locale::load_locale,
    discord_presence::DiscordPresenceRuntime,
    models::{
        default_client_capabilities, ApiResult, AppStatePayload, ClientCapabilities, ClientConfig,
        DiscordCustomAsset, DiscordPresenceSnapshot, PlatformSelfTestResult,
    },
    platform, state_store,
};

const PLATFORM_SELF_TEST_TIMEOUT_MS: u64 = 4_000;
static PLATFORM_SELF_TEST_IN_FLIGHT: AtomicBool = AtomicBool::new(false);

struct PlatformSelfTestRunGuard;

impl Drop for PlatformSelfTestRunGuard {
    fn drop(&mut self) {
        PLATFORM_SELF_TEST_IN_FLIGHT.store(false, Ordering::Release);
    }
}

#[cfg(desktop)]
use crate::realtime_reporter::{snapshot_result, ReporterRuntime};
#[cfg(desktop)]
use crate::tray;
#[cfg(desktop)]
use tauri_plugin_autostart::ManagerExt;

#[cfg(desktop)]
fn refresh_tray_menu(app: &AppHandle) {
    if let Err(error) = tray::refresh_tray(app) {
        eprintln!("Failed to refresh the tray menu: {error}");
    }
}

#[tauri::command]
pub fn load_app_state(app: AppHandle) -> Result<AppStatePayload, String> {
    state_store::load_app_state(&app)
}

#[tauri::command]
pub fn save_app_state(app: AppHandle, payload: AppStatePayload) -> Result<(), String> {
    state_store::save_app_state(&app, &payload)?;
    #[cfg(desktop)]
    if let Err(error) = tray::refresh_tray(&app) {
        eprintln!("Failed to refresh the tray menu after saving state: {error}");
    }
    Ok(())
}

#[tauri::command]
pub fn import_discord_custom_asset(
    app: AppHandle,
    name: String,
    file_name: String,
    content_type: String,
    base64_data: String,
) -> Result<ApiResult<Vec<DiscordCustomAsset>>, String> {
    match custom_assets::import_discord_custom_asset(
        &app,
        &name,
        &file_name,
        &content_type,
        &base64_data,
    ) {
        Ok(assets) => Ok(ApiResult::success(200, assets)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            None::<String>,
            error,
            None,
            None,
        )),
    }
}

#[tauri::command]
pub fn delete_discord_custom_asset(
    app: AppHandle,
    asset_id: String,
) -> Result<ApiResult<Vec<DiscordCustomAsset>>, String> {
    match custom_assets::delete_discord_custom_asset(&app, &asset_id) {
        Ok(assets) => Ok(ApiResult::success(200, assets)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            None::<String>,
            error,
            None,
            None,
        )),
    }
}

#[tauri::command]
pub fn get_discord_custom_asset_preview(
    app: AppHandle,
    asset_id: String,
) -> Result<ApiResult<String>, String> {
    match custom_assets::get_discord_custom_asset_preview(&app, &asset_id) {
        Ok(preview) => Ok(ApiResult::success(200, preview)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            None::<String>,
            error,
            None,
            None,
        )),
    }
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
pub fn set_launch_on_startup(app: AppHandle, enabled: bool) -> Result<ApiResult<bool>, String> {
    let autostart_manager = app.autolaunch();
    let result = if enabled {
        autostart_manager.enable()
    } else {
        autostart_manager.disable()
    };

    match result {
        Ok(()) => Ok(ApiResult::success(200, enabled)),
        Err(error) => Ok(ApiResult::failure_localized(
            400,
            None::<String>,
            format!("Failed to update launch with system: {error}"),
            None,
            None,
        )),
    }
}

#[cfg(desktop)]
#[tauri::command]
pub fn start_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
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

#[cfg(desktop)]
#[tauri::command]
pub fn stop_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<crate::models::RealtimeReporterSnapshot>, String> {
    let snapshot = reporter.stop();
    refresh_tray_menu(&app);
    Ok(ApiResult::success(200, snapshot))
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

#[cfg(desktop)]
#[tauri::command]
pub fn stop_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    let snapshot = discord_presence_runtime.stop();
    refresh_tray_menu(&app);
    Ok(ApiResult::success(200, snapshot))
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_discord_presence_snapshot(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    Ok(ApiResult::success(200, discord_presence_runtime.snapshot()))
}

#[tauri::command]
pub async fn run_platform_self_test() -> Result<ApiResult<PlatformSelfTestResult>, String> {
    if PLATFORM_SELF_TEST_IN_FLIGHT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Ok(ApiResult::failure_localized(
            409,
            Some("backendErrors.platformSelfTestAlreadyRunning".to_string()),
            "A previous platform self-test is still waiting on Windows.",
            None,
            None,
        ));
    }

    let timeout = Duration::from_millis(PLATFORM_SELF_TEST_TIMEOUT_MS);
    let (sender, receiver) = mpsc::channel();

    std::thread::Builder::new()
        .name("platform-self-test".to_string())
        .spawn(move || {
            let _guard = PlatformSelfTestRunGuard;
            let _ = sender.send(platform::run_self_test());
        })
        .map_err(|error| {
            PLATFORM_SELF_TEST_IN_FLIGHT.store(false, Ordering::Release);
            format!("Failed to start platform self-test worker: {error}")
        })?;

    match tauri::async_runtime::spawn_blocking(move || receiver.recv_timeout(timeout)).await {
        Ok(Ok(result)) => Ok(ApiResult::success(200, result)),
        Ok(Err(mpsc::RecvTimeoutError::Timeout)) => Ok(ApiResult::failure_localized(
            408,
            Some("backendErrors.platformSelfTestTimedOut".to_string()),
            "Platform self-test timed out. A Windows media or window API did not return in time.",
            None,
            Some(json!({ "timeoutMs": PLATFORM_SELF_TEST_TIMEOUT_MS })),
        )),
        Ok(Err(mpsc::RecvTimeoutError::Disconnected)) => Ok(ApiResult::failure_localized(
            500,
            Some("backendErrors.platformSelfTestFailed".to_string()),
            "Platform self-test worker stopped before returning a result.",
            None,
            None,
        )),
        Err(error) => Ok(ApiResult::failure_localized(
            500,
            Some("backendErrors.platformSelfTestFailed".to_string()),
            format!("Platform self-test worker failed: {error}"),
            None,
            None,
        )),
    }
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

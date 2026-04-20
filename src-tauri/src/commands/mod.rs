mod assets;
mod platform;
mod state;

#[cfg(desktop)]
mod runtime;

use tauri::AppHandle;

#[cfg(desktop)]
use tauri::State;

use crate::models::{
    ApiResult, AppStatePayload, ClientCapabilities, DiscordCustomAsset, PlatformSelfTestResult,
};

#[cfg(desktop)]
use crate::{
    discord_presence::DiscordPresenceRuntime,
    models::{ClientConfig, DiscordPresenceSnapshot, RealtimeReporterSnapshot},
    realtime_reporter::ReporterRuntime,
};

#[tauri::command]
pub fn load_app_state(app: AppHandle) -> Result<AppStatePayload, String> {
    state::load_app_state(app)
}

#[tauri::command]
pub fn save_app_state(app: AppHandle, payload: AppStatePayload) -> Result<(), String> {
    state::save_app_state(app, payload)
}

#[tauri::command]
pub fn import_discord_custom_asset(
    app: AppHandle,
    name: String,
    file_name: String,
    content_type: String,
    base64_data: String,
) -> Result<ApiResult<Vec<DiscordCustomAsset>>, String> {
    assets::import_discord_custom_asset(app, name, file_name, content_type, base64_data)
}

#[tauri::command]
pub fn delete_discord_custom_asset(
    app: AppHandle,
    asset_id: String,
) -> Result<ApiResult<Vec<DiscordCustomAsset>>, String> {
    assets::delete_discord_custom_asset(app, asset_id)
}

#[tauri::command]
pub fn get_discord_custom_asset_preview(
    app: AppHandle,
    asset_id: String,
) -> Result<ApiResult<String>, String> {
    assets::get_discord_custom_asset_preview(app, asset_id)
}

#[tauri::command]
pub fn get_client_capabilities() -> Result<ApiResult<ClientCapabilities>, String> {
    state::get_client_capabilities()
}

#[cfg(desktop)]
#[tauri::command]
pub fn hide_to_tray(app: AppHandle) -> Result<(), String> {
    runtime::hide_to_tray(app)
}

#[cfg(desktop)]
#[tauri::command]
pub fn start_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    runtime::start_realtime_reporter(app, reporter, config)
}

#[cfg(desktop)]
#[tauri::command]
pub fn stop_realtime_reporter(
    app: AppHandle,
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    runtime::stop_realtime_reporter(app, reporter)
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_realtime_reporter_snapshot(
    reporter: State<'_, ReporterRuntime>,
) -> Result<ApiResult<RealtimeReporterSnapshot>, String> {
    runtime::get_realtime_reporter_snapshot(reporter)
}

#[cfg(desktop)]
#[tauri::command]
pub fn start_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
    config: ClientConfig,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    runtime::start_discord_presence_sync(app, discord_presence_runtime, config)
}

#[cfg(desktop)]
#[tauri::command]
pub fn stop_discord_presence_sync(
    app: AppHandle,
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    runtime::stop_discord_presence_sync(app, discord_presence_runtime)
}

#[cfg(desktop)]
#[tauri::command]
pub fn get_discord_presence_snapshot(
    discord_presence_runtime: State<'_, DiscordPresenceRuntime>,
) -> Result<ApiResult<DiscordPresenceSnapshot>, String> {
    runtime::get_discord_presence_snapshot(discord_presence_runtime)
}

#[tauri::command]
pub async fn run_platform_self_test() -> Result<ApiResult<PlatformSelfTestResult>, String> {
    platform::run_platform_self_test().await
}

#[tauri::command]
pub fn request_accessibility_permission() -> Result<ApiResult<bool>, String> {
    platform::request_accessibility_permission()
}

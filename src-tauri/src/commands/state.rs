use tauri::AppHandle;

use crate::{
    models::{default_client_capabilities, ApiResult, AppStatePayload, ClientCapabilities},
    state_store,
};

#[cfg(desktop)]
use crate::tray;

pub fn load_app_state(app: AppHandle) -> Result<AppStatePayload, String> {
    state_store::load_app_state(&app)
}

pub fn save_app_state(app: AppHandle, payload: AppStatePayload) -> Result<(), String> {
    state_store::save_app_state(&app, &payload)?;
    refresh_saved_state_tray(&app);
    Ok(())
}

pub fn get_client_capabilities() -> Result<ApiResult<ClientCapabilities>, String> {
    Ok(ApiResult::success(200, default_client_capabilities()))
}

#[cfg(desktop)]
fn refresh_saved_state_tray(app: &AppHandle) {
    if let Err(error) = tray::refresh_tray(app) {
        eprintln!("Failed to refresh the tray menu after saving state: {error}");
    }
}

#[cfg(not(desktop))]
fn refresh_saved_state_tray(_: &AppHandle) {}

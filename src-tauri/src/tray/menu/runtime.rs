use tauri::{
    menu::{MenuItem, Submenu},
    AppHandle, Manager,
};

use crate::{
    backend_locale::BackendLocale, discord_presence::DiscordPresenceRuntime,
    realtime_reporter::ReporterRuntime,
};

use super::super::{
    ids::{MENU_ID_RUNTIME, MENU_ID_RUNTIME_START, MENU_ID_RUNTIME_STOP},
    text::{format_tray_error, tray_text, TrayText},
};

pub(super) fn build_runtime_submenu(
    app: &AppHandle,
    locale: BackendLocale,
) -> Result<Submenu<tauri::Wry>, String> {
    let reporter_running = app.state::<ReporterRuntime>().snapshot().running;
    let discord_running = app.state::<DiscordPresenceRuntime>().snapshot().running;
    let runtime_running = reporter_running || discord_running;
    let runtime_submenu = Submenu::with_id(
        app,
        MENU_ID_RUNTIME,
        tray_text(locale, TrayText::Runtime),
        true,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    let start_item = MenuItem::with_id(
        app,
        MENU_ID_RUNTIME_START,
        tray_text(locale, TrayText::Start),
        !runtime_running,
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let stop_item = MenuItem::with_id(
        app,
        MENU_ID_RUNTIME_STOP,
        tray_text(locale, TrayText::Stop),
        runtime_running,
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    runtime_submenu
        .append(&start_item)
        .and_then(|_| runtime_submenu.append(&stop_item))
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    Ok(runtime_submenu)
}

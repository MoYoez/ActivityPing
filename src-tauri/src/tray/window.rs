use tauri::{AppHandle, Manager};

use crate::backend_locale::load_locale;

use super::text::{tray_text, TrayText};

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let locale = load_locale(app);
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| tray_text(locale, TrayText::MainWindowNotFound).to_string())?;

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        let _ = app.set_dock_visibility(true);
    }

    let _ = window.set_skip_taskbar(false);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let locale = load_locale(app);
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| tray_text(locale, TrayText::MainWindowNotFound).to_string())?;
    let _ = window.hide();
    let _ = window.set_skip_taskbar(true);

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_dock_visibility(false);
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    Ok(())
}

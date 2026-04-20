mod quick_switch;
mod runtime;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    AppHandle,
};

use crate::{backend_locale::load_locale, state_store};

use super::{
    ids::{MENU_ID_HIDE, MENU_ID_QUIT, MENU_ID_SHOW},
    text::{format_tray_error, tray_text, TrayText},
};

pub(super) fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let locale = load_locale(app);
    let config = state_store::load_app_state(app)
        .map(|state| state.config)
        .unwrap_or_default();

    let menu = Menu::new(app)
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let show_item = MenuItem::with_id(
        app,
        MENU_ID_SHOW,
        tray_text(locale, TrayText::Show),
        true,
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let hide_item = MenuItem::with_id(
        app,
        MENU_ID_HIDE,
        tray_text(locale, TrayText::Hide),
        true,
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let runtime_submenu = runtime::build_runtime_submenu(app, locale)?;
    let quick_switch = quick_switch::build_quick_switch_submenu(app, locale, &config)?;
    let quit_item = MenuItem::with_id(
        app,
        MENU_ID_QUIT,
        tray_text(locale, TrayText::Quit),
        true,
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let separator_one = PredefinedMenuItem::separator(app)
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let separator_two = PredefinedMenuItem::separator(app)
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let separator_three = PredefinedMenuItem::separator(app)
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    menu.append(&show_item)
        .and_then(|_| menu.append(&hide_item))
        .and_then(|_| menu.append(&separator_one))
        .and_then(|_| menu.append(&runtime_submenu))
        .and_then(|_| menu.append(&separator_two))
        .and_then(|_| menu.append(&quick_switch))
        .and_then(|_| menu.append(&separator_three))
        .and_then(|_| menu.append(&quit_item))
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    Ok(menu)
}

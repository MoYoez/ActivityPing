use tauri::{
    menu::{CheckMenuItem, PredefinedMenuItem, Submenu},
    AppHandle,
};

use crate::{
    backend_locale::BackendLocale,
    models::{ClientConfig, DiscordReportMode},
};

use super::super::{
    ids::{
        MENU_ID_QUICK_SWITCH, MENU_ID_SWITCH_APP, MENU_ID_SWITCH_CUSTOM,
        MENU_ID_SWITCH_CUSTOM_CURRENT, MENU_ID_SWITCH_MIXED, MENU_ID_SWITCH_MUSIC,
    },
    quick_switch::{active_custom_preset_index, custom_preset_menu_id},
    text::{format_tray_error, mode_label, preset_label, tray_text, TrayText},
};

fn build_custom_submenu(
    app: &AppHandle,
    locale: BackendLocale,
    config: &ClientConfig,
) -> Result<Submenu<tauri::Wry>, String> {
    let active_preset_index = active_custom_preset_index(config);
    let submenu = Submenu::with_id(
        app,
        MENU_ID_SWITCH_CUSTOM,
        tray_text(locale, TrayText::Custom),
        true,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    let current_item = CheckMenuItem::with_id(
        app,
        MENU_ID_SWITCH_CUSTOM_CURRENT,
        tray_text(locale, TrayText::CurrentCustomSettings),
        true,
        config.discord_report_mode == DiscordReportMode::Custom && active_preset_index.is_none(),
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    submenu
        .append(&current_item)
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    if !config.discord_custom_presets.is_empty() {
        let separator = PredefinedMenuItem::separator(app)
            .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
        submenu
            .append(&separator)
            .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

        for (index, preset) in config.discord_custom_presets.iter().enumerate() {
            let preset_item = CheckMenuItem::with_id(
                app,
                custom_preset_menu_id(index),
                preset_label(locale, preset, index),
                true,
                active_preset_index == Some(index),
                None::<&str>,
            )
            .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
            submenu
                .append(&preset_item)
                .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
        }
    }

    Ok(submenu)
}

pub(super) fn build_quick_switch_submenu(
    app: &AppHandle,
    locale: BackendLocale,
    config: &ClientConfig,
) -> Result<Submenu<tauri::Wry>, String> {
    let submenu = Submenu::with_id(
        app,
        MENU_ID_QUICK_SWITCH,
        tray_text(locale, TrayText::QuickSwitch),
        true,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    let mixed_item = CheckMenuItem::with_id(
        app,
        MENU_ID_SWITCH_MIXED,
        mode_label(locale, &DiscordReportMode::Mixed),
        true,
        config.discord_report_mode == DiscordReportMode::Mixed,
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let music_item = CheckMenuItem::with_id(
        app,
        MENU_ID_SWITCH_MUSIC,
        mode_label(locale, &DiscordReportMode::Music),
        true,
        config.discord_report_mode == DiscordReportMode::Music,
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let app_item = CheckMenuItem::with_id(
        app,
        MENU_ID_SWITCH_APP,
        mode_label(locale, &DiscordReportMode::App),
        true,
        config.discord_report_mode == DiscordReportMode::App,
        None::<&str>,
    )
    .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;
    let custom_submenu = build_custom_submenu(app, locale, config)?;
    let separator = PredefinedMenuItem::separator(app)
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    submenu
        .append(&mixed_item)
        .and_then(|_| submenu.append(&music_item))
        .and_then(|_| submenu.append(&app_item))
        .and_then(|_| submenu.append(&separator))
        .and_then(|_| submenu.append(&custom_submenu))
        .map_err(|error| format_tray_error(locale, TrayText::CreateMenuFailed, error))?;

    Ok(submenu)
}

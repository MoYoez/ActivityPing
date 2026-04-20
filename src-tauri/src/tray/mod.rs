mod ids;
mod menu;
mod notice;
mod quick_switch;
mod runtime;
mod text;
mod window;

use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

use self::{
    ids::{
        MENU_ID_HIDE, MENU_ID_QUIT, MENU_ID_RUNTIME_START, MENU_ID_RUNTIME_STOP, MENU_ID_SHOW,
        TRAY_ID,
    },
    menu::build_tray_menu,
    quick_switch::parse_switch_target,
    runtime::{handle_quick_switch, handle_runtime_start, handle_runtime_stop},
    text::{format_tray_error, TrayText},
};
pub use window::{hide_main_window, show_main_window};

pub fn refresh_tray(app: &AppHandle) -> Result<(), String> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    let menu = build_tray_menu(app)?;
    tray.set_menu(Some(menu)).map_err(|error| {
        format_tray_error(
            crate::backend_locale::load_locale(app),
            TrayText::CreateMenuFailed,
            error,
        )
    })
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let locale = crate::backend_locale::load_locale(app);
    let menu = build_tray_menu(app)?;
    let icon = Image::from_bytes(include_bytes!("../../icons/32x32.png"))
        .map_err(|error| format_tray_error(locale, TrayText::LoadIconFailed, error))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_ID_SHOW => {
                let _ = show_main_window(app);
            }
            MENU_ID_HIDE => {
                let _ = hide_main_window(app);
            }
            MENU_ID_RUNTIME_START => {
                handle_runtime_start(app);
            }
            MENU_ID_RUNTIME_STOP => {
                handle_runtime_stop(app);
            }
            MENU_ID_QUIT => {
                app.exit(0);
            }
            menu_id => {
                if parse_switch_target(menu_id).is_some() {
                    handle_quick_switch(app, menu_id);
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|error| format_tray_error(locale, TrayText::CreateTrayFailed, error))?;

    Ok(())
}

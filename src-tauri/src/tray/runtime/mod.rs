mod control;
mod services;
mod switch;

use tauri::AppHandle;

pub(super) fn handle_runtime_start(app: &AppHandle) {
    control::handle_runtime_start(app);
}

pub(super) fn handle_runtime_stop(app: &AppHandle) {
    control::handle_runtime_stop(app);
}

pub(super) fn handle_quick_switch(app: &AppHandle, menu_id: &str) {
    switch::handle_quick_switch(app, menu_id);
}

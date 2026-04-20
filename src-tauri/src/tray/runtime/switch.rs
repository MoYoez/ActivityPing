use tauri::AppHandle;

use crate::{backend_locale::load_locale, rules::normalize_client_config, state_store};

use super::{
    super::{
        notice::{emit_tray_notice, failure_notice, show_tray_notice, success_notice},
        quick_switch::{apply_switch_target, configs_equal, parse_switch_target},
        refresh_tray,
    },
    services::restart_running_services,
};

pub(super) fn handle_quick_switch(app: &AppHandle, menu_id: &str) {
    let locale = load_locale(app);
    let notice = match parse_switch_target(menu_id) {
        Some(target) => {
            let mut state = match state_store::load_app_state(app) {
                Ok(state) => state,
                Err(error) => {
                    let notice = failure_notice(locale, error, false);
                    show_tray_notice(app, &notice);
                    emit_tray_notice(app, &notice);
                    return;
                }
            };
            let mut current_config = state.config.clone();
            normalize_client_config(&mut current_config);
            let mut next_config = current_config.clone();
            let selection = match apply_switch_target(&mut next_config, &target, locale) {
                Ok(selection) => selection,
                Err(error) => {
                    let notice = failure_notice(locale, error, false);
                    show_tray_notice(app, &notice);
                    emit_tray_notice(app, &notice);
                    return;
                }
            };
            normalize_client_config(&mut next_config);
            let changed = !configs_equal(&current_config, &next_config);

            if changed {
                state.config = next_config.clone();
                if let Err(error) = state_store::save_app_state(app, &state) {
                    let notice = failure_notice(locale, error, false);
                    show_tray_notice(app, &notice);
                    emit_tray_notice(app, &notice);
                    return;
                }
                if let Err(error) = refresh_tray(app) {
                    eprintln!("Failed to refresh the tray menu after a quick switch: {error}");
                }
            }

            if !changed {
                success_notice(locale, &selection, false, false)
            } else {
                match restart_running_services(app, &next_config) {
                    Ok(runtime_restarted) => {
                        success_notice(locale, &selection, true, runtime_restarted)
                    }
                    Err(error) => failure_notice(
                        locale,
                        if locale.is_en() {
                            format!(
                                "The new mode was saved, but the running services could not be updated: {error}"
                            )
                        } else {
                            format!("新模式已保存，但运行中的服务更新失败：{error}")
                        },
                        true,
                    ),
                }
            }
        }
        None => failure_notice(
            locale,
            if locale.is_en() {
                "Unsupported tray quick switch.".to_string()
            } else {
                "不支持的托盘快速切换操作。".to_string()
            },
            false,
        ),
    };

    show_tray_notice(app, &notice);
    emit_tray_notice(app, &notice);
}

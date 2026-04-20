use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

use crate::{backend_locale::BackendLocale, models::DiscordReportMode};

use super::{
    ids::TRAY_QUICK_SWITCH_EVENT,
    text::{mode_label, tray_text, TrayText},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TrayQuickSwitchNotice {
    tone: String,
    title: String,
    detail: String,
    reload_state: bool,
    refresh_runtime: bool,
}

#[derive(Clone)]
pub(super) enum AppliedSelection {
    Mode(DiscordReportMode),
    CustomCurrent,
    CustomPreset(String),
}

pub(super) fn success_notice(
    locale: BackendLocale,
    selection: &AppliedSelection,
    changed: bool,
    runtime_restarted: bool,
) -> TrayQuickSwitchNotice {
    let (title, tone) = match (selection, changed) {
        (AppliedSelection::Mode(mode), true) => (
            if locale.is_en() {
                format!("Switched to {} mode", mode_label(locale, mode))
            } else {
                format!("已切换到{}模式", mode_label(locale, mode))
            },
            "success",
        ),
        (AppliedSelection::Mode(mode), false) => (
            if locale.is_en() {
                format!("{} mode already active", mode_label(locale, mode))
            } else {
                format!("{}模式已在使用中", mode_label(locale, mode))
            },
            "info",
        ),
        (AppliedSelection::CustomCurrent, true) => (
            if locale.is_en() {
                "Switched to Custom mode".to_string()
            } else {
                "已切换到自定义模式".to_string()
            },
            "success",
        ),
        (AppliedSelection::CustomCurrent, false) => (
            if locale.is_en() {
                "Custom mode already active".to_string()
            } else {
                "自定义模式已在使用中".to_string()
            },
            "info",
        ),
        (AppliedSelection::CustomPreset(label), true) => (
            if locale.is_en() {
                format!("Applied custom preset: {label}")
            } else {
                format!("已应用自定义预设：{label}")
            },
            "success",
        ),
        (AppliedSelection::CustomPreset(label), false) => (
            if locale.is_en() {
                format!("Custom preset already active: {label}")
            } else {
                format!("该自定义预设已生效：{label}")
            },
            "info",
        ),
    };

    let detail = if !changed {
        if locale.is_en() {
            "No restart was needed."
        } else {
            "当前已经是这个配置，无需重启运行时。"
        }
    } else if runtime_restarted {
        if locale.is_en() {
            "Running services were updated immediately."
        } else {
            "运行中的服务已立即更新。"
        }
    } else if locale.is_en() {
        "The setting was saved and will be used when the runtime starts."
    } else {
        "设置已保存，运行时下次启动时会使用新配置。"
    };

    TrayQuickSwitchNotice {
        tone: tone.to_string(),
        title,
        detail: detail.to_string(),
        reload_state: changed,
        refresh_runtime: false,
    }
}

pub(super) fn failure_notice(
    locale: BackendLocale,
    detail: impl Into<String>,
    reload_state: bool,
) -> TrayQuickSwitchNotice {
    TrayQuickSwitchNotice {
        tone: "error".to_string(),
        title: tray_text(locale, TrayText::SwitchFailed).to_string(),
        detail: detail.into(),
        reload_state,
        refresh_runtime: false,
    }
}

pub(super) fn runtime_notice(
    tone: &str,
    title: impl Into<String>,
    detail: impl Into<String>,
) -> TrayQuickSwitchNotice {
    TrayQuickSwitchNotice {
        tone: tone.to_string(),
        title: title.into(),
        detail: detail.into(),
        reload_state: false,
        refresh_runtime: true,
    }
}

pub(super) fn show_tray_notice(app: &AppHandle, notice: &TrayQuickSwitchNotice) {
    let _ = app
        .notification()
        .builder()
        .title(&notice.title)
        .body(&notice.detail)
        .show();
}

pub(super) fn emit_tray_notice(app: &AppHandle, notice: &TrayQuickSwitchNotice) {
    let _ = app.emit(TRAY_QUICK_SWITCH_EVENT, notice.clone());
}

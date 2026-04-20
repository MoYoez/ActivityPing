use crate::{
    backend_locale::BackendLocale,
    models::{DiscordCustomPreset, DiscordReportMode},
};

pub(super) enum TrayText {
    MainWindowNotFound,
    Show,
    Hide,
    Runtime,
    Start,
    Stop,
    QuickSwitch,
    Custom,
    CurrentCustomSettings,
    Quit,
    CreateMenuFailed,
    LoadIconFailed,
    CreateTrayFailed,
    SwitchFailed,
}

pub(super) fn tray_text(locale: BackendLocale, key: TrayText) -> &'static str {
    match (locale.is_en(), key) {
        (true, TrayText::MainWindowNotFound) => "Main window not found.",
        (true, TrayText::Show) => "Open main window",
        (true, TrayText::Hide) => "Hide to background",
        (true, TrayText::Runtime) => "Runtime",
        (true, TrayText::Start) => "Start",
        (true, TrayText::Stop) => "Stop",
        (true, TrayText::QuickSwitch) => "Quick switch",
        (true, TrayText::Custom) => "Custom",
        (true, TrayText::CurrentCustomSettings) => "Current custom settings",
        (true, TrayText::Quit) => "Quit app",
        (true, TrayText::CreateMenuFailed) => "Failed to create the tray menu",
        (true, TrayText::LoadIconFailed) => "Failed to load the tray icon",
        (true, TrayText::CreateTrayFailed) => "Failed to create the system tray",
        (true, TrayText::SwitchFailed) => "Tray switch failed",
        (false, TrayText::MainWindowNotFound) => "找不到主窗口。",
        (false, TrayText::Show) => "打开主窗口",
        (false, TrayText::Hide) => "隐藏到后台",
        (false, TrayText::Runtime) => "运行时",
        (false, TrayText::Start) => "启动",
        (false, TrayText::Stop) => "关闭",
        (false, TrayText::QuickSwitch) => "快速切换",
        (false, TrayText::Custom) => "自定义",
        (false, TrayText::CurrentCustomSettings) => "当前自定义设置",
        (false, TrayText::Quit) => "退出应用",
        (false, TrayText::CreateMenuFailed) => "创建托盘菜单失败",
        (false, TrayText::LoadIconFailed) => "加载托盘图标失败",
        (false, TrayText::CreateTrayFailed) => "创建系统托盘失败",
        (false, TrayText::SwitchFailed) => "托盘切换失败",
    }
}

pub(super) fn mode_label(locale: BackendLocale, mode: &DiscordReportMode) -> &'static str {
    match (locale.is_en(), mode) {
        (true, DiscordReportMode::Mixed) => "Smart",
        (true, DiscordReportMode::Music) => "Music",
        (true, DiscordReportMode::App) => "App",
        (true, DiscordReportMode::Custom) => "Custom",
        (false, DiscordReportMode::Mixed) => "Smart",
        (false, DiscordReportMode::Music) => "Music",
        (false, DiscordReportMode::App) => "App",
        (false, DiscordReportMode::Custom) => "自定义",
    }
}

fn preset_fallback_label(locale: BackendLocale, index: usize) -> String {
    if locale.is_en() {
        format!("Preset {}", index + 1)
    } else {
        format!("预设 {}", index + 1)
    }
}

pub(super) fn preset_label(
    locale: BackendLocale,
    preset: &DiscordCustomPreset,
    index: usize,
) -> String {
    let name = preset.name.trim();
    if name.is_empty() {
        preset_fallback_label(locale, index)
    } else {
        name.to_string()
    }
}

pub(super) fn format_tray_error(
    locale: BackendLocale,
    key: TrayText,
    error: impl std::fmt::Display,
) -> String {
    format!("{}: {error}", tray_text(locale, key))
}

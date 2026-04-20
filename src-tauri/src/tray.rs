use serde::Serialize;
use tauri::{
    image::Image,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tauri_plugin_notification::NotificationExt;

use crate::{
    backend_locale::{load_locale, BackendLocale},
    discord_presence::DiscordPresenceRuntime,
    models::{
        ClientConfig, DiscordCustomPreset, DiscordReportMode, DiscordRichPresenceButtonConfig,
    },
    realtime_reporter::ReporterRuntime,
    rules::normalize_client_config,
    state_store,
};

pub const TRAY_QUICK_SWITCH_EVENT: &str = "tray-quick-switch-applied";

const TRAY_ID: &str = "activityping-tray";
const MENU_ID_SHOW: &str = "tray_show";
const MENU_ID_HIDE: &str = "tray_hide";
const MENU_ID_QUIT: &str = "tray_quit";
const MENU_ID_RUNTIME: &str = "tray_runtime";
const MENU_ID_RUNTIME_START: &str = "tray_runtime_start";
const MENU_ID_RUNTIME_STOP: &str = "tray_runtime_stop";
const MENU_ID_QUICK_SWITCH: &str = "tray_quick_switch";
const MENU_ID_SWITCH_MIXED: &str = "tray_switch_mixed";
const MENU_ID_SWITCH_MUSIC: &str = "tray_switch_music";
const MENU_ID_SWITCH_APP: &str = "tray_switch_app";
const MENU_ID_SWITCH_CUSTOM: &str = "tray_switch_custom";
const MENU_ID_SWITCH_CUSTOM_CURRENT: &str = "tray_switch_custom_current";
const MENU_ID_SWITCH_CUSTOM_PRESET_PREFIX: &str = "tray_switch_custom_preset::";
const DISCORD_CUSTOM_LINE_CUSTOM_VALUE: &str = "__custom__";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrayQuickSwitchNotice {
    tone: String,
    title: String,
    detail: String,
    reload_state: bool,
    refresh_runtime: bool,
}

#[derive(Clone)]
enum TraySwitchTarget {
    Mode(DiscordReportMode),
    CustomCurrent,
    CustomPreset(usize),
}

#[derive(Clone)]
enum AppliedSelection {
    Mode(DiscordReportMode),
    CustomCurrent,
    CustomPreset(String),
}

enum TrayText {
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

fn tray_text(locale: BackendLocale, key: TrayText) -> &'static str {
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

fn mode_label(locale: BackendLocale, mode: &DiscordReportMode) -> &'static str {
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

fn preset_label(locale: BackendLocale, preset: &DiscordCustomPreset, index: usize) -> String {
    let name = preset.name.trim();
    if name.is_empty() {
        preset_fallback_label(locale, index)
    } else {
        name.to_string()
    }
}

fn format_tray_error(
    locale: BackendLocale,
    key: TrayText,
    error: impl std::fmt::Display,
) -> String {
    format!("{}: {error}", tray_text(locale, key))
}

fn success_notice(
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

fn failure_notice(
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

fn runtime_notice(
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

fn show_tray_notice(app: &AppHandle, notice: &TrayQuickSwitchNotice) {
    let _ = app
        .notification()
        .builder()
        .title(&notice.title)
        .body(&notice.detail)
        .show();
}

fn emit_tray_notice(app: &AppHandle, notice: &TrayQuickSwitchNotice) {
    let _ = app.emit(TRAY_QUICK_SWITCH_EVENT, notice.clone());
}

fn custom_preset_menu_id(index: usize) -> String {
    format!("{MENU_ID_SWITCH_CUSTOM_PRESET_PREFIX}{index}")
}

fn parse_switch_target(id: &str) -> Option<TraySwitchTarget> {
    match id {
        MENU_ID_SWITCH_MIXED => Some(TraySwitchTarget::Mode(DiscordReportMode::Mixed)),
        MENU_ID_SWITCH_MUSIC => Some(TraySwitchTarget::Mode(DiscordReportMode::Music)),
        MENU_ID_SWITCH_APP => Some(TraySwitchTarget::Mode(DiscordReportMode::App)),
        MENU_ID_SWITCH_CUSTOM_CURRENT => Some(TraySwitchTarget::CustomCurrent),
        _ => id
            .strip_prefix(MENU_ID_SWITCH_CUSTOM_PRESET_PREFIX)
            .and_then(|value| value.parse::<usize>().ok())
            .map(TraySwitchTarget::CustomPreset),
    }
}

fn normalize_discord_line(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == DISCORD_CUSTOM_LINE_CUSTOM_VALUE {
        String::new()
    } else {
        trimmed.to_string()
    }
}

fn normalize_discord_buttons(
    buttons: &[DiscordRichPresenceButtonConfig],
) -> Vec<DiscordRichPresenceButtonConfig> {
    buttons
        .iter()
        .map(|button| DiscordRichPresenceButtonConfig {
            label: button.label.trim().to_string(),
            url: button.url.trim().to_string(),
        })
        .filter(|button| !button.label.is_empty() && !button.url.is_empty())
        .take(2)
        .collect()
}

fn normalize_party_size(value: Option<u32>) -> Option<u32> {
    value.filter(|size| *size > 0)
}

fn normalized_custom_preset(preset: &DiscordCustomPreset) -> DiscordCustomPreset {
    DiscordCustomPreset {
        name: preset.name.trim().to_string(),
        activity_type: preset.activity_type.clone(),
        status_display: preset.status_display.clone(),
        app_name_mode: preset.app_name_mode.clone(),
        custom_app_name: preset.custom_app_name.trim().to_string(),
        details_format: normalize_discord_line(&preset.details_format),
        state_format: normalize_discord_line(&preset.state_format),
        buttons: normalize_discord_buttons(&preset.buttons),
        party_id: preset.party_id.trim().to_string(),
        party_size_current: normalize_party_size(preset.party_size_current),
        party_size_max: normalize_party_size(preset.party_size_max),
        join_secret: preset.join_secret.trim().to_string(),
        spectate_secret: preset.spectate_secret.trim().to_string(),
        match_secret: preset.match_secret.trim().to_string(),
    }
}

fn apply_custom_preset_to_config(config: &mut ClientConfig, preset: &DiscordCustomPreset) {
    let preset = normalized_custom_preset(preset);
    config.discord_report_mode = DiscordReportMode::Custom;
    config.discord_activity_type = preset.activity_type;
    config.discord_custom_mode_status_display = preset.status_display;
    config.discord_custom_mode_app_name_mode = preset.app_name_mode;
    config.discord_custom_mode_custom_app_name = preset.custom_app_name;
    config.discord_details_format = preset.details_format;
    config.discord_state_format = preset.state_format;
    config.discord_custom_buttons = preset.buttons;
    config.discord_custom_party_id = preset.party_id;
    config.discord_custom_party_size_current = preset.party_size_current;
    config.discord_custom_party_size_max = preset.party_size_max;
    config.discord_custom_join_secret = preset.join_secret;
    config.discord_custom_spectate_secret = preset.spectate_secret;
    config.discord_custom_match_secret = preset.match_secret;
}

fn config_matches_custom_preset(config: &ClientConfig, preset: &DiscordCustomPreset) -> bool {
    let preset = normalized_custom_preset(preset);

    config.discord_activity_type == preset.activity_type
        && config.discord_custom_mode_status_display == preset.status_display
        && config.discord_custom_mode_app_name_mode == preset.app_name_mode
        && config.discord_custom_mode_custom_app_name.trim() == preset.custom_app_name
        && normalize_discord_line(&config.discord_details_format) == preset.details_format
        && normalize_discord_line(&config.discord_state_format) == preset.state_format
        && normalize_discord_buttons(&config.discord_custom_buttons) == preset.buttons
        && config.discord_custom_party_id.trim() == preset.party_id
        && normalize_party_size(config.discord_custom_party_size_current)
            == preset.party_size_current
        && normalize_party_size(config.discord_custom_party_size_max) == preset.party_size_max
        && config.discord_custom_join_secret.trim() == preset.join_secret
        && config.discord_custom_spectate_secret.trim() == preset.spectate_secret
        && config.discord_custom_match_secret.trim() == preset.match_secret
}

fn active_custom_preset_index(config: &ClientConfig) -> Option<usize> {
    if config.discord_report_mode != DiscordReportMode::Custom {
        return None;
    }

    config
        .discord_custom_presets
        .iter()
        .position(|preset| config_matches_custom_preset(config, preset))
}

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

fn build_quick_switch_submenu(
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

fn build_runtime_submenu(
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

fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
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
    let runtime_submenu = build_runtime_submenu(app, locale)?;
    let quick_switch = build_quick_switch_submenu(app, locale, &config)?;
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

fn apply_switch_target(
    config: &mut ClientConfig,
    target: &TraySwitchTarget,
    locale: BackendLocale,
) -> Result<AppliedSelection, String> {
    match target {
        TraySwitchTarget::Mode(mode) => {
            config.discord_report_mode = mode.clone();
            Ok(AppliedSelection::Mode(mode.clone()))
        }
        TraySwitchTarget::CustomCurrent => {
            config.discord_report_mode = DiscordReportMode::Custom;
            Ok(AppliedSelection::CustomCurrent)
        }
        TraySwitchTarget::CustomPreset(index) => {
            let preset = config
                .discord_custom_presets
                .get(*index)
                .cloned()
                .ok_or_else(|| {
                    if locale.is_en() {
                        format!("Custom preset {} was not found.", index + 1)
                    } else {
                        format!("找不到自定义预设 {}", index + 1)
                    }
                })?;
            let label = preset_label(locale, &preset, *index);
            apply_custom_preset_to_config(config, &preset);
            Ok(AppliedSelection::CustomPreset(label))
        }
    }
}

fn configs_equal(left: &ClientConfig, right: &ClientConfig) -> bool {
    match (serde_json::to_value(left), serde_json::to_value(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn restart_running_services(app: &AppHandle, config: &ClientConfig) -> Result<bool, String> {
    let reporter = app.state::<ReporterRuntime>();
    let discord = app.state::<DiscordPresenceRuntime>();
    let reporter_was_running = reporter.snapshot().running;
    let discord_was_running = discord.snapshot().running;

    if !reporter_was_running && !discord_was_running {
        return Ok(false);
    }

    if reporter_was_running {
        reporter.stop();
    }
    if discord_was_running {
        discord.stop();
    }

    let locale = load_locale(app);
    let mut discord_started = false;

    if discord_was_running {
        discord.start(config.clone(), locale)?;
        discord_started = true;
    }

    if reporter_was_running {
        if let Err(error) = reporter.start(config.clone(), locale) {
            if discord_started {
                let _ = discord.stop();
            }
            return Err(error);
        }
    }

    Ok(true)
}

fn handle_runtime_start(app: &AppHandle) {
    let locale = load_locale(app);
    let reporter = app.state::<ReporterRuntime>();
    let discord = app.state::<DiscordPresenceRuntime>();
    let reporter_running = reporter.snapshot().running;
    let discord_running = discord.snapshot().running;

    let notice = if reporter_running && discord_running {
        runtime_notice(
            "info",
            if locale.is_en() {
                "Runtime already online"
            } else {
                "运行时已启动"
            },
            if locale.is_en() {
                "Local capture and Discord RPC are already running."
            } else {
                "本地监控和 Discord RPC 已经在运行。"
            },
        )
    } else {
        let mut config = match state_store::load_app_state(app) {
            Ok(state) => state.config,
            Err(error) => {
                let notice = failure_notice(locale, error, false);
                show_tray_notice(app, &notice);
                emit_tray_notice(app, &notice);
                return;
            }
        };
        normalize_client_config(&mut config);

        let mut started_discord_this_run = false;
        if !discord_running {
            if let Err(error) = discord.start(config.clone(), locale) {
                let notice = runtime_notice(
                    "error",
                    if locale.is_en() {
                        "Runtime start failed"
                    } else {
                        "启动运行时失败"
                    },
                    error,
                );
                show_tray_notice(app, &notice);
                emit_tray_notice(app, &notice);
                return;
            }
            started_discord_this_run = true;
        }

        if !reporter_running {
            if let Err(error) = reporter.start(config.clone(), locale) {
                if started_discord_this_run {
                    let _ = discord.stop();
                }
                let notice = runtime_notice(
                    "error",
                    if locale.is_en() {
                        "Runtime start failed"
                    } else {
                        "启动运行时失败"
                    },
                    error,
                );
                show_tray_notice(app, &notice);
                emit_tray_notice(app, &notice);
                return;
            }
        }

        if let Err(error) = refresh_tray(app) {
            eprintln!("Failed to refresh the tray menu after starting runtime: {error}");
        }

        runtime_notice(
            "success",
            if locale.is_en() {
                "Runtime online"
            } else {
                "运行时已启动"
            },
            if locale.is_en() {
                "Local capture and Discord RPC are now running."
            } else {
                "本地监控和 Discord RPC 已开始运行。"
            },
        )
    };

    show_tray_notice(app, &notice);
    emit_tray_notice(app, &notice);
}

fn handle_runtime_stop(app: &AppHandle) {
    let locale = load_locale(app);
    let reporter = app.state::<ReporterRuntime>();
    let discord = app.state::<DiscordPresenceRuntime>();
    let reporter_running = reporter.snapshot().running;
    let discord_running = discord.snapshot().running;

    let notice = if !reporter_running && !discord_running {
        runtime_notice(
            "info",
            if locale.is_en() {
                "Runtime already stopped"
            } else {
                "运行时已关闭"
            },
            if locale.is_en() {
                "Local capture and Discord RPC are already stopped."
            } else {
                "本地监控和 Discord RPC 已经停止。"
            },
        )
    } else {
        if reporter_running {
            reporter.stop();
        }
        if discord_running {
            discord.stop();
        }

        if let Err(error) = refresh_tray(app) {
            eprintln!("Failed to refresh the tray menu after stopping runtime: {error}");
        }

        runtime_notice(
            "info",
            if locale.is_en() {
                "Runtime stopped"
            } else {
                "运行时已关闭"
            },
            if locale.is_en() {
                "Local capture and Discord RPC have been stopped."
            } else {
                "本地监控和 Discord RPC 已停止。"
            },
        )
    };

    show_tray_notice(app, &notice);
    emit_tray_notice(app, &notice);
}

fn handle_quick_switch(app: &AppHandle, menu_id: &str) {
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

pub fn refresh_tray(app: &AppHandle) -> Result<(), String> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    let menu = build_tray_menu(app)?;
    tray.set_menu(Some(menu))
        .map_err(|error| format_tray_error(load_locale(app), TrayText::CreateMenuFailed, error))
}

pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let locale = load_locale(app);
    let menu = build_tray_menu(app)?;
    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))
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

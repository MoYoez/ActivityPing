use tauri::{AppHandle, Manager};

use crate::{
    backend_locale::load_locale, discord_presence::DiscordPresenceRuntime,
    realtime_reporter::ReporterRuntime, rules::normalize_client_config, state_store,
};

use super::super::{
    notice::{emit_tray_notice, failure_notice, runtime_notice, show_tray_notice},
    refresh_tray,
};

pub(super) fn handle_runtime_start(app: &AppHandle) {
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

pub(super) fn handle_runtime_stop(app: &AppHandle) {
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

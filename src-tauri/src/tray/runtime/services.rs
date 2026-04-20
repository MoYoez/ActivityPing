use tauri::{AppHandle, Manager};

use crate::{
    backend_locale::load_locale, discord_presence::DiscordPresenceRuntime, models::ClientConfig,
    realtime_reporter::ReporterRuntime,
};

pub(super) fn restart_running_services(
    app: &AppHandle,
    config: &ClientConfig,
) -> Result<bool, String> {
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

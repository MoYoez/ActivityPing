use discord_rich_presence::{DiscordIpc, DiscordIpcClient};

use crate::backend_locale::BackendLocale;

pub(super) fn clear_discord_presence(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    locale: BackendLocale,
) -> Result<(), String> {
    with_discord_client(client_slot, application_id, locale, |client| {
        client
            .clear_activity()
            .map_err(|error| format_error(locale, "Failed to clear Discord presence", error))
    })
}

pub(super) fn with_discord_client<T, F>(
    client_slot: &mut Option<DiscordIpcClient>,
    application_id: &str,
    locale: BackendLocale,
    mut action: F,
) -> Result<T, String>
where
    F: FnMut(&mut DiscordIpcClient) -> Result<T, String>,
{
    for _ in 0..2 {
        if client_slot.is_none() {
            let mut client = DiscordIpcClient::new(application_id);
            client
                .connect()
                .map_err(|error| format_error(locale, "Failed to connect to Discord IPC", error))?;
            *client_slot = Some(client);
        }

        let Some(client) = client_slot.as_mut() else {
            continue;
        };

        match action(client) {
            Ok(value) => return Ok(value),
            Err(_) => {
                *client_slot = None;
            }
        }
    }

    Err(discord_ipc_unavailable(locale))
}

pub(super) fn format_error(
    locale: BackendLocale,
    prefix: &str,
    error: impl std::fmt::Display,
) -> String {
    if locale.is_en() {
        format!("{prefix}: {error}")
    } else {
        format!("{prefix}: {error}")
    }
}

pub(super) fn discord_ipc_unavailable(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord IPC is unavailable. Make sure Discord Desktop is running.".into()
    } else {
        "Discord IPC is unavailable. Make sure Discord Desktop is running.".into()
    }
}

pub(super) fn discord_config_app_id_missing(locale: BackendLocale) -> String {
    if locale.is_en() {
        "Discord application ID is required before Discord RPC can start.".into()
    } else {
        "Discord application ID is required before Discord RPC can start.".into()
    }
}

use crate::{
    artwork_server::artwork_publishing_enabled, backend_locale::BackendLocale, models::ClientConfig,
};

use super::super::client::discord_config_app_id_missing;

pub(super) fn validate_discord_presence_config(
    config: &ClientConfig,
    locale: BackendLocale,
) -> Result<(), String> {
    if config.discord_application_id.trim().is_empty() {
        return Err(discord_config_app_id_missing(locale));
    }
    if artwork_publishing_enabled(config)
        && config.discord_artwork_worker_upload_url.trim().is_empty()
    {
        return Err(
            "Discord artwork uploader service URL is required when app artwork, music artwork, or Custom Gallery images are enabled."
                .into(),
        );
    }
    Ok(())
}

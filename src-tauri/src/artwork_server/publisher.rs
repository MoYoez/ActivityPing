use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use reqwest::blocking::Client;

use crate::models::{ClientConfig, DiscordCustomAppIconSource, DiscordCustomArtworkSource};

use super::{
    normalize::normalize_asset,
    types::{ArtworkPublisher, PreparedUpload, PublishAssetKind, UploadRequest, UploadResponse},
};

const UPLOADER_SERVICE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const URL_EXPIRES_IN_SECS: u64 = 3_600;

impl ArtworkPublisher {
    pub fn publish(
        &self,
        bytes: Vec<u8>,
        etag: String,
        kind: PublishAssetKind,
    ) -> Result<String, String> {
        let prepared = normalize_asset(bytes, kind)?;
        let payload = build_upload_request(&prepared, &etag);

        let mut request = self.client.post(&self.endpoint_url).json(&payload);
        if let Some(token) = self
            .token
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .map_err(|error| format!("Failed to upload artwork to uploader service: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|error| format!("Failed to read uploader service response: {error}"))?;

        let parsed = serde_json::from_str::<UploadResponse>(&body).unwrap_or_default();
        let response_url = parsed
            .url
            .clone()
            .or_else(|| parsed.data.as_ref().and_then(|data| data.url.clone()))
            .unwrap_or_default();
        if status.is_success() && !response_url.trim().is_empty() {
            return Ok(response_url);
        }

        let error_message = parsed
            .error
            .clone()
            .or_else(|| parsed.data.as_ref().and_then(|data| data.error.clone()))
            .or_else(|| {
                if !status.is_success() {
                    Some(format!(
                        "Uploader service request failed with HTTP {}",
                        status.as_u16()
                    ))
                } else if parsed.ok == Some(false) {
                    Some("Uploader service returned ok=false.".to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                if body.trim().is_empty() {
                    "Uploader service did not return a public url.".to_string()
                } else {
                    format!("Uploader service did not return a public url: {body}")
                }
            });

        Err(format_uploader_error(kind, &error_message))
    }
}

pub fn prepare_artwork_publisher(
    config: &ClientConfig,
) -> Result<Option<ArtworkPublisher>, String> {
    if !artwork_publishing_enabled(config) {
        return Ok(None);
    }

    let endpoint_url = config.discord_artwork_worker_upload_url.trim();
    if endpoint_url.is_empty() {
        return Err(
            "Discord artwork uploader service URL is required when app artwork, music artwork, or Custom Gallery images are enabled."
                .into(),
        );
    }

    reqwest::Url::parse(endpoint_url)
        .map_err(|error| format!("Invalid artwork uploader service URL: {error}"))?;
    let client = Client::builder()
        .timeout(UPLOADER_SERVICE_TIMEOUT)
        .build()
        .map_err(|error| format!("Failed to create uploader service client: {error}"))?;

    let token = config.discord_artwork_worker_token.trim().to_string();
    Ok(Some(ArtworkPublisher {
        client,
        endpoint_url: endpoint_url.to_string(),
        token: if token.is_empty() { None } else { Some(token) },
    }))
}

pub fn artwork_publishing_enabled(config: &ClientConfig) -> bool {
    config.discord_use_app_artwork
        || config.discord_use_music_artwork
        || matches!(
            config.discord_custom_artwork_source,
            DiscordCustomArtworkSource::Music | DiscordCustomArtworkSource::App
        )
        || (config.discord_custom_artwork_source == DiscordCustomArtworkSource::Library
            && !config.discord_custom_artwork_asset_id.trim().is_empty())
        || matches!(
            config.discord_custom_app_icon_source,
            DiscordCustomAppIconSource::App | DiscordCustomAppIconSource::Source
        )
        || (config.discord_custom_app_icon_source == DiscordCustomAppIconSource::Library
            && !config.discord_custom_app_icon_asset_id.trim().is_empty())
}

fn build_upload_request(prepared: &PreparedUpload, etag: &str) -> UploadRequest {
    let encoded = BASE64_STANDARD.encode(&prepared.bytes);
    let data_url = format!("data:{};base64,{encoded}", prepared.content_type);

    UploadRequest {
        base64: encoded,
        image_base64: data_url,
        file_name: format!("{etag}.{}", prepared.file_extension),
        expires_in: URL_EXPIRES_IN_SECS,
    }
}

fn format_uploader_error(kind: PublishAssetKind, error_message: &str) -> String {
    if kind == PublishAssetKind::AppIcon
        && error_message
            .to_ascii_lowercase()
            .contains("only jpeg/jpg is allowed")
    {
        return format!(
            "{error_message} The current uploader only accepts JPEG, but app icons are uploaded as PNG to preserve transparency."
        );
    }
    error_message.to_string()
}

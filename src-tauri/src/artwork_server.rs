use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::models::ClientConfig;

const UPLOADER_SERVICE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const MAX_DIMENSION: u32 = 128;
const JPEG_QUALITY: u8 = 85;
const URL_EXPIRES_IN_SECS: u64 = 3_600;

#[derive(Clone)]
pub struct ArtworkPublisher {
    client: Client,
    endpoint_url: String,
    token: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadRequest {
    base64: String,
    image_base64: String,
    file_name: String,
    expires_in: u64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct UploadResponse {
    ok: Option<bool>,
    url: Option<String>,
    error: Option<String>,
    data: Option<UploadResponseData>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct UploadResponseData {
    url: Option<String>,
    error: Option<String>,
}

impl ArtworkPublisher {
    pub fn publish(&self, bytes: Vec<u8>, etag: String) -> Result<String, String> {
        let jpeg_bytes = normalize_artwork(bytes)?;
        let encoded = BASE64_STANDARD.encode(jpeg_bytes);
        let base64 = format!("data:image/jpeg;base64,{encoded}");
        let file_name = format!("{etag}.jpg");

        let payload = UploadRequest {
            base64: base64.clone(),
            image_base64: base64,
            file_name,
            expires_in: URL_EXPIRES_IN_SECS,
        };

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

        Err(error_message)
    }
}

fn normalize_artwork(bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    let image = image::load_from_memory(&bytes)
        .map_err(|error| format!("Failed to decode media artwork: {error}"))?;
    let prepared = if image.width() > MAX_DIMENSION || image.height() > MAX_DIMENSION {
        image.resize(MAX_DIMENSION, MAX_DIMENSION, FilterType::Lanczos3)
    } else {
        image
    };

    let mut output = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut output, JPEG_QUALITY);
    encoder
        .encode_image(&prepared)
        .map_err(|error| format!("Failed to encode media artwork as JPEG: {error}"))?;
    Ok(output)
}

pub fn prepare_artwork_publisher(
    config: &ClientConfig,
) -> Result<Option<ArtworkPublisher>, String> {
    if !config.discord_use_app_artwork && !config.discord_use_music_artwork {
        return Ok(None);
    }

    let endpoint_url = config.discord_artwork_worker_upload_url.trim();
    if endpoint_url.is_empty() {
        return Err("Discord artwork uploader service URL is required.".into());
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

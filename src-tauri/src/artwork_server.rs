use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use image::{
    codecs::{
        jpeg::JpegEncoder,
        png::{CompressionType as PngCompressionType, FilterType as PngFilterType, PngEncoder},
    },
    imageops::FilterType,
    ColorType, DynamicImage, ImageEncoder,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::models::ClientConfig;

const UPLOADER_SERVICE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const URL_EXPIRES_IN_SECS: u64 = 3_600;
const MAX_UPLOAD_BYTES: usize = 30 * 1024;
const APP_ICON_MAX_DIMENSION: u32 = 256;
const MUSIC_ARTWORK_TARGET_DIMENSION: u32 = 256;
const APP_ICON_DIMENSION_STEPS: [u32; 8] = [256, 224, 192, 176, 160, 144, 128, 96];
const MUSIC_ARTWORK_DIMENSION_STEPS: [u32; 8] = [256, 224, 192, 176, 160, 144, 128, 96];
const MUSIC_ARTWORK_QUALITY_STEPS: [u8; 11] = [92, 90, 88, 86, 84, 82, 80, 78, 76, 74, 72];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishAssetKind {
    AppIcon,
    MusicArtwork,
}

impl PublishAssetKind {
    pub fn content_type(self) -> &'static str {
        match self {
            Self::AppIcon => "image/png",
            Self::MusicArtwork => "image/jpeg",
        }
    }

    fn file_extension(self) -> &'static str {
        match self {
            Self::AppIcon => "png",
            Self::MusicArtwork => "jpg",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::AppIcon => "app icon",
            Self::MusicArtwork => "music artwork",
        }
    }
}

struct PreparedUpload {
    bytes: Vec<u8>,
    content_type: &'static str,
    file_extension: &'static str,
}

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

fn build_upload_request(prepared: &PreparedUpload, etag: &str) -> UploadRequest {
    let encoded = BASE64_STANDARD.encode(&prepared.bytes);
    let data_url = format!("data:{};base64,{encoded}", prepared.content_type);

    UploadRequest {
        // Some uploaders expect a raw base64 string here and reject a data URL
        // with "Invalid base64". Keep the MIME-prefixed form in imageBase64.
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

fn normalize_asset(bytes: Vec<u8>, kind: PublishAssetKind) -> Result<PreparedUpload, String> {
    let image = image::load_from_memory(&bytes)
        .map_err(|error| format!("Failed to decode {}: {error}", kind.label()))?;
    match kind {
        PublishAssetKind::AppIcon => normalize_app_icon(image),
        PublishAssetKind::MusicArtwork => normalize_music_artwork(image),
    }
}

fn normalize_app_icon(image: DynamicImage) -> Result<PreparedUpload, String> {
    let source_dimension = image
        .width()
        .max(image.height())
        .min(APP_ICON_MAX_DIMENSION);
    let candidates = build_dimension_candidates(source_dimension, &APP_ICON_DIMENSION_STEPS);

    for dimension in candidates {
        let prepared = resize_for_upload(&image, dimension);
        let bytes = encode_png(&prepared)?;
        if bytes.len() <= MAX_UPLOAD_BYTES {
            return Ok(PreparedUpload {
                bytes,
                content_type: PublishAssetKind::AppIcon.content_type(),
                file_extension: PublishAssetKind::AppIcon.file_extension(),
            });
        }
    }

    let fallback = resize_for_upload(
        &image,
        APP_ICON_DIMENSION_STEPS.last().copied().unwrap_or(96),
    );
    let bytes = encode_png(&fallback)?;
    Ok(PreparedUpload {
        bytes,
        content_type: PublishAssetKind::AppIcon.content_type(),
        file_extension: PublishAssetKind::AppIcon.file_extension(),
    })
}

fn normalize_music_artwork(image: DynamicImage) -> Result<PreparedUpload, String> {
    let source_dimension = image
        .width()
        .max(image.height())
        .min(MUSIC_ARTWORK_TARGET_DIMENSION);
    let candidates = build_dimension_candidates(source_dimension, &MUSIC_ARTWORK_DIMENSION_STEPS);

    let mut best_fallback = None;
    for dimension in candidates {
        let prepared = resize_for_upload(&image, dimension);
        for quality in MUSIC_ARTWORK_QUALITY_STEPS {
            let bytes = encode_jpeg(&prepared, quality)?;
            if bytes.len() <= MAX_UPLOAD_BYTES {
                return Ok(PreparedUpload {
                    bytes,
                    content_type: PublishAssetKind::MusicArtwork.content_type(),
                    file_extension: PublishAssetKind::MusicArtwork.file_extension(),
                });
            }
            best_fallback = Some(bytes);
        }
    }

    Ok(PreparedUpload {
        bytes: best_fallback.unwrap_or_default(),
        content_type: PublishAssetKind::MusicArtwork.content_type(),
        file_extension: PublishAssetKind::MusicArtwork.file_extension(),
    })
}

fn build_dimension_candidates(source_dimension: u32, steps: &[u32]) -> Vec<u32> {
    let mut candidates = Vec::new();
    if source_dimension > 0 {
        candidates.push(source_dimension);
    }
    for step in steps {
        if *step > 0 && *step <= source_dimension && !candidates.contains(step) {
            candidates.push(*step);
        }
    }
    if candidates.is_empty() {
        candidates.push(source_dimension.max(1));
    }
    candidates
}

fn resize_for_upload(image: &DynamicImage, max_dimension: u32) -> DynamicImage {
    if max_dimension == 0 {
        return image.clone();
    }

    if image.width() <= max_dimension && image.height() <= max_dimension {
        image.clone()
    } else {
        image.resize(max_dimension, max_dimension, FilterType::Lanczos3)
    }
}

fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = Vec::new();
    PngEncoder::new_with_quality(
        &mut output,
        PngCompressionType::Best,
        PngFilterType::Adaptive,
    )
    .write_image(&rgba, width, height, ColorType::Rgba8.into())
    .map_err(|error| format!("Failed to encode app icon as PNG: {error}"))?;
    Ok(output)
}

fn encode_jpeg(image: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut output, quality);
    encoder
        .encode_image(image)
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn encode_test_png(image: &DynamicImage) -> Vec<u8> {
        encode_png(image).expect("encode png")
    }

    #[test]
    fn app_icon_normalization_keeps_png_and_alpha() {
        let mut image = RgbaImage::new(512, 512);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            let alpha = if (x + y) % 2 == 0 { 0 } else { 255 };
            *pixel = Rgba([255, 32, 64, alpha]);
        }

        let prepared = normalize_asset(
            encode_test_png(&DynamicImage::ImageRgba8(image)),
            PublishAssetKind::AppIcon,
        )
        .expect("normalize app icon");

        assert_eq!(prepared.content_type, "image/png");
        assert_eq!(prepared.file_extension, "png");
        assert!(prepared.bytes.len() <= MAX_UPLOAD_BYTES);

        let decoded = image::load_from_memory(&prepared.bytes).expect("decode normalized icon");
        assert!(decoded.width() <= APP_ICON_MAX_DIMENSION);
        assert!(decoded.height() <= APP_ICON_MAX_DIMENSION);
        assert!(decoded.to_rgba8().pixels().any(|pixel| pixel[3] < 255));
    }

    #[test]
    fn music_artwork_normalization_uses_jpeg_and_budget() {
        let mut image = RgbaImage::new(1600, 1600);
        for (x, y, pixel) in image.enumerate_pixels_mut() {
            *pixel = Rgba([(x % 255) as u8, (y % 255) as u8, ((x + y) % 255) as u8, 255]);
        }

        let prepared = normalize_asset(
            encode_test_png(&DynamicImage::ImageRgba8(image)),
            PublishAssetKind::MusicArtwork,
        )
        .expect("normalize music artwork");

        assert_eq!(prepared.content_type, "image/jpeg");
        assert_eq!(prepared.file_extension, "jpg");
        assert!(!prepared.bytes.is_empty());
        assert!(prepared.bytes.len() <= MAX_UPLOAD_BYTES);

        let decoded = image::load_from_memory(&prepared.bytes).expect("decode normalized artwork");
        assert!(decoded.width() <= MUSIC_ARTWORK_TARGET_DIMENSION);
        assert!(decoded.height() <= MUSIC_ARTWORK_TARGET_DIMENSION);
    }

    #[test]
    fn upload_request_uses_raw_base64_and_data_url_forms() {
        let prepared = PreparedUpload {
            bytes: vec![1, 2, 3, 4],
            content_type: "image/png",
            file_extension: "png",
        };

        let payload = build_upload_request(&prepared, "etag");

        assert_eq!(payload.base64, "AQIDBA==");
        assert_eq!(payload.image_base64, "data:image/png;base64,AQIDBA==");
        assert_eq!(payload.file_name, "etag.png");
        assert_eq!(payload.expires_in, URL_EXPIRES_IN_SECS);
    }

    #[test]
    fn app_icon_jpeg_only_error_gets_a_clear_hint() {
        let error = format_uploader_error(
            PublishAssetKind::AppIcon,
            "Only JPEG/JPG is allowed",
        );

        assert!(error.contains("Only JPEG/JPG is allowed"));
        assert!(error.contains("app icons are uploaded as PNG"));
    }
}

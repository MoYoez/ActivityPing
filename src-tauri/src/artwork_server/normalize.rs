use image::{
    codecs::{
        jpeg::JpegEncoder,
        png::{CompressionType as PngCompressionType, FilterType as PngFilterType, PngEncoder},
    },
    imageops::FilterType,
    ColorType, DynamicImage, ImageEncoder,
};

use super::types::{PreparedUpload, PublishAssetKind};

const MAX_UPLOAD_BYTES: usize = 30 * 1024;
const APP_ICON_MAX_DIMENSION: u32 = 256;
const MUSIC_ARTWORK_TARGET_DIMENSION: u32 = 256;
const APP_ICON_DIMENSION_STEPS: [u32; 8] = [256, 224, 192, 176, 160, 144, 128, 96];
const MUSIC_ARTWORK_DIMENSION_STEPS: [u32; 8] = [256, 224, 192, 176, 160, 144, 128, 96];
const MUSIC_ARTWORK_QUALITY_STEPS: [u8; 11] = [92, 90, 88, 86, 84, 82, 80, 78, 76, 74, 72];

pub(super) fn normalize_asset(
    bytes: Vec<u8>,
    kind: PublishAssetKind,
) -> Result<PreparedUpload, String> {
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

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
    let error = format_uploader_error(PublishAssetKind::AppIcon, "Only JPEG/JPG is allowed");

    assert!(error.contains("Only JPEG/JPG is allowed"));
    assert!(error.contains("app icons are uploaded as PNG"));
}

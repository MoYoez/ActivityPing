use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

pub(super) fn decode_base64_image_payload(value: &str) -> Option<Vec<u8>> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
        return None;
    }

    let encoded = trimmed
        .split_once(',')
        .map(|(_, payload)| payload)
        .unwrap_or(trimmed);

    BASE64_STANDARD
        .decode(encoded.trim())
        .ok()
        .filter(|bytes| !bytes.is_empty())
}

pub(super) fn detect_image_content_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

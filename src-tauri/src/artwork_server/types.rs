use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

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

    pub(super) fn file_extension(self) -> &'static str {
        match self {
            Self::AppIcon => "png",
            Self::MusicArtwork => "jpg",
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::AppIcon => "app icon",
            Self::MusicArtwork => "music artwork",
        }
    }
}

pub(super) struct PreparedUpload {
    pub(super) bytes: Vec<u8>,
    pub(super) content_type: &'static str,
    pub(super) file_extension: &'static str,
}

#[derive(Clone)]
pub struct ArtworkPublisher {
    pub(super) client: Client,
    pub(super) endpoint_url: String,
    pub(super) token: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UploadRequest {
    pub(super) base64: String,
    pub(super) image_base64: String,
    pub(super) file_name: String,
    pub(super) expires_in: u64,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct UploadResponse {
    pub(super) ok: Option<bool>,
    pub(super) url: Option<String>,
    pub(super) error: Option<String>,
    pub(super) data: Option<UploadResponseData>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct UploadResponseData {
    pub(super) url: Option<String>,
    pub(super) error: Option<String>,
}

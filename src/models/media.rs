//! The media library.

use serde::Deserialize;

use super::common::MediaType;

/// A file in the library.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaLibraryItem {
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// Points at the delivery endpoint, which re-checks access per request.
    pub url: String,
    #[serde(rename = "type", default)]
    pub media_type: Option<MediaType>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub size: Option<i64>,
    #[serde(default)]
    pub alt_text: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A file that just landed in the library.
#[derive(Debug, Clone, Deserialize)]
pub struct UploadedMedia {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub media_type: MediaType,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub size: Option<i64>,
}

/// A file to upload: its bytes, its name, and its content type.
#[derive(Debug, Clone)]
pub struct MediaUpload {
    pub filename: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

impl MediaUpload {
    pub fn new(
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            filename: filename.into(),
            mime_type: mime_type.into(),
            bytes: bytes.into(),
        }
    }
}

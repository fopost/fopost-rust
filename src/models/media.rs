//! The media library.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

/// What to ask for before uploading a file straight to storage.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignUpload {
    pub workspace_id: String,
    pub filename: String,
    pub mime_type: String,
    /// Byte length of the file; the upload must send exactly this many.
    pub size: u64,
}

impl PresignUpload {
    pub fn new(
        workspace_id: impl Into<String>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        size: u64,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            filename: filename.into(),
            mime_type: mime_type.into(),
            size,
        }
    }
}

/// A one-off upload slot: PUT the bytes to `upload_url` with `headers`, then complete it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignedUpload {
    pub upload_id: String,
    pub upload_url: String,
    #[serde(default)]
    pub method: Option<String>,
    /// Sent verbatim on the upload; `Content-Type` is always among them.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

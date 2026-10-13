//! The media library.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::common::{string_enum, MediaType};

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

/// Where a file's bytes come from: memory, or a path streamed from disk.
#[derive(Debug, Clone)]
pub enum FileSource {
    Bytes(Vec<u8>),
    Path(PathBuf),
}

impl FileSource {
    /// Byte length, reading the metadata when the source is a path.
    pub async fn len(&self) -> std::io::Result<u64> {
        match self {
            FileSource::Bytes(bytes) => Ok(bytes.len() as u64),
            FileSource::Path(path) => Ok(tokio::fs::metadata(path).await?.len()),
        }
    }
}

/// A file to attach to a post or upload directly: name, content type, and bytes or a path.
#[derive(Debug, Clone)]
pub struct MediaFile {
    pub filename: String,
    pub mime_type: String,
    pub source: FileSource,
}

impl MediaFile {
    pub fn new(
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            filename: filename.into(),
            mime_type: mime_type.into(),
            source: FileSource::Bytes(bytes.into()),
        }
    }

    /// A file streamed from disk; the name is the path's file name.
    pub fn from_path(path: impl Into<PathBuf>, mime_type: impl Into<String>) -> Self {
        let path = path.into();
        let filename = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            filename,
            mime_type: mime_type.into(),
            source: FileSource::Path(path),
        }
    }

    /// The kind the API files this under, read from the content type.
    pub fn media_type(&self) -> MediaType {
        match self.mime_type.as_str() {
            "image/gif" => MediaType::Gif,
            mime if mime.starts_with("image/") => MediaType::Image,
            mime if mime.starts_with("video/") => MediaType::Video,
            _ => MediaType::Document,
        }
    }
}

string_enum! {
    /// Why an upload is being made.
    pub enum UploadOrigin {
        Chat => "chat",
    }
}

string_enum! {
    /// The part a file plays in a brand kit.
    pub enum BrandRole {
        Logo => "logo",
        Product => "product",
        Person => "person",
        Background => "background",
    }
}

/// Attestation that a person pictured in a `person` asset agreed to its use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonConsent {
    pub subject: String,
    pub statement: String,
}

/// What to ask for before uploading a file straight to storage.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignUpload {
    pub workspace_id: String,
    pub filename: String,
    pub mime_type: String,
    /// Byte length of the file, up to 5 GB; the upload must send exactly this many.
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<UploadOrigin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_role: Option<BrandRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_voice_id: Option<String>,
    /// Required when `brand_role` is `person`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_consent: Option<PersonConsent>,
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
            folder_id: None,
            origin: None,
            brand_role: None,
            brand_voice_id: None,
            person_consent: None,
        }
    }

    pub fn folder(mut self, folder_id: impl Into<String>) -> Self {
        self.folder_id = Some(folder_id.into());
        self
    }

    pub fn origin(mut self, origin: UploadOrigin) -> Self {
        self.origin = Some(origin);
        self
    }

    pub fn brand_role(mut self, role: BrandRole) -> Self {
        self.brand_role = Some(role);
        self
    }

    pub fn brand_voice(mut self, brand_voice_id: impl Into<String>) -> Self {
        self.brand_voice_id = Some(brand_voice_id.into());
        self
    }

    pub fn person_consent(
        mut self,
        subject: impl Into<String>,
        statement: impl Into<String>,
    ) -> Self {
        self.person_consent = Some(PersonConsent {
            subject: subject.into(),
            statement: statement.into(),
        });
        self
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
    /// The largest file this account may upload, in bytes.
    #[serde(default)]
    pub max_file_bytes: Option<u64>,
}

string_enum! {
    /// What sets the account's upload ceiling.
    pub enum UploadLimitSource {
        Accounts => "accounts",
        AddOn => "add_on",
        Bypass => "bypass",
    }
}

/// The next ceiling connecting more accounts would unlock.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadLimitTier {
    /// `None` on the top tier.
    #[serde(default)]
    pub up_to_accounts: Option<u32>,
    pub max_file_bytes: u64,
}

/// What this account may upload, from `GET /media/limits`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadLimits {
    /// The largest file allowed by any route.
    pub max_file_bytes: u64,
    #[serde(default)]
    pub source: Option<UploadLimitSource>,
    #[serde(default)]
    pub connected_accounts: u32,
    #[serde(default)]
    pub large_uploads_enabled: bool,
    #[serde(default)]
    pub next_tier: Option<UploadLimitTier>,
    /// The most a multipart request carries per file; bigger files go direct.
    pub multipart_max_file_bytes: u64,
    #[serde(default)]
    pub max_files_per_upload: u32,
    #[serde(default)]
    pub direct_upload_available: bool,
    #[serde(default)]
    pub allowed_types: Vec<String>,
}

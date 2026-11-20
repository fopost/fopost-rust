//! Designs — editable stories, posts, carousels and thumbnails in the media library.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::common::{string_enum, Platform, PostStatus};

string_enum! {
    /// The format a design is laid out for.
    pub enum DesignKind {
        Story => "story",
        Post => "post",
        Carousel => "carousel",
        Thumbnail => "thumbnail",
        Card => "card",
        Collage => "collage",
    }
}

string_enum! {
    /// What `export` renders a design to.
    pub enum DesignExportFormat {
        Png => "png",
        Jpg => "jpg",
        Pdf => "pdf",
        Mp4 => "mp4",
        Gif => "gif",
    }
}

string_enum! {
    /// The layout `from_post` turns a post's text into.
    pub enum DesignPostStyle {
        Quote => "quote",
        Post => "post",
        Plain => "plain",
    }
}

/// A design's editor document (pages, layers, background), passed through as JSON.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DesignDocument(pub serde_json::Value);

impl DesignDocument {
    pub fn into_inner(self) -> serde_json::Value {
        self.0
    }
}

impl From<serde_json::Value> for DesignDocument {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for DesignDocument {
    type Target = serde_json::Value;

    fn deref(&self) -> &serde_json::Value {
        &self.0
    }
}

impl std::ops::DerefMut for DesignDocument {
    fn deref_mut(&mut self) -> &mut serde_json::Value {
        &mut self.0
    }
}

/// A saved design.
#[derive(Debug, Clone, Deserialize)]
pub struct Design {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub kind: DesignKind,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub document: DesignDocument,
    #[serde(default)]
    pub thumbnail_media_id: Option<String>,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// A saved snapshot of a design. `document` is only present when asked for.
#[derive(Debug, Clone, Deserialize)]
pub struct DesignVersion {
    pub id: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub document: Option<DesignDocument>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// What `update` and `restore_version` answer: the design plus the version saved, if any.
#[derive(Debug, Clone, Deserialize)]
pub struct DesignSaved {
    #[serde(rename = "data")]
    pub design: Design,
    #[serde(default)]
    pub version: Option<DesignVersion>,
}

impl std::ops::Deref for DesignSaved {
    type Target = Design;

    fn deref(&self) -> &Design {
        &self.design
    }
}

/// What `delete` answers.
#[derive(Debug, Clone, Deserialize)]
pub struct DesignDeleted {
    pub id: String,
    #[serde(default)]
    pub deleted: bool,
}

/// One rendered file of an export, stored in the media library.
#[derive(Debug, Clone, Deserialize)]
pub struct DesignExportFile {
    pub media_id: String,
    #[serde(default)]
    pub page_index: Option<u32>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub preview_url: Option<String>,
}

/// What `export` answers.
#[derive(Debug, Clone, Deserialize)]
pub struct DesignExport {
    pub format: DesignExportFormat,
    #[serde(default)]
    pub files: Vec<DesignExportFile>,
}

/// A post that uses an exported file of a design.
#[derive(Debug, Clone, Deserialize)]
pub struct DesignPost {
    pub id: String,
    pub status: PostStatus,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub schedule_at: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub engagement: Option<u64>,
    #[serde(default)]
    pub views: Option<u64>,
}

/// A slide outline from `text_to_slides`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SlidePoint {
    #[serde(default)]
    pub heading: String,
    #[serde(default)]
    pub body: String,
}

/// The cover slide of a `text_to_slides` outline.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SlideCover {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
}

/// Heading and body fonts from the brand kit.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SlideFonts {
    #[serde(default)]
    pub heading: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

/// The brand kit a `text_to_slides` outline was written for.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SlideBrand {
    #[serde(default)]
    pub colors: Vec<String>,
    #[serde(default)]
    pub fonts: SlideFonts,
    #[serde(default)]
    pub logo_media_id: Option<String>,
}

/// What `text_to_slides` answers: an outline, not a saved design.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SlideOutline {
    #[serde(default)]
    pub cover: SlideCover,
    #[serde(default)]
    pub slides: Vec<SlidePoint>,
    #[serde(default)]
    pub cta: SlidePoint,
    #[serde(default)]
    pub brand: SlideBrand,
}

/// Query parameters for `GET /designs`.
#[derive(Debug, Clone, Default)]
pub struct ListDesigns<'a> {
    pub workspace_id: &'a str,
    pub kind: Option<DesignKind>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// The body of `POST /designs`. Build it with [`CreateDesign::new`].
#[derive(Debug, Clone, Serialize)]
pub struct CreateDesign {
    pub workspace_id: String,
    pub kind: DesignKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<DesignDocument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_voice_id: Option<String>,
}

impl CreateDesign {
    pub fn new(workspace_id: impl Into<String>, kind: DesignKind) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            kind,
            name: None,
            document: None,
            brand_voice_id: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn document(mut self, document: impl Into<DesignDocument>) -> Self {
        self.document = Some(document.into());
        self
    }

    pub fn brand_voice(mut self, brand_voice_id: impl Into<String>) -> Self {
        self.brand_voice_id = Some(brand_voice_id.into());
        self
    }
}

/// The body of `PATCH /designs/{id}`. Only the fields set are sent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateDesign {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<DesignKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<DesignDocument>,
    /// Save a version snapshot alongside the change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_version: Option<bool>,
}

impl UpdateDesign {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn kind(mut self, kind: DesignKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn document(mut self, document: impl Into<DesignDocument>) -> Self {
        self.document = Some(document.into());
        self
    }

    pub fn create_version(mut self, create_version: bool) -> Self {
        self.create_version = Some(create_version);
        self
    }
}

/// The body of `POST /designs/{id}/export`. Build it with [`ExportDesign::new`].
#[derive(Debug, Clone, Serialize)]
pub struct ExportDesign {
    pub format: DesignExportFormat,
    /// Zero-based pages to render; every page when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_indexes: Option<Vec<u32>>,
    /// Length of an `mp4` or `gif` export.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<u32>,
}

impl ExportDesign {
    pub fn new(format: DesignExportFormat) -> Self {
        Self {
            format,
            page_indexes: None,
            duration_sec: None,
        }
    }

    pub fn pages(mut self, page_indexes: impl IntoIterator<Item = u32>) -> Self {
        self.page_indexes = Some(page_indexes.into_iter().collect());
        self
    }

    pub fn duration_sec(mut self, seconds: u32) -> Self {
        self.duration_sec = Some(seconds);
        self
    }
}

/// The body of `POST /designs/{id}/bulk`: one new design per row, with each
/// column filling the text layer of the same name.
#[derive(Debug, Clone, Serialize)]
pub struct BulkCreateDesigns {
    pub rows: Vec<BTreeMap<String, String>>,
    /// The column whose value names each new design.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_column: Option<String>,
}

impl BulkCreateDesigns {
    pub fn new(rows: impl IntoIterator<Item = BTreeMap<String, String>>) -> Self {
        Self {
            rows: rows.into_iter().collect(),
            name_column: None,
        }
    }

    pub fn name_column(mut self, column: impl Into<String>) -> Self {
        self.name_column = Some(column.into());
        self
    }
}

/// The body of `POST /designs/text-to-slides`. Pass `text` or `post_id`.
#[derive(Debug, Clone, Serialize)]
pub struct TextToSlides {
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_slides: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_voice_id: Option<String>,
}

impl TextToSlides {
    pub fn new(workspace_id: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            text: None,
            post_id: None,
            max_slides: None,
            brand_voice_id: None,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn post(mut self, post_id: impl Into<String>) -> Self {
        self.post_id = Some(post_id.into());
        self
    }

    pub fn max_slides(mut self, max_slides: u32) -> Self {
        self.max_slides = Some(max_slides);
        self
    }

    pub fn brand_voice(mut self, brand_voice_id: impl Into<String>) -> Self {
        self.brand_voice_id = Some(brand_voice_id.into());
        self
    }
}

/// The body of `POST /designs/from-post`. Pass `text` or `post_id`.
#[derive(Debug, Clone, Serialize)]
pub struct DesignFromPost {
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<DesignPostStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    /// The account whose name and avatar the design shows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carousel: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_voice_id: Option<String>,
}

impl DesignFromPost {
    pub fn new(workspace_id: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            text: None,
            post_id: None,
            style: None,
            preset: None,
            account_id: None,
            carousel: None,
            name: None,
            brand_voice_id: None,
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn post(mut self, post_id: impl Into<String>) -> Self {
        self.post_id = Some(post_id.into());
        self
    }

    pub fn style(mut self, style: DesignPostStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn preset(mut self, preset: impl Into<String>) -> Self {
        self.preset = Some(preset.into());
        self
    }

    pub fn account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn carousel(mut self, carousel: bool) -> Self {
        self.carousel = Some(carousel);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn brand_voice(mut self, brand_voice_id: impl Into<String>) -> Self {
        self.brand_voice_id = Some(brand_voice_id.into());
        self
    }
}

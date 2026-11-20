//! Fonts — the shipped library plus a workspace's own uploads.

use serde::Deserialize;

use super::common::string_enum;

string_enum! {
    /// Upright or italic.
    pub enum FontStyle {
        Normal => "normal",
        Italic => "italic",
    }
}

/// One file of a library font.
#[derive(Debug, Clone, Deserialize)]
pub struct LibraryFontFile {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub subset: Option<String>,
    #[serde(default)]
    pub weight: Option<u32>,
    #[serde(default)]
    pub style: Option<FontStyle>,
    #[serde(default)]
    pub unicode_range: Option<String>,
}

/// A font every workspace can use.
#[derive(Debug, Clone, Deserialize)]
pub struct LibraryFont {
    pub slug: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub weights: Vec<u32>,
    #[serde(default)]
    pub files: Vec<LibraryFontFile>,
}

/// A font file a workspace uploaded.
#[derive(Debug, Clone, Deserialize)]
pub struct UploadedFont {
    pub id: String,
    #[serde(default)]
    pub family: String,
    pub style: FontStyle,
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub file_url: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// What `fonts().list` answers.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct FontList {
    #[serde(default)]
    pub library: Vec<LibraryFont>,
    #[serde(default)]
    pub uploaded: Vec<UploadedFont>,
}

/// Optional fields of a font upload. The family defaults to the file name,
/// the weight to 400 and the style to normal.
#[derive(Debug, Clone, Default)]
pub struct FontUpload {
    pub family: Option<String>,
    /// 100 to 1000.
    pub weight: Option<u32>,
    pub style: Option<FontStyle>,
}

impl FontUpload {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    pub fn weight(mut self, weight: u32) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn style(mut self, style: FontStyle) -> Self {
        self.style = Some(style);
        self
    }
}

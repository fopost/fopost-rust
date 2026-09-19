//! Validate — check content, length, or a media URL against platform rules
//! without creating a post. Nothing is stored.

use serde::{Deserialize, Serialize};

use super::posts::ContentSignal;

/// A media item to check as part of `POST /validate/post`.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateMediaItem {
    /// A public http(s) URL.
    pub url: String,
    pub mime_type: String,
    /// File size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

impl ValidateMediaItem {
    pub fn new(url: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            mime_type: mime_type.into(),
            size: None,
        }
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }
}

/// The body of `POST /validate/post`.
#[derive(Debug, Clone, Serialize)]
pub struct ValidatePost {
    /// Platform slugs, at least one.
    pub platforms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub media: Vec<ValidateMediaItem>,
}

impl ValidatePost {
    pub fn new<I, S>(platforms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            platforms: platforms.into_iter().map(Into::into).collect(),
            content: None,
            media: Vec::new(),
        }
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn media(mut self, media: impl IntoIterator<Item = ValidateMediaItem>) -> Self {
        self.media = media.into_iter().collect();
        self
    }
}

/// One platform's readiness, from `POST /validate/post`.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatePostPlatform {
    pub platform: String,
    #[serde(default)]
    pub ready: bool,
    /// Hard blockers that would prevent publishing.
    #[serde(default)]
    pub issues: Vec<String>,
    /// Advisory 0-100 content quality score.
    #[serde(default)]
    pub score: Option<f64>,
    /// Advisory signals — they never block publishing.
    #[serde(default)]
    pub signals: Vec<ContentSignal>,
}

/// What `POST /validate/post` answers with.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatePostResult {
    /// True only when every platform is ready.
    #[serde(default)]
    pub ready: bool,
    #[serde(default)]
    pub platforms: Vec<ValidatePostPlatform>,
}

/// The body of `POST /validate/length`.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateLength {
    pub text: String,
    /// Platform slugs, at least one.
    pub platforms: Vec<String>,
}

impl ValidateLength {
    pub fn new<I, S>(text: impl Into<String>, platforms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            text: text.into(),
            platforms: platforms.into_iter().map(Into::into).collect(),
        }
    }
}

/// One platform's length check, from `POST /validate/length`.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidateLengthPlatform {
    pub platform: String,
    /// What the platform counts, in `unit`.
    #[serde(default)]
    pub length: u64,
    /// `None` when the platform has no text limit.
    #[serde(default)]
    pub limit: Option<u64>,
    /// `chars` or `bytes`.
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub signals: Vec<ContentSignal>,
}

/// What `POST /validate/length` answers with.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidateLengthResult {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub platforms: Vec<ValidateLengthPlatform>,
}

/// The body of `POST /validate/media`.
#[derive(Debug, Clone, Serialize)]
pub struct ValidateMedia {
    /// A public http(s) URL of the file.
    pub url: String,
}

impl ValidateMedia {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

/// What `POST /validate/media` answers with — 200 even when a check fails.
#[derive(Debug, Clone, Deserialize)]
pub struct ValidateMediaResult {
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub issues: Vec<String>,
    #[serde(default)]
    pub name: String,
    /// Bytes fetched.
    #[serde(default)]
    pub size: u64,
    /// Present only when `ok`.
    #[serde(default)]
    pub mime_type: Option<String>,
    /// `image`, `video`, `audio` or `document`; present only when `ok`.
    #[serde(rename = "type", default)]
    pub media_type: Option<String>,
}

/// What `GET /validate/subreddit` answers with.
#[derive(Debug, Clone, Deserialize)]
pub struct SubredditCheck {
    #[serde(default)]
    pub subreddit: String,
    #[serde(default)]
    pub exists: bool,
    #[serde(default)]
    pub can_post: bool,
    #[serde(default)]
    pub over_18: bool,
    #[serde(default)]
    pub flair_enabled: bool,
    /// True when the subreddit exists and takes a post from this account.
    #[serde(default)]
    pub ok: bool,
}

//! The knowledge base — what a workspace has told FoPost about itself.

use serde::{Deserialize, Serialize};

/// One thing the workspace has told FoPost about itself: an FAQ, a note, a
/// page on its own site, or a plain-text/CSV file from the media library.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSource {
    pub id: String,
    /// `faq`, `text`, `url` or `file`.
    pub kind: String,
    pub title: String,
    /// `pending`, `syncing`, `ready` or `failed`. Only a `ready` source is searched.
    pub status: String,
    /// Why the last sync failed, in plain words.
    #[serde(default)]
    pub status_message: Option<String>,
    /// Set for `url` sources.
    #[serde(default)]
    pub url: Option<String>,
    /// Set for `file` sources: the media library item read.
    #[serde(default)]
    pub media_id: Option<String>,
    /// `None` means the source serves the whole workspace.
    #[serde(default)]
    pub brand_voice_id: Option<String>,
    /// Searchable passages the last sync produced.
    #[serde(default)]
    pub chunk_count: i64,
    /// The typed text, for `faq` and `text` sources only.
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub last_synced_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// One retrieved passage, with the source it came from so a reply can cite it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeMatch {
    pub source_id: String,
    pub source_title: String,
    pub source_kind: String,
    #[serde(default)]
    pub source_url: Option<String>,
    pub text: String,
    /// Similarity to the question, 0-1.
    #[serde(default)]
    pub score: f64,
}

/// What a sync answers: the source, and that it is queued.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSyncResult {
    pub id: String,
    pub status: String,
}

/// What a delete answers.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeDeleted {
    pub id: String,
    #[serde(default)]
    pub deleted: bool,
}

/// The body of `POST /knowledge/sources`.
///
/// `kind` is `faq`, `text`, `url` or `file`. An `faq` or `text` source needs
/// `content`, a `url` source needs `url`, and a `file` source needs `media_id`
/// pointing at a plain-text or CSV item in the same workspace.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CreateKnowledgeSource {
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_voice_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

impl CreateKnowledgeSource {
    pub fn new(kind: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            title: title.into(),
            ..Default::default()
        }
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn media_id(mut self, media_id: impl Into<String>) -> Self {
        self.media_id = Some(media_id.into());
        self
    }

    pub fn brand_voice_id(mut self, brand_voice_id: impl Into<String>) -> Self {
        self.brand_voice_id = Some(brand_voice_id.into());
        self
    }

    pub fn workspace_id(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

/// The body of `PATCH /knowledge/sources/{id}`. Only the fields you set are
/// sent, so it stays a partial update.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateKnowledgeSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_voice_id: Option<String>,
}

impl UpdateKnowledgeSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn brand_voice_id(mut self, brand_voice_id: impl Into<String>) -> Self {
        self.brand_voice_id = Some(brand_voice_id.into());
        self
    }
}

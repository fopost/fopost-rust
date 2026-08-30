//! Labels — the campaign tags posts are grouped and reported by.

use serde::{Deserialize, Serialize};

/// The workspace a label belongs to.
#[derive(Debug, Clone, Deserialize)]
pub struct LabelWorkspaceRef {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(rename = "type", default)]
    pub workspace_type: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

/// A label.
#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub workspace: Option<LabelWorkspaceRef>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// The body of `POST /labels`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateLabel {
    pub workspace_id: String,
    pub name: String,
    /// A hex color, `#4F46E5` style.
    pub color: String,
}

impl CreateLabel {
    pub fn new(
        workspace_id: impl Into<String>,
        name: impl Into<String>,
        color: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            name: name.into(),
            color: color.into(),
        }
    }
}

/// The body of `PUT /labels/{id}`. The API wants both fields.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateLabel {
    pub name: String,
    pub color: String,
}

impl UpdateLabel {
    pub fn new(name: impl Into<String>, color: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: color.into(),
        }
    }
}

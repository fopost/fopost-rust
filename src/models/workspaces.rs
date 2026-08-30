//! Workspaces — the tenant every other resource is scoped to.

use serde::{Deserialize, Serialize};

use super::common::WorkspaceType;

/// An account listed inside a workspace.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAccountRef {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
}

/// A workspace, with its accounts when the endpoint includes them.
#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(rename = "type", default)]
    pub workspace_type: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Present on `list` and `get`, empty on `create` and `update`.
    #[serde(default)]
    pub accounts: Vec<WorkspaceAccountRef>,
}

/// One account's audience inside a workspace roll-up.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAccountAnalytics {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub followers: Option<i64>,
    #[serde(default)]
    pub following: Option<i64>,
    #[serde(default)]
    pub total_posts: Option<i64>,
    #[serde(default)]
    pub fetched_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAnalyticsTotals {
    #[serde(default)]
    pub followers: i64,
    #[serde(default)]
    pub total_posts: i64,
}

/// A workspace's audience, per account and in total.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAnalytics {
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub accounts: Vec<WorkspaceAccountAnalytics>,
    #[serde(default)]
    pub totals: Option<WorkspaceAnalyticsTotals>,
}

/// The body of `POST /workspaces`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWorkspace {
    pub name: String,
    pub slug: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub workspace_type: Option<WorkspaceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl CreateWorkspace {
    pub fn new(name: impl Into<String>, slug: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            slug: slug.into(),
            workspace_type: None,
            logo: None,
            website: None,
            timezone: None,
            country: None,
            description: None,
            language: None,
        }
    }

    pub fn workspace_type(mut self, workspace_type: WorkspaceType) -> Self {
        self.workspace_type = Some(workspace_type);
        self
    }

    /// An IANA zone, which is what a schedule is interpreted against.
    pub fn timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }
}

/// The body of `PUT /workspaces/{id}`. Only the fields you set are sent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWorkspace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub workspace_type: Option<WorkspaceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Hold every post for approval before it can be scheduled.
    #[serde(rename = "requireApproval", skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    #[serde(rename = "aiAltTextEnabled", skip_serializing_if = "Option::is_none")]
    pub ai_alt_text_enabled: Option<bool>,
    #[serde(rename = "brandColor", skip_serializing_if = "Option::is_none")]
    pub brand_color: Option<String>,
}

impl UpdateWorkspace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn require_approval(mut self, require: bool) -> Self {
        self.require_approval = Some(require);
        self
    }
}

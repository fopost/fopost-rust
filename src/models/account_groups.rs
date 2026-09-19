//! Account groups: named sets of connected accounts a post can target at once.

use serde::{Deserialize, Serialize};

/// An account group.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub account_ids: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// The body of `POST /account-groups`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateAccountGroup {
    pub workspace_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_ids: Option<Vec<String>>,
}

impl CreateAccountGroup {
    pub fn new(workspace_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            name: name.into(),
            account_ids: None,
        }
    }

    /// The accounts the group starts with.
    pub fn account_ids<S: Into<String>>(mut self, ids: impl IntoIterator<Item = S>) -> Self {
        self.account_ids = Some(ids.into_iter().map(Into::into).collect());
        self
    }
}

/// The body of `PATCH /account-groups/{id}`.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateAccountGroup {
    pub name: String,
}

impl UpdateAccountGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

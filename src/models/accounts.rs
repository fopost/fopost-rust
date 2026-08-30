//! Connected social accounts, their health, and X communities.

use serde::{Deserialize, Serialize};

use super::common::{HealthStatus, Platform};

/// A connected account as the list endpoint returns it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub platform: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub is_primary: Option<bool>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub health_status: Option<HealthStatus>,
    #[serde(default)]
    pub last_health_check: Option<String>,
}

/// The workspace an account belongs to, as `GET /accounts/{id}` reports it.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountWorkspaceRef {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(rename = "type", default)]
    pub workspace_type: Option<String>,
}

/// One account in full, with the workspace it belongs to.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountDetail {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub platform: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub workspace: Option<AccountWorkspaceRef>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// A newly connected account.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountCreated {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub platform: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Whether one account's credentials still work.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountHealth {
    pub id: String,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub health_status: Option<HealthStatus>,
    #[serde(default)]
    pub last_health_check: Option<String>,
}

/// One row of the account health sweep.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountHealthRow {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub health_status: Option<HealthStatus>,
    #[serde(default)]
    pub last_health_check: Option<String>,
}

/// How many accounts sit in each health state.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountHealthCounts {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub healthy: u32,
    #[serde(default)]
    pub degraded: u32,
    #[serde(default)]
    pub expired: u32,
    #[serde(default)]
    pub revoked: u32,
    #[serde(default)]
    pub unknown: u32,
}

/// The whole account health picture, per account and in total.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountsHealthSummary {
    #[serde(default)]
    pub accounts: Vec<AccountHealthRow>,
    pub summary: AccountHealthCounts,
}

/// The result of toggling the primary flag.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryToggled {
    pub id: String,
    #[serde(default)]
    pub is_primary: bool,
}

/// The verdict of a credential check.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialCheck {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub valid: bool,
    #[serde(default)]
    pub health_status: Option<HealthStatus>,
}

/// The result of refreshing an OAuth token.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRefreshed {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// One snapshot of an account's audience, oldest first.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountHistoryPoint {
    #[serde(default)]
    pub followers: Option<i64>,
    #[serde(default)]
    pub following: Option<i64>,
    #[serde(default)]
    pub total_posts: Option<i64>,
    #[serde(default)]
    pub reach: Option<i64>,
    #[serde(default)]
    pub profile_views: Option<i64>,
    #[serde(default)]
    pub fetched_at: Option<String>,
}

/// An account's audience over time.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAnalyticsHistory {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub history: Vec<AccountHistoryPoint>,
}

/// An X community an account can post into.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Community {
    /// The row id, which is what removing one takes.
    pub id: i64,
    #[serde(default)]
    pub account_id: Option<String>,
    /// The id X itself uses.
    #[serde(default)]
    pub community_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub member_count: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub last_synced_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A community search hit, straight from X.
#[derive(Debug, Clone, Deserialize)]
pub struct CommunitySearchResult {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub member_count: Option<i64>,
}

/// The body of `POST /accounts` — connecting an account with credentials you
/// already hold, instead of running the OAuth flow in the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct CreateAccount {
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    pub platform: Platform,
    pub username: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// Platform credentials. What belongs here differs per platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl CreateAccount {
    pub fn new(
        workspace_id: impl Into<String>,
        platform: Platform,
        username: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            platform,
            username: username.into(),
            name: name.into(),
            avatar: None,
            credentials: None,
        }
    }

    pub fn credentials(
        mut self,
        credentials: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn avatar(mut self, avatar: impl Into<String>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }
}

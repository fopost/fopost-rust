//! Hosted connect links — let an end user connect an account with no FoPost login.

use serde::{Deserialize, Serialize};

use super::common::{string_enum, Platform};

string_enum! {
    /// Where a connect session stands.
    pub enum ConnectSessionStatus {
        Pending => "pending",
        Used => "used",
        Expired => "expired",
    }
}

/// The name and logo the hosted page shows.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConnectSessionBranding {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub logo_url: Option<String>,
}

/// A single-use hosted connect link.
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectSession {
    pub id: String,
    pub status: ConnectSessionStatus,
    #[serde(default)]
    pub external_user_id: String,
    #[serde(default)]
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub redirect_url: Option<String>,
    #[serde(default)]
    pub branding: ConnectSessionBranding,
    /// The connected account, once the session is used.
    #[serde(default)]
    pub account_id: Option<String>,
    /// The hosted link to hand to your user. Present on creation only.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub used_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// The branding sent on `POST /connect-sessions`.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectBranding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

/// The body of `POST /connect-sessions`. Build it with [`CreateConnectSession::new`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectSession {
    /// May be omitted for a key bound to one workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Your own id for the end user; the account is tagged with it.
    pub external_user_id: String,
    /// The networks the page offers, one to nine.
    pub platforms: Vec<Platform>,
    /// Must be on a domain in the workspace redirect allowlist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding: Option<ConnectBranding>,
}

impl CreateConnectSession {
    pub fn new(
        external_user_id: impl Into<String>,
        platforms: impl IntoIterator<Item = Platform>,
    ) -> Self {
        Self {
            workspace_id: None,
            external_user_id: external_user_id.into(),
            platforms: platforms.into_iter().collect(),
            redirect_url: None,
            branding: None,
        }
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn redirect_url(mut self, url: impl Into<String>) -> Self {
        self.redirect_url = Some(url.into());
        self
    }

    pub fn branding(mut self, branding: ConnectBranding) -> Self {
        self.branding = Some(branding);
        self
    }
}

//! `client.connect_sessions()` — hosted links that connect an end user's account.

use reqwest::Method;

use crate::error::Result;
use crate::http::HttpClient;
use crate::models::{ConnectSession, CreateConnectSession};

/// Hosted connect links.
#[derive(Debug, Clone)]
pub struct ConnectSessions<'a> {
    pub(crate) http: &'a HttpClient,
}

impl ConnectSessions<'_> {
    /// Mint a single-use link, open for one hour, that lets one of your end
    /// users connect a social account into your workspace with no FoPost
    /// login. Hand the answer's `url` to the user; they land on `redirect_url`
    /// with `status`, `platform`, `account_id` and `external_user_id` once
    /// connected.
    pub async fn create(&self, session: &CreateConnectSession) -> Result<ConnectSession> {
        self.http
            .send(Method::POST, "/connect-sessions", None, Some(session))
            .await
    }

    /// One session: whether it was used, and which account it connected.
    pub async fn get(&self, id: &str) -> Result<ConnectSession> {
        self.http
            .send::<ConnectSession, ()>(Method::GET, &format!("/connect-sessions/{id}"), None, None)
            .await
    }
}

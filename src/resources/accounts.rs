//! `client.accounts()` — the social accounts a workspace publishes through.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    Account, AccountAnalyticsHistory, AccountCreated, AccountDetail, AccountHealth,
    AccountsHealthSummary, Community, CommunitySearchResult, CreateAccount, CredentialCheck,
    Message, PrimaryToggled, TokenRefreshed,
};

/// Connected accounts, their health, and their X communities.
#[derive(Debug, Clone)]
pub struct Accounts<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Accounts<'_> {
    /// Connected accounts, across every workspace the key reaches unless one is named.
    pub async fn list(&self, workspace_id: Option<&str>) -> Result<Vec<Account>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspaceId", workspace_id);
        let body: Envelope<Vec<Account>> = self
            .http
            .send::<_, ()>(Method::GET, "/accounts", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// One account in full, with the workspace it belongs to.
    pub async fn get(&self, id: &str) -> Result<AccountDetail> {
        let body: Envelope<AccountDetail> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/accounts/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Connect an account with credentials you already hold. The dashboard's
    /// OAuth flow is the other way in, and the usual one.
    pub async fn create(&self, account: &CreateAccount) -> Result<AccountCreated> {
        let body: Envelope<AccountCreated> = self
            .http
            .send(Method::POST, "/accounts", None, Some(account))
            .await?;
        Ok(body.data)
    }

    /// Disconnect an account. Scheduled posts targeting it will fail.
    pub async fn delete(&self, id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, &format!("/accounts/{id}"), None, None)
            .await
    }

    /// Make this the primary account for its platform, or clear the flag.
    pub async fn toggle_primary(&self, id: &str) -> Result<PrimaryToggled> {
        let body: Envelope<PrimaryToggled> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/accounts/{id}/primary"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Ask the platform whether the stored credentials still work.
    pub async fn validate(&self, id: &str) -> Result<CredentialCheck> {
        let body: Envelope<CredentialCheck> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/accounts/{id}/validate"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The stored health verdict. Set `refresh` to re-check with the platform first.
    pub async fn health(&self, id: &str, refresh: bool) -> Result<AccountHealth> {
        let mut query: Query = Vec::new();
        if refresh {
            query.push(("refresh", "true".into()));
        }
        let body: Envelope<AccountHealth> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/health"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Health for every account the key can see, with the counts.
    pub async fn health_summary(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<AccountsHealthSummary> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspaceId", workspace_id);
        let body: Envelope<AccountsHealthSummary> = self
            .http
            .send::<_, ()>(Method::GET, "/accounts/health", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Force an OAuth token refresh instead of waiting for the scheduled one.
    pub async fn refresh_token(&self, id: &str) -> Result<TokenRefreshed> {
        let body: Envelope<TokenRefreshed> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/accounts/{id}/refresh-token"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The account's audience over time, newest first.
    pub async fn analytics(&self, id: &str, limit: Option<u32>) -> Result<AccountAnalyticsHistory> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "limit", limit);
        let body: Envelope<AccountAnalyticsHistory> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/analytics"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    // ─── X communities ─────────────────────────────────────────────

    /// The X communities this account can post into.
    pub async fn communities(&self, account_id: &str) -> Result<Vec<Community>> {
        let body: Envelope<Vec<Community>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{account_id}/communities"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Re-read the account's communities from X.
    pub async fn sync_communities(&self, account_id: &str) -> Result<Vec<Community>> {
        let body: Envelope<Vec<Community>> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/accounts/{account_id}/communities/sync"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Search X's communities by name.
    pub async fn search_communities(
        &self,
        account_id: &str,
        query: &str,
    ) -> Result<Vec<CommunitySearchResult>> {
        let params: Query = vec![("q", query.to_string())];
        let body: Envelope<Vec<CommunitySearchResult>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{account_id}/communities/search"),
                Some(params),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Add a community by the id X uses, for one search cannot reach.
    pub async fn add_community(
        &self,
        account_id: &str,
        community_id: &str,
        name: Option<&str>,
    ) -> Result<Community> {
        let payload = serde_json::json!({ "communityId": community_id, "name": name });
        let body: Envelope<Community> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{account_id}/communities/manual"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove a community. `community_id` is the row id, not X's own id.
    pub async fn remove_community(&self, account_id: &str, community_id: i64) -> Result<bool> {
        #[derive(serde::Deserialize)]
        struct Removed {
            success: bool,
        }
        let body: Removed = self
            .http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/accounts/{account_id}/communities/{community_id}"),
                None,
                None,
            )
            .await?;
        Ok(body.success)
    }
}

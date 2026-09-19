//! `client.account_groups()` — named sets of accounts a post can target at once.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{AccountGroup, CreateAccountGroup, Message, UpdateAccountGroup};

/// Account groups. Needs the `accounts` scope.
#[derive(Debug, Clone)]
pub struct AccountGroups<'a> {
    pub(crate) http: &'a HttpClient,
}

impl AccountGroups<'_> {
    /// Account groups, across every workspace the key reaches unless one is named.
    pub async fn list(&self, workspace_id: Option<&str>) -> Result<Vec<AccountGroup>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", workspace_id);
        let body: Envelope<Vec<AccountGroup>> = self
            .http
            .send::<_, ()>(Method::GET, "/account-groups", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// One account group.
    pub async fn get(&self, id: &str) -> Result<AccountGroup> {
        let body: Envelope<AccountGroup> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/account-groups/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Create an account group in a workspace.
    pub async fn create(&self, group: &CreateAccountGroup) -> Result<AccountGroup> {
        let body: Envelope<AccountGroup> = self
            .http
            .send(Method::POST, "/account-groups", None, Some(group))
            .await?;
        Ok(body.data)
    }

    /// Rename an account group.
    pub async fn update(&self, id: &str, changes: &UpdateAccountGroup) -> Result<AccountGroup> {
        let body: Envelope<AccountGroup> = self
            .http
            .send(
                Method::PATCH,
                &format!("/account-groups/{id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Delete an account group. Its accounts stay connected.
    pub async fn delete(&self, id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, &format!("/account-groups/{id}"), None, None)
            .await
    }

    /// Replace the group's members with `account_ids`. An empty list empties it.
    pub async fn set_members<S: AsRef<str>>(
        &self,
        id: &str,
        account_ids: impl IntoIterator<Item = S>,
    ) -> Result<AccountGroup> {
        let account_ids: Vec<String> = account_ids
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();
        let payload = serde_json::json!({ "account_ids": account_ids });
        let body: Envelope<AccountGroup> = self
            .http
            .send(
                Method::PUT,
                &format!("/account-groups/{id}/members"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }
}

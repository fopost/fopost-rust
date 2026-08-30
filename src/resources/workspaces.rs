//! `client.workspaces()` — the tenant every other resource is scoped to.

use reqwest::Method;

use crate::error::Result;
use crate::http::{Envelope, HttpClient};
use crate::models::{CreateWorkspace, Message, UpdateWorkspace, Workspace, WorkspaceAnalytics};

/// Workspaces the API key can reach.
#[derive(Debug, Clone)]
pub struct Workspaces<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Workspaces<'_> {
    /// Every workspace the key reaches, each with its connected accounts. A key
    /// bound to one workspace sees only that one.
    pub async fn list(&self) -> Result<Vec<Workspace>> {
        let body: Envelope<Vec<Workspace>> = self
            .http
            .send::<_, ()>(Method::GET, "/workspaces", None, None)
            .await?;
        Ok(body.data)
    }

    /// One workspace, with its connected accounts.
    pub async fn get(&self, id: &str) -> Result<Workspace> {
        let body: Envelope<Workspace> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/workspaces/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Create a workspace. The slug has to be unique on the account.
    pub async fn create(&self, workspace: &CreateWorkspace) -> Result<Workspace> {
        let body: Envelope<Workspace> = self
            .http
            .send(Method::POST, "/workspaces", None, Some(workspace))
            .await?;
        Ok(body.data)
    }

    /// Partial update — only the fields set on the body are sent.
    pub async fn update(&self, id: &str, changes: &UpdateWorkspace) -> Result<Workspace> {
        let body: Envelope<Workspace> = self
            .http
            .send(
                Method::PUT,
                &format!("/workspaces/{id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Delete a workspace and everything in it. This cannot be undone.
    pub async fn delete(&self, id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, &format!("/workspaces/{id}"), None, None)
            .await
    }

    /// The workspace's audience, per account and in total.
    pub async fn analytics(&self, id: &str) -> Result<WorkspaceAnalytics> {
        let body: Envelope<WorkspaceAnalytics> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/workspaces/{id}/analytics"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }
}

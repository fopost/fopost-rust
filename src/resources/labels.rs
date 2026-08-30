//! `client.labels()` — the campaign tags posts are grouped and reported by.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{CreateLabel, Label, Message, UpdateLabel};

/// Labels.
#[derive(Debug, Clone)]
pub struct Labels<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Labels<'_> {
    /// Labels, across every workspace the key reaches unless one is named.
    pub async fn list(&self, workspace_id: Option<&str>) -> Result<Vec<Label>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", workspace_id);
        let body: Envelope<Vec<Label>> = self
            .http
            .send::<_, ()>(Method::GET, "/labels", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// One label.
    pub async fn get(&self, id: &str) -> Result<Label> {
        self.http
            .send::<Label, ()>(Method::GET, &format!("/labels/{id}"), None, None)
            .await
    }

    /// Create a label in a workspace.
    pub async fn create(&self, label: &CreateLabel) -> Result<Label> {
        self.http
            .send(Method::POST, "/labels", None, Some(label))
            .await
    }

    /// Rename or recolor a label. The API wants both fields.
    pub async fn update(&self, id: &str, changes: &UpdateLabel) -> Result<Label> {
        self.http
            .send(Method::PUT, &format!("/labels/{id}"), None, Some(changes))
            .await
    }

    /// Delete a label. The posts carrying it keep everything but the label.
    pub async fn delete(&self, id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, &format!("/labels/{id}"), None, None)
            .await
    }
}

//! `client.webhooks()` — the push side of the API.

use reqwest::Method;

use crate::error::Result;
use crate::http::{Envelope, HttpClient};
use crate::models::{CreateWebhook, Message, UpdateWebhook, Webhook, WebhookCreated};

/// Webhook subscriptions.
#[derive(Debug, Clone)]
pub struct Webhooks<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Webhooks<'_> {
    /// Every webhook the key can reach.
    pub async fn list(&self) -> Result<Vec<Webhook>> {
        let body: Envelope<Vec<Webhook>> = self
            .http
            .send::<_, ()>(Method::GET, "/webhooks", None, None)
            .await?;
        Ok(body.data)
    }

    /// Register an endpoint. The `secret` on the answer is shown once — store
    /// it, it signs every delivery.
    pub async fn create(&self, webhook: &CreateWebhook) -> Result<WebhookCreated> {
        let body: Envelope<WebhookCreated> = self
            .http
            .send(Method::POST, "/webhooks", None, Some(webhook))
            .await?;
        Ok(body.data)
    }

    /// Partial update — only the fields set on the body are sent.
    pub async fn update(&self, id: &str, changes: &UpdateWebhook) -> Result<Webhook> {
        let body: Envelope<Webhook> = self
            .http
            .send(Method::PUT, &format!("/webhooks/{id}"), None, Some(changes))
            .await?;
        Ok(body.data)
    }

    /// Delete a webhook. Nothing further is delivered to it.
    pub async fn delete(&self, id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, &format!("/webhooks/{id}"), None, None)
            .await
    }

    /// Send a sample event, to check the endpoint before something real happens.
    pub async fn test(&self, id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::POST, &format!("/webhooks/{id}/test"), None, None)
            .await
    }
}

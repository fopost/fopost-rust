//! `client.webhooks()` — the push side of the API.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    CreateWebhook, Message, UpdateWebhook, Webhook, WebhookCreated, WebhookDelivery,
    WebhookDeliveryDetail, WebhookDeliveryStatus,
};

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

    /// Recent delivery attempts to this webhook, newest first. `limit` is 50
    /// by default and 200 at most.
    pub async fn deliveries(
        &self,
        id: &str,
        status: Option<WebhookDeliveryStatus>,
        limit: Option<u32>,
    ) -> Result<Vec<WebhookDelivery>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "status", status);
        push_opt(&mut query, "limit", limit);
        let body: Envelope<Vec<WebhookDelivery>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/webhooks/{id}/deliveries"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// One delivery attempt, with the body exactly as it was sent.
    pub async fn delivery(&self, id: &str, delivery_id: &str) -> Result<WebhookDeliveryDetail> {
        self.http
            .send::<WebhookDeliveryDetail, ()>(
                Method::GET,
                &format!("/webhooks/{id}/deliveries/{delivery_id}"),
                None,
                None,
            )
            .await
    }

    /// Send a delivery's payload again, as a new attempt under the same
    /// delivery key. 409 while the webhook is paused.
    pub async fn redeliver(&self, id: &str, delivery_id: &str) -> Result<()> {
        self.http
            .send::<serde_json::Value, ()>(
                Method::POST,
                &format!("/webhooks/{id}/deliveries/{delivery_id}/redeliver"),
                None,
                None,
            )
            .await?;
        Ok(())
    }
}

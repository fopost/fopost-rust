//! Webhooks — the push side of the API.

use serde::{Deserialize, Serialize};

use super::common::string_enum;

string_enum! {
    /// What a webhook can subscribe to.
    pub enum WebhookEvent {
        PostPublished => "post.published",
        PostFailed => "post.failed",
        PostPartiallyFailed => "post.partially_failed",
        DeliveryPublished => "delivery.published",
        DeliveryFailed => "delivery.failed",
        DeliveryDelayed => "delivery.delayed",
        AccountHealthChanged => "account.health_changed",
        ArticlePublished => "article.published",
        /// The sample a blog Website target receives from `test_website`.
        ArticleTest => "article.test",
    }
}

/// A registered webhook.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub url: String,
    #[serde(default)]
    pub events: Vec<WebhookEvent>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub last_triggered_at: Option<String>,
    /// Consecutive delivery failures. The API stops calling a dead endpoint.
    #[serde(default)]
    pub failure_count: u32,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A newly created webhook. `secret` is shown once — store it now, it signs
/// every delivery so you can verify the payload really came from FoPost.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookCreated {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub url: String,
    /// The signing secret, returned only at creation.
    pub secret: String,
    #[serde(default)]
    pub events: Vec<WebhookEvent>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// The body of `POST /webhooks`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhook {
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    pub url: String,
    pub events: Vec<WebhookEvent>,
}

impl CreateWebhook {
    pub fn new(
        workspace_id: impl Into<String>,
        url: impl Into<String>,
        events: impl IntoIterator<Item = WebhookEvent>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            url: url.into(),
            events: events.into_iter().collect(),
        }
    }
}

/// The body of `PUT /webhooks/{id}`. Only the fields you set are sent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWebhook {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<WebhookEvent>>,
    /// Pause a webhook without deleting it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

impl UpdateWebhook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn events(mut self, events: impl IntoIterator<Item = WebhookEvent>) -> Self {
        self.events = Some(events.into_iter().collect());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }
}

string_enum! {
    /// How one delivery attempt ended.
    pub enum WebhookDeliveryStatus {
        Success => "success",
        Failed => "failed",
    }
}

/// One attempt to deliver an event to a webhook endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookDelivery {
    pub id: String,
    #[serde(default)]
    pub webhook_id: Option<String>,
    #[serde(default)]
    pub event: Option<WebhookEvent>,
    /// The `X-FoPost-Delivery` id; every attempt of one event shares it.
    #[serde(default)]
    pub delivery_key: Option<String>,
    /// 1 for the first send; retries and redeliveries count up.
    #[serde(default)]
    pub attempt: i64,
    #[serde(default)]
    pub status: Option<WebhookDeliveryStatus>,
    /// `None` when no response arrived.
    #[serde(default)]
    pub response_status: Option<i64>,
    /// Up to 500 characters of what the endpoint answered, secrets redacted.
    #[serde(default)]
    pub response_snippet: Option<String>,
    #[serde(default)]
    pub duration_ms: i64,
    /// SHA-256 of the body that was sent.
    #[serde(default)]
    pub payload_hash: Option<String>,
    #[serde(default)]
    pub next_retry_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A single delivery read in full: the row plus the body exactly as it was sent.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookDeliveryDetail {
    #[serde(flatten)]
    pub delivery: WebhookDelivery,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

impl std::ops::Deref for WebhookDeliveryDetail {
    type Target = WebhookDelivery;

    fn deref(&self) -> &WebhookDelivery {
        &self.delivery
    }
}

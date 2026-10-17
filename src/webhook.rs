//! Verify a delivery the API pushed to you.
//!
//! Every outbound FoPost webhook, and every signed delivery a blog Website
//! target receives, carries the same headers, all keyed by the secret shown
//! once when the subscription or the website URL was created:
//!
//! - `X-FoPost-Signature: sha256=<hex>`, HMAC-SHA256 over the raw body
//! - `X-FoPost-Timestamp: <unix seconds>`
//! - `X-FoPost-Signature-256: sha256=<hex>`, HMAC-SHA256 over `<timestamp>.<raw body>`
//!
//! [`verify_webhook`] prefers the timestamped signature and refuses a delivery
//! older than [`WEBHOOK_TOLERANCE`], so a captured delivery cannot be replayed.
//! It falls back to the plain body signature only when no timestamp header is
//! present. Always hand it the request bytes exactly as received: re-serialised
//! JSON changes the bytes and the signature no longer matches.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use sha2::Sha256;

use crate::models::WebhookEvent;

/// Header carrying the HMAC over the raw body.
pub const WEBHOOK_SIGNATURE_HEADER: &str = "x-fopost-signature";
/// Header carrying the HMAC over `<timestamp>.<raw body>`.
pub const WEBHOOK_SIGNATURE_256_HEADER: &str = "x-fopost-signature-256";
/// Header carrying the unix timestamp of the attempt.
pub const WEBHOOK_TIMESTAMP_HEADER: &str = "x-fopost-timestamp";
/// Header naming the event, also present inside the envelope.
pub const WEBHOOK_EVENT_HEADER: &str = "x-fopost-event";
/// Header carrying the delivery id, the same across retries of one event.
pub const WEBHOOK_DELIVERY_HEADER: &str = "x-fopost-delivery";
/// How far a delivery's timestamp may sit from the current clock.
pub const WEBHOOK_TOLERANCE: Duration = Duration::from_secs(300);

/// The signing headers of one delivery, pulled out of whatever header type
/// your server uses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WebhookSignature {
    /// `X-FoPost-Signature`.
    pub signature: Option<String>,
    /// `X-FoPost-Signature-256`.
    pub signature_256: Option<String>,
    /// `X-FoPost-Timestamp`.
    pub timestamp: Option<String>,
}

impl WebhookSignature {
    /// Read the three headers out of a `HeaderMap`.
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let get = |name: &str| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(|value| value.trim().to_string())
        };
        Self {
            signature: get(WEBHOOK_SIGNATURE_HEADER),
            signature_256: get(WEBHOOK_SIGNATURE_256_HEADER),
            timestamp: get(WEBHOOK_TIMESTAMP_HEADER),
        }
    }
}

impl From<&HeaderMap> for WebhookSignature {
    fn from(headers: &HeaderMap) -> Self {
        Self::from_headers(headers)
    }
}

/// The envelope every delivery carries. `data` is the event-specific body;
/// decode it with [`WebhookEnvelope::data_as`].
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEnvelope {
    /// Which event this is.
    pub event: WebhookEvent,
    /// The event-specific body.
    #[serde(default)]
    pub data: serde_json::Value,
    /// When the event happened, as an ISO 8601 string.
    #[serde(default)]
    pub timestamp: String,
}

impl WebhookEnvelope {
    /// Decode `data` into a typed payload, such as
    /// [`BlogArticlePublished`](crate::models::BlogArticlePublished) for
    /// `article.published`.
    pub fn data_as<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }
}

/// Why a delivery was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum WebhookError {
    /// Neither signature header was present.
    #[error("fopost: no signature header on the delivery")]
    MissingSignature,
    /// A signature header was present but did not match the body.
    #[error("fopost: the signature does not match the body")]
    InvalidSignature,
    /// `X-FoPost-Timestamp` was present but unreadable.
    #[error("fopost: the timestamp header is not a unix time")]
    InvalidTimestamp,
    /// The delivery sits further than [`WEBHOOK_TOLERANCE`] from the clock.
    #[error("fopost: the delivery is outside the replay tolerance")]
    StaleTimestamp,
    /// The signature checked out but the body is not a webhook envelope.
    #[error("fopost: the body is not a webhook envelope: {0}")]
    InvalidPayload(String),
}

/// Verify a delivery against the secret and return its envelope.
///
/// ```no_run
/// use fopost::{verify_webhook, WebhookSignature};
/// use fopost::models::BlogArticlePublished;
///
/// # fn handle(raw_body: &[u8], headers: &reqwest::header::HeaderMap) -> Result<(), Box<dyn std::error::Error>> {
/// let secret = std::env::var("FOPOST_WEBHOOK_SECRET")?;
/// let envelope = verify_webhook(raw_body, &WebhookSignature::from_headers(headers), &secret)?;
/// if envelope.event.as_str() == "article.published" {
///     let article: BlogArticlePublished = envelope.data_as()?;
///     println!("{} is live at {:?}", article.title, article.published_url);
/// }
/// # Ok(()) }
/// ```
pub fn verify_webhook(
    raw_body: &[u8],
    signature: &WebhookSignature,
    secret: &str,
) -> Result<WebhookEnvelope, WebhookError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    verify_webhook_at(raw_body, signature, secret, now)
}

/// [`verify_webhook`] against a clock you supply, as unix seconds.
pub fn verify_webhook_at(
    raw_body: &[u8],
    signature: &WebhookSignature,
    secret: &str,
    now_unix: u64,
) -> Result<WebhookEnvelope, WebhookError> {
    match (&signature.timestamp, &signature.signature_256) {
        (Some(timestamp), Some(signature_256)) => {
            let sent: u64 = timestamp
                .parse()
                .map_err(|_| WebhookError::InvalidTimestamp)?;
            if now_unix.abs_diff(sent) > WEBHOOK_TOLERANCE.as_secs() {
                return Err(WebhookError::StaleTimestamp);
            }
            let mut mac = hmac(secret);
            mac.update(timestamp.as_bytes());
            mac.update(b".");
            mac.update(raw_body);
            check(mac, signature_256)?;
        }
        (None, _) => {
            let header = signature
                .signature
                .as_deref()
                .ok_or(WebhookError::MissingSignature)?;
            let mut mac = hmac(secret);
            mac.update(raw_body);
            check(mac, header)?;
        }
        // A timestamp with no timestamped signature is not a delivery we sent.
        (Some(_), None) => return Err(WebhookError::MissingSignature),
    }

    serde_json::from_slice(raw_body).map_err(|err| WebhookError::InvalidPayload(err.to_string()))
}

/// The `X-FoPost-Signature` value the API sends for these exact bytes.
pub fn sign_webhook(raw_body: &[u8], secret: &str) -> String {
    let mut mac = hmac(secret);
    mac.update(raw_body);
    format!("sha256={}", hex(&mac.finalize().into_bytes()))
}

/// The `X-FoPost-Signature-256` value the API sends for these bytes at
/// `timestamp` (unix seconds).
pub fn sign_webhook_at(raw_body: &[u8], secret: &str, timestamp: u64) -> String {
    let mut mac = hmac(secret);
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(raw_body);
    format!("sha256={}", hex(&mac.finalize().into_bytes()))
}

fn hmac(secret: &str) -> Hmac<Sha256> {
    // HMAC accepts a key of any length, so this cannot fail.
    Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac accepts any key length")
}

fn check(mac: Hmac<Sha256>, header: &str) -> Result<(), WebhookError> {
    let hex_digest = header
        .trim()
        .strip_prefix("sha256=")
        .ok_or(WebhookError::InvalidSignature)?;
    let digest = unhex(hex_digest).ok_or(WebhookError::InvalidSignature)?;
    mac.verify_slice(&digest)
        .map_err(|_| WebhookError::InvalidSignature)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

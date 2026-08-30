//! Errors returned by the SDK.
//!
//! Every non-2xx response becomes [`Error::Api`]. The API answers with an
//! `{"error": "<code>", "message": "<explanation>"}` envelope, which maps onto
//! the `code` and `message` fields.

use serde::Deserialize;

/// Convenience alias for every fallible SDK call.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything that can go wrong talking to the FoPost API.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The API answered with a non-2xx status.
    #[error("{0}")]
    Api(#[from] ApiError),

    /// The request never completed: DNS, TLS, connection, or timeout.
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),

    /// A 2xx response body did not match the shape the SDK expected.
    #[error("could not decode the response body: {source}")]
    Decode {
        /// The serde failure.
        #[source]
        source: serde_json::Error,
        /// The raw body, so the caller can see what actually arrived.
        body: String,
    },

    /// The client was built with something the SDK cannot use.
    #[error("{0}")]
    Config(String),
}

impl Error {
    /// The HTTP status, when the failure came from a response.
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Api(err) => Some(err.status),
            Error::Transport(err) => err.status().map(|s| s.as_u16()),
            _ => None,
        }
    }

    /// The machine-readable error code (`not_found`, `subscription_required`, …).
    pub fn code(&self) -> Option<&str> {
        match self {
            Error::Api(err) => err.code.as_deref(),
            _ => None,
        }
    }

    /// True for 429, whether or not the retry budget was exhausted.
    pub fn is_rate_limited(&self) -> bool {
        self.status() == Some(429)
    }

    /// Seconds to wait before retrying, when the API said so.
    pub fn retry_after(&self) -> Option<f64> {
        match self {
            Error::Api(err) => err.retry_after,
            _ => None,
        }
    }
}

/// A failed response, decoded.
#[derive(Debug, Clone)]
pub struct ApiError {
    /// HTTP status code.
    pub status: u16,
    /// The `error` field — a stable machine-readable code.
    pub code: Option<String>,
    /// The `message` field, or a fallback built from the status.
    pub message: String,
    /// The whole decoded body, for fields the SDK does not model.
    pub body: Option<serde_json::Value>,
    /// Parsed from `Retry-After`, in seconds, on a 429.
    pub retry_after: Option<f64>,
}

impl ApiError {
    /// Where to send the user when a 402 says the plan is out of room.
    pub fn upgrade_url(&self) -> Option<&str> {
        self.body.as_ref()?.get("upgrade_url")?.as_str()
    }

    /// 401 — missing, invalid, or expired API key.
    pub fn is_unauthorized(&self) -> bool {
        self.status == 401
    }

    /// 402 — no active subscription, or AI credits exhausted.
    pub fn is_payment_required(&self) -> bool {
        self.status == 402
    }

    /// 403 — the key is valid but lacks the scope or the workspace.
    pub fn is_forbidden(&self) -> bool {
        self.status == 403
    }

    /// 404 — no such resource, or it is outside the key's reach.
    pub fn is_not_found(&self) -> bool {
        self.status == 404
    }

    /// 429 — the per-key, per-minute ceiling was hit.
    pub fn is_rate_limited(&self) -> bool {
        self.status == 429
    }

    pub(crate) fn from_body(
        status: u16,
        body: Option<serde_json::Value>,
        retry_after: Option<f64>,
    ) -> Self {
        #[derive(Deserialize)]
        struct Envelope {
            error: Option<String>,
            message: Option<String>,
        }

        let envelope = body
            .clone()
            .and_then(|value| serde_json::from_value::<Envelope>(value).ok());
        let code = envelope.as_ref().and_then(|e| e.error.clone());
        let message = envelope
            .and_then(|e| e.message)
            .filter(|m| !m.is_empty())
            .or_else(|| code.clone())
            .unwrap_or_else(|| format!("HTTP {status}"));

        ApiError {
            status,
            code,
            message,
            body,
            retry_after,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.code {
            Some(code) => write!(f, "[{} {}] {}", self.status, code, self.message),
            None => write!(f, "[{}] {}", self.status, self.message),
        }
    }
}

impl std::error::Error for ApiError {}

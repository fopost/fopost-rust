//! Internal transport: auth headers, JSON coding, retries, envelope unwrap.

use std::time::Duration;

use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{ApiError, Error, Result};

/// Where the API lives, version path included.
pub const DEFAULT_BASE_URL: &str = "https://api.fopost.com/v1";

pub(crate) const USER_AGENT: &str = concat!("fopost-rust/", env!("CARGO_PKG_VERSION"));

/// Never wait longer than this between attempts, whatever `Retry-After` says.
const MAX_RETRY_WAIT: Duration = Duration::from_secs(60);

/// Statuses worth a second attempt: the rate limiter, and a gateway blip.
fn is_retryable(status: StatusCode) -> bool {
    matches!(status.as_u16(), 429 | 502 | 503 | 504)
}

/// The `{"data": …}` envelope most endpoints wrap their payload in.
#[derive(Debug, Deserialize)]
pub(crate) struct Envelope<T> {
    pub data: T,
}

/// One query parameter, already stringified. `None` values are dropped.
pub(crate) type Query = Vec<(&'static str, String)>;

/// Push `name=value` when the value is present.
pub(crate) fn push_opt<T: ToString>(query: &mut Query, name: &'static str, value: Option<T>) {
    if let Some(value) = value {
        query.push((name, value.to_string()));
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HttpClient {
    inner: reqwest::Client,
    api_key: String,
    base_url: Url,
    max_retries: u32,
}

impl HttpClient {
    pub(crate) fn new(
        inner: reqwest::Client,
        api_key: String,
        base_url: &str,
        max_retries: u32,
    ) -> Result<Self> {
        if api_key.is_empty() {
            return Err(Error::Config("fopost: an api key is required".into()));
        }
        if max_retries < 1 {
            return Err(Error::Config(
                "fopost: max_retries must be at least 1".into(),
            ));
        }
        // A trailing slash keeps `Url::join` from eating the last path segment.
        let normalized = format!("{}/", base_url.trim_end_matches('/'));
        let base_url = Url::parse(&normalized)
            .map_err(|err| Error::Config(format!("fopost: invalid base_url — {err}")))?;

        Ok(Self {
            inner,
            api_key,
            base_url,
            max_retries,
        })
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }

    /// The configured base URL, without the trailing slash it is stored with.
    pub(crate) fn base_url(&self) -> &str {
        self.base_url.as_str().trim_end_matches('/')
    }

    fn url(&self, path: &str) -> Result<Url> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|err| {
                Error::Config(format!("fopost: could not build a url for {path} — {err}"))
            })
    }

    /// Send a request and decode the body into `T`.
    pub(crate) async fn send<T, B>(
        &self,
        method: Method,
        path: &str,
        query: Option<Query>,
        body: Option<&B>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let value = self.send_value(method, path, query, body).await?;
        decode(value)
    }

    /// Same, but hands back the raw JSON — for endpoints the SDK does not model.
    pub(crate) async fn send_value<B>(
        &self,
        method: Method,
        path: &str,
        query: Option<Query>,
        body: Option<&B>,
    ) -> Result<serde_json::Value>
    where
        B: Serialize + ?Sized,
    {
        let url = self.url(path)?;
        let mut attempt = 0;

        loop {
            attempt += 1;

            let mut request = self
                .inner
                .request(method.clone(), url.clone())
                .header("X-API-Key", &self.api_key)
                .header("Accept", "application/json")
                .header("User-Agent", USER_AGENT);
            if let Some(query) = &query {
                request = request.query(query);
            }
            if let Some(body) = body {
                request = request.json(body);
            }

            let response = request.send().await?;
            let status = response.status();

            if is_retryable(status) && attempt < self.max_retries {
                let wait = retry_after(&response).unwrap_or(Duration::from_secs(1));
                tokio::time::sleep(wait.min(MAX_RETRY_WAIT)).await;
                continue;
            }

            return finish(response).await;
        }
    }

    /// Send an already-built request (multipart, say) through the same decoding.
    #[cfg(feature = "multipart")]
    pub(crate) async fn send_request(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<serde_json::Value> {
        finish(request.send().await?).await
    }

    /// A request builder carrying the auth headers but nothing else.
    #[cfg(feature = "multipart")]
    pub(crate) fn raw_request(
        &self,
        method: Method,
        path: &str,
    ) -> Result<reqwest::RequestBuilder> {
        Ok(self
            .inner
            .request(method, self.url(path)?)
            .header("X-API-Key", &self.api_key)
            .header("Accept", "application/json")
            .header("User-Agent", USER_AGENT))
    }
}

/// Turn a finished response into JSON, or into an [`ApiError`].
async fn finish(response: reqwest::Response) -> Result<serde_json::Value> {
    let status = response.status();
    let wait = retry_after(&response).map(|d| d.as_secs_f64());
    let text = response.text().await?;

    let json: Option<serde_json::Value> = if text.trim().is_empty() {
        None
    } else {
        serde_json::from_str(&text).ok()
    };

    if status.is_success() {
        return match json {
            Some(value) => Ok(value),
            // 204s and empty 200s are real answers; give the caller a null.
            None if text.trim().is_empty() => Ok(serde_json::Value::Null),
            None => Err(Error::Decode {
                source: serde_json::from_str::<serde_json::Value>(&text).unwrap_err(),
                body: text,
            }),
        };
    }

    let json =
        json.or_else(|| (!text.trim().is_empty()).then(|| serde_json::json!({ "message": text })));
    Err(Error::Api(ApiError::from_body(status.as_u16(), json, wait)))
}

pub(crate) fn decode<T: DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    serde_json::from_value(value.clone()).map_err(|source| Error::Decode {
        source,
        body: value.to_string(),
    })
}

/// `Retry-After` is either delta-seconds or an HTTP date; we only read the former,
/// falling back to the API's own `retry_after` body field elsewhere.
fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    let raw = response.headers().get("retry-after")?.to_str().ok()?;
    let seconds: f64 = raw.trim().parse().ok()?;
    (seconds.is_finite() && seconds >= 0.0).then(|| Duration::from_secs_f64(seconds))
}

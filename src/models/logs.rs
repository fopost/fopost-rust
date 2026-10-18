//! Request logs — what the API saw from your keys.

use serde::Deserialize;

/// One request the API handled.
#[derive(Debug, Clone, Deserialize)]
pub struct RequestLog {
    pub id: i64,
    /// Workspace the request ran against; `None` for account-level requests.
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    /// `jwt` or `api_key`.
    #[serde(default)]
    pub auth_method: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    /// Route pattern, e.g. `/v1/posts/{id}`.
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub status_code: Option<i64>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub platform: Option<String>,
    /// Error text for failed requests.
    #[serde(default)]
    pub message: Option<String>,
}

/// Requests and errors in one time bucket.
#[derive(Debug, Clone, Deserialize)]
pub struct RequestLogBucket {
    #[serde(default)]
    pub at: Option<String>,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub errors: i64,
}

/// Totals over the window the page covers.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RequestLogSummary {
    #[serde(default)]
    pub since: Option<String>,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub errors: i64,
    #[serde(default)]
    pub buckets: Vec<RequestLogBucket>,
}

/// The cursor for the next page; `next_cursor` is `None` at the end.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RequestLogMeta {
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_more: bool,
}

/// One page of request logs, newest first, with the window's totals.
#[derive(Debug, Clone, Deserialize)]
pub struct RequestLogPage {
    #[serde(default)]
    pub data: Vec<RequestLog>,
    #[serde(default)]
    pub meta: RequestLogMeta,
    #[serde(default)]
    pub summary: RequestLogSummary,
}

/// How to narrow the log. Leave `workspace_id` unset to read every workspace
/// the key can reach.
#[derive(Debug, Clone, Default)]
pub struct ListLogs<'a> {
    pub workspace_id: Option<&'a str>,
    /// 1 to 30, 7 by default.
    pub days: Option<u32>,
    /// `success` or `failed`.
    pub status: Option<&'a str>,
    pub method: Option<&'a str>,
    pub status_code: Option<u16>,
    pub platform: Option<&'a str>,
    /// Exact route pattern, e.g. `/v1/posts/{id}`.
    pub route: Option<&'a str>,
    /// Substring match against path and message.
    pub q: Option<&'a str>,
    /// 1 to 200, 50 by default.
    pub limit: Option<u32>,
    /// `meta.next_cursor` from the previous page.
    pub before: Option<&'a str>,
}

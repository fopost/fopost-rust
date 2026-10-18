//! `client.logs()` — the requests the API handled for your keys.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, HttpClient, Query};
use crate::models::{ListLogs, RequestLogPage};

/// Request logs.
#[derive(Debug, Clone)]
pub struct Logs<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Logs<'_> {
    /// Requests newest first, with the window's totals. Page by passing
    /// `meta.next_cursor` back as `before`.
    pub async fn list(&self, params: &ListLogs<'_>) -> Result<RequestLogPage> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id);
        push_opt(&mut query, "days", params.days);
        push_opt(&mut query, "status", params.status);
        push_opt(&mut query, "method", params.method);
        push_opt(&mut query, "status_code", params.status_code);
        push_opt(&mut query, "platform", params.platform);
        push_opt(&mut query, "route", params.route);
        push_opt(&mut query, "q", params.q);
        push_opt(&mut query, "limit", params.limit);
        push_opt(&mut query, "before", params.before);
        // The response carries meta and summary beside data, so it is read whole.
        self.http
            .send::<RequestLogPage, ()>(Method::GET, "/logs", Some(query), None)
            .await
    }
}

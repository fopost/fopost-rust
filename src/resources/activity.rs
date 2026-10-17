//! `client.activity()` — what happened in a workspace, and the audit log.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, HttpClient, Query};
use crate::models::{ActivityPage, ListActivity};

/// Activity.
#[derive(Debug, Clone)]
pub struct Activity<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Activity<'_> {
    /// Activity newest first.
    ///
    /// `kind: Some("security")` is the audit log: members joining, leaving or
    /// changing role and access, and changes to two-step verification,
    /// passkeys, single sign-on and signed-in devices. Those rows are
    /// append-only and never expire.
    pub async fn list(&self, params: &ListActivity<'_>) -> Result<ActivityPage> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id);
        push_opt(&mut query, "kind", params.kind);
        push_opt(&mut query, "from", params.from);
        push_opt(&mut query, "to", params.to);
        push_opt(&mut query, "cursor", params.cursor);
        push_opt(&mut query, "limit", params.limit);
        // The response carries meta beside data, so it is read whole rather
        // than unwrapped down to the list.
        self.http
            .send::<ActivityPage, ()>(Method::GET, "/activity", Some(query), None)
            .await
    }
}

//! Activity — what happened in a workspace, including the security audit log.

use serde::Deserialize;

/// Who did something: `user`, `api_key`, `agent` or `system`.
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityActor {
    #[serde(rename = "type")]
    pub actor_type: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// One thing that happened. A `security` kind is an audit row.
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityEvent {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub kind: String,
    #[serde(default)]
    pub ref_type: Option<String>,
    #[serde(default)]
    pub ref_id: Option<String>,
    pub summary: String,
    pub actor: ActivityActor,
    pub time: String,
}

/// The cursor for the next page; `None` at the end of the list.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ActivityMeta {
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// One page of activity, newest first.
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityPage {
    #[serde(default)]
    pub data: Vec<ActivityEvent>,
    #[serde(default)]
    pub meta: ActivityMeta,
}

/// How to narrow the log. Leave `workspace_id` unset to read every workspace
/// the key can reach.
#[derive(Debug, Clone, Default)]
pub struct ListActivity<'a> {
    pub workspace_id: Option<&'a str>,
    /// `security` is the audit log; its rows are append-only and never expire.
    pub kind: Option<&'a str>,
    /// ISO 8601. Only events at or after this time.
    pub from: Option<&'a str>,
    /// ISO 8601. Only events at or before this time.
    pub to: Option<&'a str>,
    /// `meta.next_cursor` from the previous page.
    pub cursor: Option<&'a str>,
    pub limit: Option<u32>,
}

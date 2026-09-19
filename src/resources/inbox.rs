//! `client.inbox()` — comments, mentions and direct messages on connected accounts.
//!
//! Every call needs the `inbox` scope.

use reqwest::Method;
use serde::Serialize;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    ApprovalDecision, InboxAccount, InboxApproval, InboxConversation, InboxItem, InboxPage,
    InboxPlatform, InboxRefreshResult, InboxReplyResult, InboxThread, ListInbox,
    ListInboxConversations, ListInboxThreads, MarkThreadRead, UpdateInboxItem,
};

/// The inbox.
#[derive(Debug, Clone)]
pub struct Inbox<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Inbox<'_> {
    /// One page of items, newest first unless `sort` says otherwise.
    pub async fn list(&self, params: &ListInbox) -> Result<InboxPage<InboxItem>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "type", params.item_type.as_ref());
        push_opt(&mut query, "state", params.state.as_ref());
        push_opt(&mut query, "platform", params.platform.as_ref());
        push_opt(&mut query, "account_id", params.account_id.as_ref());
        push_opt(&mut query, "post_id", params.post_id.as_ref());
        push_opt(
            &mut query,
            "post_external_id",
            params.post_external_id.as_ref(),
        );
        push_opt(
            &mut query,
            "conversation_id",
            params.conversation_id.as_ref(),
        );
        push_opt(&mut query, "direction", params.direction.as_ref());
        push_opt(&mut query, "q", params.q.as_ref());
        push_opt(&mut query, "sort", params.sort.as_ref());
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        self.http
            .send::<InboxPage<InboxItem>, ()>(Method::GET, "/inbox", Some(query), None)
            .await
    }

    /// One row per post with comments; `kind` of `Mentions` for posts we were tagged in.
    pub async fn threads(&self, params: &ListInboxThreads) -> Result<InboxPage<InboxThread>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "kind", params.kind.as_ref());
        push_opt(&mut query, "platform", params.platform.as_ref());
        push_opt(&mut query, "account_id", params.account_id.as_ref());
        push_opt(&mut query, "state", params.state.as_ref());
        push_opt(&mut query, "q", params.q.as_ref());
        push_opt(&mut query, "sort", params.sort.as_ref());
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        self.http
            .send::<InboxPage<InboxThread>, ()>(Method::GET, "/inbox/posts", Some(query), None)
            .await
    }

    /// One row per DM thread, latest first.
    pub async fn conversations(
        &self,
        params: &ListInboxConversations,
    ) -> Result<InboxPage<InboxConversation>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "platform", params.platform.as_ref());
        push_opt(&mut query, "account_id", params.account_id.as_ref());
        push_opt(&mut query, "state", params.state.as_ref());
        push_opt(&mut query, "q", params.q.as_ref());
        push_opt(&mut query, "sort", params.sort.as_ref());
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        self.http
            .send::<InboxPage<InboxConversation>, ()>(
                Method::GET,
                "/inbox/conversations",
                Some(query),
                None,
            )
            .await
    }

    /// How many items are unread.
    pub async fn unread_count(&self, workspace_id: Option<&str>) -> Result<u64> {
        #[derive(serde::Deserialize)]
        struct Count {
            #[serde(default)]
            count: u64,
        }
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", workspace_id);
        let body: Count = self
            .http
            .send::<_, ()>(Method::GET, "/inbox/unread-count", Some(query), None)
            .await?;
        Ok(body.count)
    }

    /// Every active account, flagged with whether comments and DMs can be read for it.
    pub async fn accounts(&self, workspace_id: Option<&str>) -> Result<Vec<InboxAccount>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", workspace_id);
        let body: Envelope<Vec<InboxAccount>> = self
            .http
            .send::<_, ()>(Method::GET, "/inbox/accounts", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// What the inbox can read on each platform. Not tenant data.
    pub async fn platforms(&self) -> Result<Vec<InboxPlatform>> {
        let body: Envelope<Vec<InboxPlatform>> = self
            .http
            .send::<_, ()>(Method::GET, "/inbox/platforms", None, None)
            .await?;
        Ok(body.data)
    }

    /// Mark a whole comment thread or DM thread read. Returns how many items changed.
    pub async fn mark_thread_read(&self, thread: &MarkThreadRead) -> Result<u64> {
        #[derive(serde::Deserialize)]
        struct Updated {
            #[serde(default)]
            updated: u64,
        }
        let body: Envelope<Updated> = self
            .http
            .send(Method::POST, "/inbox/read", None, Some(thread))
            .await?;
        Ok(body.data.updated)
    }

    /// Poll every inbox-capable account in the workspace now.
    pub async fn refresh(&self, workspace_id: &str) -> Result<InboxRefreshResult> {
        #[derive(Serialize)]
        struct Refresh<'a> {
            workspace_id: &'a str,
        }
        let body: Envelope<InboxRefreshResult> = self
            .http
            .send(
                Method::POST,
                "/inbox/refresh",
                None,
                Some(&Refresh { workspace_id }),
            )
            .await?;
        Ok(body.data)
    }

    /// Mark an item unread, read, resolved or snoozed.
    pub async fn update(&self, id: &str, changes: &UpdateInboxItem) -> Result<InboxItem> {
        let body: Envelope<InboxItem> = self
            .http
            .send(Method::PATCH, &format!("/inbox/{id}"), None, Some(changes))
            .await?;
        Ok(body.data)
    }

    /// Send a reply on the platform as the connected account.
    pub async fn reply(&self, id: &str, text: &str) -> Result<InboxReplyResult> {
        #[derive(Serialize)]
        struct Reply<'a> {
            text: &'a str,
        }
        let body: Envelope<InboxReplyResult> = self
            .http
            .send(
                Method::POST,
                &format!("/inbox/{id}/reply"),
                None,
                Some(&Reply { text }),
            )
            .await?;
        Ok(body.data)
    }

    /// Hide a comment on the platform.
    pub async fn hide(&self, id: &str) -> Result<InboxItem> {
        let body: Envelope<InboxItem> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/inbox/{id}/hide"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Show a hidden comment again.
    pub async fn unhide(&self, id: &str) -> Result<InboxItem> {
        let body: Envelope<InboxItem> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/inbox/{id}/unhide"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Delete a comment on the platform.
    pub async fn delete(&self, id: &str) -> Result<bool> {
        #[derive(serde::Deserialize)]
        struct Deleted {
            #[serde(default)]
            deleted: bool,
        }
        let body: Envelope<Deleted> = self
            .http
            .send::<_, ()>(Method::DELETE, &format!("/inbox/{id}"), None, None)
            .await?;
        Ok(body.data.deleted)
    }

    /// Replies an automation or the agent drafted that a person still has to send.
    pub async fn approvals(&self, workspace_id: Option<&str>) -> Result<Vec<InboxApproval>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", workspace_id);
        let body: Envelope<Vec<InboxApproval>> = self
            .http
            .send::<_, ()>(Method::GET, "/inbox/approvals", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Send the draft, or `text` in its place.
    pub async fn approve_reply(&self, id: u64, text: Option<&str>) -> Result<ApprovalDecision> {
        #[derive(Serialize)]
        struct Decide<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            text: Option<&'a str>,
        }
        let body: Envelope<ApprovalDecision> = self
            .http
            .send(
                Method::POST,
                &format!("/inbox/approvals/{id}/approve"),
                None,
                Some(&Decide { text }),
            )
            .await?;
        Ok(body.data)
    }

    /// Discard a drafted reply.
    pub async fn reject_reply(&self, id: u64) -> Result<ApprovalDecision> {
        let body: Envelope<ApprovalDecision> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/inbox/approvals/{id}/reject"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }
}

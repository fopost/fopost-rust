//! `client.inbox()` — comments, mentions and direct messages on connected accounts.
//!
//! Every call needs the `inbox` scope. The calls that act on the platform as the account
//! (like, pin, react, edit, start a conversation, typing, a reply with media or quick
//! replies, deleting our own reply) also need the `publish` scope.

use reqwest::Method;
use serde::Serialize;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    ApprovalDecision, InboxAccount, InboxApproval, InboxConversation, InboxConversationStarted,
    InboxHandover, InboxItem, InboxPage, InboxPlatform, InboxRefreshResult, InboxReply,
    InboxReplyResult, InboxThread, ListInbox, ListInboxConversations, ListInboxThreads,
    MarkThreadRead, StartInboxConversation, UpdateInboxItem,
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

    /// Edit our own comment on the platform. Only where `can_edit` is true; also needs
    /// `publish`.
    pub async fn edit_comment(&self, id: &str, text: &str) -> Result<InboxItem> {
        #[derive(Serialize)]
        struct Edit<'a> {
            text: &'a str,
        }
        let body: Envelope<InboxItem> = self
            .http
            .send(
                Method::PATCH,
                &format!("/inbox/{id}"),
                None,
                Some(&Edit { text }),
            )
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

    /// Send a reply carrying media or quick replies; either also needs `publish`.
    pub async fn reply_with(&self, id: &str, reply: &InboxReply) -> Result<InboxReplyResult> {
        let body: Envelope<InboxReplyResult> = self
            .http
            .send(
                Method::POST,
                &format!("/inbox/{id}/reply"),
                None,
                Some(reply),
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

    /// Delete a comment on the platform, whether someone else wrote it or it is our own
    /// reply. Deleting our own reply also needs `publish`.
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

    /// Like an item: a like, an upvote on Reddit, a favourite on Mastodon. Only where
    /// `can_like` is true; also needs `publish`.
    pub async fn like(&self, id: &str) -> Result<InboxItem> {
        let body: Envelope<InboxItem> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/inbox/{id}/like"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Remove our like. Only where `can_like` is true; also needs `publish`.
    pub async fn unlike(&self, id: &str) -> Result<InboxItem> {
        let body: Envelope<InboxItem> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/inbox/{id}/unlike"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Pin our own comment. Only where `can_pin` is true; also needs `publish`.
    pub async fn pin(&self, id: &str) -> Result<InboxItem> {
        let body: Envelope<InboxItem> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/inbox/{id}/pin"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Unpin our own comment. Only where `can_pin` is true; also needs `publish`.
    pub async fn unpin(&self, id: &str) -> Result<InboxItem> {
        let body: Envelope<InboxItem> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/inbox/{id}/unpin"), None, None)
            .await?;
        Ok(body.data)
    }

    /// React to a message with an emoji, or pass `None` to remove ours. Only where
    /// `can_react` is true; also needs `publish`.
    pub async fn react(&self, id: &str, reaction: Option<&str>) -> Result<InboxItem> {
        #[derive(Serialize)]
        struct React<'a> {
            reaction: Option<&'a str>,
        }
        let body: Envelope<InboxItem> = self
            .http
            .send(
                Method::POST,
                &format!("/inbox/{id}/react"),
                None,
                Some(&React { reaction }),
            )
            .await?;
        Ok(body.data)
    }

    /// Open a DM, by handle or as a private reply to a comment. Also needs `publish`.
    pub async fn start_conversation(
        &self,
        conversation: &StartInboxConversation,
    ) -> Result<InboxConversationStarted> {
        let body: Envelope<InboxConversationStarted> = self
            .http
            .send(
                Method::POST,
                "/inbox/conversations",
                None,
                Some(conversation),
            )
            .await?;
        Ok(body.data)
    }

    /// Show (`on`) or clear the typing indicator in a DM thread. Returns whether it is now
    /// shown. Also needs `publish`.
    pub async fn set_typing(
        &self,
        conversation_id: &str,
        account_id: &str,
        on: bool,
    ) -> Result<bool> {
        #[derive(Serialize)]
        struct Typing<'a> {
            account_id: &'a str,
            on: bool,
        }
        #[derive(serde::Deserialize)]
        struct State {
            #[serde(default)]
            typing: bool,
        }
        let body: Envelope<State> = self
            .http
            .send(
                Method::POST,
                &format!("/inbox/conversations/{conversation_id}/typing"),
                None,
                Some(&Typing { account_id, on }),
            )
            .await?;
        Ok(body.data.typing)
    }

    /// Pass a Messenger thread to another Meta app, or take it back when `app_id` is
    /// `None`. Also needs the `publish` scope.
    pub async fn handover(
        &self,
        conversation_id: &str,
        account_id: &str,
        app_id: Option<&str>,
        metadata: Option<&str>,
    ) -> Result<InboxHandover> {
        #[derive(Serialize)]
        struct Handover<'a> {
            account_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            app_id: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            metadata: Option<&'a str>,
        }
        let body: Envelope<InboxHandover> = self
            .http
            .send(
                Method::POST,
                &format!("/inbox/conversations/{conversation_id}/handover"),
                None,
                Some(&Handover {
                    account_id,
                    app_id,
                    metadata,
                }),
            )
            .await?;
        Ok(body.data)
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

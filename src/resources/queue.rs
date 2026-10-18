//! `client.queue()` — the posting queue's weekly slots.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    CreateQueueSlot, QueueNext, QueueSlot, QueueSlotDeleted, QueueSlotUpdated, UpdateQueueSlot,
};

/// The posting queue. A post joins it with [`CreatePost::queue`](crate::models::CreatePost::queue)
/// or [`UpdatePost::queue`](crate::models::UpdatePost::queue).
#[derive(Debug, Clone)]
pub struct Queue<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Queue<'_> {
    /// Every slot in the workspace.
    pub async fn slots(&self, workspace_id: &str) -> Result<Vec<QueueSlot>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", Some(workspace_id));
        let body: Envelope<Vec<QueueSlot>> = self
            .http
            .send::<_, ()>(Method::GET, "/queue/slots", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Add a weekly slot.
    pub async fn create_slot(&self, slot: &CreateQueueSlot) -> Result<QueueSlot> {
        self.http
            .send(Method::POST, "/queue/slots", None, Some(slot))
            .await
    }

    /// Change a slot. Queued posts that sat in it are moved, and the answer
    /// says where.
    pub async fn update_slot(
        &self,
        id: &str,
        changes: &UpdateQueueSlot,
    ) -> Result<QueueSlotUpdated> {
        self.http
            .send(
                Method::PUT,
                &format!("/queue/slots/{id}"),
                None,
                Some(changes),
            )
            .await
    }

    /// Remove a slot. Queued posts that sat in it are moved to the next free one.
    pub async fn delete_slot(&self, id: &str) -> Result<QueueSlotDeleted> {
        self.http
            .send::<QueueSlotDeleted, ()>(Method::DELETE, &format!("/queue/slots/{id}"), None, None)
            .await
    }

    /// The time a post added to the queue now would get, or `None` when no
    /// slot applies. Pass `account_id` to ask for one account.
    pub async fn next(
        &self,
        workspace_id: &str,
        account_id: Option<&str>,
    ) -> Result<Option<String>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", Some(workspace_id));
        push_opt(&mut query, "account_id", account_id);
        let body: QueueNext = self
            .http
            .send::<_, ()>(Method::GET, "/queue/next", Some(query), None)
            .await?;
        Ok(body.next)
    }

    /// The next free times in order, as a post-by-post preview of the queue.
    /// `count` is 5 by default and 20 at most.
    pub async fn preview(
        &self,
        workspace_id: &str,
        account_id: Option<&str>,
        count: Option<u32>,
    ) -> Result<Vec<String>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", Some(workspace_id));
        push_opt(&mut query, "account_id", account_id);
        push_opt(&mut query, "count", count);
        let body: Envelope<Vec<String>> = self
            .http
            .send::<_, ()>(Method::GET, "/queue/preview", Some(query), None)
            .await?;
        Ok(body.data)
    }
}

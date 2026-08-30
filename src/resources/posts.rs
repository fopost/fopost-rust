//! `client.posts()` — create, schedule, publish, and inspect posts.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    BulkImportRollback, BulkPostAction, BulkResult, CancelResult, CreatePost, Delivery,
    DuplicatedPost, ListPosts, Message, Page, Post, PostAnalytics, PreflightResult, PublishOptions,
    PublishOutcome, PublishRun, RetryOptions, RetryResult, UpdatePost,
};

/// Posts and everything that happens to them.
#[derive(Debug, Clone)]
pub struct Posts<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Posts<'_> {
    /// One page of posts, newest first.
    pub async fn list(&self, params: &ListPosts) -> Result<Page<Post>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "status", params.status.as_ref());
        push_opt(&mut query, "search", params.search.as_ref());
        push_opt(&mut query, "platform", params.platform.as_ref());
        push_opt(&mut query, "label", params.label.as_ref());
        push_opt(&mut query, "account_id", params.account_id.as_ref());
        push_opt(&mut query, "date", params.date.as_ref());
        push_opt(&mut query, "from", params.from.as_ref());
        push_opt(&mut query, "to", params.to.as_ref());
        push_opt(&mut query, "sort", params.sort.as_ref());
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);

        self.http
            .send::<Page<Post>, ()>(Method::GET, "/posts", Some(query), None)
            .await
    }

    /// Every matching post, walking the pages for you.
    ///
    /// Fetches until the API says there is nothing left, so bound it with a
    /// filter on anything but a small workspace.
    pub async fn list_all(&self, params: &ListPosts) -> Result<Vec<Post>> {
        let mut params = params.clone();
        let mut page_number = params.page.unwrap_or(1);
        let mut out = Vec::new();

        loop {
            params.page = Some(page_number);
            let page = self.list(&params).await?;
            let has_next = page.has_next() && !page.is_empty();
            out.extend(page.items);
            if !has_next {
                return Ok(out);
            }
            page_number += 1;
        }
    }

    /// One post, with its content, targets, and per-account delivery state.
    pub async fn get(&self, id: &str) -> Result<Post> {
        self.http
            .send::<Post, ()>(Method::GET, &format!("/posts/{id}"), None, None)
            .await
    }

    /// Create a draft, or a scheduled post when the body carries a schedule.
    ///
    /// To send something out now, create it and call [`Posts::publish`].
    pub async fn create(&self, post: &CreatePost) -> Result<Post> {
        self.http
            .send(Method::POST, "/posts", None, Some(post))
            .await
    }

    /// Partial update — only the fields set on the body are sent.
    pub async fn update(&self, id: &str, changes: &UpdatePost) -> Result<Post> {
        self.http
            .send(Method::PUT, &format!("/posts/{id}"), None, Some(changes))
            .await
    }

    /// Delete a post. Anything already published stays on the platform.
    pub async fn delete(&self, id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, &format!("/posts/{id}"), None, None)
            .await
    }

    /// Copy a post into a fresh draft in the same workspace.
    pub async fn duplicate(&self, id: &str) -> Result<DuplicatedPost> {
        let body: Envelope<DuplicatedPost> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/posts/{id}/duplicate"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Queue the post for delivery to its accounts.
    ///
    /// Pass [`PublishOptions::dry_run`] to get the same validation without
    /// anything leaving the building.
    pub async fn publish(&self, id: &str, options: &PublishOptions) -> Result<PublishOutcome> {
        let body: Envelope<PublishOutcome> = self
            .http
            .send(
                Method::POST,
                &format!("/posts/{id}/publish"),
                None,
                Some(options),
            )
            .await?;
        Ok(body.data)
    }

    /// Retry the deliveries that failed, leaving the successful ones alone.
    pub async fn retry(&self, id: &str, options: &RetryOptions) -> Result<RetryResult> {
        let body: Envelope<RetryResult> = self
            .http
            .send(
                Method::POST,
                &format!("/posts/{id}/retry"),
                None,
                Some(options),
            )
            .await?;
        Ok(body.data)
    }

    /// Cancel the deliveries that have not gone out yet. `None` cancels them all.
    pub async fn cancel(&self, id: &str, account_ids: Option<&[String]>) -> Result<CancelResult> {
        let payload = serde_json::json!({ "accountIds": account_ids });
        let body: Envelope<CancelResult> = self
            .http
            .send(
                Method::POST,
                &format!("/posts/{id}/cancel"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Per-account blockers and advisory content signals, without publishing.
    pub async fn preflight(&self, id: &str) -> Result<PreflightResult> {
        let body: Envelope<PreflightResult> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/posts/{id}/preflight"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Every delivery of this post, one per targeted account.
    pub async fn deliveries(&self, id: &str) -> Result<Vec<Delivery>> {
        let body: Envelope<Vec<Delivery>> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/posts/{id}/deliveries"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Each pass of publishing this post, newest first.
    pub async fn publish_runs(&self, id: &str) -> Result<Vec<PublishRun>> {
        let body: Envelope<Vec<PublishRun>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/posts/{id}/publish-runs"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// How the post did, in total and per platform.
    pub async fn analytics(&self, id: &str) -> Result<PostAnalytics> {
        let body: Envelope<PostAnalytics> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/posts/{id}/analytics"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Reschedule, relabel, or delete a selection in one transaction. A
    /// selection containing one already-published post changes nothing at all.
    pub async fn bulk(&self, action: &BulkPostAction) -> Result<BulkResult> {
        self.http
            .send(Method::POST, "/posts/bulk", None, Some(action))
            .await
    }

    /// Roll a committed bulk import back, deleting every post it created.
    pub async fn rollback_import(&self, batch_id: &str) -> Result<BulkImportRollback> {
        self.http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/posts/bulk-import/{batch_id}"),
                None,
                None,
            )
            .await
    }
}

#[cfg(feature = "multipart")]
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
impl Posts<'_> {
    /// Dry-run a bulk-import CSV: what each row would create, and why a row
    /// would not. Nothing is written.
    pub async fn validate_import(
        &self,
        workspace_id: &str,
        csv: crate::models::MediaUpload,
    ) -> Result<crate::models::BulkImportValidation> {
        let value = self
            .send_csv("/posts/bulk-import/validate", workspace_id, csv)
            .await?;
        crate::http::decode(value)
    }

    /// Commit a bulk-import CSV, creating a post per row. Keep the returned
    /// `batch_id` — it is what [`Posts::rollback_import`] takes.
    pub async fn commit_import(
        &self,
        workspace_id: &str,
        csv: crate::models::MediaUpload,
    ) -> Result<crate::models::BulkImportCommit> {
        let value = self
            .send_csv("/posts/bulk-import/commit", workspace_id, csv)
            .await?;
        crate::http::decode(value)
    }

    async fn send_csv(
        &self,
        path: &str,
        workspace_id: &str,
        csv: crate::models::MediaUpload,
    ) -> Result<serde_json::Value> {
        let part = reqwest::multipart::Part::bytes(csv.bytes)
            .file_name(csv.filename)
            .mime_str(&csv.mime_type)
            .map_err(crate::error::Error::Transport)?;
        let form = reqwest::multipart::Form::new()
            .text("workspace_id", workspace_id.to_string())
            .part("file", part);

        self.http
            .send_request(self.http.raw_request(Method::POST, path)?.multipart(form))
            .await
    }
}

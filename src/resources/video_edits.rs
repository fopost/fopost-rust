//! `client.video_edits()` — background jobs that edit a library video.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{CreateVideoEdit, ListVideoEdits, VideoEdit, VideoEditOpInfo};

/// Video edits. Every call needs the `posts` scope.
#[derive(Debug, Clone)]
pub struct VideoEdits<'a> {
    pub(crate) http: &'a HttpClient,
}

impl VideoEdits<'_> {
    /// Every operation, whether this deployment can run it, and whether it uses AI.
    pub async fn ops(&self) -> Result<Vec<VideoEditOpInfo>> {
        let body: Envelope<Vec<VideoEditOpInfo>> = self
            .http
            .send::<_, ()>(Method::GET, "/video-edits/ops", None, None)
            .await?;
        Ok(body.data)
    }

    /// Recent edits, newest first. `limit` is 20 by default.
    pub async fn list(&self, params: &ListVideoEdits<'_>) -> Result<Vec<VideoEdit>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id);
        push_opt(&mut query, "media_id", params.media_id);
        push_opt(&mut query, "limit", params.limit);
        let body: Envelope<Vec<VideoEdit>> = self
            .http
            .send::<_, ()>(Method::GET, "/video-edits", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Start an edit. It runs in the background; read it back with [`Self::get`]
    /// until `status` is `succeeded` or `failed`. Operations that use AI spend
    /// AI credits, and a `402` means the balance ran out.
    pub async fn create(&self, input: &CreateVideoEdit) -> Result<VideoEdit> {
        self.http
            .send(Method::POST, "/video-edits", None, Some(input))
            .await
    }

    /// One edit and its progress.
    pub async fn get(&self, id: &str, workspace_id: Option<&str>) -> Result<VideoEdit> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", workspace_id);
        self.http
            .send::<_, ()>(
                Method::GET,
                &format!("/video-edits/{id}"),
                Some(query),
                None,
            )
            .await
    }
}

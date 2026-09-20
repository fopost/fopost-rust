//! `client.knowledge()` — what the workspace has told FoPost about itself.
//!
//! A source is an FAQ, a note, a page on your own site, or a plain-text/CSV
//! item from the media library. Retrieval over these is what grounds a drafted
//! inbox reply in your own answers instead of an invented one. Needs the
//! `inbox` scope.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    CreateKnowledgeSource, KnowledgeDeleted, KnowledgeMatch, KnowledgeSource, KnowledgeSyncResult,
    UpdateKnowledgeSource,
};

/// The knowledge base.
#[derive(Debug, Clone)]
pub struct Knowledge<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Knowledge<'_> {
    /// The workspace's knowledge sources. Only a `ready` one is searched.
    pub async fn list(&self, workspace_id: Option<&str>) -> Result<Vec<KnowledgeSource>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", workspace_id);
        let body: Envelope<Vec<KnowledgeSource>> = self
            .http
            .send::<_, ()>(Method::GET, "/knowledge/sources", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Add a source and queue it for indexing, so it comes back `pending`.
    pub async fn create(&self, source: &CreateKnowledgeSource) -> Result<KnowledgeSource> {
        let body: Envelope<KnowledgeSource> = self
            .http
            .send(Method::POST, "/knowledge/sources", None, Some(source))
            .await?;
        Ok(body.data)
    }

    /// Edit a source. Changing the content or the URL returns it to `pending`
    /// and re-indexes it.
    pub async fn update(
        &self,
        id: &str,
        changes: &UpdateKnowledgeSource,
    ) -> Result<KnowledgeSource> {
        let body: Envelope<KnowledgeSource> = self
            .http
            .send(
                Method::PATCH,
                &format!("/knowledge/sources/{id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove a source and every passage indexed from it.
    pub async fn delete(&self, id: &str) -> Result<KnowledgeDeleted> {
        let body: Envelope<KnowledgeDeleted> = self
            .http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/knowledge/sources/{id}"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Read the source again — a `url` source is re-fetched. Returns once the
    /// re-index is queued, not once it has finished.
    pub async fn sync(&self, id: &str) -> Result<KnowledgeSyncResult> {
        let body: Envelope<KnowledgeSyncResult> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/knowledge/sources/{id}/sync"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The passages closest to a question, best first. An empty vector is the
    /// honest answer when nothing stored answers it. `top_k` defaults to 5 and
    /// caps at 20.
    pub async fn search(
        &self,
        q: &str,
        top_k: Option<u32>,
        brand_voice_id: Option<&str>,
        workspace_id: Option<&str>,
    ) -> Result<Vec<KnowledgeMatch>> {
        let mut query: Query = vec![("q", q.to_string())];
        push_opt(&mut query, "top_k", top_k);
        push_opt(&mut query, "brand_voice_id", brand_voice_id);
        push_opt(&mut query, "workspace_id", workspace_id);
        let body: Envelope<Vec<KnowledgeMatch>> = self
            .http
            .send::<_, ()>(Method::GET, "/knowledge/search", Some(query), None)
            .await?;
        Ok(body.data)
    }
}

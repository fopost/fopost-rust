//! `client.designs()` — editable designs and what they export to.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    BulkCreateDesigns, CreateDesign, Design, DesignDeleted, DesignExport, DesignFromPost,
    DesignPost, DesignSaved, DesignVersion, ExportDesign, ListDesigns, SlideOutline, TextToSlides,
    UpdateDesign,
};

/// Designs. Every call needs the `posts` scope.
#[derive(Debug, Clone)]
pub struct Designs<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Designs<'_> {
    /// A workspace's designs, newest first.
    pub async fn list(&self, params: &ListDesigns<'_>) -> Result<Vec<Design>> {
        let mut query: Query = vec![("workspace_id", params.workspace_id.to_string())];
        push_opt(&mut query, "kind", params.kind.as_ref());
        push_opt(&mut query, "limit", params.limit);
        push_opt(&mut query, "offset", params.offset);
        let body: Envelope<Vec<Design>> = self
            .http
            .send::<_, ()>(Method::GET, "/designs", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Create a design. Without a document it starts from a blank page of its kind.
    pub async fn create(&self, input: &CreateDesign) -> Result<Design> {
        let body: Envelope<Design> = self
            .http
            .send(Method::POST, "/designs", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// One design, with its document.
    pub async fn get(&self, id: &str) -> Result<Design> {
        let body: Envelope<Design> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/designs/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Change a design. With `create_version`, the answer carries the saved version.
    pub async fn update(&self, id: &str, changes: &UpdateDesign) -> Result<DesignSaved> {
        self.http
            .send(
                Method::PATCH,
                &format!("/designs/{id}"),
                None,
                Some(changes),
            )
            .await
    }

    /// Delete a design.
    pub async fn delete(&self, id: &str) -> Result<DesignDeleted> {
        let body: Envelope<DesignDeleted> = self
            .http
            .send::<_, ()>(Method::DELETE, &format!("/designs/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Saved versions, newest first. `include_document` adds each version's document.
    pub async fn versions(&self, id: &str, include_document: bool) -> Result<Vec<DesignVersion>> {
        let mut query: Query = Vec::new();
        push_opt(
            &mut query,
            "include",
            include_document.then_some("document"),
        );
        let body: Envelope<Vec<DesignVersion>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/designs/{id}/versions"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Put a saved version back as the current document.
    pub async fn restore_version(&self, id: &str, version_id: &str) -> Result<DesignSaved> {
        self.http
            .send::<_, ()>(
                Method::POST,
                &format!("/designs/{id}/versions/{version_id}/restore"),
                None,
                None,
            )
            .await
    }

    /// Copy a design. `name` defaults to the original's name with a suffix.
    pub async fn duplicate(&self, id: &str, name: Option<&str>) -> Result<Design> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
        }
        let body: Envelope<Design> = self
            .http
            .send(
                Method::POST,
                &format!("/designs/{id}/duplicate"),
                None,
                Some(&Body { name }),
            )
            .await?;
        Ok(body.data)
    }

    /// Render a design into media library files a post can attach.
    pub async fn export(&self, id: &str, input: &ExportDesign) -> Result<DesignExport> {
        let body: Envelope<DesignExport> = self
            .http
            .send(
                Method::POST,
                &format!("/designs/{id}/export"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Create one design per row from this design, filling text layers by column.
    pub async fn bulk(&self, id: &str, input: &BulkCreateDesigns) -> Result<Vec<Design>> {
        let body: Envelope<Vec<Design>> = self
            .http
            .send(
                Method::POST,
                &format!("/designs/{id}/bulk"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// The posts that use this design's exported files.
    pub async fn posts(&self, id: &str) -> Result<Vec<DesignPost>> {
        let body: Envelope<Vec<DesignPost>> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/designs/{id}/posts"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Outline text or a post as carousel slides. Saves nothing and uses AI credits;
    /// a `402` means the balance ran out.
    pub async fn text_to_slides(&self, input: &TextToSlides) -> Result<SlideOutline> {
        let body: Envelope<SlideOutline> = self
            .http
            .send(Method::POST, "/designs/text-to-slides", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// Turn a post's text into a new design.
    pub async fn from_post(&self, input: &DesignFromPost) -> Result<Design> {
        let body: Envelope<Design> = self
            .http
            .send(Method::POST, "/designs/from-post", None, Some(input))
            .await?;
        Ok(body.data)
    }
}

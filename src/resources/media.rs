//! `client.media()` — the media library posts attach from.

use reqwest::Method;

use crate::error::Result;
use crate::http::{Envelope, HttpClient, Query};
use crate::models::MediaLibraryItem;

/// The media library.
#[derive(Debug, Clone)]
pub struct Media<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Media<'_> {
    /// Everything in one workspace's library, newest first.
    pub async fn list(&self, workspace_id: &str) -> Result<Vec<MediaLibraryItem>> {
        let query: Query = vec![("workspaceId", workspace_id.to_string())];
        let body: Envelope<Vec<MediaLibraryItem>> = self
            .http
            .send::<_, ()>(Method::GET, "/media", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Delete a file. Posts already published keep what the platform stored.
    pub async fn delete(&self, id: &str) -> Result<bool> {
        #[derive(serde::Deserialize)]
        struct Deleted {
            success: bool,
        }
        let body: Deleted = self
            .http
            .send::<_, ()>(Method::DELETE, &format!("/media/{id}"), None, None)
            .await?;
        Ok(body.success)
    }
}

#[cfg(feature = "multipart")]
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
impl Media<'_> {
    /// Upload files into a workspace's library. The urls that come back are
    /// what a post's [`crate::models::MediaItem`] points at.
    ///
    /// Needs the `multipart` feature, which is on by default.
    pub async fn upload(
        &self,
        workspace_id: &str,
        files: impl IntoIterator<Item = crate::models::MediaUpload>,
    ) -> Result<Vec<crate::models::UploadedMedia>> {
        let mut form =
            reqwest::multipart::Form::new().text("workspaceId", workspace_id.to_string());

        for file in files {
            let part = reqwest::multipart::Part::bytes(file.bytes)
                .file_name(file.filename)
                .mime_str(&file.mime_type)
                .map_err(crate::error::Error::Transport)?;
            form = form.part("files", part);
        }

        let value = self
            .http
            .send_request(
                self.http
                    .raw_request(Method::POST, "/media/upload")?
                    .multipart(form),
            )
            .await?;
        let body: Envelope<Vec<crate::models::UploadedMedia>> = crate::http::decode(value)?;
        Ok(body.data)
    }
}

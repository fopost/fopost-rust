//! `client.media()` — the media library posts attach from.

use reqwest::Method;

use crate::error::{ApiError, Error, Result};
use crate::http::{Envelope, HttpClient, Query};
use crate::models::{MediaLibraryItem, PresignUpload, PresignedUpload, UploadedMedia};

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

    /// Reserve a direct-upload slot. PUT the bytes to `upload_url` with the returned
    /// headers and a `Content-Length` equal to `size`, then call [`Self::complete`].
    pub async fn presign(&self, input: &PresignUpload) -> Result<PresignedUpload> {
        let body: Envelope<PresignedUpload> = self
            .http
            .send(Method::POST, "/media/presign", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// Turn a finished direct upload into a library file.
    pub async fn complete(&self, upload_id: &str) -> Result<UploadedMedia> {
        let body: Envelope<UploadedMedia> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/media/presign/{upload_id}/complete"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Upload one file straight to storage: presign, PUT the bytes, complete.
    /// Needs no `multipart` feature.
    pub async fn upload_direct(
        &self,
        workspace_id: &str,
        filename: &str,
        mime_type: &str,
        data: Vec<u8>,
    ) -> Result<UploadedMedia> {
        let presigned = self
            .presign(&PresignUpload::new(
                workspace_id,
                filename,
                mime_type,
                data.len() as u64,
            ))
            .await?;

        // The slot is authorised by its url, so the API key stays home.
        let mut request = self
            .http
            .inner()
            .request(Method::PUT, &presigned.upload_url);
        for (name, value) in &presigned.headers {
            request = request.header(name, value);
        }
        let response = request.body(data).send().await?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await?;
            let json = (!text.trim().is_empty()).then(|| serde_json::json!({ "message": text }));
            return Err(Error::Api(ApiError::from_body(status.as_u16(), json, None)));
        }

        self.complete(&presigned.upload_id).await
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

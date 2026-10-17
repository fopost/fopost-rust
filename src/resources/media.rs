//! `client.media()` — the media library posts attach from.

use reqwest::Method;

use crate::error::{ApiError, Error, Result};
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    FileSource, MediaLibraryItem, PresignUpload, PresignedUpload, UploadLimits, UploadedMedia,
};

/// What a multipart request carries per file when `limits` cannot be read.
pub const DEFAULT_MULTIPART_MAX_FILE_BYTES: u64 = 100 * 1024 * 1024;

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

    /// What this account may upload: the per-file ceiling by any route, the
    /// multipart ceiling, and the next tier more connected accounts would unlock.
    pub async fn limits(&self, workspace_id: Option<&str>) -> Result<UploadLimits> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspaceId", workspace_id);
        let body: Envelope<UploadLimits> = self
            .http
            .send::<_, ()>(Method::GET, "/media/limits", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Reserve a direct-upload slot for a file of up to 5 GB. PUT the bytes to
    /// `upload_url` with the returned headers and a `Content-Length` equal to
    /// `size`, then call [`Self::complete`].
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
        let input = PresignUpload::new(workspace_id, filename, mime_type, 0);
        self.upload_direct_with(input, FileSource::Bytes(data))
            .await
    }

    /// [`Self::upload_direct`] with every presign option. `input.size` is replaced by
    /// the source's real length, and a path is streamed rather than read into memory.
    pub async fn upload_direct_with(
        &self,
        mut input: PresignUpload,
        source: FileSource,
    ) -> Result<UploadedMedia> {
        input.size = source.len().await.map_err(|err| {
            Error::Config(format!("fopost: could not read {} — {err}", input.filename))
        })?;
        let presigned = self.presign(&input).await?;

        // The slot is authorised by its url, so the API key stays home.
        let mut request = self
            .http
            .inner()
            .request(Method::PUT, &presigned.upload_url)
            .header("Content-Length", input.size);
        for (name, value) in &presigned.headers {
            request = request.header(name, value);
        }
        let body = match source {
            FileSource::Bytes(bytes) => reqwest::Body::from(bytes),
            FileSource::Path(path) => {
                let file = tokio::fs::File::open(&path).await.map_err(|err| {
                    Error::Config(format!("fopost: could not open {} — {err}", path.display()))
                })?;
                reqwest::Body::wrap_stream(tokio_util::io::ReaderStream::new(file))
            }
        };
        let response = request.body(body).send().await?;
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

//! `client.fonts()` — the font library and a workspace's uploaded fonts.

use reqwest::Method;

use crate::error::Result;
use crate::http::{HttpClient, Query};
use crate::models::FontList;

/// Fonts designs and burned-in subtitles can use. Every call needs the `posts` scope.
#[derive(Debug, Clone)]
pub struct Fonts<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Fonts<'_> {
    /// The shipped library plus the fonts this workspace uploaded.
    pub async fn list(&self, workspace_id: &str) -> Result<FontList> {
        let query: Query = vec![("workspace_id", workspace_id.to_string())];
        self.http
            .send::<_, ()>(Method::GET, "/fonts", Some(query), None)
            .await
    }

    /// Remove an uploaded font.
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.http
            .send::<serde_json::Value, ()>(Method::DELETE, &format!("/fonts/{id}"), None, None)
            .await?;
        Ok(())
    }
}

#[cfg(feature = "multipart")]
#[cfg_attr(docsrs, doc(cfg(feature = "multipart")))]
impl Fonts<'_> {
    /// Upload one font file (TTF, OTF, WOFF or WOFF2, up to 5 MB) from bytes
    /// or a path, as [`MediaFile::new`](crate::models::MediaFile::new) or
    /// [`MediaFile::from_path`](crate::models::MediaFile::from_path) builds it.
    ///
    /// Needs the `multipart` feature, which is on by default.
    pub async fn upload(
        &self,
        workspace_id: &str,
        file: crate::models::MediaFile,
        options: &crate::models::FontUpload,
    ) -> Result<crate::models::UploadedFont> {
        let len = file.source.len().await.map_err(|err| {
            crate::error::Error::Config(format!("fopost: could not read {} — {err}", file.filename))
        })?;
        let body = match file.source {
            crate::models::FileSource::Bytes(bytes) => reqwest::Body::from(bytes),
            crate::models::FileSource::Path(path) => {
                let opened = tokio::fs::File::open(&path).await.map_err(|err| {
                    crate::error::Error::Config(format!(
                        "fopost: could not open {} — {err}",
                        path.display()
                    ))
                })?;
                reqwest::Body::wrap_stream(tokio_util::io::ReaderStream::new(opened))
            }
        };
        let part = reqwest::multipart::Part::stream_with_length(body, len)
            .file_name(file.filename)
            .mime_str(&file.mime_type)
            .map_err(crate::error::Error::Transport)?;

        let mut form = reqwest::multipart::Form::new()
            .text("workspace_id", workspace_id.to_string())
            .part("file", part);
        if let Some(family) = &options.family {
            form = form.text("family", family.clone());
        }
        if let Some(weight) = options.weight {
            form = form.text("weight", weight.to_string());
        }
        if let Some(style) = &options.style {
            form = form.text("style", style.to_string());
        }

        let value = self
            .http
            .send_request(
                self.http
                    .raw_request(Method::POST, "/fonts")?
                    .multipart(form),
            )
            .await?;
        crate::http::decode(value)
    }
}

//! `client.validate()` — check content, length, or a media URL against
//! platform rules without creating a post.

use reqwest::Method;

use crate::error::Result;
use crate::http::{Envelope, HttpClient};
use crate::models::{
    ValidateLength, ValidateLengthResult, ValidateMedia, ValidateMediaResult, ValidatePost,
    ValidatePostResult,
};

/// Validation. Needs the `posts` scope; nothing is stored.
#[derive(Debug, Clone)]
pub struct Validate<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Validate<'_> {
    /// Per-platform blockers and advisory signals for content that is not a post yet.
    pub async fn post(&self, payload: &ValidatePost) -> Result<ValidatePostResult> {
        let body: Envelope<ValidatePostResult> = self
            .http
            .send(Method::POST, "/validate/post", None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// How each platform counts the text and whether it fits.
    pub async fn length(&self, payload: &ValidateLength) -> Result<ValidateLengthResult> {
        let body: Envelope<ValidateLengthResult> = self
            .http
            .send(Method::POST, "/validate/length", None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// Fetch a public file and check it. Answers 200 even when a check fails;
    /// an unfetchable URL is a 400 `unsupported`.
    pub async fn media(&self, payload: &ValidateMedia) -> Result<ValidateMediaResult> {
        let body: Envelope<ValidateMediaResult> = self
            .http
            .send(Method::POST, "/validate/media", None, Some(payload))
            .await?;
        Ok(body.data)
    }
}

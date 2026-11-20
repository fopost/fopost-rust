//! Video edits — background jobs that turn a library video into a new one.

use serde::{Deserialize, Serialize};

use super::common::string_enum;

string_enum! {
    /// Where a video edit stands.
    pub enum VideoEditStatus {
        Queued => "queued",
        Running => "running",
        Succeeded => "succeeded",
        Failed => "failed",
    }
}

string_enum! {
    /// An edit step. `video_edits().ops()` says which are available.
    pub enum VideoEditOp {
        Trim => "trim",
        Cut => "cut",
        Speed => "speed",
        Merge => "merge",
        Volume => "volume",
        RemoveSilence => "remove_silence",
        Denoise => "denoise",
        Subtitles => "subtitles",
        TranslateSubtitles => "translate_subtitles",
        Clips => "clips",
        Music => "music",
        Voiceover => "voiceover",
        Watermark => "watermark",
        Cover => "cover",
        Reframe => "reframe",
        Audiogram => "audiogram",
    }
}

/// One operation and whether this deployment can run it.
#[derive(Debug, Clone, Deserialize)]
pub struct VideoEditOpInfo {
    pub name: VideoEditOp,
    #[serde(default)]
    pub available: bool,
    #[serde(default)]
    pub uses_ai: bool,
}

/// One step of an edit. `params` is the operation's own JSON object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoEditOperation {
    pub op: VideoEditOp,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
}

impl VideoEditOperation {
    pub fn new(op: VideoEditOp) -> Self {
        Self {
            op,
            params: serde_json::Value::Null,
        }
    }

    pub fn params(mut self, params: serde_json::Value) -> Self {
        self.params = params;
        self
    }
}

/// The finished video.
#[derive(Debug, Clone, Deserialize)]
pub struct VideoEditResultMedia {
    pub id: String,
    #[serde(default)]
    pub preview_url: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
}

/// A file an edit produced beside the main result, such as a subtitle track or a clip.
#[derive(Debug, Clone, Deserialize)]
pub struct VideoEditOutput {
    pub media_id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub mime: Option<String>,
    #[serde(default)]
    pub preview_url: Option<String>,
}

/// A video edit job.
#[derive(Debug, Clone, Deserialize)]
pub struct VideoEdit {
    pub id: String,
    pub status: VideoEditStatus,
    /// 0 to 1.
    #[serde(default)]
    pub progress: f64,
    #[serde(default)]
    pub source_media_id: String,
    #[serde(default)]
    pub operations: Vec<VideoEditOperation>,
    #[serde(default)]
    pub result_media_id: Option<String>,
    #[serde(default)]
    pub result_media: Option<VideoEditResultMedia>,
    #[serde(default)]
    pub extra_outputs: Vec<VideoEditOutput>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// Query parameters for `GET /video-edits`.
#[derive(Debug, Clone, Default)]
pub struct ListVideoEdits<'a> {
    pub workspace_id: Option<&'a str>,
    /// Only edits of this source video.
    pub media_id: Option<&'a str>,
    pub limit: Option<u32>,
}

/// The body of `POST /video-edits`. Build it with [`CreateVideoEdit::new`].
#[derive(Debug, Clone, Serialize)]
pub struct CreateVideoEdit {
    /// May be omitted for a key bound to one workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    pub media_id: String,
    pub operations: Vec<VideoEditOperation>,
}

impl CreateVideoEdit {
    pub fn new(
        media_id: impl Into<String>,
        operations: impl IntoIterator<Item = VideoEditOperation>,
    ) -> Self {
        Self {
            workspace_id: None,
            media_id: media_id.into(),
            operations: operations.into_iter().collect(),
        }
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

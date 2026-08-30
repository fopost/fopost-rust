//! Posts, deliveries, publish runs, and bulk import.

use serde::{Deserialize, Serialize};

use super::common::{string_enum, ContentType, DeliveryStatus, GapUnit, MediaItem, PostStatus};

/// One block of a post's body. A thread is several blocks in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub media: Vec<MediaItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

impl ContentBlock {
    /// A text-only block.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            id: None,
            text: Some(text.into()),
            media: Vec::new(),
            position: None,
        }
    }

    /// Attach media to this block.
    pub fn with_media(mut self, media: impl IntoIterator<Item = MediaItem>) -> Self {
        self.media.extend(media);
        self
    }
}

impl From<&str> for ContentBlock {
    fn from(text: &str) -> Self {
        ContentBlock::text(text)
    }
}

impl From<String> for ContentBlock {
    fn from(text: String) -> Self {
        ContentBlock::text(text)
    }
}

/// An account a post targets, plus how its delivery went.
#[derive(Debug, Clone, Deserialize)]
pub struct PostAccount {
    /// Social account id.
    pub id: String,
    pub platform: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub publish_status: Option<DeliveryStatus>,
    #[serde(default)]
    pub posted_at: Option<String>,
    #[serde(default)]
    pub platform_post_id: Option<String>,
    #[serde(default)]
    pub external_url: Option<String>,
    #[serde(default)]
    pub error_code: Option<String>,
    /// User-friendly error message.
    #[serde(default)]
    pub error_message: Option<String>,
    /// Raw error message from the platform.
    #[serde(default)]
    pub raw_error_message: Option<String>,
    #[serde(default)]
    pub attempts: Option<i32>,
    #[serde(default)]
    pub max_attempts: Option<i32>,
}

/// A label as it appears on a post.
#[derive(Debug, Clone, Deserialize)]
pub struct PostLabel {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

/// A post: its body, its targets, and its schedule.
#[derive(Debug, Clone, Deserialize)]
pub struct Post {
    pub id: String,
    pub workspace_id: String,
    pub status: PostStatus,
    pub content_type: ContentType,
    #[serde(default)]
    pub schedule_at: Option<String>,
    #[serde(default)]
    pub repeatable: Option<bool>,
    #[serde(default)]
    pub repeatable_times: Option<i32>,
    #[serde(default)]
    pub repeatable_gap: Option<i32>,
    #[serde(default)]
    pub repeatable_gap_unit: Option<GapUnit>,
    #[serde(default)]
    pub remaining_posts: Option<i32>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub auto_plug: Option<bool>,
    #[serde(default)]
    pub auto_plug_content: Option<String>,
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    #[serde(default)]
    pub accounts: Vec<PostAccount>,
    #[serde(default)]
    pub labels: Vec<PostLabel>,
    /// Per-account platform settings, keyed by account id.
    #[serde(default)]
    pub settings: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// An account whose credentials look shaky, reported alongside a publish.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthWarning {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub health_status: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

/// A delivery as the publish, retry, and cancel endpoints report it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishDelivery {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub status: Option<DeliveryStatus>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub platform_post_id: Option<String>,
    #[serde(default)]
    pub external_url: Option<String>,
    #[serde(default)]
    pub posted_at: Option<String>,
    #[serde(default)]
    pub attempts: Option<i32>,
    #[serde(default)]
    pub scheduled_publish_at: Option<String>,
    #[serde(default)]
    pub delay_reason: Option<String>,
    #[serde(default)]
    pub delay_message: Option<String>,
}

/// What `publish` answers with once the deliveries are queued.
#[derive(Debug, Clone, Deserialize)]
pub struct PublishResult {
    #[serde(default)]
    pub post_status: Option<PostStatus>,
    #[serde(default)]
    pub deliveries: Vec<PublishDelivery>,
    #[serde(default, rename = "healthWarnings")]
    pub health_warnings: Vec<HealthWarning>,
}

/// A post reference carried inside publish and preflight answers.
#[derive(Debug, Clone, Deserialize)]
pub struct PostRef {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<PostStatus>,
}

/// What `publish` answers with when `dry_run` was set: nothing was sent.
#[derive(Debug, Clone, Deserialize)]
pub struct DryRunResult {
    /// Required, and the field that tells a dry run apart from a real publish.
    #[serde(rename = "dryRun")]
    pub dry_run: bool,
    #[serde(default)]
    pub post: Option<PostRef>,
    #[serde(default)]
    pub accounts: Vec<DryRunAccount>,
    #[serde(default, rename = "healthWarnings")]
    pub health_warnings: Vec<HealthWarning>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DryRunAccount {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

/// Either outcome of `publish`: queued for real, or a dry run.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PublishOutcome {
    /// `dry_run` was set — nothing left the building.
    DryRun(DryRunResult),
    /// Deliveries were queued.
    Queued(PublishResult),
}

impl PublishOutcome {
    /// The queued deliveries, empty on a dry run.
    pub fn deliveries(&self) -> &[PublishDelivery] {
        match self {
            PublishOutcome::Queued(result) => &result.deliveries,
            PublishOutcome::DryRun(_) => &[],
        }
    }
}

/// An account that has already burned its retry budget.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceededAccount {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub attempts: Option<i32>,
}

/// What `retry` answers with.
#[derive(Debug, Clone, Deserialize)]
pub struct RetryResult {
    #[serde(default)]
    pub post_status: Option<PostStatus>,
    #[serde(default)]
    pub deliveries: Vec<PublishDelivery>,
    /// Accounts left alone because they are out of attempts.
    #[serde(default)]
    pub exceeded: Vec<ExceededAccount>,
}

/// What `cancel` answers with.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelResult {
    #[serde(default)]
    pub post_status: Option<PostStatus>,
    #[serde(default)]
    pub deliveries: Vec<PublishDelivery>,
}

string_enum! {
    /// How seriously to take a content signal. Signals never block a publish.
    pub enum SignalLevel {
        Info => "info",
        Warn => "warn",
    }
}

/// An advisory note about the content — never a blocker.
#[derive(Debug, Clone, Deserialize)]
pub struct ContentSignal {
    pub level: SignalLevel,
    pub code: String,
    pub message: String,
}

/// One account's readiness, from a preflight check.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightAccount {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub ready: bool,
    /// Hard blockers that prevent publishing.
    #[serde(default)]
    pub issues: Vec<String>,
    /// Advisory 0-100 content quality score for this platform.
    #[serde(default)]
    pub score: Option<f64>,
    /// Advisory signals — they never block publishing.
    #[serde(default)]
    pub signals: Vec<ContentSignal>,
}

/// What `preflight` answers with: blockers and advice, nothing sent.
#[derive(Debug, Clone, Deserialize)]
pub struct PreflightResult {
    #[serde(default)]
    pub ready: bool,
    #[serde(default)]
    pub post: Option<PostRef>,
    #[serde(default)]
    pub accounts: Vec<PreflightAccount>,
}

/// A delivery row on `GET /posts/{id}/deliveries`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delivery {
    pub id: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub status: Option<DeliveryStatus>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub attempts: Option<i32>,
    #[serde(default)]
    pub max_attempts: Option<i32>,
    #[serde(default)]
    pub scheduled_publish_at: Option<String>,
    #[serde(default)]
    pub delay_reason: Option<String>,
    #[serde(default)]
    pub delay_message: Option<String>,
    #[serde(default)]
    pub posted_at: Option<String>,
    #[serde(default)]
    pub last_attempt_at: Option<String>,
    #[serde(default)]
    pub platform_post_id: Option<String>,
    #[serde(default)]
    pub external_url: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub account_name: Option<String>,
}

/// One account's attempt inside a publish run.
#[derive(Debug, Clone, Deserialize)]
pub struct PublishRunDelivery {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub account_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub attempt_number: Option<i32>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub platform_post_id: Option<String>,
    #[serde(default)]
    pub external_url: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
}

/// One pass of publishing a post to its accounts.
#[derive(Debug, Clone, Deserialize)]
pub struct PublishRun {
    pub id: String,
    #[serde(default)]
    pub run_number: Option<i32>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub deliveries: Vec<PublishRunDelivery>,
}

/// A freshly duplicated post.
#[derive(Debug, Clone, Deserialize)]
pub struct DuplicatedPost {
    pub id: String,
    #[serde(default)]
    pub status: Option<String>,
}

string_enum! {
    /// What a bulk action does to the selection.
    pub enum BulkAction {
        Shift => "shift",
        Label => "label",
        Delete => "delete",
    }
}

string_enum! {
    /// How a bulk relabel treats the labels already on a post.
    pub enum LabelMode {
        /// Wipe the existing labels; an empty list clears them.
        Replace => "replace",
        Add => "add",
        Remove => "remove",
    }
}

/// What a bulk action changed.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkResult {
    pub updated: u32,
    pub action: BulkAction,
    #[serde(default)]
    pub mode: Option<LabelMode>,
}

/// One CSV row's verdict from a bulk-import dry run.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportRow {
    pub row: u32,
    #[serde(default)]
    pub content_preview: Option<String>,
    #[serde(default)]
    pub schedule_at: Option<String>,
    #[serde(default)]
    pub accounts: Vec<String>,
    #[serde(default)]
    pub labels: u32,
    #[serde(default)]
    pub has_media: bool,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// The dry run of a bulk-import CSV: what would happen, row by row.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportValidation {
    pub total_rows: u32,
    pub valid_rows: u32,
    pub invalid_rows: u32,
    #[serde(default)]
    pub rows: Vec<BulkImportRow>,
}

/// A post created by a bulk import.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportPost {
    pub id: String,
    #[serde(default)]
    pub schedule_at: Option<String>,
}

/// What a committed bulk import created, with the batch id that rolls it back.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportCommit {
    pub batch_id: String,
    pub created: u32,
    #[serde(default)]
    pub posts: Vec<BulkImportPost>,
}

/// What a bulk-import rollback deleted.
#[derive(Debug, Clone, Deserialize)]
pub struct BulkImportRollback {
    pub message: String,
    pub deleted: u32,
}

/// Post metrics, summed across every platform the post reached.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostAnalyticsTotals {
    #[serde(default)]
    pub impressions: i64,
    #[serde(default)]
    pub reach: i64,
    #[serde(default)]
    pub engagements: i64,
    #[serde(default)]
    pub likes: i64,
    #[serde(default)]
    pub comments: i64,
    #[serde(default)]
    pub shares: i64,
    #[serde(default)]
    pub reposts: i64,
    #[serde(default)]
    pub clicks: i64,
    #[serde(default)]
    pub saves: i64,
    #[serde(default)]
    pub video_views: i64,
    #[serde(default)]
    pub follows: i64,
}

/// Metrics for one post on one platform.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostAnalyticsMetrics {
    #[serde(default)]
    pub impressions: Option<i64>,
    #[serde(default)]
    pub reach: Option<i64>,
    #[serde(default)]
    pub engagements: Option<i64>,
    #[serde(default)]
    pub likes: Option<i64>,
    #[serde(default)]
    pub comments: Option<i64>,
    #[serde(default)]
    pub shares: Option<i64>,
    #[serde(default)]
    pub reposts: Option<i64>,
    #[serde(default)]
    pub clicks: Option<i64>,
    #[serde(default)]
    pub saves: Option<i64>,
    #[serde(default)]
    pub video_views: Option<i64>,
    #[serde(default)]
    pub watch_time_ms: Option<i64>,
    #[serde(default)]
    pub avg_watch_time_ms: Option<i64>,
    #[serde(default)]
    pub reactions_breakdown: Option<serde_json::Value>,
    #[serde(default)]
    pub follows: Option<i64>,
    #[serde(default)]
    pub fetched_at: Option<String>,
}

/// How one post did on one platform.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostAnalyticsPlatform {
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub external_post_id: Option<String>,
    #[serde(default)]
    pub permalink: Option<String>,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub posted_at: Option<String>,
    #[serde(default)]
    pub metrics: Option<PostAnalyticsMetrics>,
}

/// How a post did, in total and per platform.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostAnalytics {
    #[serde(default)]
    pub post_id: Option<String>,
    #[serde(default)]
    pub totals: Option<PostAnalyticsTotals>,
    #[serde(default)]
    pub platforms: Vec<PostAnalyticsPlatform>,
    #[serde(default)]
    pub last_fetched_at: Option<String>,
}

// ─── Request bodies ────────────────────────────────────────────────

/// The body of `POST /posts`. Build it with [`CreatePost::new`].
#[derive(Debug, Clone, Serialize)]
pub struct CreatePost {
    pub workspace_id: String,
    pub accounts: Vec<String>,
    pub content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<super::common::ArtifactType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PostStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable_times: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable_gap: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable_gap_unit: Option<GapUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_plug: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_plug_content: Option<String>,
    /// Per-account platform settings, keyed by account id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl CreatePost {
    /// A draft in `workspace_id` targeting `accounts`, from one or more blocks.
    pub fn new<C>(workspace_id: impl Into<String>, content: impl IntoIterator<Item = C>) -> Self
    where
        C: Into<ContentBlock>,
    {
        Self {
            workspace_id: workspace_id.into(),
            accounts: Vec::new(),
            content: content.into_iter().map(Into::into).collect(),
            content_type: None,
            artifact_type: None,
            status: None,
            schedule_at: None,
            repeatable: None,
            repeatable_times: None,
            repeatable_gap: None,
            repeatable_gap_unit: None,
            labels: None,
            title: None,
            internal_title: None,
            summary: None,
            auto_plug: None,
            auto_plug_content: None,
            settings: None,
        }
    }

    /// A single-block post — the common case.
    pub fn text(workspace_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self::new(workspace_id, [ContentBlock::text(text)])
    }

    /// The social accounts to publish to.
    pub fn accounts<S: Into<String>>(mut self, accounts: impl IntoIterator<Item = S>) -> Self {
        self.accounts = accounts.into_iter().map(Into::into).collect();
        self
    }

    /// Schedule it: sets `status` to `scheduled` and the time to send.
    pub fn schedule_at(mut self, at: impl Into<String>) -> Self {
        self.status = Some(PostStatus::Scheduled);
        self.schedule_at = Some(at.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn labels<S: Into<String>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = Some(labels.into_iter().map(Into::into).collect());
        self
    }

    /// Per-account platform settings, keyed by account id.
    pub fn settings(
        mut self,
        settings: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        self.settings = Some(settings);
        self
    }
}

/// The body of `PUT /posts/{id}`. Only the fields you set are sent, so an
/// update never clears something by omission.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdatePost {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<super::common::ArtifactType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PostStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable_times: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable_gap: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeatable_gap_unit: Option<GapUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_plug: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_plug_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl UpdatePost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content<C: Into<ContentBlock>>(mut self, content: impl IntoIterator<Item = C>) -> Self {
        self.content = Some(content.into_iter().map(Into::into).collect());
        self
    }

    pub fn accounts<S: Into<String>>(mut self, accounts: impl IntoIterator<Item = S>) -> Self {
        self.accounts = Some(accounts.into_iter().map(Into::into).collect());
        self
    }

    pub fn schedule_at(mut self, at: impl Into<String>) -> Self {
        self.status = Some(PostStatus::Scheduled);
        self.schedule_at = Some(at.into());
        self
    }

    pub fn status(mut self, status: PostStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn labels<S: Into<String>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = Some(labels.into_iter().map(Into::into).collect());
        self
    }
}

/// Filters for `GET /posts`.
#[derive(Debug, Clone, Default)]
pub struct ListPosts {
    pub workspace_id: Option<String>,
    pub status: Option<PostStatus>,
    pub search: Option<String>,
    pub platform: Option<String>,
    pub label: Option<String>,
    pub account_id: Option<String>,
    /// A single YYYY-MM-DD day.
    pub date: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    /// `oldest` flips the default newest-first order.
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListPosts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn status(mut self, status: PostStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

/// The body of `POST /posts/bulk`: one action over a selection of posts.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BulkPostAction {
    /// Move every selected post's schedule by `offset_minutes`, which may be negative.
    Shift {
        workspace_id: String,
        post_ids: Vec<String>,
        offset_minutes: i64,
    },
    /// Set, add, or remove labels across the selection.
    Label {
        workspace_id: String,
        post_ids: Vec<String>,
        label_ids: Vec<String>,
        mode: LabelMode,
    },
    /// Delete the selection.
    Delete {
        workspace_id: String,
        post_ids: Vec<String>,
    },
}

impl BulkPostAction {
    pub fn shift<S: Into<String>>(
        workspace_id: impl Into<String>,
        post_ids: impl IntoIterator<Item = S>,
        offset_minutes: i64,
    ) -> Self {
        BulkPostAction::Shift {
            workspace_id: workspace_id.into(),
            post_ids: post_ids.into_iter().map(Into::into).collect(),
            offset_minutes,
        }
    }

    pub fn label<S: Into<String>, L: Into<String>>(
        workspace_id: impl Into<String>,
        post_ids: impl IntoIterator<Item = S>,
        label_ids: impl IntoIterator<Item = L>,
        mode: LabelMode,
    ) -> Self {
        BulkPostAction::Label {
            workspace_id: workspace_id.into(),
            post_ids: post_ids.into_iter().map(Into::into).collect(),
            label_ids: label_ids.into_iter().map(Into::into).collect(),
            mode,
        }
    }

    pub fn delete<S: Into<String>>(
        workspace_id: impl Into<String>,
        post_ids: impl IntoIterator<Item = S>,
    ) -> Self {
        BulkPostAction::Delete {
            workspace_id: workspace_id.into(),
            post_ids: post_ids.into_iter().map(Into::into).collect(),
        }
    }
}

/// Options for `publish`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PublishOptions {
    /// Publish to a subset of the post's accounts instead of all of them.
    #[serde(rename = "accountIds", skip_serializing_if = "Option::is_none")]
    pub account_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<PublishFlags>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PublishFlags {
    /// Validate and report, but send nothing.
    #[serde(rename = "dryRun")]
    pub dry_run: bool,
}

impl PublishOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accounts<S: Into<String>>(mut self, account_ids: impl IntoIterator<Item = S>) -> Self {
        self.account_ids = Some(account_ids.into_iter().map(Into::into).collect());
        self
    }

    /// Report what would happen without publishing anything.
    pub fn dry_run(mut self) -> Self {
        self.options = Some(PublishFlags { dry_run: true });
        self
    }
}

/// Options for `retry`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RetryOptions {
    #[serde(rename = "accountIds", skip_serializing_if = "Option::is_none")]
    pub account_ids: Option<Vec<String>>,
    /// Also re-send to accounts that already published. Off by default.
    #[serde(rename = "includePublished", skip_serializing_if = "Option::is_none")]
    pub include_published: Option<bool>,
}

impl RetryOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accounts<S: Into<String>>(mut self, account_ids: impl IntoIterator<Item = S>) -> Self {
        self.account_ids = Some(account_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn include_published(mut self, include: bool) -> Self {
        self.include_published = Some(include);
        self
    }
}

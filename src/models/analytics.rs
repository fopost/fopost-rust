//! Analytics — the read-only reporting surface.

use serde::Deserialize;

use super::common::{string_enum, DeliveryStatus};

/// Period-over-period change, as a fraction. `None` where there is no baseline.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewDeltas {
    #[serde(default)]
    pub followers: Option<f64>,
    #[serde(default)]
    pub posts: Option<f64>,
    #[serde(default)]
    pub engagement: Option<f64>,
    #[serde(default)]
    pub impressions: Option<f64>,
    #[serde(default)]
    pub likes: Option<f64>,
    #[serde(default)]
    pub comments: Option<f64>,
    #[serde(default)]
    pub shares: Option<f64>,
    #[serde(default)]
    pub profile_views: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayStats {
    #[serde(default)]
    pub posts: i64,
    #[serde(default)]
    pub follower_change: i64,
    #[serde(default)]
    pub engagement: i64,
}

/// Accounts and followers on one platform.
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformTotals {
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub accounts: i64,
    #[serde(default)]
    pub followers: i64,
}

/// A day of an account's follower count.
#[derive(Debug, Clone, Deserialize)]
pub struct FollowerPoint {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub followers: Option<i64>,
}

/// One account's line in the overview.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewAccount {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub followers: Option<i64>,
    #[serde(default)]
    pub total_posts: Option<i64>,
    #[serde(default)]
    pub fetched_at: Option<String>,
    /// Daily follower history, up to 30 days, oldest first.
    #[serde(default)]
    pub history: Vec<FollowerPoint>,
}

/// Everything on the analytics dashboard, in one call.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsOverview {
    #[serde(default)]
    pub total_accounts: i64,
    #[serde(default)]
    pub total_followers: i64,
    #[serde(default)]
    pub total_posts: i64,
    #[serde(default)]
    pub total_engagement: i64,
    #[serde(default)]
    pub total_impressions: i64,
    #[serde(default)]
    pub total_reach: i64,
    #[serde(default)]
    pub total_likes: i64,
    #[serde(default)]
    pub total_comments: i64,
    #[serde(default)]
    pub total_shares: i64,
    #[serde(default)]
    pub total_reposts: i64,
    #[serde(default)]
    pub total_saves: i64,
    #[serde(default)]
    pub total_clicks: i64,
    #[serde(default)]
    pub total_video_views: i64,
    #[serde(default)]
    pub total_profile_views: i64,
    #[serde(default)]
    pub engagement_rate: Option<f64>,
    #[serde(default)]
    pub deltas: Option<OverviewDeltas>,
    #[serde(default)]
    pub today_stats: Option<TodayStats>,
    #[serde(default)]
    pub platforms: Vec<PlatformTotals>,
    #[serde(default)]
    pub accounts: Vec<OverviewAccount>,
}

/// One day of the time series.
#[derive(Debug, Clone, Deserialize)]
pub struct TimeSeriesPoint {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub engagements: i64,
    #[serde(default)]
    pub impressions: i64,
    #[serde(default)]
    pub likes: i64,
    #[serde(default)]
    pub comments: i64,
    #[serde(default)]
    pub shares: i64,
    #[serde(default)]
    pub followers: i64,
    #[serde(default)]
    pub posts: i64,
}

/// Daily metrics over a window.
#[derive(Debug, Clone, Deserialize)]
pub struct TimeSeries {
    #[serde(default)]
    pub days: u32,
    #[serde(default)]
    pub series: Vec<TimeSeriesPoint>,
}

string_enum! {
    /// Where a top post came from: published through FoPost, or found on the platform.
    pub enum TopPostSource {
        Fopost => "fopost",
        Platform => "platform",
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopPostPlatform {
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopPostLabel {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopPostMetrics {
    #[serde(default)]
    pub engagements: i64,
    #[serde(default)]
    pub impressions: i64,
    #[serde(default)]
    pub reach: i64,
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
}

/// A high performer.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopPost {
    #[serde(default)]
    pub rank: Option<u32>,
    #[serde(default)]
    pub post_id: Option<String>,
    #[serde(default)]
    pub external_post_id: Option<String>,
    #[serde(default)]
    pub source: Option<TopPostSource>,
    #[serde(default)]
    pub preview: Option<String>,
    #[serde(default)]
    pub permalink: Option<String>,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub platforms: Vec<TopPostPlatform>,
    #[serde(default)]
    pub labels: Vec<TopPostLabel>,
    #[serde(default)]
    pub metrics: Option<TopPostMetrics>,
}

/// A campaign roll-up: how everything under one label did.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelAnalytics {
    #[serde(default)]
    pub label_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub post_count: i64,
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
    pub engagement_rate: Option<f64>,
    #[serde(default)]
    pub follower_delta: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostsTablePlatform {
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub delivery_status: Option<DeliveryStatus>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeliverySummary {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub published: u32,
    #[serde(default)]
    pub failed: u32,
    #[serde(default)]
    pub pending: u32,
}

/// One row of the posts table.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostsTableRow {
    #[serde(default)]
    pub post_id: Option<String>,
    /// First 120 characters of the post text.
    #[serde(default)]
    pub preview: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub scheduled_at: Option<String>,
    #[serde(default)]
    pub platforms: Vec<PostsTablePlatform>,
    #[serde(default)]
    pub delivery_summary: Option<DeliverySummary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusSummary {
    #[serde(default)]
    pub draft: u32,
    #[serde(default)]
    pub scheduled: u32,
    #[serde(default)]
    pub published: u32,
    #[serde(default)]
    pub failed: u32,
    #[serde(default)]
    pub pending: u32,
    #[serde(default)]
    pub total: u32,
}

/// Posts with their delivery breakdown, paginated.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostsTable {
    #[serde(default)]
    pub posts: Vec<PostsTableRow>,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub limit: u32,
    #[serde(default)]
    pub status_summary: Option<StatusSummary>,
}

/// One day of posting activity.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreakDay {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub published_count: u32,
    #[serde(default)]
    pub failed_count: u32,
    #[serde(default)]
    pub scheduled_count: u32,
}

string_enum! {
    /// Which audience a demographics breakdown describes.
    pub enum Audience {
        Followers => "followers",
        Engaged => "engaged",
        Reached => "reached",
    }
}

/// One slice of a demographic dimension.
#[derive(Debug, Clone, Deserialize)]
pub struct DemographicsBucket {
    pub key: String,
    #[serde(default)]
    pub value: f64,
    /// This bucket's share of the whole, 0 to 1.
    #[serde(default)]
    pub share: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DemographicsDimensions {
    #[serde(default)]
    pub age: Vec<DemographicsBucket>,
    #[serde(default)]
    pub gender: Vec<DemographicsBucket>,
    #[serde(default)]
    pub country: Vec<DemographicsBucket>,
    #[serde(default)]
    pub city: Vec<DemographicsBucket>,
}

/// An account that did, or could not, contribute to a breakdown.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemographicsAccount {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

/// Who the audience is, across the accounts that report it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Demographics {
    #[serde(default)]
    pub audience: Option<Audience>,
    #[serde(default)]
    pub dimensions: Option<DemographicsDimensions>,
    #[serde(default)]
    pub contributing_accounts: Vec<DemographicsAccount>,
    /// Accounts whose platform exposes no demographics.
    #[serde(default)]
    pub unsupported_accounts: Vec<DemographicsAccount>,
}

string_enum! {
    /// Which part of a collection run failed.
    pub enum CollectStage {
        Account => "account",
        Demographics => "demographics",
        Timeline => "timeline",
        Post => "post",
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectError {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub stage: Option<CollectStage>,
    #[serde(default)]
    pub message: Option<String>,
}

/// What a collection pass fetched, and what it could not.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectSummary {
    #[serde(default)]
    pub accounts: u32,
    #[serde(default)]
    pub posts: u32,
    #[serde(default)]
    pub demographics: u32,
    #[serde(default)]
    pub errors: u32,
    #[serde(default)]
    pub error_details: Vec<CollectError>,
}

/// Filters shared by most analytics endpoints.
#[derive(Debug, Clone, Default)]
pub struct AnalyticsQuery {
    pub workspace_id: Option<String>,
    pub account_id: Option<String>,
    /// A rolling window, in days. Ignored when `from`/`to` are set.
    pub days: Option<u32>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub label: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    /// `recent` on top-posts orders by date instead of engagement.
    pub sort: Option<String>,
}

impl AnalyticsQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn days(mut self, days: u32) -> Self {
        self.days = Some(days);
        self
    }

    /// An explicit YYYY-MM-DD range, which wins over `days`.
    pub fn range(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self.to = Some(to.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

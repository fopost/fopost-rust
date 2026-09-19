//! `client.analytics()` — the read-only reporting surface.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    AnalyticsOverview, AnalyticsQuery, Audience, CollectPostResult, CollectSummary, ContentDecay,
    Demographics, InboxPage, LabelAnalytics, MetricChangePage, MetricChangesQuery, NativePost,
    NativePostsQuery, PostTimeline, PostingFrequency, PostsTable, StreakDay, TimeSeries, TopPost,
};

/// Reporting across every connected account.
#[derive(Debug, Clone)]
pub struct Analytics<'a> {
    pub(crate) http: &'a HttpClient,
}

/// Percent-encode one path segment. A post can be addressed by its permalink,
/// which carries the slashes and colons that would otherwise split the path.
fn encode_segment(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn base_query(params: &AnalyticsQuery) -> Query {
    let mut query: Query = Vec::new();
    push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
    push_opt(&mut query, "accountId", params.account_id.as_ref());
    push_opt(&mut query, "days", params.days);
    push_opt(&mut query, "from", params.from.as_ref());
    push_opt(&mut query, "to", params.to.as_ref());
    query
}

impl Analytics<'_> {
    /// Everything on the analytics dashboard, in one call.
    pub async fn overview(&self, params: &AnalyticsQuery) -> Result<AnalyticsOverview> {
        let body: Envelope<AnalyticsOverview> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/analytics/overview",
                Some(base_query(params)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Daily metrics over the window.
    pub async fn time_series(&self, params: &AnalyticsQuery) -> Result<TimeSeries> {
        let body: Envelope<TimeSeries> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/analytics/time-series",
                Some(base_query(params)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The best performing posts, by engagement unless `sort` says otherwise.
    pub async fn top_posts(&self, params: &AnalyticsQuery) -> Result<Vec<TopPost>> {
        let mut query = base_query(params);
        push_opt(&mut query, "limit", params.limit);
        push_opt(&mut query, "label", params.label.as_ref());
        push_opt(&mut query, "sort", params.sort.as_ref());
        let body: Envelope<Vec<TopPost>> = self
            .http
            .send::<_, ()>(Method::GET, "/analytics/top-posts", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// A campaign roll-up: how everything under each label did.
    pub async fn labels(&self, params: &AnalyticsQuery) -> Result<Vec<LabelAnalytics>> {
        let body: Envelope<Vec<LabelAnalytics>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/analytics/labels",
                Some(base_query(params)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Posts with their delivery breakdown, paginated.
    pub async fn posts_table(&self, params: &AnalyticsQuery) -> Result<PostsTable> {
        let mut query = base_query(params);
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "limit", params.limit);
        let body: Envelope<PostsTable> = self
            .http
            .send::<_, ()>(Method::GET, "/analytics/posts-table", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// 365 days of posting activity, for a contribution-graph style view.
    pub async fn posting_streak(&self, workspace_id: Option<&str>) -> Result<Vec<StreakDay>> {
        #[derive(serde::Deserialize)]
        struct Streak {
            #[serde(default)]
            streak: Vec<StreakDay>,
        }
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", workspace_id);
        let body: Envelope<Streak> = self
            .http
            .send::<_, ()>(Method::GET, "/analytics/posting-streak", Some(query), None)
            .await?;
        Ok(body.data.streak)
    }

    /// Who the audience is, across the accounts whose platform reports it.
    pub async fn demographics(
        &self,
        audience: Option<Audience>,
        params: &AnalyticsQuery,
    ) -> Result<Demographics> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "audience", audience);
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "accountId", params.account_id.as_ref());
        let body: Envelope<Demographics> = self
            .http
            .send::<_, ()>(Method::GET, "/analytics/demographics", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Pull fresh numbers from the platforms now, instead of waiting for the
    /// scheduled collection. Rate limited by every platform involved.
    pub async fn collect(&self, account_id: Option<&str>) -> Result<CollectSummary> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "accountId", account_id);
        let body: Envelope<CollectSummary> = self
            .http
            .send::<_, ()>(Method::POST, "/analytics/collect", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// How long a post keeps earning: engagement grouped by the post's age at
    /// the moment each reading was taken. `days` selects posts by publish
    /// time, not reading time.
    pub async fn decay(&self, params: &AnalyticsQuery) -> Result<ContentDecay> {
        let body: Envelope<ContentDecay> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/analytics/decay",
                Some(base_query(params)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Whether posting more earned more: weekly cadence against what each
    /// cadence earned per post.
    pub async fn frequency(&self, params: &AnalyticsQuery) -> Result<PostingFrequency> {
        let body: Envelope<PostingFrequency> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/analytics/frequency",
                Some(base_query(params)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Every reading held for one post, oldest first, with what moved between
    /// them and one timeline per delivery. `id_or_permalink` is a FoPost post
    /// id or the permalink of a post made natively on the network.
    pub async fn timeline(&self, id_or_permalink: &str) -> Result<PostTimeline> {
        let path = format!(
            "/analytics/posts/{}/timeline",
            encode_segment(id_or_permalink)
        );
        let body: Envelope<PostTimeline> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Readings recorded after `since`, oldest first, with a cursor to
    /// continue. Poll it to mirror the metrics into your own store instead of
    /// refetching the whole history.
    pub async fn changes(&self, params: &MetricChangesQuery) -> Result<MetricChangePage> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "since", params.since.as_ref());
        push_opt(&mut query, "limit", params.limit);
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "accountId", params.account_id.as_ref());
        let body: Envelope<MetricChangePage> = self
            .http
            .send::<_, ()>(Method::GET, "/analytics/changes", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Re-read one post from the network now. Spends the same per-user budget
    /// as [`collect`](Self::collect), so a burst answers 429.
    pub async fn collect_post(&self, id_or_permalink: &str) -> Result<CollectPostResult> {
        let path = format!(
            "/posts/{}/analytics/collect",
            encode_segment(id_or_permalink)
        );
        let body: Envelope<CollectPostResult> = self
            .http
            .send::<_, ()>(Method::POST, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Posts on the account that never went out through FoPost, newest first.
    pub async fn native_posts(
        &self,
        account_id: &str,
        params: &NativePostsQuery,
    ) -> Result<InboxPage<NativePost>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        push_opt(&mut query, "days", params.days);
        let path = format!("/accounts/{account_id}/native-posts");
        self.http
            .send::<_, ()>(Method::GET, &path, Some(query), None)
            .await
    }
}

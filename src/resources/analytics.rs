//! `client.analytics()` — the read-only reporting surface.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    AnalyticsOverview, AnalyticsQuery, Audience, CollectSummary, Demographics, LabelAnalytics,
    PostsTable, StreakDay, TimeSeries, TopPost,
};

/// Reporting across every connected account.
#[derive(Debug, Clone)]
pub struct Analytics<'a> {
    pub(crate) http: &'a HttpClient,
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
}

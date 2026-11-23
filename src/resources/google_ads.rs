//! `client.google_ads()`: the Google Ads surface no other network has.
//!
//! Ads, audiences and insights are on [`crate::resources::ads::Ads`] and
//! dispatch by connection. What is here (bid strategies, the ad schedule,
//! Local Services leads, conversions and raw GAQL) is Google only, and a
//! connection on another network answers 400.
//!
//! Every call needs the `ads` scope; anything that changes what a live account
//! serves or bids also needs `publish`.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    CreateGoogleBidStrategy, CreateGoogleConversionAction, GoogleAdScheduleSlot, GoogleBidStrategy,
    GoogleConversionAction, GoogleDateRange, GoogleLocalServicesLead, GoogleQuery,
    GoogleQueryResult, GoogleScope, SetGoogleAdSchedule, UploadGoogleConversionAdjustments,
    UploadGoogleConversions,
};

/// Google Ads.
#[derive(Debug, Clone)]
pub struct GoogleAds<'a> {
    pub(crate) http: &'a HttpClient,
}

fn scope_query(scope: &GoogleScope) -> Query {
    let mut query: Query = Vec::new();
    push_opt(&mut query, "workspace_id", scope.workspace_id.as_deref());
    query.push(("connection_id", scope.connection_id.clone()));
    query.push(("customer_id", scope.customer_id.clone()));
    query
}

#[derive(serde::Deserialize)]
struct Created {
    #[serde(default)]
    id: String,
}

#[derive(serde::Deserialize)]
struct Uploaded {
    #[serde(default)]
    uploaded: i64,
}

#[derive(serde::Deserialize)]
struct Slots {
    #[serde(default)]
    slots: i64,
}

impl GoogleAds<'_> {
    // ── Bid strategies and ad schedule ──

    /// The account's portfolio bid strategies.
    pub async fn bid_strategies(&self, scope: &GoogleScope) -> Result<Vec<GoogleBidStrategy>> {
        let body: Envelope<Vec<GoogleBidStrategy>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/google/bid-strategies",
                Some(scope_query(scope)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Add a bid strategy. Needs `publish` as well as `ads`.
    pub async fn create_bid_strategy(&self, input: &CreateGoogleBidStrategy) -> Result<String> {
        self.created(Method::POST, "/ads/google/bid-strategies", input)
            .await
    }

    /// A campaign's ad schedule.
    pub async fn ad_schedule(
        &self,
        scope: &GoogleScope,
        campaign_id: &str,
    ) -> Result<Vec<GoogleAdScheduleSlot>> {
        let mut query = scope_query(scope);
        query.push(("campaign_id", campaign_id.to_string()));
        let body: Envelope<Vec<GoogleAdScheduleSlot>> = self
            .http
            .send::<_, ()>(Method::GET, "/ads/google/ad-schedule", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Replace a campaign's schedule. Needs `publish` as well as `ads`.
    pub async fn set_ad_schedule(&self, input: &SetGoogleAdSchedule) -> Result<i64> {
        let body: Envelope<Slots> = self
            .http
            .send(Method::PUT, "/ads/google/ad-schedule", None, Some(input))
            .await?;
        Ok(body.data.slots)
    }

    // ── Local Services leads ──

    /// Leads from Local Services Ads, read live and never stored.
    pub async fn local_services_leads(
        &self,
        scope: &GoogleScope,
        range: &GoogleDateRange,
    ) -> Result<Vec<GoogleLocalServicesLead>> {
        let body: Envelope<Vec<GoogleLocalServicesLead>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/google/local-services",
                Some(with_range(scope_query(scope), range)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    // ── Conversions ──

    /// The account's conversion actions.
    pub async fn conversion_actions(
        &self,
        scope: &GoogleScope,
    ) -> Result<Vec<GoogleConversionAction>> {
        let body: Envelope<Vec<GoogleConversionAction>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/google/conversions",
                Some(scope_query(scope)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Add a conversion action. Needs `publish` as well as `ads`.
    pub async fn create_conversion_action(
        &self,
        input: &CreateGoogleConversionAction,
    ) -> Result<String> {
        self.created(Method::POST, "/ads/google/conversions", input)
            .await
    }

    /// Send offline conversions; answers how many landed. Needs `publish`.
    pub async fn upload_conversions(&self, input: &UploadGoogleConversions) -> Result<i64> {
        self.uploaded("/ads/google/conversions/upload", input).await
    }

    /// Send conversion adjustments; answers how many landed. Needs `publish`.
    pub async fn upload_conversion_adjustments(
        &self,
        input: &UploadGoogleConversionAdjustments,
    ) -> Result<i64> {
        self.uploaded("/ads/google/conversions/adjustments", input)
            .await
    }

    // ── GAQL ──

    /// Run a read-only GAQL SELECT; rows come back as Google sends them.
    pub async fn query(&self, input: &GoogleQuery) -> Result<GoogleQueryResult> {
        let body: Envelope<GoogleQueryResult> = self
            .http
            .send(Method::POST, "/ads/insights/query", None, Some(input))
            .await?;
        Ok(body.data)
    }

    async fn created<B: serde::Serialize>(
        &self,
        method: Method,
        path: &str,
        input: &B,
    ) -> Result<String> {
        let body: Envelope<Created> = self.http.send(method, path, None, Some(input)).await?;
        Ok(body.data.id)
    }

    async fn uploaded<B: serde::Serialize>(&self, path: &str, input: &B) -> Result<i64> {
        let body: Envelope<Uploaded> = self
            .http
            .send(Method::POST, path, None, Some(input))
            .await?;
        Ok(body.data.uploaded)
    }
}

fn with_range(mut query: Query, range: &GoogleDateRange) -> Query {
    query.push(("since", range.since.clone()));
    query.push(("until", range.until.clone()));
    query
}

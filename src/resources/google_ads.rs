//! `client.google_ads()` — the Google Ads surface no other network has.
//!
//! Campaigns, ad groups, ads, audiences and insights are on [`crate::resources::ads::Ads`]
//! and dispatch by connection. What is here — keywords, assets, Performance
//! Max asset groups, Local Services leads, conversions and raw GAQL — is
//! Google only, and a connection on another network answers 400.
//!
//! Every call needs the `ads` scope; anything that changes what a live account
//! serves or bids also needs `publish`.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    AddGoogleNegativeKeywords, AttachGoogleAsset, AttachGoogleNegativeKeywordList,
    CreateGoogleAsset, CreateGoogleAssetGroup, CreateGoogleBidStrategy,
    CreateGoogleConversionAction, CreateGoogleKeyword, CreateGoogleNegativeKeywordList,
    GoogleAdScheduleSlot, GoogleAssetGroup, GoogleAssets, GoogleBidStrategy,
    GoogleConversionAction, GoogleDateRange, GoogleKeyword, GoogleKeywordIdea, GoogleKeywordIdeas,
    GoogleKeywordMetrics, GoogleLocalServicesLead, GoogleQuery, GoogleQueryResult, GoogleScope,
    GoogleSearchTerm, GoogleSharedSet, Message, SetGoogleAdSchedule, UpdateGoogleAssetGroup,
    UpdateGoogleKeyword, UploadGoogleConversionAdjustments, UploadGoogleConversions,
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
struct Added {
    #[serde(default)]
    added: i64,
}

#[derive(serde::Deserialize)]
struct Slots {
    #[serde(default)]
    slots: i64,
}

impl GoogleAds<'_> {
    // ── Keywords ──

    /// Keywords on the account, or on one ad group.
    pub async fn keywords(
        &self,
        scope: &GoogleScope,
        ad_group_id: Option<&str>,
    ) -> Result<Vec<GoogleKeyword>> {
        let mut query = scope_query(scope);
        push_opt(&mut query, "ad_group_id", ad_group_id);
        let body: Envelope<Vec<GoogleKeyword>> = self
            .http
            .send::<_, ()>(Method::GET, "/ads/google/keywords", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Add a keyword. Needs `publish` as well as `ads`.
    pub async fn create_keyword(&self, input: &CreateGoogleKeyword) -> Result<String> {
        self.created(Method::POST, "/ads/google/keywords", input)
            .await
    }

    /// Pause, resume or rebid a keyword. Needs `publish` as well as `ads`.
    pub async fn update_keyword(&self, id: &str, input: &UpdateGoogleKeyword) -> Result<String> {
        self.created(Method::PATCH, &format!("/ads/google/keywords/{id}"), input)
            .await
    }

    /// Remove a keyword. Needs `publish` as well as `ads`.
    pub async fn delete_keyword(&self, id: &str, scope: &GoogleScope) -> Result<Message> {
        self.http
            .send(
                Method::DELETE,
                &format!("/ads/google/keywords/{id}"),
                None,
                Some(scope),
            )
            .await
    }

    /// Ideas from seed keywords, a landing page, or both.
    pub async fn keyword_ideas(
        &self,
        input: &GoogleKeywordIdeas,
    ) -> Result<Vec<GoogleKeywordIdea>> {
        let body: Envelope<Vec<GoogleKeywordIdea>> = self
            .http
            .send(Method::POST, "/ads/google/keyword-ideas", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// Historical metrics for keywords you already have.
    pub async fn keyword_metrics(
        &self,
        input: &GoogleKeywordMetrics,
    ) -> Result<Vec<GoogleKeywordIdea>> {
        let body: Envelope<Vec<GoogleKeywordIdea>> = self
            .http
            .send(
                Method::POST,
                "/ads/google/keyword-metrics",
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// What people actually searched, with the metrics each term earned.
    pub async fn search_terms(
        &self,
        scope: &GoogleScope,
        range: &GoogleDateRange,
    ) -> Result<Vec<GoogleSearchTerm>> {
        let body: Envelope<Vec<GoogleSearchTerm>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/google/search-terms",
                Some(with_range(scope_query(scope), range)),
                None,
            )
            .await?;
        Ok(body.data)
    }

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

    // ── Negative keyword lists ──

    /// The account's negative keyword lists.
    pub async fn negative_keyword_lists(
        &self,
        scope: &GoogleScope,
    ) -> Result<Vec<GoogleSharedSet>> {
        let body: Envelope<Vec<GoogleSharedSet>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/google/negative-keywords",
                Some(scope_query(scope)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Create a negative keyword list. Needs `publish` as well as `ads`.
    pub async fn create_negative_keyword_list(
        &self,
        input: &CreateGoogleNegativeKeywordList,
    ) -> Result<String> {
        self.created(Method::POST, "/ads/google/negative-keywords", input)
            .await
    }

    /// Add keywords to a list; answers how many landed. Needs `publish`.
    pub async fn add_negative_keywords(&self, input: &AddGoogleNegativeKeywords) -> Result<i64> {
        let body: Envelope<Added> = self
            .http
            .send(
                Method::POST,
                "/ads/google/negative-keywords/keywords",
                None,
                Some(input),
            )
            .await?;
        Ok(body.data.added)
    }

    /// Put a list on a campaign. Needs `publish` as well as `ads`.
    pub async fn attach_negative_keyword_list(
        &self,
        input: &AttachGoogleNegativeKeywordList,
    ) -> Result<Message> {
        self.http
            .send(
                Method::POST,
                "/ads/google/negative-keywords/attach",
                None,
                Some(input),
            )
            .await
    }

    // ── Assets ──

    /// Sitelinks, callouts and snippets, with the links that place each one.
    pub async fn assets(&self, scope: &GoogleScope) -> Result<GoogleAssets> {
        let body: Envelope<GoogleAssets> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/google/assets",
                Some(scope_query(scope)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Add an asset to the library. Needs `publish` as well as `ads`.
    pub async fn create_asset(&self, input: &CreateGoogleAsset) -> Result<String> {
        self.created(Method::POST, "/ads/google/assets", input)
            .await
    }

    /// Put an asset under the ads it belongs to. Needs `publish`.
    pub async fn attach_asset(&self, input: &AttachGoogleAsset) -> Result<Message> {
        self.http
            .send(Method::POST, "/ads/google/assets/attach", None, Some(input))
            .await
    }

    /// Remove the links that put an asset under an ad; on Google the asset
    /// itself is permanent. Needs `publish` as well as `ads`.
    pub async fn delete_asset(&self, id: &str, scope: &GoogleScope) -> Result<Message> {
        self.http
            .send(
                Method::DELETE,
                &format!("/ads/google/assets/{id}"),
                None,
                Some(scope),
            )
            .await
    }

    // ── Performance Max asset groups ──

    /// Performance Max asset groups on the account, or on one campaign.
    pub async fn asset_groups(
        &self,
        scope: &GoogleScope,
        campaign_id: Option<&str>,
    ) -> Result<Vec<GoogleAssetGroup>> {
        let mut query = scope_query(scope);
        push_opt(&mut query, "campaign_id", campaign_id);
        let body: Envelope<Vec<GoogleAssetGroup>> = self
            .http
            .send::<_, ()>(Method::GET, "/ads/google/asset-groups", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Create an asset group. Needs `publish` as well as `ads`.
    pub async fn create_asset_group(&self, input: &CreateGoogleAssetGroup) -> Result<String> {
        self.created(Method::POST, "/ads/google/asset-groups", input)
            .await
    }

    /// Rename, pause or resume an asset group. Needs `publish`.
    pub async fn update_asset_group(
        &self,
        id: &str,
        input: &UpdateGoogleAssetGroup,
    ) -> Result<String> {
        self.created(
            Method::PATCH,
            &format!("/ads/google/asset-groups/{id}"),
            input,
        )
        .await
    }

    /// Remove an asset group. Needs `publish` as well as `ads`.
    pub async fn delete_asset_group(&self, id: &str, scope: &GoogleScope) -> Result<Message> {
        self.http
            .send(
                Method::DELETE,
                &format!("/ads/google/asset-groups/{id}"),
                None,
                Some(scope),
            )
            .await
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

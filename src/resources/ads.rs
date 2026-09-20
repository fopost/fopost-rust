//! `client.ads()` — boosts, standalone ads, the campaign tree, creatives,
//! catalogs, audiences, predictions, the public ad archive, insights and lead
//! forms on a Meta Ads connection.
//!
//! Every call needs the `ads` scope. The calls that spend money also need
//! `publish`: [`Ads::boost`], [`Ads::create`], [`Ads::set_status`],
//! [`Ads::delete`], [`Ads::set_statuses`], and every create, update, delete and
//! duplicate on campaigns, ad sets and network ads. A boost, ad or new campaign
//! object starts paused unless `paused` is set to `false`.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    Ad, AdAccountQuery, AdAccountTree, AdActivityLog, AdActivityQuery, AdAudience, AdAudiences,
    AdCampaign, AdConnection, AdCreatives, AdInsightsQuery, AdLabel, AdLabelInput, AdLibraryPage,
    AdLibraryQuery, AdObjectQuery, AdSet, AdSource, AdStatus, AdStatusResult, AdStudy,
    ApplyAdLabel, ArchiveLeadForm, AudienceCreated, AudiencesQuery, AuthorizeMetaAds, BoostPost,
    BoostablePost, CatalogBatchResult, CatalogProducts, CatalogProductsQuery, CreateAd,
    CreateAdSet, CreateAdStudy, CreateAudience, CreateCampaign, CreateCatalog, CreateCreative,
    CreateHighDemandPeriod, CreateLeadForm, CreateNetworkAd, CreateProductFeed,
    CreateReachFrequency, CreateValueRuleSet, Creative, CreativesQuery, EstimateReach, ExternalAd,
    HighDemandPeriod, InsightsQuery, InsightsReport, IosCampaignLimits, LeadFormDetail,
    LeadFormQuery, LeadFormSource, LeadPage, LeadPageSubscribed, LeadsFeedPage, LeadsFeedQuery,
    LeadsPage, LeadsQuery, Message, NetworkAd, PartnershipCreator, PartnershipQuery,
    ProductCatalog, ProductCatalogs, ProductFeed, ProductFeedUpload, ProductSet, ProductSetInput,
    ReachEstimate, ReachFrequencyAction, ReachFrequencyPrediction, ReachFrequencyPredictions,
    RequestPartnership, SetAdStatus, SetAdStatuses, StartFeedUpload, SubscribeLeadPage,
    TargetingOption, TargetingSearch, UpdateAdSet, UpdateAudience, UpdateCampaign, UpdateCatalog,
    UpdateNetworkAd, ValueRuleSet, WriteCatalogProducts,
};

/// Ads.
#[derive(Debug, Clone)]
pub struct Ads<'a> {
    pub(crate) http: &'a HttpClient,
}

fn workspace_query(workspace_id: Option<&str>) -> Query {
    let mut query: Query = Vec::new();
    push_opt(&mut query, "workspace_id", workspace_id);
    query
}

fn object_query(params: &AdObjectQuery) -> Query {
    let mut query = workspace_query(params.workspace_id.as_deref());
    query.push(("connection_id", params.connection_id.clone()));
    query
}

fn account_query(params: &AdAccountQuery) -> Query {
    let mut query = workspace_query(params.workspace_id.as_deref());
    query.push(("connection_id", params.connection_id.clone()));
    query.push(("ad_account_id", params.ad_account_id.clone()));
    query
}

fn partnership_query(params: &PartnershipQuery) -> Query {
    let mut query = workspace_query(params.workspace_id.as_deref());
    query.push(("connection_id", params.connection_id.clone()));
    query.push(("page_id", params.page_id.clone()));
    query
}

#[derive(serde::Deserialize)]
struct Duplicated {
    #[serde(default)]
    id: String,
}

impl Ads<'_> {
    /// Boosts and ads created through FoPost, with insights from their last refresh.
    pub async fn list(&self, workspace_id: Option<&str>) -> Result<Vec<Ad>> {
        let body: Envelope<Vec<Ad>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads",
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Ads on the connected ad accounts that were made elsewhere. Read live, never stored.
    pub async fn external(&self, workspace_id: Option<&str>) -> Result<Vec<ExternalAd>> {
        let body: Envelope<Vec<ExternalAd>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/external",
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Published posts with a delivery on an account a connection reaches.
    pub async fn boostable(&self, workspace_id: Option<&str>) -> Result<Vec<BoostablePost>> {
        let body: Envelope<Vec<BoostablePost>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/boostable",
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The Meta Ads connections in a workspace.
    pub async fn connections(&self, workspace_id: Option<&str>) -> Result<Vec<AdConnection>> {
        let body: Envelope<Vec<AdConnection>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/connections",
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Each connection with the ad accounts and Pages its grant reaches.
    pub async fn sources(&self, workspace_id: Option<&str>) -> Result<Vec<AdSource>> {
        let body: Envelope<Vec<AdSource>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/sources",
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The Meta login URL. The caller finishes it in their own browser, because
    /// the callback checks that the same user came back.
    pub async fn authorize_meta(&self, input: &AuthorizeMetaAds) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct Authorized {
            #[serde(default)]
            url: String,
        }
        let body: Envelope<Authorized> = self
            .http
            .send(
                Method::POST,
                "/ads/connections/meta/authorize",
                None,
                Some(input),
            )
            .await?;
        Ok(body.data.url)
    }

    /// Remove a connection. Also deletes every ad record created through it.
    pub async fn delete_connection(&self, id: &str, workspace_id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(
                Method::DELETE,
                &format!("/ads/connections/{id}"),
                Some(workspace_query(Some(workspace_id))),
                None,
            )
            .await
    }

    /// Promote a post FoPost already published.
    ///
    /// Needs the `publish` scope as well as `ads`. The boost starts paused
    /// unless `paused` is `false`, so nothing spends until someone resumes it.
    pub async fn boost(&self, input: &BoostPost) -> Result<Ad> {
        let body: Envelope<Ad> = self
            .http
            .send(Method::POST, "/ads/boost", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// Create a standalone ad from a creative.
    ///
    /// Needs the `publish` scope as well as `ads`. The ad starts paused unless
    /// `paused` is `false`, so nothing spends until someone resumes it.
    pub async fn create(&self, input: &CreateAd) -> Result<Ad> {
        let body: Envelope<Ad> = self
            .http
            .send(Method::POST, "/ads", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// Read the delivery status and lifetime insights from Meta.
    pub async fn refresh(&self, id: &str, workspace_id: &str) -> Result<Ad> {
        let body: Envelope<Ad> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/ads/{id}/refresh"),
                Some(workspace_query(Some(workspace_id))),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Resume or pause delivery. Needs the `publish` scope as well as `ads`.
    pub async fn set_status(&self, id: &str, workspace_id: &str, status: AdStatus) -> Result<Ad> {
        let body: Envelope<Ad> = self
            .http
            .send(
                Method::PATCH,
                &format!("/ads/{id}"),
                Some(workspace_query(Some(workspace_id))),
                Some(&SetAdStatus { status }),
            )
            .await?;
        Ok(body.data)
    }

    /// End delivery and delete the ad on Meta as well as here. Needs the
    /// `publish` scope as well as `ads`.
    pub async fn delete(&self, id: &str, workspace_id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(
                Method::DELETE,
                &format!("/ads/{id}"),
                Some(workspace_query(Some(workspace_id))),
                None,
            )
            .await
    }

    /// The audiences and pixels on one ad account.
    pub async fn audiences(&self, params: &AudiencesQuery) -> Result<AdAudiences> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        query.push(("ad_account_id", params.ad_account_id.clone()));
        let body: Envelope<AdAudiences> = self
            .http
            .send::<_, ()>(Method::GET, "/ads/audiences", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Build a custom, lookalike or website audience.
    pub async fn create_audience(&self, input: &CreateAudience) -> Result<AudienceCreated> {
        let body: Envelope<AudienceCreated> = self
            .http
            .send(Method::POST, "/ads/audiences", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// Locations, interests, behaviours and income brackets as Meta names them.
    pub async fn search_targeting(&self, params: &TargetingSearch) -> Result<Vec<TargetingOption>> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        query.push(("type", params.search_type.to_string()));
        push_opt(&mut query, "q", params.q.as_ref());
        let body: Envelope<Vec<TargetingOption>> = self
            .http
            .send::<_, ()>(Method::GET, "/ads/targeting/search", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Each connection's Page with its Instant Forms.
    pub async fn lead_forms(&self, workspace_id: Option<&str>) -> Result<Vec<LeadFormSource>> {
        let body: Envelope<Vec<LeadFormSource>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/ads/lead-forms",
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Create an Instant Form on a Page. Returns its id.
    pub async fn create_lead_form(&self, input: &CreateLeadForm) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct Created {
            #[serde(default)]
            id: String,
        }
        let body: Envelope<Created> = self
            .http
            .send(Method::POST, "/ads/lead-forms", None, Some(input))
            .await?;
        Ok(body.data.id)
    }

    /// One page of leads on a form. Pass `next_cursor` back as `after` for the next.
    pub async fn leads(&self, form_id: &str, params: &LeadsQuery) -> Result<LeadsPage> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        query.push(("page_id", params.page_id.clone()));
        push_opt(&mut query, "after", params.after.as_ref());
        let body: Envelope<LeadsPage> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/ads/lead-forms/{form_id}/leads"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    // ─── Goals ──────────────────────────────────────────────────────

    /// The goals this connection's network can run right now. Ask rather than
    /// assume: a goal the deployment is not set up for is absent here and is
    /// refused if you send it anyway.
    pub async fn goals(&self, params: &AdObjectQuery) -> Result<Vec<String>> {
        self.get("/ads/goals", object_query(params)).await
    }

    // ─── Product catalogs ───────────────────────────────────────────

    /// Catalogs the connection's business portfolios reach. Read live, never stored.
    pub async fn catalogs(&self, params: &AdObjectQuery) -> Result<ProductCatalogs> {
        self.get("/ads/catalogs", object_query(params)).await
    }

    /// Created on the connection's business portfolio. Also needs `publish`.
    pub async fn create_catalog(&self, input: &CreateCatalog) -> Result<ProductCatalog> {
        self.post_json("/ads/catalogs", input).await
    }

    /// Reads one catalog.
    pub async fn catalog(&self, id: &str, params: &AdObjectQuery) -> Result<ProductCatalog> {
        self.get(&format!("/ads/catalogs/{id}"), object_query(params))
            .await
    }

    /// Also needs `publish`.
    pub async fn update_catalog(
        &self,
        id: &str,
        params: &AdObjectQuery,
        input: &UpdateCatalog,
    ) -> Result<ProductCatalog> {
        self.change(&format!("/ads/catalogs/{id}"), params, input)
            .await
    }

    /// Deletes every product, feed and set in it. Also needs `publish`.
    pub async fn delete_catalog(&self, id: &str, params: &AdObjectQuery) -> Result<Message> {
        self.remove(&format!("/ads/catalogs/{id}"), params).await
    }

    /// One page of products; pass `next_cursor` back as `after`.
    pub async fn catalog_products(
        &self,
        id: &str,
        params: &CatalogProductsQuery,
    ) -> Result<CatalogProducts> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        push_opt(&mut query, "after", params.after.as_deref());
        self.get(&format!("/ads/catalogs/{id}/products"), query)
            .await
    }

    /// Up to 500 upserts and deletes in one batch, keyed by your own retailer
    /// id. Also needs `publish`.
    pub async fn write_catalog_products(
        &self,
        id: &str,
        input: &WriteCatalogProducts,
    ) -> Result<CatalogBatchResult> {
        self.post_json(&format!("/ads/catalogs/{id}/products"), input)
            .await
    }

    /// The feeds keeping a catalog in step with a hosted product file.
    pub async fn product_feeds(
        &self,
        id: &str,
        params: &AdObjectQuery,
    ) -> Result<Vec<ProductFeed>> {
        self.get(&format!("/ads/catalogs/{id}/feeds"), object_query(params))
            .await
    }

    /// Also needs `publish`.
    pub async fn create_product_feed(
        &self,
        id: &str,
        input: &CreateProductFeed,
    ) -> Result<ProductFeed> {
        self.post_json(&format!("/ads/catalogs/{id}/feeds"), input)
            .await
    }

    /// Also needs `publish`.
    pub async fn delete_product_feed(
        &self,
        id: &str,
        feed_id: &str,
        params: &AdObjectQuery,
    ) -> Result<Message> {
        self.remove(&format!("/ads/catalogs/{id}/feeds/{feed_id}"), params)
            .await
    }

    /// Each run the network made of the feed.
    pub async fn feed_uploads(
        &self,
        id: &str,
        feed_id: &str,
        params: &AdObjectQuery,
    ) -> Result<Vec<ProductFeedUpload>> {
        self.get(
            &format!("/ads/catalogs/{id}/feeds/{feed_id}/uploads"),
            object_query(params),
        )
        .await
    }

    /// Fetches the feed now; the id of the run. Also needs `publish`.
    pub async fn start_feed_upload(
        &self,
        id: &str,
        feed_id: &str,
        input: &StartFeedUpload,
    ) -> Result<String> {
        let body: Envelope<Duplicated> = self
            .http
            .send(
                Method::POST,
                &format!("/ads/catalogs/{id}/feeds/{feed_id}/uploads"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data.id)
    }

    /// A catalog ad runs from a product set, not the whole catalog.
    pub async fn product_sets(&self, id: &str, params: &AdObjectQuery) -> Result<Vec<ProductSet>> {
        self.get(
            &format!("/ads/catalogs/{id}/product-sets"),
            object_query(params),
        )
        .await
    }

    /// Without a `filter` the set is the whole catalog. Also needs `publish`.
    pub async fn create_product_set(
        &self,
        id: &str,
        input: &ProductSetInput,
    ) -> Result<ProductSet> {
        self.post_json(&format!("/ads/catalogs/{id}/product-sets"), input)
            .await
    }

    /// Also needs `publish`.
    pub async fn update_product_set(
        &self,
        id: &str,
        set_id: &str,
        params: &AdObjectQuery,
        input: &ProductSetInput,
    ) -> Result<ProductSet> {
        self.change(
            &format!("/ads/catalogs/{id}/product-sets/{set_id}"),
            params,
            input,
        )
        .await
    }

    /// Also needs `publish`.
    pub async fn delete_product_set(
        &self,
        id: &str,
        set_id: &str,
        params: &AdObjectQuery,
    ) -> Result<Message> {
        self.remove(&format!("/ads/catalogs/{id}/product-sets/{set_id}"), params)
            .await
    }

    // ─── Reach and frequency ────────────────────────────────────────

    /// The reach-and-frequency predictions on one ad account.
    pub async fn reach_frequency(
        &self,
        params: &AdAccountQuery,
    ) -> Result<ReachFrequencyPredictions> {
        self.get("/ads/reach-frequency", account_query(params))
            .await
    }

    /// Prices a flight. Nothing is bought until you reserve it.
    pub async fn create_reach_frequency(
        &self,
        input: &CreateReachFrequency,
    ) -> Result<ReachFrequencyPrediction> {
        self.post_json("/ads/reach-frequency", input).await
    }

    /// Reads one prediction.
    pub async fn reach_frequency_prediction(
        &self,
        id: &str,
        params: &AdAccountQuery,
    ) -> Result<ReachFrequencyPrediction> {
        self.get(&format!("/ads/reach-frequency/{id}"), account_query(params))
            .await
    }

    /// Holds the inventory the prediction priced. Also needs `publish`.
    pub async fn reserve_reach_frequency(
        &self,
        id: &str,
        input: &ReachFrequencyAction,
    ) -> Result<ReachFrequencyPrediction> {
        self.post_json(&format!("/ads/reach-frequency/{id}/reserve"), input)
            .await
    }

    /// Also needs `publish`.
    pub async fn cancel_reach_frequency(
        &self,
        id: &str,
        input: &ReachFrequencyAction,
    ) -> Result<ReachFrequencyPrediction> {
        self.post_json(&format!("/ads/reach-frequency/{id}/cancel"), input)
            .await
    }

    // ─── Ad Library ─────────────────────────────────────────────────

    /// The public ad archive: ads anyone is running, by keyword or by Page.
    /// Read live on every call and stored nowhere, so an ad that stops running
    /// is simply absent from the next search.
    pub async fn library(&self, params: &AdLibraryQuery) -> Result<AdLibraryPage> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        query.push(("countries", params.countries.join(",")));
        push_opt(&mut query, "q", params.q.as_deref());
        if !params.page_ids.is_empty() {
            query.push(("page_ids", params.page_ids.join(",")));
        }
        push_opt(&mut query, "active_status", params.active_status.as_deref());
        if let Some(limit) = params.limit {
            query.push(("limit", limit.to_string()));
        }
        push_opt(&mut query, "after", params.after.as_deref());
        self.get("/ads/library", query).await
    }

    // ─── Partnership ads ────────────────────────────────────────────

    /// Creators who allowlisted this Page to run partnership ads on their posts.
    pub async fn partnership_creators(
        &self,
        params: &PartnershipQuery,
    ) -> Result<Vec<PartnershipCreator>> {
        self.get("/ads/partnership/creators", partnership_query(params))
            .await
    }

    /// Asks a creator for permission; the list as it now stands.
    pub async fn request_partnership(
        &self,
        input: &RequestPartnership,
    ) -> Result<Vec<PartnershipCreator>> {
        self.post_json("/ads/partnership/creators", input).await
    }

    /// Revokes a creator's partnership permission.
    pub async fn revoke_partnership(
        &self,
        creator_id: &str,
        params: &PartnershipQuery,
    ) -> Result<Message> {
        self.http
            .send::<Message, ()>(
                Method::DELETE,
                &format!("/ads/partnership/creators/{creator_id}"),
                Some(partnership_query(params)),
                None,
            )
            .await
    }

    // ─── Ad account settings ────────────────────────────────────────

    /// Who changed what on the ad account, and when.
    pub async fn account_activity(&self, params: &AdActivityQuery) -> Result<AdActivityLog> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        query.push(("ad_account_id", params.ad_account_id.clone()));
        push_opt(&mut query, "since", params.since.as_deref());
        push_opt(&mut query, "until", params.until.as_deref());
        self.get("/ads/account/activity", query).await
    }

    /// The labels on an ad account.
    pub async fn labels(&self, params: &AdAccountQuery) -> Result<Vec<AdLabel>> {
        self.get("/ads/account/labels", account_query(params)).await
    }

    /// Creates a label.
    pub async fn create_label(&self, input: &AdLabelInput) -> Result<AdLabel> {
        self.post_json("/ads/account/labels", input).await
    }

    /// Renames a label.
    pub async fn update_label(
        &self,
        id: &str,
        params: &AdObjectQuery,
        input: &AdLabelInput,
    ) -> Result<AdLabel> {
        self.change(&format!("/ads/account/labels/{id}"), params, input)
            .await
    }

    /// Deletes a label.
    pub async fn delete_label(&self, id: &str, params: &AdAccountQuery) -> Result<Message> {
        self.remove_account(&format!("/ads/account/labels/{id}"), params)
            .await
    }

    /// Keeps whatever labels the object already carries.
    pub async fn apply_label(&self, id: &str, input: &ApplyAdLabel) -> Result<Message> {
        self.http
            .send(
                Method::POST,
                &format!("/ads/account/labels/{id}/apply"),
                None,
                Some(input),
            )
            .await
    }

    /// The A/B studies on an ad account.
    pub async fn studies(&self, params: &AdAccountQuery) -> Result<Vec<AdStudy>> {
        self.get("/ads/account/studies", account_query(params))
            .await
    }

    /// Splits traffic evenly across the cells for the length of the flight.
    pub async fn create_study(&self, input: &CreateAdStudy) -> Result<AdStudy> {
        self.post_json("/ads/account/studies", input).await
    }

    /// Reads one A/B study.
    pub async fn study(&self, id: &str, params: &AdAccountQuery) -> Result<AdStudy> {
        self.get(&format!("/ads/account/studies/{id}"), account_query(params))
            .await
    }

    /// Deletes an A/B study.
    pub async fn delete_study(&self, id: &str, params: &AdAccountQuery) -> Result<Message> {
        self.remove_account(&format!("/ads/account/studies/{id}"), params)
            .await
    }

    /// How many iOS 14 campaigns the account may run at once, per app.
    pub async fn ios_campaign_limits(
        &self,
        params: &AdAccountQuery,
    ) -> Result<Vec<IosCampaignLimits>> {
        self.get("/ads/account/ios-limits", account_query(params))
            .await
    }

    /// The high-demand windows declared on an ad account.
    pub async fn high_demand_periods(
        &self,
        params: &AdAccountQuery,
    ) -> Result<Vec<HighDemandPeriod>> {
        self.get("/ads/account/high-demand-periods", account_query(params))
            .await
    }

    /// Tells the network to expect heavier spend over a window, so pacing
    /// allows for it.
    pub async fn create_high_demand_period(
        &self,
        input: &CreateHighDemandPeriod,
    ) -> Result<HighDemandPeriod> {
        self.post_json("/ads/account/high-demand-periods", input)
            .await
    }

    /// Deletes a high-demand window.
    pub async fn delete_high_demand_period(
        &self,
        id: &str,
        params: &AdAccountQuery,
    ) -> Result<Message> {
        self.remove_account(&format!("/ads/account/high-demand-periods/{id}"), params)
            .await
    }

    /// The value rule sets on an ad account.
    pub async fn value_rule_sets(&self, params: &AdAccountQuery) -> Result<Vec<ValueRuleSet>> {
        self.get("/ads/account/value-rule-sets", account_query(params))
            .await
    }

    /// Weights conversions so some audiences count for more than others.
    pub async fn create_value_rule_set(&self, input: &CreateValueRuleSet) -> Result<ValueRuleSet> {
        self.post_json("/ads/account/value-rule-sets", input).await
    }

    /// Deletes a value rule set.
    pub async fn delete_value_rule_set(
        &self,
        id: &str,
        params: &AdAccountQuery,
    ) -> Result<Message> {
        self.remove_account(&format!("/ads/account/value-rule-sets/{id}"), params)
            .await
    }

    async fn post_json<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        input: &B,
    ) -> Result<T> {
        let body: Envelope<T> = self
            .http
            .send(Method::POST, path, None, Some(input))
            .await?;
        Ok(body.data)
    }

    async fn change<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        params: &AdObjectQuery,
        input: &B,
    ) -> Result<T> {
        let body: Envelope<T> = self
            .http
            .send(Method::PATCH, path, Some(object_query(params)), Some(input))
            .await?;
        Ok(body.data)
    }

    async fn remove_account(&self, path: &str, params: &AdAccountQuery) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, path, Some(account_query(params)), None)
            .await
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str, query: Query) -> Result<T> {
        let body: Envelope<T> = self
            .http
            .send::<_, ()>(Method::GET, path, Some(query), None)
            .await?;
        Ok(body.data)
    }

    async fn remove(&self, path: &str, params: &AdObjectQuery) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, path, Some(object_query(params)), None)
            .await
    }

    async fn duplicate(
        &self,
        path: &str,
        params: &AdObjectQuery,
        paused: Option<bool>,
    ) -> Result<String> {
        let input = paused.map(|paused| serde_json::json!({ "paused": paused }));
        let body: Envelope<Duplicated> = self
            .http
            .send(
                Method::POST,
                path,
                Some(object_query(params)),
                input.as_ref(),
            )
            .await?;
        Ok(body.data.id)
    }

    /// Every campaign on an ad account with its ad sets and ads, whoever made
    /// them. Read live, never stored.
    pub async fn tree(&self, ad_account_id: &str, params: &AdObjectQuery) -> Result<AdAccountTree> {
        self.get(
            &format!("/ads/accounts/{ad_account_id}/tree"),
            object_query(params),
        )
        .await
    }

    /// Create a campaign. Needs the `publish` scope as well as `ads`.
    pub async fn create_campaign(&self, input: &CreateCampaign) -> Result<AdCampaign> {
        let body: Envelope<AdCampaign> = self
            .http
            .send(Method::POST, "/ads/campaigns", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// One campaign.
    pub async fn campaign(&self, id: &str, params: &AdObjectQuery) -> Result<AdCampaign> {
        self.get(&format!("/ads/campaigns/{id}"), object_query(params))
            .await
    }

    /// Rename, pause or resume a campaign. Needs the `publish` scope as well as `ads`.
    pub async fn update_campaign(
        &self,
        id: &str,
        params: &AdObjectQuery,
        input: &UpdateCampaign,
    ) -> Result<AdCampaign> {
        let body: Envelope<AdCampaign> = self
            .http
            .send(
                Method::PATCH,
                &format!("/ads/campaigns/{id}"),
                Some(object_query(params)),
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Delete a campaign on Meta. Needs the `publish` scope as well as `ads`.
    pub async fn delete_campaign(&self, id: &str, params: &AdObjectQuery) -> Result<Message> {
        self.remove(&format!("/ads/campaigns/{id}"), params).await
    }

    /// Copy a campaign; returns the copy's Meta id. The copy starts paused
    /// unless `paused` is `Some(false)`. Needs the `publish` scope as well as `ads`.
    pub async fn duplicate_campaign(
        &self,
        id: &str,
        params: &AdObjectQuery,
        paused: Option<bool>,
    ) -> Result<String> {
        self.duplicate(&format!("/ads/campaigns/{id}/duplicate"), params, paused)
            .await
    }

    /// Create an ad set in a campaign. Needs the `publish` scope as well as `ads`.
    pub async fn create_ad_set(&self, input: &CreateAdSet) -> Result<AdSet> {
        let body: Envelope<AdSet> = self
            .http
            .send(Method::POST, "/ads/ad-sets", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// One ad set.
    pub async fn ad_set(&self, id: &str, params: &AdObjectQuery) -> Result<AdSet> {
        self.get(&format!("/ads/ad-sets/{id}"), object_query(params))
            .await
    }

    /// Change an ad set's name, status, budget, end or targeting. Needs the
    /// `publish` scope as well as `ads`.
    pub async fn update_ad_set(
        &self,
        id: &str,
        params: &AdObjectQuery,
        input: &UpdateAdSet,
    ) -> Result<AdSet> {
        let body: Envelope<AdSet> = self
            .http
            .send(
                Method::PATCH,
                &format!("/ads/ad-sets/{id}"),
                Some(object_query(params)),
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Delete an ad set on Meta. Needs the `publish` scope as well as `ads`.
    pub async fn delete_ad_set(&self, id: &str, params: &AdObjectQuery) -> Result<Message> {
        self.remove(&format!("/ads/ad-sets/{id}"), params).await
    }

    /// Copy an ad set; returns the copy's Meta id. Needs the `publish` scope as
    /// well as `ads`.
    pub async fn duplicate_ad_set(
        &self,
        id: &str,
        params: &AdObjectQuery,
        paused: Option<bool>,
    ) -> Result<String> {
        self.duplicate(&format!("/ads/ad-sets/{id}/duplicate"), params, paused)
            .await
    }

    /// Create an ad inside an ad set from a creative. Unlike [`Ads::create`] it
    /// builds no campaign. Needs the `publish` scope as well as `ads`.
    pub async fn create_network_ad(&self, input: &CreateNetworkAd) -> Result<NetworkAd> {
        let body: Envelope<NetworkAd> = self
            .http
            .send(Method::POST, "/ads/ads", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// One ad, by its Meta id.
    pub async fn network_ad(&self, id: &str, params: &AdObjectQuery) -> Result<NetworkAd> {
        self.get(&format!("/ads/ads/{id}"), object_query(params))
            .await
    }

    /// Rename, pause, resume or swap the creative of an ad. Needs the
    /// `publish` scope as well as `ads`.
    pub async fn update_network_ad(
        &self,
        id: &str,
        params: &AdObjectQuery,
        input: &UpdateNetworkAd,
    ) -> Result<NetworkAd> {
        let body: Envelope<NetworkAd> = self
            .http
            .send(
                Method::PATCH,
                &format!("/ads/ads/{id}"),
                Some(object_query(params)),
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Delete an ad on Meta. Needs the `publish` scope as well as `ads`.
    pub async fn delete_network_ad(&self, id: &str, params: &AdObjectQuery) -> Result<Message> {
        self.remove(&format!("/ads/ads/{id}"), params).await
    }

    /// Copy an ad; returns the copy's Meta id. Needs the `publish` scope as
    /// well as `ads`.
    pub async fn duplicate_network_ad(
        &self,
        id: &str,
        params: &AdObjectQuery,
        paused: Option<bool>,
    ) -> Result<String> {
        self.duplicate(&format!("/ads/ads/{id}/duplicate"), params, paused)
            .await
    }

    /// Pause or resume up to 50 campaigns, ad sets and ads; each succeeds or
    /// fails on its own. Needs the `publish` scope as well as `ads`.
    pub async fn set_statuses(&self, input: &SetAdStatuses) -> Result<Vec<AdStatusResult>> {
        let body: Envelope<Vec<AdStatusResult>> = self
            .http
            .send(Method::POST, "/ads/status", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// An ad account's creative library.
    pub async fn creatives(&self, params: &CreativesQuery) -> Result<AdCreatives> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        query.push(("ad_account_id", params.ad_account_id.clone()));
        self.get("/ads/creatives", query).await
    }

    /// Add an image, video or carousel creative to an ad account.
    pub async fn create_creative(&self, input: &CreateCreative) -> Result<Creative> {
        let body: Envelope<Creative> = self
            .http
            .send(Method::POST, "/ads/creatives", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// One creative.
    pub async fn creative(&self, id: &str, params: &AdObjectQuery) -> Result<Creative> {
        self.get(&format!("/ads/creatives/{id}"), object_query(params))
            .await
    }

    /// Delete a creative on Meta.
    pub async fn delete_creative(&self, id: &str, params: &AdObjectQuery) -> Result<Message> {
        self.remove(&format!("/ads/creatives/{id}"), params).await
    }

    /// One audience.
    pub async fn audience(&self, id: &str, params: &AdObjectQuery) -> Result<AdAudience> {
        self.get(&format!("/ads/audiences/{id}"), object_query(params))
            .await
    }

    /// Rename an audience or change its description.
    pub async fn update_audience(
        &self,
        id: &str,
        params: &AdObjectQuery,
        input: &UpdateAudience,
    ) -> Result<AdAudience> {
        let body: Envelope<AdAudience> = self
            .http
            .send(
                Method::PATCH,
                &format!("/ads/audiences/{id}"),
                Some(object_query(params)),
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Delete an audience on Meta.
    pub async fn delete_audience(&self, id: &str, params: &AdObjectQuery) -> Result<Message> {
        self.remove(&format!("/ads/audiences/{id}"), params).await
    }

    /// Add emails to a custom audience, hashed before they leave the API.
    /// Returns how many were sent to Meta.
    pub async fn add_audience_users<I, S>(
        &self,
        id: &str,
        params: &AdObjectQuery,
        emails: I,
    ) -> Result<u64>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        #[derive(serde::Deserialize)]
        struct Added {
            #[serde(default)]
            added: u64,
        }
        let emails: Vec<String> = emails.into_iter().map(Into::into).collect();
        let body: Envelope<Added> = self
            .http
            .send(
                Method::POST,
                &format!("/ads/audiences/{id}/users"),
                Some(object_query(params)),
                Some(&serde_json::json!({ "emails": emails })),
            )
            .await?;
        Ok(body.data.added)
    }

    /// How many people a targeting reaches.
    pub async fn estimate_reach(&self, input: &EstimateReach) -> Result<ReachEstimate> {
        let body: Envelope<ReachEstimate> = self
            .http
            .send(Method::POST, "/ads/reach-estimate", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// Delivery for any Meta object over a date range, optionally split by
    /// one breakdown and by day.
    pub async fn insights(&self, params: &InsightsQuery) -> Result<InsightsReport> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        query.push(("object_id", params.object_id.clone()));
        query.push(("since", params.since.clone()));
        query.push(("until", params.until.clone()));
        push_opt(&mut query, "breakdown", params.breakdown.as_ref());
        push_opt(&mut query, "daily", params.daily);
        self.get("/ads/insights", query).await
    }

    /// Delivery over a date range for a boost or ad created through FoPost,
    /// by its FoPost id.
    pub async fn ad_insights(&self, id: &str, params: &AdInsightsQuery) -> Result<InsightsReport> {
        let mut query = workspace_query(Some(&params.workspace_id));
        query.push(("since", params.since.clone()));
        query.push(("until", params.until.clone()));
        push_opt(&mut query, "breakdown", params.breakdown.as_ref());
        push_opt(&mut query, "daily", params.daily);
        self.get(&format!("/ads/{id}/insights"), query).await
    }

    /// One Instant Form with its settings.
    pub async fn lead_form(&self, form_id: &str, params: &LeadFormQuery) -> Result<LeadFormDetail> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        query.push(("connection_id", params.connection_id.clone()));
        query.push(("page_id", params.page_id.clone()));
        self.get(&format!("/ads/lead-forms/{form_id}"), query).await
    }

    /// Stop an Instant Form collecting. Its leads stay readable.
    pub async fn archive_lead_form(
        &self,
        form_id: &str,
        input: &ArchiveLeadForm,
    ) -> Result<LeadFormDetail> {
        let body: Envelope<LeadFormDetail> = self
            .http
            .send(
                Method::POST,
                &format!("/ads/lead-forms/{form_id}/archive"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Leads collected from subscribed Pages, newest first. Pass `next_cursor`
    /// back as `cursor` for the next page.
    pub async fn leads_feed(&self, params: &LeadsFeedQuery) -> Result<LeadsFeedPage> {
        let mut query = workspace_query(params.workspace_id.as_deref());
        push_opt(&mut query, "form_id", params.form_id.as_ref());
        push_opt(&mut query, "page_id", params.page_id.as_ref());
        push_opt(&mut query, "cursor", params.cursor.as_ref());
        push_opt(&mut query, "limit", params.limit);
        self.get("/ads/leads", query).await
    }

    /// The Pages whose leads are collected into the feed.
    pub async fn lead_pages(&self, workspace_id: Option<&str>) -> Result<Vec<LeadPage>> {
        self.get("/ads/lead-pages", workspace_query(workspace_id))
            .await
    }

    /// Start collecting a Page's leads and backfill its most recent ones.
    pub async fn subscribe_lead_page(
        &self,
        input: &SubscribeLeadPage,
    ) -> Result<LeadPageSubscribed> {
        let body: Envelope<LeadPageSubscribed> = self
            .http
            .send(Method::POST, "/ads/lead-pages", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// Stop collecting a Page's leads.
    pub async fn unsubscribe_lead_page(
        &self,
        page_id: &str,
        params: &AdObjectQuery,
    ) -> Result<Message> {
        self.remove(&format!("/ads/lead-pages/{page_id}"), params)
            .await
    }
}

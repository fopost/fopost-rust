//! `client.ads()` — boosts, standalone ads, the campaign tree, creatives,
//! audiences, insights and lead forms on an ad connection.
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
    Ad, AdAccountTree, AdAudience, AdAudiences, AdCampaign, AdConnection, AdCreatives,
    AdInsightsQuery, AdObjectQuery, AdSet, AdSource, AdStatus, AdStatusResult, ArchiveLeadForm,
    AudienceCreated, AudiencesQuery, AuthorizeAds, BoostPost, BoostablePost, CreateAd, CreateAdSet,
    CreateAudience, CreateCampaign, CreateCreative, CreateLeadForm, CreateNetworkAd, Creative,
    CreativesQuery, EstimateReach, ExternalAd, InsightsQuery, InsightsReport, LeadFormDetail,
    LeadFormQuery, LeadFormSource, LeadPage, LeadPageSubscribed, LeadsFeedPage, LeadsFeedQuery,
    LeadsPage, LeadsQuery, Message, NetworkAd, ReachEstimate, SetAdStatus, SetAdStatuses,
    SubscribeLeadPage, TargetingOption, TargetingSearch, UpdateAdSet, UpdateAudience,
    UpdateCampaign, UpdateNetworkAd,
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

    /// The ad connections in a workspace.
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

    /// The network's login URL. The caller finishes it in their own browser,
    /// because the callback checks that the same user came back. A network that
    /// is not available on the deployment answers 503.
    pub async fn authorize(&self, input: &AuthorizeAds) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct Authorized {
            #[serde(default)]
            url: String,
        }
        let provider = if input.provider.is_empty() {
            "meta"
        } else {
            &input.provider
        };
        let body: Envelope<Authorized> = self
            .http
            .send(
                Method::POST,
                &format!("/ads/connections/{provider}/authorize"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data.url)
    }

    /// The Meta login URL.
    #[deprecated(note = "use authorize, which takes a provider")]
    pub async fn authorize_meta(&self, input: &AuthorizeAds) -> Result<String> {
        self.authorize(input).await
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

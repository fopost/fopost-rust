//! `client.ads()` — boosts, standalone ads, audiences and lead forms on a Meta
//! Ads connection.
//!
//! Every call needs the `ads` scope. The four that spend money — [`Ads::boost`],
//! [`Ads::create`], [`Ads::set_status`] and [`Ads::delete`] — also need `publish`.
//! A boost or ad starts paused unless `paused` is set to `false` on the body.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    Ad, AdAudiences, AdConnection, AdSource, AdStatus, AudienceCreated, AudiencesQuery,
    AuthorizeMetaAds, BoostPost, BoostablePost, CreateAd, CreateAudience, CreateLeadForm,
    ExternalAd, LeadFormSource, LeadsPage, LeadsQuery, Message, SetAdStatus, TargetingOption,
    TargetingSearch,
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
}

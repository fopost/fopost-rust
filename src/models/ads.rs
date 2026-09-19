//! Ads — boosts, standalone ads, the campaign tree, creatives, audiences,
//! insights and lead forms on a Meta Ads connection.
//!
//! Amounts are in the ad account's currency, in minor units: `1500` is $15.00
//! on a USD account.

use serde::{Deserialize, Serialize};

use super::common::string_enum;

string_enum! {
    /// What an ad optimises for.
    pub enum AdGoal {
        Engagement => "engagement",
        Traffic => "traffic",
        Awareness => "awareness",
        VideoViews => "video_views",
    }
}

string_enum! {
    /// How a budget is spent.
    pub enum AdBudgetType {
        Daily => "daily",
        Lifetime => "lifetime",
    }
}

string_enum! {
    /// Gender targeting.
    pub enum AdGender {
        All => "all",
        Male => "male",
        Female => "female",
    }
}

// `string_enum!` fixes the derive list, so Default is written out.
#[allow(clippy::derivable_impls)]
impl Default for AdGender {
    fn default() -> Self {
        AdGender::All
    }
}

string_enum! {
    /// Whether an ad promotes a published post or carries its own creative.
    pub enum AdKind {
        Boost => "boost",
        Ad => "ad",
    }
}

string_enum! {
    /// What was asked of an ad: delivering or not.
    pub enum AdStatus {
        Active => "active",
        Paused => "paused",
    }
}

string_enum! {
    /// A targeting location below country level.
    pub enum AdLocationType {
        Region => "region",
        City => "city",
        Zip => "zip",
        GeoMarket => "geo_market",
    }
}

string_enum! {
    /// What `GET /ads/targeting/search` looks up.
    pub enum TargetingSearchType {
        Country => "country",
        Region => "region",
        City => "city",
        Zip => "zip",
        Metro => "metro",
        Interest => "interest",
        Behavior => "behavior",
        Income => "income",
    }
}

string_enum! {
    /// How a Meta Ads connection is authorised.
    pub enum MetaLoginMethod {
        /// Facebook Login for Business. The default.
        Business => "business",
        /// A personal login.
        User => "user",
    }
}

string_enum! {
    /// A question on an Instant Form.
    pub enum LeadQuestion {
        Email => "EMAIL",
        FullName => "FULL_NAME",
        Phone => "PHONE",
    }
}

/// An interest, behaviour or income bracket as Meta names it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdTargetingItem {
    pub id: String,
    pub name: String,
}

impl AdTargetingItem {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

/// A location below country level, from `GET /ads/targeting/search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdTargetingLocation {
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub location_type: AdLocationType,
}

impl AdTargetingLocation {
    pub fn new(
        key: impl Into<String>,
        name: impl Into<String>,
        location_type: AdLocationType,
    ) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            location_type,
        }
    }
}

/// Who an ad is shown to. At least one country or one location is required.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdTargeting {
    /// ISO 3166-1 alpha-2 codes.
    #[serde(default)]
    pub countries: Vec<String>,
    #[serde(default)]
    pub age_min: u32,
    #[serde(default)]
    pub age_max: u32,
    #[serde(default)]
    pub gender: AdGender,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audience_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locations: Vec<AdTargetingLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interests: Vec<AdTargetingItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub behaviors: Vec<AdTargetingItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub income: Vec<AdTargetingItem>,
}

impl AdTargeting {
    /// The required fields: countries and an age band, shown to everyone.
    pub fn new<I, S>(countries: I, age_min: u32, age_max: u32) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            countries: countries.into_iter().map(Into::into).collect(),
            age_min,
            age_max,
            ..Self::default()
        }
    }

    pub fn gender(mut self, gender: AdGender) -> Self {
        self.gender = gender;
        self
    }

    pub fn audiences<I, S>(mut self, audience_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.audience_ids = audience_ids.into_iter().map(Into::into).collect();
        self
    }

    pub fn locations(mut self, locations: impl IntoIterator<Item = AdTargetingLocation>) -> Self {
        self.locations = locations.into_iter().collect();
        self
    }

    pub fn interests(mut self, interests: impl IntoIterator<Item = AdTargetingItem>) -> Self {
        self.interests = interests.into_iter().collect();
        self
    }

    pub fn behaviors(mut self, behaviors: impl IntoIterator<Item = AdTargetingItem>) -> Self {
        self.behaviors = behaviors.into_iter().collect();
        self
    }

    pub fn income(mut self, income: impl IntoIterator<Item = AdTargetingItem>) -> Self {
        self.income = income.into_iter().collect();
        self
    }
}

/// What an ad may spend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdBudget {
    /// Ad account currency, minor units.
    pub minor: u64,
    #[serde(rename = "type")]
    pub budget_type: AdBudgetType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_at: Option<String>,
}

impl AdBudget {
    /// `minor` per day, open-ended.
    pub fn daily(minor: u64) -> Self {
        Self {
            minor,
            budget_type: AdBudgetType::Daily,
            end_at: None,
        }
    }

    /// `minor` in total, until `end_at` (ISO 8601).
    pub fn lifetime(minor: u64, end_at: impl Into<String>) -> Self {
        Self {
            minor,
            budget_type: AdBudgetType::Lifetime,
            end_at: Some(end_at.into()),
        }
    }
}

/// Lifetime numbers from the last refresh.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdInsights {
    #[serde(default)]
    pub impressions: u64,
    #[serde(default)]
    pub reach: u64,
    #[serde(default)]
    pub clicks: u64,
    /// Ad account currency, minor units.
    #[serde(default)]
    pub spend_minor: u64,
}

/// The creative of a standalone ad.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdCreative {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub headline: Option<String>,
    #[serde(default)]
    pub destination_url: Option<String>,
    #[serde(default)]
    pub media_url: Option<String>,
    #[serde(default)]
    pub url_tags: Option<String>,
}

/// A boost or ad created through FoPost.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ad {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub kind: AdKind,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub goal: Option<AdGoal>,
    /// What was asked for: `active` or `paused`.
    #[serde(default)]
    pub status: Option<AdStatus>,
    /// Meta's own delivery status, from the last refresh.
    #[serde(default)]
    pub effective_status: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    /// The connected account a boost was built from.
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub ad_account_id: Option<String>,
    /// The FoPost post a boost promotes.
    #[serde(default)]
    pub source_post_id: Option<String>,
    #[serde(default)]
    pub budget_minor: u64,
    #[serde(default)]
    pub budget_type: Option<AdBudgetType>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default)]
    pub targeting: AdTargeting,
    #[serde(default)]
    pub creative: Option<AdCreative>,
    #[serde(default)]
    pub insights: Option<AdInsights>,
    #[serde(default)]
    pub insights_at: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// An ad on a connected ad account that was made elsewhere. Read live, never stored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAd {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub effective_status: Option<String>,
    #[serde(default)]
    pub campaign_id: Option<String>,
    #[serde(default)]
    pub campaign_name: Option<String>,
    #[serde(default)]
    pub objective: Option<String>,
    #[serde(default)]
    pub budget_minor: Option<u64>,
    #[serde(default)]
    pub budget_type: Option<AdBudgetType>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub ad_account_id: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// A Meta Ads connection.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdConnection {
    pub id: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub auth_type: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub business_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// An ad account a connection reaches.
#[derive(Debug, Clone, Deserialize)]
pub struct AdAccountRef {
    /// `act_…`
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub status: Option<i64>,
}

/// A Facebook Page a connection reaches.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdPageRef {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub instagram_user_id: Option<String>,
}

/// A connection with the ad accounts and Pages its grant reaches.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdSource {
    pub connection_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub ad_accounts: Vec<AdAccountRef>,
    #[serde(default)]
    pub pages: Vec<AdPageRef>,
    /// Set when Meta refused the listing, usually a revoked grant.
    #[serde(default)]
    pub error: Option<String>,
}

/// One delivery of a boostable post.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostableDelivery {
    pub account_id: String,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub external_url: Option<String>,
    #[serde(default)]
    pub posted_at: Option<String>,
}

/// A published post with a delivery on an account a connection reaches.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostablePost {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub deliveries: Vec<BoostableDelivery>,
}

/// A custom, lookalike or website audience on an ad account.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdAudience {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub size_lower: Option<u64>,
    #[serde(default)]
    pub size_upper: Option<u64>,
    #[serde(default)]
    pub delivery_status: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A pixel on an ad account, the seed of a website audience.
#[derive(Debug, Clone, Deserialize)]
pub struct AdPixel {
    pub id: String,
    #[serde(default)]
    pub name: String,
}

/// The answer to `GET /ads/audiences`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdAudiences {
    #[serde(default)]
    pub audiences: Vec<AdAudience>,
    #[serde(default)]
    pub pixels: Vec<AdPixel>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// A location, interest, behaviour or income bracket as Meta names it.
#[derive(Debug, Clone, Deserialize)]
pub struct TargetingOption {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub detail: Option<String>,
}

/// An Instant Form on a Page.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadForm {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub leads_count: u64,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub questions: Vec<String>,
}

/// A connection's Page with its Instant Forms.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadFormSource {
    pub connection_id: String,
    #[serde(default)]
    pub connection_name: Option<String>,
    #[serde(default)]
    pub page_id: Option<String>,
    #[serde(default)]
    pub page_name: Option<String>,
    #[serde(default)]
    pub forms: Vec<LeadForm>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// One answer on a lead.
#[derive(Debug, Clone, Deserialize)]
pub struct LeadField {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub values: Vec<String>,
}

/// A submission on an Instant Form.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lead {
    pub id: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub fields: Vec<LeadField>,
    #[serde(default)]
    pub ad_name: Option<String>,
    #[serde(default)]
    pub campaign_name: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub is_organic: bool,
}

/// One page of leads. Pass `next_cursor` back as `after` for the next.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadsPage {
    #[serde(default)]
    pub leads: Vec<Lead>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// The body of `POST /ads/connections/meta/authorize`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeMetaAds {
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<MetaLoginMethod>,
    /// Dashboard path to land on after Meta redirects back.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_to: Option<String>,
}

impl AuthorizeMetaAds {
    pub fn new(workspace_id: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            method: None,
            return_to: None,
        }
    }

    pub fn method(mut self, method: MetaLoginMethod) -> Self {
        self.method = Some(method);
        self
    }

    pub fn return_to(mut self, return_to: impl Into<String>) -> Self {
        self.return_to = Some(return_to.into());
        self
    }
}

/// The body of `POST /ads/boost`: promote a post FoPost already published.
///
/// Needs the `publish` scope as well as `ads`. The boost starts paused unless
/// `paused` is set to `false`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostPost {
    pub workspace_id: String,
    /// A Meta Ads connection in the workspace.
    pub connection_id: String,
    /// `act_…`, from `sources()`.
    pub ad_account_id: String,
    /// A published FoPost post, from `boostable()`.
    pub post_id: String,
    /// The account the post was delivered to.
    pub account_id: String,
    /// What the campaign is called on Meta.
    pub name: String,
    pub goal: AdGoal,
    pub budget: AdBudget,
    pub targeting: AdTargeting,
    /// Default `true`: created paused, spending nothing until resumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
}

impl BoostPost {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        ad_account_id: impl Into<String>,
        post_id: impl Into<String>,
        account_id: impl Into<String>,
        name: impl Into<String>,
        goal: AdGoal,
        budget: AdBudget,
        targeting: AdTargeting,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            ad_account_id: ad_account_id.into(),
            post_id: post_id.into(),
            account_id: account_id.into(),
            name: name.into(),
            goal,
            budget,
            targeting,
            paused: None,
        }
    }

    /// `false` to start delivering at once.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = Some(paused);
        self
    }
}

/// The body of `POST /ads`: a standalone ad from a creative.
///
/// Needs the `publish` scope as well as `ads`. The ad starts paused unless
/// `paused` is set to `false`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAd {
    pub workspace_id: String,
    /// A Meta Ads connection in the workspace.
    pub connection_id: String,
    /// `act_…`, from `sources()`.
    pub ad_account_id: String,
    /// Facebook Page id, from `sources()`.
    pub page_id: String,
    /// What the campaign is called on Meta.
    pub name: String,
    pub goal: AdGoal,
    pub budget: AdBudget,
    pub targeting: AdTargeting,
    /// Up to 125 characters.
    pub text: String,
    /// Up to 40 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_url: Option<String>,
    /// A media library asset url.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    /// Query string appended to every link in the ad, e.g. `utm_source=meta&utm_medium=paid`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_tags: Option<String>,
    /// Default `true`: created paused, spending nothing until resumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
}

impl CreateAd {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        ad_account_id: impl Into<String>,
        page_id: impl Into<String>,
        name: impl Into<String>,
        goal: AdGoal,
        budget: AdBudget,
        targeting: AdTargeting,
        text: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            ad_account_id: ad_account_id.into(),
            page_id: page_id.into(),
            name: name.into(),
            goal,
            budget,
            targeting,
            text: text.into(),
            headline: None,
            destination_url: None,
            media_url: None,
            url_tags: None,
            paused: None,
        }
    }

    pub fn headline(mut self, headline: impl Into<String>) -> Self {
        self.headline = Some(headline.into());
        self
    }

    pub fn destination_url(mut self, destination_url: impl Into<String>) -> Self {
        self.destination_url = Some(destination_url.into());
        self
    }

    pub fn media_url(mut self, media_url: impl Into<String>) -> Self {
        self.media_url = Some(media_url.into());
        self
    }

    pub fn url_tags(mut self, url_tags: impl Into<String>) -> Self {
        self.url_tags = Some(url_tags.into());
        self
    }

    /// `false` to start delivering at once.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = Some(paused);
        self
    }
}

/// The body of `PATCH /ads/{id}`.
#[derive(Debug, Clone, Serialize)]
pub struct SetAdStatus {
    pub status: AdStatus,
}

/// How an audience is built. Serialised with a `subtype` tag.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "subtype")]
pub enum AudienceSpec {
    /// A customer list. Emails are hashed before they leave the API.
    #[serde(rename = "CUSTOM")]
    Custom {
        #[serde(skip_serializing_if = "Vec::is_empty")]
        emails: Vec<String>,
    },
    /// People like an existing audience, in one country.
    #[serde(rename = "LOOKALIKE")]
    Lookalike {
        #[serde(rename = "originAudienceId")]
        origin_audience_id: String,
        /// ISO 3166-1 alpha-2.
        country: String,
        /// 0.01 to 0.2; defaults to 0.01.
        #[serde(skip_serializing_if = "Option::is_none")]
        ratio: Option<f64>,
    },
    /// Visitors a pixel saw.
    #[serde(rename = "WEBSITE")]
    Website {
        #[serde(rename = "pixelId")]
        pixel_id: String,
        /// 1 to 180; defaults to 30.
        #[serde(rename = "retentionDays", skip_serializing_if = "Option::is_none")]
        retention_days: Option<u32>,
        #[serde(rename = "urlContains", skip_serializing_if = "Option::is_none")]
        url_contains: Option<String>,
    },
}

/// The body of `POST /ads/audiences`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAudience {
    pub workspace_id: String,
    pub connection_id: String,
    /// `act_…`
    pub ad_account_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub spec: AudienceSpec,
}

impl CreateAudience {
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        ad_account_id: impl Into<String>,
        name: impl Into<String>,
        spec: AudienceSpec,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            ad_account_id: ad_account_id.into(),
            name: name.into(),
            description: None,
            spec,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// The answer to `POST /ads/audiences`.
#[derive(Debug, Clone, Deserialize)]
pub struct AudienceCreated {
    #[serde(default)]
    pub id: String,
    /// Emails accepted by Meta, for a custom audience.
    #[serde(default)]
    pub added: u64,
}

/// Filters for `GET /ads/audiences`.
#[derive(Debug, Clone)]
pub struct AudiencesQuery {
    pub connection_id: String,
    /// `act_…`
    pub ad_account_id: String,
    pub workspace_id: Option<String>,
}

impl AudiencesQuery {
    pub fn new(connection_id: impl Into<String>, ad_account_id: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            ad_account_id: ad_account_id.into(),
            workspace_id: None,
        }
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

/// Filters for `GET /ads/targeting/search`.
#[derive(Debug, Clone)]
pub struct TargetingSearch {
    pub connection_id: String,
    pub search_type: TargetingSearchType,
    pub q: Option<String>,
    pub workspace_id: Option<String>,
}

impl TargetingSearch {
    pub fn new(connection_id: impl Into<String>, search_type: TargetingSearchType) -> Self {
        Self {
            connection_id: connection_id.into(),
            search_type,
            q: None,
            workspace_id: None,
        }
    }

    pub fn query(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

/// The body of `POST /ads/lead-forms`: an Instant Form on a Page.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLeadForm {
    pub workspace_id: String,
    pub connection_id: String,
    /// Facebook Page id.
    pub page_id: String,
    pub name: String,
    /// One to three questions.
    pub questions: Vec<LeadQuestion>,
    pub privacy_policy_url: String,
    /// Up to 500 characters.
    pub thank_you_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_up_url: Option<String>,
}

impl CreateLeadForm {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        page_id: impl Into<String>,
        name: impl Into<String>,
        questions: impl IntoIterator<Item = LeadQuestion>,
        privacy_policy_url: impl Into<String>,
        thank_you_message: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            page_id: page_id.into(),
            name: name.into(),
            questions: questions.into_iter().collect(),
            privacy_policy_url: privacy_policy_url.into(),
            thank_you_message: thank_you_message.into(),
            follow_up_url: None,
        }
    }

    pub fn follow_up_url(mut self, follow_up_url: impl Into<String>) -> Self {
        self.follow_up_url = Some(follow_up_url.into());
        self
    }
}

/// Filters for `GET /ads/lead-forms/{formId}/leads`.
#[derive(Debug, Clone)]
pub struct LeadsQuery {
    pub connection_id: String,
    pub page_id: String,
    /// The `next_cursor` of the previous page.
    pub after: Option<String>,
    pub workspace_id: Option<String>,
}

impl LeadsQuery {
    pub fn new(connection_id: impl Into<String>, page_id: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            page_id: page_id.into(),
            after: None,
            workspace_id: None,
        }
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

string_enum! {
    /// Which kind of Meta object a bulk status change names.
    pub enum AdObjectLevel {
        Campaign => "campaign",
        AdSet => "ad_set",
        Ad => "ad",
    }
}

string_enum! {
    /// How insights are split.
    pub enum InsightsBreakdown {
        Age => "age",
        Gender => "gender",
        Placement => "placement",
        Country => "country",
    }
}

string_enum! {
    /// What a creative carries. Meta's catch-all `other` parses as `Other("other")`.
    pub enum CreativeFormat {
        Image => "image",
        Video => "video",
        Carousel => "carousel",
        Post => "post",
    }
}

string_enum! {
    /// The button on a creative.
    pub enum CallToAction {
        LearnMore => "LEARN_MORE",
        ShopNow => "SHOP_NOW",
        SignUp => "SIGN_UP",
        Subscribe => "SUBSCRIBE",
        ContactUs => "CONTACT_US",
        Download => "DOWNLOAD",
        GetOffer => "GET_OFFER",
        BookNow => "BOOK_NOW",
        ApplyNow => "APPLY_NOW",
        WatchMore => "WATCH_MORE",
    }
}

/// The connection a Meta object is read or changed through. Writes also need
/// the workspace.
#[derive(Debug, Clone)]
pub struct AdObjectQuery {
    pub connection_id: String,
    pub workspace_id: Option<String>,
}

impl AdObjectQuery {
    pub fn new(connection_id: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            workspace_id: None,
        }
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

/// A Meta campaign. Read live, never stored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdCampaign {
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// Meta's `ACTIVE`, `PAUSED`, `DELETED` or `ARCHIVED`.
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub effective_status: Option<String>,
    #[serde(default)]
    pub objective: Option<String>,
    /// `None` when the budget lives on the ad sets.
    #[serde(default)]
    pub budget_minor: Option<u64>,
    #[serde(default)]
    pub budget_type: Option<AdBudgetType>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A Meta ad set. Read live, never stored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdSet {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub campaign_id: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub effective_status: Option<String>,
    #[serde(default)]
    pub budget_minor: Option<u64>,
    #[serde(default)]
    pub budget_type: Option<AdBudgetType>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default)]
    pub optimization_goal: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// An ad inside a Meta ad set. Read live, never stored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAd {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub campaign_id: Option<String>,
    #[serde(default)]
    pub ad_set_id: Option<String>,
    #[serde(default)]
    pub creative_id: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub effective_status: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// An ad set with its ads, inside the account tree.
#[derive(Debug, Clone, Deserialize)]
pub struct TreeAdSet {
    #[serde(flatten)]
    pub ad_set: AdSet,
    #[serde(default)]
    pub ads: Vec<NetworkAd>,
}

/// A campaign with its ad sets, inside the account tree.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeCampaign {
    #[serde(flatten)]
    pub campaign: AdCampaign,
    #[serde(default)]
    pub ad_sets: Vec<TreeAdSet>,
}

/// Every campaign on an ad account with its ad sets and ads.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdAccountTree {
    #[serde(default)]
    pub ad_account_id: String,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub campaigns: Vec<TreeCampaign>,
}

/// The body of `POST /ads/campaigns`. Created paused unless `paused` is `false`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCampaign {
    pub workspace_id: String,
    pub connection_id: String,
    /// `act_…`
    pub ad_account_id: String,
    pub name: String,
    pub goal: AdGoal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
}

impl CreateCampaign {
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        ad_account_id: impl Into<String>,
        name: impl Into<String>,
        goal: AdGoal,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            ad_account_id: ad_account_id.into(),
            name: name.into(),
            goal,
            paused: None,
        }
    }

    /// `false` to start delivering at once.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = Some(paused);
        self
    }
}

/// The body of `PATCH /ads/campaigns/{id}`: a new name, a new status, or both.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCampaign {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AdStatus>,
}

impl UpdateCampaign {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn status(mut self, status: AdStatus) -> Self {
        self.status = Some(status);
        self
    }
}

/// The body of `POST /ads/ad-sets`. Created paused unless `paused` is `false`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAdSet {
    pub workspace_id: String,
    pub connection_id: String,
    pub campaign_id: String,
    /// The Page the ads in this set run as.
    pub page_id: String,
    pub name: String,
    pub goal: AdGoal,
    pub budget: AdBudget,
    pub targeting: AdTargeting,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
}

impl CreateAdSet {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        campaign_id: impl Into<String>,
        page_id: impl Into<String>,
        name: impl Into<String>,
        goal: AdGoal,
        budget: AdBudget,
        targeting: AdTargeting,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            campaign_id: campaign_id.into(),
            page_id: page_id.into(),
            name: name.into(),
            goal,
            budget,
            targeting,
            paused: None,
        }
    }

    /// `false` to start delivering at once.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = Some(paused);
        self
    }
}

/// The body of `PATCH /ads/ad-sets/{id}`. A new budget keeps the budget type
/// set at creation.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAdSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AdStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_minor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targeting: Option<AdTargeting>,
}

impl UpdateAdSet {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn status(mut self, status: AdStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn budget_minor(mut self, budget_minor: u64) -> Self {
        self.budget_minor = Some(budget_minor);
        self
    }

    pub fn end_at(mut self, end_at: impl Into<String>) -> Self {
        self.end_at = Some(end_at.into());
        self
    }

    pub fn targeting(mut self, targeting: AdTargeting) -> Self {
        self.targeting = Some(targeting);
        self
    }
}

/// The body of `POST /ads/ads`: an ad inside an ad set. Created paused unless
/// `paused` is `false`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNetworkAd {
    pub workspace_id: String,
    pub connection_id: String,
    pub ad_set_id: String,
    /// From `create_creative()` or `creatives()`.
    pub creative_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paused: Option<bool>,
}

impl CreateNetworkAd {
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        ad_set_id: impl Into<String>,
        creative_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            ad_set_id: ad_set_id.into(),
            creative_id: creative_id.into(),
            name: name.into(),
            paused: None,
        }
    }

    /// `false` to start delivering at once.
    pub fn paused(mut self, paused: bool) -> Self {
        self.paused = Some(paused);
        self
    }
}

/// The body of `PATCH /ads/ads/{id}`.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNetworkAd {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AdStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creative_id: Option<String>,
}

impl UpdateNetworkAd {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn status(mut self, status: AdStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn creative(mut self, creative_id: impl Into<String>) -> Self {
        self.creative_id = Some(creative_id.into());
        self
    }
}

/// One object in a bulk status change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdObjectRef {
    pub id: String,
    pub level: AdObjectLevel,
}

impl AdObjectRef {
    pub fn new(id: impl Into<String>, level: AdObjectLevel) -> Self {
        Self {
            id: id.into(),
            level,
        }
    }
}

/// The body of `POST /ads/status`: up to 50 objects.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetAdStatuses {
    pub workspace_id: String,
    pub connection_id: String,
    pub status: AdStatus,
    pub objects: Vec<AdObjectRef>,
}

impl SetAdStatuses {
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        status: AdStatus,
        objects: impl IntoIterator<Item = AdObjectRef>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            status,
            objects: objects.into_iter().collect(),
        }
    }
}

/// The outcome for one object of a bulk status change.
#[derive(Debug, Clone, Deserialize)]
pub struct AdStatusResult {
    pub id: String,
    pub level: AdObjectLevel,
    #[serde(default)]
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
}

/// A creative in an ad account's library.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Creative {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub format: Option<CreativeFormat>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub call_to_action: Option<String>,
    #[serde(default)]
    pub url_tags: Option<String>,
}

/// The answer to `GET /ads/creatives`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdCreatives {
    #[serde(default)]
    pub creatives: Vec<Creative>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// Filters for `GET /ads/creatives`.
#[derive(Debug, Clone)]
pub struct CreativesQuery {
    pub connection_id: String,
    /// `act_…`
    pub ad_account_id: String,
    pub workspace_id: Option<String>,
}

impl CreativesQuery {
    pub fn new(connection_id: impl Into<String>, ad_account_id: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            ad_account_id: ad_account_id.into(),
            workspace_id: None,
        }
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

/// One card of a carousel creative.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CarouselCard {
    /// A library image.
    pub media_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CarouselCard {
    pub fn new(media_url: impl Into<String>) -> Self {
        Self {
            media_url: media_url.into(),
            destination_url: None,
            headline: None,
            description: None,
        }
    }

    pub fn destination_url(mut self, destination_url: impl Into<String>) -> Self {
        self.destination_url = Some(destination_url.into());
        self
    }

    pub fn headline(mut self, headline: impl Into<String>) -> Self {
        self.headline = Some(headline.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// The body of `POST /ads/creatives`. A video needs `media_url`; a carousel
/// needs 2 to 10 `cards`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCreative {
    pub workspace_id: String,
    pub connection_id: String,
    /// `act_…`
    pub ad_account_id: String,
    pub page_id: String,
    pub name: String,
    pub format: CreativeFormat,
    /// Primary text.
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_url: Option<String>,
    /// `LEARN_MORE` when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_to_action: Option<CallToAction>,
    /// Query string appended to every link in the ad.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_tags: Option<String>,
    /// A media library asset url: the image, or the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
    /// A video's poster frame, as a library image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_media_url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cards: Vec<CarouselCard>,
}

impl CreateCreative {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        ad_account_id: impl Into<String>,
        page_id: impl Into<String>,
        name: impl Into<String>,
        format: CreativeFormat,
        text: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            ad_account_id: ad_account_id.into(),
            page_id: page_id.into(),
            name: name.into(),
            format,
            text: text.into(),
            headline: None,
            destination_url: None,
            call_to_action: None,
            url_tags: None,
            media_url: None,
            thumbnail_media_url: None,
            cards: Vec::new(),
        }
    }

    pub fn headline(mut self, headline: impl Into<String>) -> Self {
        self.headline = Some(headline.into());
        self
    }

    pub fn destination_url(mut self, destination_url: impl Into<String>) -> Self {
        self.destination_url = Some(destination_url.into());
        self
    }

    pub fn call_to_action(mut self, call_to_action: CallToAction) -> Self {
        self.call_to_action = Some(call_to_action);
        self
    }

    pub fn url_tags(mut self, url_tags: impl Into<String>) -> Self {
        self.url_tags = Some(url_tags.into());
        self
    }

    pub fn media_url(mut self, media_url: impl Into<String>) -> Self {
        self.media_url = Some(media_url.into());
        self
    }

    pub fn thumbnail_media_url(mut self, thumbnail_media_url: impl Into<String>) -> Self {
        self.thumbnail_media_url = Some(thumbnail_media_url.into());
        self
    }

    pub fn cards(mut self, cards: impl IntoIterator<Item = CarouselCard>) -> Self {
        self.cards = cards.into_iter().collect();
        self
    }
}

/// The body of `PATCH /ads/audiences/{id}`.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAudience {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl UpdateAudience {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// The body of `POST /ads/reach-estimate`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateReach {
    pub workspace_id: String,
    pub connection_id: String,
    /// `act_…`
    pub ad_account_id: String,
    pub page_id: String,
    pub targeting: AdTargeting,
}

impl EstimateReach {
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        ad_account_id: impl Into<String>,
        page_id: impl Into<String>,
        targeting: AdTargeting,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            ad_account_id: ad_account_id.into(),
            page_id: page_id.into(),
            targeting,
        }
    }
}

/// Meta's audience size range for a targeting. `ready` is false while Meta is
/// still computing it.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ReachEstimate {
    #[serde(default)]
    pub lower: Option<u64>,
    #[serde(default)]
    pub upper: Option<u64>,
    #[serde(default)]
    pub ready: bool,
}

/// Delivery numbers over a range.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightsMetrics {
    #[serde(default)]
    pub impressions: u64,
    #[serde(default)]
    pub reach: u64,
    #[serde(default)]
    pub clicks: u64,
    /// Account currency, minor units.
    #[serde(default)]
    pub spend_minor: u64,
    /// Clicks per impression, as a percentage.
    #[serde(default)]
    pub ctr: f64,
    #[serde(default)]
    pub leads: u64,
}

/// One value of the requested breakdown.
#[derive(Debug, Clone, Deserialize)]
pub struct InsightsBreakdownRow {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub metrics: InsightsMetrics,
}

/// One day of the timeline.
#[derive(Debug, Clone, Deserialize)]
pub struct InsightsDay {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub metrics: InsightsMetrics,
}

/// An object's delivery over a date range. `totals` is `None` when Meta has
/// nothing for the range.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsightsReport {
    #[serde(default)]
    pub object_id: String,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub since: String,
    #[serde(default)]
    pub until: String,
    #[serde(default)]
    pub breakdown_by: Option<InsightsBreakdown>,
    #[serde(default)]
    pub totals: Option<InsightsMetrics>,
    #[serde(default)]
    pub breakdown: Vec<InsightsBreakdownRow>,
    #[serde(default)]
    pub timeline: Vec<InsightsDay>,
}

/// Filters for `GET /ads/insights`. `object_id` is an ad account (`act_…`),
/// campaign, ad set or ad; `since` and `until` are inclusive `YYYY-MM-DD` dates.
#[derive(Debug, Clone)]
pub struct InsightsQuery {
    pub connection_id: String,
    pub object_id: String,
    pub since: String,
    pub until: String,
    pub breakdown: Option<InsightsBreakdown>,
    /// Add the per-day timeline.
    pub daily: Option<bool>,
    pub workspace_id: Option<String>,
}

impl InsightsQuery {
    pub fn new(
        connection_id: impl Into<String>,
        object_id: impl Into<String>,
        since: impl Into<String>,
        until: impl Into<String>,
    ) -> Self {
        Self {
            connection_id: connection_id.into(),
            object_id: object_id.into(),
            since: since.into(),
            until: until.into(),
            breakdown: None,
            daily: None,
            workspace_id: None,
        }
    }

    pub fn breakdown(mut self, breakdown: InsightsBreakdown) -> Self {
        self.breakdown = Some(breakdown);
        self
    }

    pub fn daily(mut self, daily: bool) -> Self {
        self.daily = Some(daily);
        self
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

/// Filters for `GET /ads/{id}/insights`, on a boost or ad created through
/// FoPost.
#[derive(Debug, Clone)]
pub struct AdInsightsQuery {
    pub workspace_id: String,
    pub since: String,
    pub until: String,
    pub breakdown: Option<InsightsBreakdown>,
    pub daily: Option<bool>,
}

impl AdInsightsQuery {
    pub fn new(
        workspace_id: impl Into<String>,
        since: impl Into<String>,
        until: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            since: since.into(),
            until: until.into(),
            breakdown: None,
            daily: None,
        }
    }

    pub fn breakdown(mut self, breakdown: InsightsBreakdown) -> Self {
        self.breakdown = Some(breakdown);
        self
    }

    pub fn daily(mut self, daily: bool) -> Self {
        self.daily = Some(daily);
        self
    }
}

/// An Instant Form with its settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadFormDetail {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub leads_count: u64,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub questions: Vec<String>,
    #[serde(default)]
    pub page_id: Option<String>,
    #[serde(default)]
    pub privacy_policy_url: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
}

/// Filters for `GET /ads/lead-forms/{formId}`.
#[derive(Debug, Clone)]
pub struct LeadFormQuery {
    pub connection_id: String,
    pub page_id: String,
    pub workspace_id: Option<String>,
}

impl LeadFormQuery {
    pub fn new(connection_id: impl Into<String>, page_id: impl Into<String>) -> Self {
        Self {
            connection_id: connection_id.into(),
            page_id: page_id.into(),
            workspace_id: None,
        }
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

/// The body of `POST /ads/lead-forms/{formId}/archive`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLeadForm {
    pub workspace_id: String,
    pub connection_id: String,
    pub page_id: String,
}

impl ArchiveLeadForm {
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        page_id: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            page_id: page_id.into(),
        }
    }
}

/// A lead collected from a subscribed Page.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedLead {
    pub id: String,
    /// Meta's lead id.
    #[serde(default)]
    pub lead_id: String,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub page_id: Option<String>,
    #[serde(default)]
    pub form_id: Option<String>,
    #[serde(default)]
    pub ad_id: Option<String>,
    #[serde(default)]
    pub ad_name: Option<String>,
    #[serde(default)]
    pub campaign_name: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub is_organic: bool,
    #[serde(default)]
    pub fields: Vec<LeadField>,
    #[serde(default)]
    pub submitted_at: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// One page of the leads feed. Pass `next_cursor` back as `cursor` for the
/// next; `None` on the last page.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadsFeedPage {
    #[serde(default)]
    pub leads: Vec<FeedLead>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// Filters for `GET /ads/leads`.
#[derive(Debug, Clone, Default)]
pub struct LeadsFeedQuery {
    pub workspace_id: Option<String>,
    pub form_id: Option<String>,
    pub page_id: Option<String>,
    /// The `next_cursor` of the previous page.
    pub cursor: Option<String>,
    /// 1 to 100.
    pub limit: Option<u32>,
}

impl LeadsFeedQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn form(mut self, form_id: impl Into<String>) -> Self {
        self.form_id = Some(form_id.into());
        self
    }

    pub fn page(mut self, page_id: impl Into<String>) -> Self {
        self.page_id = Some(page_id.into());
        self
    }

    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// A Page whose leads are collected into the feed.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadPage {
    #[serde(default)]
    pub connection_id: String,
    pub page_id: String,
    #[serde(default)]
    pub page_name: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// The body of `POST /ads/lead-pages`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeLeadPage {
    pub workspace_id: String,
    pub connection_id: String,
    pub page_id: String,
}

impl SubscribeLeadPage {
    pub fn new(
        workspace_id: impl Into<String>,
        connection_id: impl Into<String>,
        page_id: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            connection_id: connection_id.into(),
            page_id: page_id.into(),
        }
    }
}

/// The answer to `POST /ads/lead-pages`. `backfilled` counts the recent leads
/// copied into the feed.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadPageSubscribed {
    #[serde(default)]
    pub page_id: String,
    #[serde(default)]
    pub backfilled: u64,
}

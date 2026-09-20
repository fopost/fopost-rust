//! Google Ads — keywords, assets, Performance Max asset groups, Local
//! Services leads, conversions and raw GAQL.
//!
//! Amounts are in the account's currency, in minor units: `1500` is $15.00 on
//! a USD account. Google's own micros never cross this boundary.
//!
//! An object id carries the account it belongs to, because a Google resource
//! name cannot ride in a URL path segment: `1234567890~campaign~55`,
//! `1234567890~adGroup~77`, `1234567890~ad~77~88`.

use serde::{Deserialize, Serialize};

use super::common::string_enum;

string_enum! {
    /// How closely a search has to match a keyword.
    pub enum GoogleMatchType {
        Exact => "EXACT",
        Phrase => "PHRASE",
        Broad => "BROAD",
    }
}

string_enum! {
    /// A portfolio bid strategy.
    pub enum GoogleBidStrategyType {
        TargetSpend => "TARGET_SPEND",
        MaximizeConversions => "MAXIMIZE_CONVERSIONS",
        MaximizeConversionValue => "MAXIMIZE_CONVERSION_VALUE",
        TargetCpa => "TARGET_CPA",
        TargetRoas => "TARGET_ROAS",
    }
}

string_enum! {
    /// Where an asset renders.
    pub enum GoogleAssetFieldType {
        Sitelink => "SITELINK",
        Callout => "CALLOUT",
        StructuredSnippet => "STRUCTURED_SNIPPET",
    }
}

string_enum! {
    /// A day of the week on an ad schedule.
    pub enum GoogleDayOfWeek {
        Monday => "MONDAY",
        Tuesday => "TUESDAY",
        Wednesday => "WEDNESDAY",
        Thursday => "THURSDAY",
        Friday => "FRIDAY",
        Saturday => "SATURDAY",
        Sunday => "SUNDAY",
    }
}

string_enum! {
    /// What a conversion adjustment does to a conversion already counted.
    pub enum GoogleAdjustmentType {
        Restatement => "RESTATEMENT",
        Retraction => "RETRACTION",
        Enhancement => "ENHANCEMENT",
    }
}

/// The connection and the Google Ads account a call runs against.
///
/// `customer_id` is digits only and has to name an account the connection's
/// grant reaches: any other answers 404. `workspace_id` may be left out on a
/// read, and is required on a write.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    pub connection_id: String,
    pub customer_id: String,
}

impl GoogleScope {
    pub fn new(connection_id: impl Into<String>, customer_id: impl Into<String>) -> Self {
        Self {
            workspace_id: None,
            connection_id: connection_id.into(),
            customer_id: customer_id.into(),
        }
    }

    /// A write names the workspace the connection lives in.
    pub fn in_workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }
}

/// A date range in the account's time zone, `YYYY-MM-DD` and inclusive.
#[derive(Debug, Clone)]
pub struct GoogleDateRange {
    pub since: String,
    pub until: String,
}

impl GoogleDateRange {
    pub fn new(since: impl Into<String>, until: impl Into<String>) -> Self {
        Self {
            since: since.into(),
            until: until.into(),
        }
    }
}

/// A keyword on an ad group.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleKeyword {
    /// `<customer_id>~keyword~<ad_group_id>~<criterion_id>`
    pub id: String,
    #[serde(default)]
    pub ad_group_id: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub match_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub cpc_bid_minor: Option<i64>,
    #[serde(default)]
    pub negative: bool,
}

/// A keyword idea, or the historical metrics of one.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleKeywordIdea {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub avg_monthly_searches: i64,
    #[serde(default)]
    pub competition: Option<String>,
    #[serde(default)]
    pub low_top_of_page_bid_minor: Option<i64>,
    #[serde(default)]
    pub high_top_of_page_bid_minor: Option<i64>,
}

/// What someone actually searched, with the metrics it earned.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearchTerm {
    #[serde(default)]
    pub term: String,
    #[serde(default)]
    pub ad_group_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub metrics: serde_json::Value,
}

/// A portfolio bid strategy on the account.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleBidStrategy {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub campaign_count: i64,
}

/// One slot of a campaign's ad schedule.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAdScheduleSlot {
    pub id: String,
    #[serde(default)]
    pub day_of_week: String,
    #[serde(default)]
    pub start_hour: i32,
    #[serde(default)]
    pub end_hour: i32,
    #[serde(default)]
    pub bid_modifier: Option<f64>,
}

/// A negative keyword list.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSharedSet {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub member_count: i64,
}

/// A sitelink, callout or structured snippet.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAsset {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: String,
    /// What a sitelink, callout or snippet renders.
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub final_url: Option<String>,
}

/// Where an asset is attached; an asset with no links serves nowhere.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAssetLink {
    pub id: String,
    #[serde(default)]
    pub asset_id: String,
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub field_type: String,
    #[serde(default)]
    pub status: String,
}

/// The account's assets with the links that place them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAssets {
    #[serde(default)]
    pub assets: Vec<GoogleAsset>,
    #[serde(default)]
    pub links: Vec<GoogleAssetLink>,
}

/// A Performance Max asset group.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAssetGroup {
    pub id: String,
    #[serde(default)]
    pub campaign_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub final_urls: Vec<String>,
}

/// A lead from Local Services Ads, read live and never stored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleLocalServicesLead {
    pub id: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub service: Option<String>,
    #[serde(default)]
    pub contact_name: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A conversion action on the account.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleConversionAction {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub counting_type: Option<String>,
    #[serde(default)]
    pub value_minor: Option<i64>,
}

// ─── Request bodies ────────────────────────────────────────────────

/// Start a Google Ads connection.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizeGoogleAds {
    pub workspace_id: String,
    /// Dashboard path to land on after Google redirects back.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_to: Option<String>,
}

impl AuthorizeGoogleAds {
    pub fn new(workspace_id: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            return_to: None,
        }
    }
}

/// Add a keyword to an ad group.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoogleKeyword {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub ad_group_id: String,
    pub text: String,
    pub match_type: GoogleMatchType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpc_bid_minor: Option<i64>,
}

/// Pause, resume or rebid a keyword.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoogleKeyword {
    #[serde(flatten)]
    pub scope: GoogleScope,
    /// `active` or `paused`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpc_bid_minor: Option<i64>,
}

/// Ask for keyword ideas from seeds, a landing page, or both.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleKeywordIdeas {
    #[serde(flatten)]
    pub scope: GoogleScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seeds: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_target_ids: Option<Vec<String>>,
}

/// Read the historical metrics of keywords you already have.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleKeywordMetrics {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub keywords: Vec<String>,
}

/// Add a portfolio bid strategy.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoogleBidStrategy {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: GoogleBidStrategyType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_minor: Option<i64>,
}

/// One slot to put on a campaign's schedule.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAdScheduleInput {
    pub day_of_week: GoogleDayOfWeek,
    pub start_hour: i32,
    pub end_hour: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bid_modifier: Option<f64>,
}

/// Replace a campaign's schedule; Google has no partial edit for one.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetGoogleAdSchedule {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub campaign_id: String,
    pub slots: Vec<GoogleAdScheduleInput>,
}

/// Create a negative keyword list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoogleNegativeKeywordList {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub name: String,
}

/// One keyword in a negative list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleNegativeKeyword {
    pub text: String,
    pub match_type: GoogleMatchType,
}

/// Add keywords to a negative list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddGoogleNegativeKeywords {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub shared_set_id: String,
    pub keywords: Vec<GoogleNegativeKeyword>,
}

/// Put a negative keyword list on a campaign.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachGoogleNegativeKeywordList {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub shared_set_id: String,
    pub campaign_id: String,
}

/// The asset to create.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GoogleAssetSpec {
    #[serde(rename = "sitelink", rename_all = "camelCase")]
    Sitelink {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description1: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description2: Option<String>,
        final_url: String,
    },
    #[serde(rename = "callout", rename_all = "camelCase")]
    Callout { text: String },
    #[serde(rename = "snippet", rename_all = "camelCase")]
    Snippet { header: String, values: Vec<String> },
}

/// Add an asset to the library.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoogleAsset {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub spec: GoogleAssetSpec,
}

/// Attach an asset to the account, or to one campaign.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachGoogleAsset {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub asset_id: String,
    pub field_type: GoogleAssetFieldType,
    /// Attaches to the account when left out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign_id: Option<String>,
}

/// Create a Performance Max asset group.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoogleAssetGroup {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub campaign_id: String,
    pub name: String,
    pub final_urls: Vec<String>,
    /// `active` or `paused`; starts paused when left out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Rename, pause or resume an asset group.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoogleAssetGroup {
    #[serde(flatten)]
    pub scope: GoogleScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Create a conversion action.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoogleConversionAction {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub name: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_minor: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counting_type: Option<String>,
}

/// One offline conversion. One of `gclid`, `gbraid` or `wbraid` is required:
/// it is what matches the click.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleClickConversion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gclid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gbraid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wbraid: Option<String>,
    pub conversion_action_id: String,
    /// `yyyy-MM-dd HH:mm:ss+|-HH:mm`, the only shape Google accepts.
    pub conversion_date_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_minor: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
}

/// Send offline conversions.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadGoogleConversions {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub conversions: Vec<GoogleClickConversion>,
}

/// Restate, retract or enhance a conversion already counted.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleConversionAdjustment {
    pub conversion_action_id: String,
    pub adjustment_type: GoogleAdjustmentType,
    pub adjustment_date_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gclid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversion_date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restatement_value_minor: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
}

/// Send conversion adjustments.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadGoogleConversionAdjustments {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub adjustments: Vec<GoogleConversionAdjustment>,
}

/// A raw read-only GAQL SELECT. The account read is `customer_id`, never
/// anything named inside `query`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleQuery {
    #[serde(flatten)]
    pub scope: GoogleScope,
    pub query: String,
}

/// Rows exactly as Google returns them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleQueryResult {
    #[serde(default)]
    pub rows: Vec<serde_json::Value>,
}

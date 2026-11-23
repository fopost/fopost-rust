//! Google Ads: bid strategies, the ad schedule, Local Services leads,
//! conversions and raw GAQL.
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

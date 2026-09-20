//! WhatsApp Business — a number the customer already owns, reached through the
//! platform's Cloud API.
//!
//! The platform owns templates, flows, the profile and the commerce settings, so
//! nothing here is a cached copy: a template's status is whatever the platform
//! assigned it in review.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ─── Profile ────────────────────────────────────────────────────────────

/// The business profile on a WhatsApp number, plus how the platform rates it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappProfile {
    #[serde(default)]
    pub about: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub vertical: Option<String>,
    #[serde(default)]
    pub websites: Vec<String>,
    #[serde(default)]
    pub profile_picture_url: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    /// The platform's review state for the display name.
    #[serde(default)]
    pub display_name_status: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub quality_rating: Option<String>,
    #[serde(default)]
    pub messaging_limit_tier: Option<String>,
}

/// A partial profile update: omitted fields keep their value.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWhatsappProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub websites: Option<Vec<String>>,
    /// A library media id, uploaded first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_picture_media_id: Option<String>,
}

// ─── Templates ──────────────────────────────────────────────────────────

/// A message template. `status` is the review outcome the platform assigned.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappTemplate {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub rejected_reason: Option<String>,
    #[serde(default)]
    pub components: Vec<Value>,
    #[serde(default)]
    pub quality_score: Option<String>,
}

/// Files a template for review.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWhatsappTemplate {
    /// Lowercase letters, digits and underscores.
    pub name: String,
    pub language: String,
    pub category: String,
    pub components: Vec<Value>,
    /// Lets the platform re-file a template it judges to be another category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_category_change: Option<bool>,
}

impl CreateWhatsappTemplate {
    pub fn new(
        name: impl Into<String>,
        language: impl Into<String>,
        category: impl Into<String>,
        components: Vec<Value>,
    ) -> Self {
        Self {
            name: name.into(),
            language: language.into(),
            category: category.into(),
            components,
            allow_category_change: None,
        }
    }

    pub fn allow_category_change(mut self, allow: bool) -> Self {
        self.allow_category_change = Some(allow);
        self
    }
}

/// Creates a template from one of the platform's library entries.
#[derive(Debug, Clone, Serialize)]
pub struct ImportWhatsappTemplate {
    pub library_template_name: String,
    pub name: String,
    pub language: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library_template_button_inputs: Option<Vec<Value>>,
}

/// Edits a template. The name cannot change; create a new one instead.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWhatsappTemplate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Value>>,
}

// ─── Groups ─────────────────────────────────────────────────────────────

/// A group on the number. Participation is invite-only: no endpoint adds anyone.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappGroup {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub participant_count: Option<i64>,
    #[serde(default)]
    pub invite_link: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Creates or updates a group.
#[derive(Debug, Clone, Default, Serialize)]
pub struct WhatsappGroupInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ─── Blocking ───────────────────────────────────────────────────────────

/// What the platform took and what it refused.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappBlockResult {
    #[serde(default)]
    pub blocked: Vec<String>,
    #[serde(default)]
    pub unblocked: Vec<String>,
    #[serde(default)]
    pub failed: Vec<String>,
}

// ─── Commerce ───────────────────────────────────────────────────────────

/// Whether the cart and catalog show on the number, and which catalog is linked.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappCommerceSettings {
    #[serde(default)]
    pub cart_enabled: Option<bool>,
    #[serde(default)]
    pub catalog_visible: Option<bool>,
    #[serde(default)]
    pub catalog_id: Option<String>,
}

/// Turns the cart or the catalog on or off.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWhatsappCommerce {
    #[serde(rename = "is_cart_enabled", skip_serializing_if = "Option::is_none")]
    pub cart_enabled: Option<bool>,
    #[serde(rename = "is_catalog_visible", skip_serializing_if = "Option::is_none")]
    pub catalog_visible: Option<bool>,
}

// ─── Flows ──────────────────────────────────────────────────────────────

/// One problem the platform found in a flow definition.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappFlowValidationError {
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub message: String,
}

/// An in-chat form. The platform validates it and owns its status.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappFlow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub validation_errors: Vec<WhatsappFlowValidationError>,
    #[serde(default)]
    pub endpoint_uri: Option<String>,
    #[serde(default)]
    pub json_version: Option<String>,
    #[serde(default)]
    pub preview_url: Option<String>,
    #[serde(default)]
    pub preview_expires_at: Option<String>,
}

/// Creates a draft flow. Its screens are uploaded separately.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWhatsappFlow {
    pub name: String,
    pub categories: Vec<String>,
    /// Where the platform calls back for a flow that reads live data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clone_flow_id: Option<String>,
}

impl CreateWhatsappFlow {
    pub fn new(name: impl Into<String>, categories: Vec<String>) -> Self {
        Self {
            name: name.into(),
            categories,
            endpoint_uri: None,
            clone_flow_id: None,
        }
    }
}

/// Changes a flow's metadata, not its screens.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWhatsappFlow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_uri: Option<String>,
}

/// The platform's verdict on an uploaded definition. It answers with the errors
/// rather than refusing the upload, so they arrive as data.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappFlowJsonResult {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub validation_errors: Vec<WhatsappFlowValidationError>,
}

/// What one person submitted through a flow.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappFlowResponse {
    #[serde(default)]
    pub message_id: String,
    #[serde(default)]
    pub wa_id: Option<String>,
    #[serde(default)]
    pub flow_token: Option<String>,
    #[serde(default)]
    pub answers: Value,
    #[serde(default)]
    pub responded_at: Option<String>,
}

/// Whether a key is registered. The key itself never comes back.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappEncryptionKeyStatus {
    #[serde(default)]
    pub has_key: bool,
    #[serde(default)]
    pub signature_status: Option<String>,
}

// ─── Sandbox ────────────────────────────────────────────────────────────

/// A sandbox invitation. Only the last four digits of the tester's number
/// travel; the number itself is never stored.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhatsappSandboxSession {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub phone_number_last4: String,
    #[serde(default)]
    pub invited_at: Option<String>,
    #[serde(default)]
    pub activated_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

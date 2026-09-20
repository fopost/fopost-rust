//! Broadcasts and drip sequences — one message into many conversations, and a
//! series of messages on a delay.
//!
//! Neither opens a cold DM: every message lands in a direct-message thread the
//! contact already started. Both honour each network's messaging window
//! server-side. Messenger and Instagram take a business-initiated message only
//! within 24 hours of the contact's last one, so a recipient outside it comes
//! back [`RecipientStatus::Skipped`] with [`SkipReason::WindowClosed`] and
//! nothing is attempted. Telegram, Slack, Bluesky and Reddit have no window.

use serde::{Deserialize, Serialize};

use super::ContactSource;

/// One custom-field clause in an audience filter.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudienceField {
    pub key: String,
    /// `is`, `is_not`, `contains`, `is_set` or `is_not_set`. Unset means `is`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Who a broadcast or an enrollment resolves to, expressed over contacts.
///
/// Every clause narrows: a contact has to match all of them.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudienceFilter {
    /// Contacts with a handle on at least one of these networks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_ids: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ContactSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<AudienceField>>,
}

impl AudienceFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn platforms<I, S>(mut self, platforms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.platforms = Some(platforms.into_iter().map(Into::into).collect());
        self
    }

    pub fn label_ids<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.label_ids = Some(ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn source(mut self, source: ContactSource) -> Self {
        self.source = Some(source);
        self
    }

    pub fn field(mut self, field: AudienceField) -> Self {
        self.fields.get_or_insert_with(Vec::new).push(field);
        self
    }
}

/// Where a broadcast stands.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BroadcastStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Cancelled,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for BroadcastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Draft => "draft",
            Self::Scheduled => "scheduled",
            Self::Sending => "sending",
            Self::Sent => "sent",
            Self::Cancelled => "cancelled",
            Self::Other(other) => other,
        };
        f.write_str(s)
    }
}

/// What became of one recipient's message.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipientStatus {
    Pending,
    Sent,
    Skipped,
    Failed,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for RecipientStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Pending => "pending",
            Self::Sent => "sent",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
            Self::Other(other) => other,
        };
        f.write_str(s)
    }
}

/// Why a recipient was skipped instead of written to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// The network's messaging window had shut, so nothing was attempted.
    WindowClosed,
    /// This contact never wrote to the sending account.
    NoConversation,
    /// The account's network takes no messages.
    UnsupportedPlatform,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::WindowClosed => "window_closed",
            Self::NoConversation => "no_conversation",
            Self::UnsupportedPlatform => "unsupported_platform",
            Self::Other(other) => other,
        };
        f.write_str(s)
    }
}

/// What became of a broadcast's recipients, by status.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BroadcastCounts {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub sent: u32,
    /// Usually the messaging window doing its job.
    #[serde(default)]
    pub skipped: u32,
    #[serde(default)]
    pub failed: u32,
    #[serde(default)]
    pub pending: u32,
}

/// One message, sent into conversations the workspace already has.
#[derive(Debug, Clone, Deserialize)]
pub struct Broadcast {
    pub id: String,
    /// Internal only; never sent to anyone.
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub audience: AudienceFilter,
    #[serde(default = "default_broadcast_status")]
    pub status: BroadcastStatus,
    #[serde(default)]
    pub scheduled_at: Option<String>,
    #[serde(default)]
    pub sent_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub counts: Option<BroadcastCounts>,
    /// Set only on a listing that spans workspaces.
    #[serde(default)]
    pub workspace_id: Option<String>,
}

fn default_broadcast_status() -> BroadcastStatus {
    BroadcastStatus::Draft
}

/// The pagination footer a broadcast or sequence listing carries.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BroadcastPageMeta {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u64,
}

/// One page of broadcasts.
#[derive(Debug, Clone, Deserialize)]
pub struct BroadcastPage {
    #[serde(rename = "data", default = "Vec::new")]
    pub items: Vec<Broadcast>,
    #[serde(default)]
    pub pagination: BroadcastPageMeta,
}

impl BroadcastPage {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<'a> IntoIterator for &'a BroadcastPage {
    type Item = &'a Broadcast;
    type IntoIter = std::slice::Iter<'a, Broadcast>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// One contact on one broadcast, and what became of their message.
#[derive(Debug, Clone, Deserialize)]
pub struct BroadcastRecipient {
    pub contact_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default = "default_recipient_status")]
    pub status: RecipientStatus,
    /// Set when `status` is `Skipped`.
    #[serde(default)]
    pub skip_reason: Option<SkipReason>,
    #[serde(default)]
    pub sent_at: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

fn default_recipient_status() -> RecipientStatus {
    RecipientStatus::Pending
}

/// One page of a broadcast's recipients.
#[derive(Debug, Clone, Deserialize)]
pub struct RecipientPage {
    #[serde(rename = "data", default = "Vec::new")]
    pub items: Vec<BroadcastRecipient>,
    #[serde(default)]
    pub pagination: BroadcastPageMeta,
}

impl<'a> IntoIterator for &'a RecipientPage {
    type Item = &'a BroadcastRecipient;
    type IntoIter = std::slice::Iter<'a, BroadcastRecipient>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// What a send started.
#[derive(Debug, Clone, Deserialize)]
pub struct BroadcastSent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: String,
    /// How many contacts matched, not how many will be messaged — the
    /// messaging window decides that.
    #[serde(default)]
    pub recipients: u32,
}

/// The broadcast's new status after a cancel.
#[derive(Debug, Clone, Deserialize)]
pub struct BroadcastCancelled {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub status: String,
}

/// One message and how long after the previous step it goes out.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SequenceStep {
    /// Hours to wait after the previous step; 0 on the first means straight away.
    pub delay_hours: f64,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_id: Option<String>,
}

impl SequenceStep {
    pub fn new(delay_hours: f64, text: impl Into<String>) -> Self {
        Self {
            delay_hours,
            text: text.into(),
            media_id: None,
        }
    }

    pub fn media(mut self, media_id: impl Into<String>) -> Self {
        self.media_id = Some(media_id.into());
        self
    }
}

/// Whether a sequence fires at all.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SequenceStatus {
    Active,
    Paused,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for SequenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Other(other) => other,
        };
        f.write_str(s)
    }
}

/// Where a contact stands on a sequence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentStatus {
    Active,
    Completed,
    Stopped,
    Failed,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for EnrollmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
            Self::Other(other) => other,
        };
        f.write_str(s)
    }
}

/// Where a sequence's enrollments stand, by status.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EnrollmentCounts {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub active: u32,
    #[serde(default)]
    pub completed: u32,
    #[serde(default)]
    pub stopped: u32,
    #[serde(default)]
    pub failed: u32,
}

/// A series of messages, each a delay after the one before.
#[derive(Debug, Clone, Deserialize)]
pub struct Sequence {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default = "Vec::new")]
    pub steps: Vec<SequenceStep>,
    #[serde(default = "default_sequence_status")]
    pub status: SequenceStatus,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub enrollments: Option<EnrollmentCounts>,
    /// Set only on a listing that spans workspaces.
    #[serde(default)]
    pub workspace_id: Option<String>,
}

fn default_sequence_status() -> SequenceStatus {
    SequenceStatus::Active
}

/// One page of sequences.
#[derive(Debug, Clone, Deserialize)]
pub struct SequencePage {
    #[serde(rename = "data", default = "Vec::new")]
    pub items: Vec<Sequence>,
    #[serde(default)]
    pub pagination: BroadcastPageMeta,
}

impl<'a> IntoIterator for &'a SequencePage {
    type Item = &'a Sequence;
    type IntoIter = std::slice::Iter<'a, Sequence>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// One contact walking one sequence.
#[derive(Debug, Clone, Deserialize)]
pub struct Enrollment {
    pub id: String,
    pub contact_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    /// Steps already sent, so also the index of the next one.
    #[serde(default)]
    pub step: u32,
    #[serde(default)]
    pub next_at: Option<String>,
    #[serde(default = "default_enrollment_status")]
    pub status: EnrollmentStatus,
    #[serde(default)]
    pub last_sent_at: Option<String>,
    /// On a skipped step, the reason it was skipped.
    #[serde(default)]
    pub error: Option<String>,
}

fn default_enrollment_status() -> EnrollmentStatus {
    EnrollmentStatus::Active
}

/// One page of a sequence's enrollments.
#[derive(Debug, Clone, Deserialize)]
pub struct EnrollmentPage {
    #[serde(rename = "data", default = "Vec::new")]
    pub items: Vec<Enrollment>,
    #[serde(default)]
    pub pagination: BroadcastPageMeta,
}

impl<'a> IntoIterator for &'a EnrollmentPage {
    type Item = &'a Enrollment;
    type IntoIter = std::slice::Iter<'a, Enrollment>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// Filters and pagination for a broadcast listing.
#[derive(Debug, Clone, Default)]
pub struct ListBroadcasts {
    pub workspace_id: Option<String>,
    pub status: Option<BroadcastStatus>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListBroadcasts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn status(mut self, status: BroadcastStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

/// A broadcast to create. It is written without being sent.
#[derive(Debug, Clone, Serialize)]
pub struct CreateBroadcast {
    pub workspace_id: String,
    /// The connected account the messages go out from.
    pub account_id: String,
    pub name: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_id: Option<String>,
    /// Unset means every contact in the workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<AudienceFilter>,
    /// Set to have it go out on its own at that time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<String>,
}

impl CreateBroadcast {
    pub fn new(
        workspace_id: impl Into<String>,
        account_id: impl Into<String>,
        name: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            account_id: account_id.into(),
            name: name.into(),
            text: text.into(),
            media_id: None,
            audience: None,
            scheduled_at: None,
        }
    }

    pub fn media(mut self, media_id: impl Into<String>) -> Self {
        self.media_id = Some(media_id.into());
        self
    }

    pub fn audience(mut self, audience: AudienceFilter) -> Self {
        self.audience = Some(audience);
        self
    }

    pub fn scheduled_at(mut self, at: impl Into<String>) -> Self {
        self.scheduled_at = Some(at.into());
        self
    }
}

/// A partial update. Only what is set is sent, and only a draft or scheduled
/// broadcast can be edited.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateBroadcast {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<AudienceFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<String>,
}

impl UpdateBroadcast {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn audience(mut self, audience: AudienceFilter) -> Self {
        self.audience = Some(audience);
        self
    }

    pub fn scheduled_at(mut self, at: impl Into<String>) -> Self {
        self.scheduled_at = Some(at.into());
        self
    }
}

/// Filters and pagination for a recipients listing.
#[derive(Debug, Clone, Default)]
pub struct ListRecipients {
    pub status: Option<RecipientStatus>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListRecipients {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: RecipientStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

/// Pagination for a sequence listing.
#[derive(Debug, Clone, Default)]
pub struct ListSequences {
    pub workspace_id: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListSequences {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

/// A sequence to create. Creating one enrolls nobody.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSequence {
    pub workspace_id: String,
    pub account_id: String,
    pub name: String,
    pub steps: Vec<SequenceStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SequenceStatus>,
}

impl CreateSequence {
    pub fn new(
        workspace_id: impl Into<String>,
        account_id: impl Into<String>,
        name: impl Into<String>,
        steps: Vec<SequenceStep>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            account_id: account_id.into(),
            name: name.into(),
            steps,
            status: None,
        }
    }

    pub fn status(mut self, status: SequenceStatus) -> Self {
        self.status = Some(status);
        self
    }
}

/// A partial update. Pausing stops every enrollment from firing without
/// ending any of them.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateSequence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<SequenceStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SequenceStatus>,
}

impl UpdateSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn steps(mut self, steps: Vec<SequenceStep>) -> Self {
        self.steps = Some(steps);
        self
    }

    pub fn status(mut self, status: SequenceStatus) -> Self {
        self.status = Some(status);
        self
    }
}

/// Who to enroll: named contacts, or the audience they are drawn from.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Enroll {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<AudienceFilter>,
}

impl Enroll {
    pub fn contacts<I, S>(ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            contact_ids: Some(ids.into_iter().map(Into::into).collect()),
            audience: None,
        }
    }

    pub fn audience(audience: AudienceFilter) -> Self {
        Self {
            contact_ids: None,
            audience: Some(audience),
        }
    }
}

/// How many contacts were put on the sequence.
#[derive(Debug, Clone, Deserialize)]
pub struct Enrolled {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub enrolled: u32,
}

/// How many enrollments were stopped.
#[derive(Debug, Clone, Deserialize)]
pub struct Unenrolled {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub stopped: u32,
}

/// Pagination for an enrollment listing.
#[derive(Debug, Clone, Default)]
pub struct ListEnrollments {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListEnrollments {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

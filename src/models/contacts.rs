//! Contacts — the people behind the inbox, and the fields a workspace keeps.
//!
//! A contact is one person however many handles they write from. An inbound
//! inbox item files its author, a reply files whoever you answered, and both
//! fold into whatever is already on file, so the same person never becomes two
//! rows.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One handle on one network.
///
/// `handle` is lower-cased with no leading `@`. `external_id` is the
/// platform's own id for this person when the network gave us one, and it is
/// what a merge prefers: a handle can be changed, an id cannot.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ContactChannel {
    pub platform: String,
    pub handle: String,
    #[serde(
        rename = "externalId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub external_id: Option<String>,
}

impl ContactChannel {
    pub fn new(platform: impl Into<String>, handle: impl Into<String>) -> Self {
        Self {
            platform: platform.into(),
            handle: handle.into(),
            external_id: None,
        }
    }

    pub fn external_id(mut self, id: impl Into<String>) -> Self {
        self.external_id = Some(id.into());
        self
    }
}

/// What first created a contact row.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactSource {
    Inbox,
    Radar,
    Import,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for ContactSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Inbox => "inbox",
            Self::Radar => "radar",
            Self::Import => "import",
            Self::Other(other) => other,
        };
        f.write_str(s)
    }
}

/// A workspace label put on a contact.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactLabel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
}

/// One person, however many handles they write from.
#[derive(Debug, Clone, Deserialize)]
pub struct Contact {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default = "Vec::new")]
    pub channels: Vec<ContactChannel>,
    #[serde(default = "default_source")]
    pub source: ContactSource,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub first_seen_at: Option<String>,
    #[serde(default)]
    pub last_seen_at: Option<String>,
    /// Custom field values, keyed by field key.
    #[serde(default)]
    pub fields: BTreeMap<String, String>,
    #[serde(default = "Vec::new")]
    pub labels: Vec<ContactLabel>,
    /// Set only on a listing that spans workspaces.
    #[serde(default)]
    pub workspace_id: Option<String>,
}

fn default_source() -> ContactSource {
    ContactSource::Inbox
}

/// The pagination footer a contacts listing carries.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ContactPageMeta {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u64,
}

/// One page of contacts: its rows plus the pagination footer.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactPage {
    #[serde(rename = "data", default = "Vec::new")]
    pub items: Vec<Contact>,
    #[serde(default)]
    pub pagination: ContactPageMeta,
}

impl ContactPage {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<'a> IntoIterator for &'a ContactPage {
    type Item = &'a Contact;
    type IntoIter = std::slice::Iter<'a, Contact>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// One thread a contact appears in.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactConversation {
    /// How the inbox groups the thread: the DM thread id, else the post the
    /// comments hang off, else the handle.
    pub key: String,
    pub account_id: String,
    #[serde(default)]
    pub account_username: Option<String>,
    pub platform: String,
    #[serde(default)]
    pub messages: u64,
    #[serde(default)]
    pub received: u64,
    #[serde(default)]
    pub sent: u64,
    #[serde(default)]
    pub last_message_at: Option<String>,
    /// An inbox item id, readable through the inbox endpoints.
    #[serde(default)]
    pub last_item_id: Option<String>,
}

/// What a delete answered.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactDeleted {
    #[serde(default)]
    pub deleted: bool,
}

/// One CSV row the import could not read.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactImportSkip {
    pub row: u32,
    pub reason: String,
}

/// What a CSV import did.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactImportResult {
    #[serde(default)]
    pub created: u32,
    /// Rows that folded into a contact already on file.
    #[serde(default)]
    pub merged: u32,
    #[serde(default = "Vec::new")]
    pub skipped: Vec<ContactImportSkip>,
    /// Columns that matched neither a reserved field nor a custom field. They
    /// are reported, never stored.
    #[serde(rename = "unknownColumns", default = "Vec::new")]
    pub unknown_columns: Vec<String>,
}

/// What a custom field accepts.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactFieldType {
    Text,
    Number,
    Date,
    Select,
    Boolean,
    #[serde(untagged)]
    Other(String),
}

impl std::fmt::Display for ContactFieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Date => "date",
            Self::Select => "select",
            Self::Boolean => "boolean",
            Self::Other(other) => other,
        };
        f.write_str(s)
    }
}

/// A column the workspace invented.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactField {
    pub id: String,
    /// The machine name, and the CSV column header. Fixed once created.
    pub key: String,
    pub name: String,
    #[serde(rename = "type", default = "default_field_type")]
    pub field_type: ContactFieldType,
    /// Allowed values when the type is `select`.
    #[serde(default = "Vec::new")]
    pub options: Vec<String>,
    #[serde(default)]
    pub position: i32,
}

fn default_field_type() -> ContactFieldType {
    ContactFieldType::Text
}

/// Filters for `GET /contacts`. Everything is optional.
#[derive(Debug, Clone, Default)]
pub struct ListContacts {
    pub workspace_id: Option<String>,
    /// Matches a display name or any of their handles.
    pub search: Option<String>,
    pub platform: Option<String>,
    pub source: Option<ContactSource>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListContacts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn source(mut self, source: ContactSource) -> Self {
        self.source = Some(source);
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

/// The body of `POST /contacts`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateContact {
    pub workspace_id: String,
    pub channels: Vec<ContactChannel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<BTreeMap<String, String>>,
}

impl CreateContact {
    pub fn new(workspace_id: impl Into<String>, channels: Vec<ContactChannel>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            channels,
            display_name: None,
            note: None,
            fields: None,
        }
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields
            .get_or_insert_with(BTreeMap::new)
            .insert(key.into(), value.into());
        self
    }
}

/// The body of `PATCH /contacts/{id}`. Only what is set is sent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<ContactChannel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Option<String>>,
    /// A value of `None` clears that field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<BTreeMap<String, Option<String>>>,
}

impl UpdateContact {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(Some(name.into()));
        self
    }

    pub fn clear_display_name(mut self) -> Self {
        self.display_name = Some(None);
        self
    }

    pub fn channels(mut self, channels: Vec<ContactChannel>) -> Self {
        self.channels = Some(channels);
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(Some(note.into()));
        self
    }

    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields
            .get_or_insert_with(BTreeMap::new)
            .insert(key.into(), Some(value.into()));
        self
    }

    /// Clear one custom field.
    pub fn clear_field(mut self, key: impl Into<String>) -> Self {
        self.fields
            .get_or_insert_with(BTreeMap::new)
            .insert(key.into(), None);
        self
    }
}

/// The body of `POST /contacts/import`.
#[derive(Debug, Clone, Serialize)]
pub struct ImportContacts {
    pub workspace_id: String,
    /// CSV text. `platform` and `handle` are required columns; any other column
    /// is read as a custom field key.
    pub csv: String,
}

impl ImportContacts {
    pub fn new(workspace_id: impl Into<String>, csv: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            csv: csv.into(),
        }
    }
}

/// The body of `POST /contacts/fields`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateContactField {
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: ContactFieldType,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<String>,
}

impl CreateContactField {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            field_type: ContactFieldType::Text,
            options: Vec::new(),
        }
    }

    pub fn field_type(mut self, field_type: ContactFieldType) -> Self {
        self.field_type = field_type;
        self
    }

    pub fn options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }
}

/// The body of `PATCH /contacts/fields/{id}`. The key and the type are fixed
/// once created; the name and options are not.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateContactField {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

impl UpdateContactField {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn options(mut self, options: Vec<String>) -> Self {
        self.options = Some(options);
        self
    }

    pub fn position(mut self, position: i32) -> Self {
        self.position = Some(position);
        self
    }
}

/// How one thread performed over the period.
#[derive(Debug, Clone, Deserialize)]
pub struct ConversationAnalyticsRow {
    pub key: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub platform: String,
    #[serde(default)]
    pub received: u64,
    #[serde(default)]
    pub sent: u64,
    #[serde(default)]
    pub answered: u64,
    #[serde(default)]
    pub open: u64,
    /// `None` when the thread was never answered.
    #[serde(rename = "medianResponseMinutes", default)]
    pub median_response_minutes: Option<f64>,
    #[serde(rename = "firstMessageAt", default)]
    pub first_message_at: Option<String>,
    #[serde(rename = "lastMessageAt", default)]
    pub last_message_at: Option<String>,
}

/// Inbox analytics broken out per thread.
#[derive(Debug, Clone, Deserialize)]
pub struct ConversationAnalytics {
    #[serde(default = "Vec::new")]
    pub conversations: Vec<ConversationAnalyticsRow>,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub page: u32,
    #[serde(rename = "perPage", default)]
    pub per_page: u32,
}

/// How a per-conversation report is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationSort {
    Volume,
    Slowest,
    Recent,
}

impl std::fmt::Display for ConversationSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Volume => "volume",
            Self::Slowest => "slowest",
            Self::Recent => "recent",
        })
    }
}

/// Filters for `GET /analytics/inbox/conversations`.
#[derive(Debug, Clone, Default)]
pub struct ConversationAnalyticsQuery {
    pub workspace_id: Option<String>,
    pub account_id: Option<String>,
    /// The reporting period, 1 to 365. Defaults to 7.
    pub days: Option<u32>,
    pub sort: Option<ConversationSort>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ConversationAnalyticsQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn days(mut self, days: u32) -> Self {
        self.days = Some(days);
        self
    }

    pub fn sort(mut self, sort: ConversationSort) -> Self {
        self.sort = Some(sort);
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

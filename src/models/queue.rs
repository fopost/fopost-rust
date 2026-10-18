//! The posting queue — weekly slots a post can be dropped into.

use serde::{Deserialize, Serialize};

/// One weekly slot. `account_id` is `None` when it applies to every account.
#[derive(Debug, Clone, Deserialize)]
pub struct QueueSlot {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    /// 0 = Sunday … 6 = Saturday.
    #[serde(default)]
    pub weekday: u8,
    /// `HH:MM`, 24-hour, in `timezone`.
    #[serde(default)]
    pub time: String,
    /// IANA zone name.
    #[serde(default)]
    pub timezone: String,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A queued post moved to a new time because a slot changed.
#[derive(Debug, Clone, Deserialize)]
pub struct QueueMove {
    /// Post id.
    pub id: String,
    pub from: String,
    pub to: String,
}

/// What `update_slot` answers with: the slot plus the posts it moved.
#[derive(Debug, Clone, Deserialize)]
pub struct QueueSlotUpdated {
    #[serde(flatten)]
    pub slot: QueueSlot,
    #[serde(default)]
    pub moved: Vec<QueueMove>,
}

impl std::ops::Deref for QueueSlotUpdated {
    type Target = QueueSlot;

    fn deref(&self) -> &QueueSlot {
        &self.slot
    }
}

/// What `delete_slot` answers with.
#[derive(Debug, Clone, Deserialize)]
pub struct QueueSlotDeleted {
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub moved: Vec<QueueMove>,
}

/// What `next` answers with. `next` is `None` when no slot applies.
#[derive(Debug, Clone, Deserialize)]
pub struct QueueNext {
    #[serde(default)]
    pub next: Option<String>,
}

/// The body of `POST /queue/slots`. Build it with [`CreateQueueSlot::new`].
#[derive(Debug, Clone, Serialize)]
pub struct CreateQueueSlot {
    pub workspace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// 0 = Sunday … 6 = Saturday.
    pub weekday: u8,
    /// `HH:MM`, 24-hour.
    pub time: String,
    /// IANA zone name.
    pub timezone: String,
}

impl CreateQueueSlot {
    /// A slot for every account in the workspace.
    pub fn new(
        workspace_id: impl Into<String>,
        weekday: u8,
        time: impl Into<String>,
        timezone: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            account_id: None,
            weekday,
            time: time.into(),
            timezone: timezone.into(),
        }
    }

    /// Limit the slot to one account.
    pub fn account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }
}

/// The body of `PUT /queue/slots/{id}`. Only the fields you set are sent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateQueueSlot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekday: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl UpdateQueueSlot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Limit the slot to one account; `None` opens it to every account.
    pub fn account(mut self, account_id: Option<String>) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub fn weekday(mut self, weekday: u8) -> Self {
        self.weekday = Some(weekday);
        self
    }

    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.time = Some(time.into());
        self
    }

    pub fn timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }
}

//! Automations — triggers and the steps they run.

use serde::{Deserialize, Serialize};

use super::common::string_enum;

string_enum! {
    /// What sets an automation running.
    pub enum TriggerType {
        CrossPost => "cross_post",
        RssFeed => "rss_feed",
        ApiWebhook => "api_webhook",
        Schedule => "schedule",
    }
}

string_enum! {
    /// What one step of an automation does.
    pub enum ActionType {
        Publish => "publish",
        Delay => "delay",
        Transform => "transform",
    }
}

/// One step in an automation, in running order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationStep {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    pub action_type: ActionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_config: Option<serde_json::Value>,
}

impl AutomationStep {
    pub fn new(action_type: ActionType) -> Self {
        Self {
            id: None,
            position: None,
            action_type,
            action_config: None,
        }
    }

    pub fn config(mut self, config: serde_json::Value) -> Self {
        self.action_config = Some(config);
        self
    }
}

/// An automation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Automation {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub trigger_type: Option<TriggerType>,
    #[serde(default)]
    pub trigger_config: Option<serde_json::Value>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub last_triggered_at: Option<String>,
    #[serde(default)]
    pub run_count: i64,
    /// Present on `get` and `update`, empty on `list`.
    #[serde(default)]
    pub steps: Vec<AutomationStep>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// A newly created automation. `secret` is shown once and signs the webhook
/// that triggers it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationCreated {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub trigger_type: Option<TriggerType>,
    #[serde(default)]
    pub trigger_config: Option<serde_json::Value>,
    #[serde(default)]
    pub active: bool,
    /// The trigger secret, returned only at creation.
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub steps: Vec<AutomationStep>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// One run of an automation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRun {
    pub id: i64,
    #[serde(default)]
    pub automation_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub current_step: Option<i32>,
    #[serde(default)]
    pub trigger_event: Option<serde_json::Value>,
    #[serde(default)]
    pub context: Option<serde_json::Value>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    /// Per-step detail, present on `get_run`.
    #[serde(default)]
    pub logs: Vec<AutomationRunLog>,
}

/// What one step of a run did.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunLog {
    pub id: i64,
    #[serde(default)]
    pub step_position: Option<i32>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub input_snapshot: Option<serde_json::Value>,
    #[serde(default)]
    pub output_snapshot: Option<serde_json::Value>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub error_message: Option<String>,
}

/// A recent run, as the stats endpoint reports it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationRunSummary {
    pub id: i64,
    #[serde(default)]
    pub automation_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// How the automations are doing overall.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationStats {
    #[serde(default)]
    pub total_automations: u32,
    #[serde(default)]
    pub active_automations: u32,
    #[serde(default)]
    pub total_runs24h: u32,
    #[serde(default)]
    pub runs_by_status: std::collections::HashMap<String, u32>,
    #[serde(default)]
    pub recent_runs: Vec<AutomationRunSummary>,
}

/// The result of toggling an automation.
#[derive(Debug, Clone, Deserialize)]
pub struct AutomationToggled {
    pub id: String,
    #[serde(default)]
    pub active: bool,
}

/// The result of deleting an automation.
#[derive(Debug, Clone, Deserialize)]
pub struct AutomationDeleted {
    pub id: String,
    #[serde(default)]
    pub deleted: bool,
}

/// The result of firing an automation's trigger webhook.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationTriggered {
    #[serde(default)]
    pub run_id: Option<i64>,
    #[serde(default)]
    pub triggered: bool,
}

/// The body of `POST /automations`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAutomation {
    pub workspace_id: String,
    pub name: String,
    pub trigger_type: TriggerType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_config: Option<serde_json::Value>,
    pub steps: Vec<AutomationStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

impl CreateAutomation {
    pub fn new(
        workspace_id: impl Into<String>,
        name: impl Into<String>,
        trigger_type: TriggerType,
        steps: impl IntoIterator<Item = AutomationStep>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            name: name.into(),
            trigger_type,
            trigger_config: None,
            steps: steps.into_iter().collect(),
            active: None,
        }
    }

    pub fn trigger_config(mut self, config: serde_json::Value) -> Self {
        self.trigger_config = Some(config);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }
}

/// The body of `PUT /automations/{id}`. Only the fields you set are sent.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAutomation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<AutomationStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

impl UpdateAutomation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn steps(mut self, steps: impl IntoIterator<Item = AutomationStep>) -> Self {
        self.steps = Some(steps.into_iter().collect());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }
}

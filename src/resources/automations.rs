//! `client.automations()` — triggers and the steps they run.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    Automation, AutomationCreated, AutomationDeleted, AutomationRun, AutomationStats,
    AutomationToggled, AutomationTriggered, CreateAutomation, Page, UpdateAutomation,
};

/// Automations.
#[derive(Debug, Clone)]
pub struct Automations<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Automations<'_> {
    /// Every automation the key can reach, without their steps.
    pub async fn list(&self) -> Result<Vec<Automation>> {
        let body: Envelope<Vec<Automation>> = self
            .http
            .send::<_, ()>(Method::GET, "/automations", None, None)
            .await?;
        Ok(body.data)
    }

    /// One automation with its steps.
    pub async fn get(&self, id: &str) -> Result<Automation> {
        let body: Envelope<Automation> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/automations/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Create an automation. The `secret` on the answer is shown once and signs
    /// the webhook that triggers it.
    pub async fn create(&self, automation: &CreateAutomation) -> Result<AutomationCreated> {
        let body: Envelope<AutomationCreated> = self
            .http
            .send(Method::POST, "/automations", None, Some(automation))
            .await?;
        Ok(body.data)
    }

    /// Partial update — only the fields set on the body are sent. Passing
    /// `steps` replaces the whole list.
    pub async fn update(&self, id: &str, changes: &UpdateAutomation) -> Result<Automation> {
        let body: Envelope<Automation> = self
            .http
            .send(
                Method::PUT,
                &format!("/automations/{id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Delete an automation. Runs already finished keep their history.
    pub async fn delete(&self, id: &str) -> Result<AutomationDeleted> {
        let body: Envelope<AutomationDeleted> = self
            .http
            .send::<_, ()>(Method::DELETE, &format!("/automations/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Flip an automation on or off without deleting it.
    pub async fn toggle(&self, id: &str) -> Result<AutomationToggled> {
        let body: Envelope<AutomationToggled> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/automations/{id}/toggle"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Runs of one automation, newest first.
    pub async fn runs(
        &self,
        id: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> Result<Page<AutomationRun>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "page", page);
        push_opt(&mut query, "per_page", per_page);
        self.http
            .send::<_, ()>(
                Method::GET,
                &format!("/automations/{id}/runs"),
                Some(query),
                None,
            )
            .await
    }

    /// One run, with the per-step log.
    pub async fn get_run(&self, id: &str, run_id: i64) -> Result<AutomationRun> {
        let body: Envelope<AutomationRun> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/automations/{id}/runs/{run_id}"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// How the automations are doing overall.
    pub async fn stats(&self) -> Result<AutomationStats> {
        let body: Envelope<AutomationStats> = self
            .http
            .send::<_, ()>(Method::GET, "/automations/stats", None, None)
            .await?;
        Ok(body.data)
    }

    /// Fire an `api_webhook` automation. The payload becomes the run's trigger
    /// event, so the steps can read from it.
    pub async fn trigger(
        &self,
        id: &str,
        payload: Option<&serde_json::Value>,
    ) -> Result<AutomationTriggered> {
        let empty = serde_json::json!({});
        let body: Envelope<AutomationTriggered> = self
            .http
            .send(
                Method::POST,
                &format!("/automations/{id}/trigger"),
                None,
                Some(payload.unwrap_or(&empty)),
            )
            .await?;
        Ok(body.data)
    }
}

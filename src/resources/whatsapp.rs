//! `client.whatsapp()` — a WhatsApp Business connection.
//!
//! The platform owns templates, flows, the business profile and the commerce
//! settings, so every method here is a live read or write against the customer's
//! own WhatsApp Business Account. Nothing is cached, and all of it answers 503
//! until WhatsApp is set up on the deployment.

use reqwest::Method;
use serde_json::{json, Value};

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    CreateWhatsappFlow, CreateWhatsappTemplate, ImportWhatsappTemplate, UpdateWhatsappCommerce,
    UpdateWhatsappFlow, UpdateWhatsappProfile, UpdateWhatsappTemplate, WhatsappBlockResult,
    WhatsappCommerceSettings, WhatsappEncryptionKeyStatus, WhatsappFlow, WhatsappFlowJsonResult,
    WhatsappFlowResponse, WhatsappGroup, WhatsappGroupInput, WhatsappProfile,
    WhatsappSandboxSession, WhatsappTemplate,
};

/// WhatsApp Business. Needs the `accounts` scope, except the sandbox, which
/// sends a template and needs `publish`.
#[derive(Debug, Clone)]
pub struct Whatsapp<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Whatsapp<'_> {
    // ─── Profile ────────────────────────────────────────────────────────

    /// The profile on the number, plus its quality rating and limit tier.
    pub async fn profile(&self, account_id: &str) -> Result<WhatsappProfile> {
        let path = format!("/accounts/{account_id}/whatsapp/profile");
        let body: Envelope<WhatsappProfile> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// A partial update: omitted fields keep their value.
    pub async fn update_profile(
        &self,
        account_id: &str,
        payload: &UpdateWhatsappProfile,
    ) -> Result<WhatsappProfile> {
        let path = format!("/accounts/{account_id}/whatsapp/profile");
        let body: Envelope<WhatsappProfile> = self
            .http
            .send(Method::PATCH, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// A review, not a write: the number keeps its old name until it passes.
    pub async fn request_display_name(&self, account_id: &str, display_name: &str) -> Result<()> {
        let path = format!("/accounts/{account_id}/whatsapp/profile/display-name");
        let payload = json!({ "display_name": display_name });
        let _: Envelope<Value> = self
            .http
            .send(Method::POST, &path, None, Some(&payload))
            .await?;
        Ok(())
    }

    /// Sets the public username on the number.
    pub async fn set_username(&self, account_id: &str, username: &str) -> Result<WhatsappProfile> {
        let path = format!("/accounts/{account_id}/whatsapp/profile/username");
        let payload = json!({ "username": username });
        let body: Envelope<WhatsappProfile> = self
            .http
            .send(Method::PUT, &path, None, Some(&payload))
            .await?;
        Ok(body.data)
    }

    // ─── Templates ──────────────────────────────────────────────────────

    /// Every template on the account, with its review status.
    pub async fn templates(
        &self,
        account_id: &str,
        after: Option<&str>,
    ) -> Result<Vec<WhatsappTemplate>> {
        let path = format!("/accounts/{account_id}/whatsapp/templates");
        let mut query: Query = Vec::new();
        push_opt(&mut query, "after", after);
        let body: Envelope<Vec<WhatsappTemplate>> = self
            .http
            .send::<_, ()>(Method::GET, &path, Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// The pre-written templates the platform offers, for adapting.
    pub async fn template_library(
        &self,
        account_id: &str,
        search: Option<&str>,
    ) -> Result<Vec<Value>> {
        let path = format!("/accounts/{account_id}/whatsapp/templates/library");
        let mut query: Query = Vec::new();
        push_opt(&mut query, "search", search);
        let body: Envelope<Vec<Value>> = self
            .http
            .send::<_, ()>(Method::GET, &path, Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// One template and the review status it currently has.
    pub async fn template(&self, account_id: &str, template_id: &str) -> Result<WhatsappTemplate> {
        let path = format!("/accounts/{account_id}/whatsapp/templates/{template_id}");
        let body: Envelope<WhatsappTemplate> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Files a template for review. The result carries the status the platform
    /// assigned, which is `PENDING` on a normal submission.
    pub async fn create_template(
        &self,
        account_id: &str,
        payload: &CreateWhatsappTemplate,
    ) -> Result<WhatsappTemplate> {
        let path = format!("/accounts/{account_id}/whatsapp/templates");
        let body: Envelope<WhatsappTemplate> = self
            .http
            .send(Method::POST, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// Creates a template from one of the platform's library entries.
    pub async fn import_template(
        &self,
        account_id: &str,
        payload: &ImportWhatsappTemplate,
    ) -> Result<WhatsappTemplate> {
        let path = format!("/accounts/{account_id}/whatsapp/templates/import");
        let body: Envelope<WhatsappTemplate> = self
            .http
            .send(Method::POST, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// Edits a template. The name cannot change.
    pub async fn update_template(
        &self,
        account_id: &str,
        template_id: &str,
        payload: &UpdateWhatsappTemplate,
    ) -> Result<WhatsappTemplate> {
        let path = format!("/accounts/{account_id}/whatsapp/templates/{template_id}");
        let body: Envelope<WhatsappTemplate> = self
            .http
            .send(Method::PATCH, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// The name is required: it is what the platform deletes by.
    pub async fn delete_template(
        &self,
        account_id: &str,
        template_id: &str,
        name: &str,
    ) -> Result<()> {
        let path = format!("/accounts/{account_id}/whatsapp/templates/{template_id}");
        let query: Query = vec![("name", name.to_string())];
        let _: Envelope<Value> = self
            .http
            .send::<_, ()>(Method::DELETE, &path, Some(query), None)
            .await?;
        Ok(())
    }

    // ─── Groups ─────────────────────────────────────────────────────────

    /// The groups this number created.
    pub async fn groups(&self, account_id: &str) -> Result<Vec<WhatsappGroup>> {
        let path = format!("/accounts/{account_id}/whatsapp/groups");
        let body: Envelope<Vec<WhatsappGroup>> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Participation is invite-only: send the invite link, there is no add.
    pub async fn create_group(
        &self,
        account_id: &str,
        payload: &WhatsappGroupInput,
    ) -> Result<WhatsappGroup> {
        let path = format!("/accounts/{account_id}/whatsapp/groups");
        let body: Envelope<WhatsappGroup> = self
            .http
            .send(Method::POST, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// One group and its participant count.
    pub async fn group(&self, account_id: &str, group_id: &str) -> Result<WhatsappGroup> {
        let path = format!("/accounts/{account_id}/whatsapp/groups/{group_id}");
        let body: Envelope<WhatsappGroup> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Changes a group's subject or description.
    pub async fn update_group(
        &self,
        account_id: &str,
        group_id: &str,
        payload: &WhatsappGroupInput,
    ) -> Result<WhatsappGroup> {
        let path = format!("/accounts/{account_id}/whatsapp/groups/{group_id}");
        let body: Envelope<WhatsappGroup> = self
            .http
            .send(Method::PATCH, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// Removes the group.
    pub async fn delete_group(&self, account_id: &str, group_id: &str) -> Result<()> {
        let path = format!("/accounts/{account_id}/whatsapp/groups/{group_id}");
        let _: Envelope<Value> = self
            .http
            .send::<_, ()>(Method::DELETE, &path, None, None)
            .await?;
        Ok(())
    }

    /// The link someone joins the group with.
    pub async fn group_invite_link(
        &self,
        account_id: &str,
        group_id: &str,
    ) -> Result<Option<String>> {
        let path = format!("/accounts/{account_id}/whatsapp/groups/{group_id}/invite-link");
        let body: Envelope<Value> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(invite_link_of(&body.data))
    }

    /// Issues a new link and invalidates the old one.
    pub async fn reset_group_invite_link(
        &self,
        account_id: &str,
        group_id: &str,
    ) -> Result<Option<String>> {
        let path = format!("/accounts/{account_id}/whatsapp/groups/{group_id}/invite-link");
        let body: Envelope<Value> = self
            .http
            .send::<_, ()>(Method::POST, &path, None, None)
            .await?;
        Ok(invite_link_of(&body.data))
    }

    /// Removes people from the group. There is no matching add.
    pub async fn remove_group_participants(
        &self,
        account_id: &str,
        group_id: &str,
        users: &[String],
    ) -> Result<()> {
        let path = format!("/accounts/{account_id}/whatsapp/groups/{group_id}/participants");
        let payload = json!({ "users": users });
        let _: Envelope<Value> = self
            .http
            .send(Method::DELETE, &path, None, Some(&payload))
            .await?;
        Ok(())
    }

    // ─── Blocking ───────────────────────────────────────────────────────

    /// The numbers this account has blocked.
    pub async fn blocked(&self, account_id: &str, after: Option<&str>) -> Result<Vec<String>> {
        let path = format!("/accounts/{account_id}/whatsapp/block");
        let mut query: Query = Vec::new();
        push_opt(&mut query, "after", after);
        let body: Envelope<Vec<String>> = self
            .http
            .send::<_, ()>(Method::GET, &path, Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Blocks up to 100 numbers, and names the ones the platform refused.
    pub async fn block_users(
        &self,
        account_id: &str,
        users: &[String],
    ) -> Result<WhatsappBlockResult> {
        let path = format!("/accounts/{account_id}/whatsapp/block");
        let payload = json!({ "users": users });
        let body: Envelope<WhatsappBlockResult> = self
            .http
            .send(Method::POST, &path, None, Some(&payload))
            .await?;
        Ok(body.data)
    }

    /// Unblocks up to 100 numbers.
    pub async fn unblock_users(
        &self,
        account_id: &str,
        users: &[String],
    ) -> Result<WhatsappBlockResult> {
        let path = format!("/accounts/{account_id}/whatsapp/block");
        let payload = json!({ "users": users });
        let body: Envelope<WhatsappBlockResult> = self
            .http
            .send(Method::DELETE, &path, None, Some(&payload))
            .await?;
        Ok(body.data)
    }

    // ─── Commerce ───────────────────────────────────────────────────────

    /// Whether the cart and catalog show on the number.
    pub async fn commerce_settings(&self, account_id: &str) -> Result<WhatsappCommerceSettings> {
        let path = format!("/accounts/{account_id}/whatsapp/commerce");
        let body: Envelope<WhatsappCommerceSettings> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Turns the cart or the catalog on or off.
    pub async fn update_commerce_settings(
        &self,
        account_id: &str,
        payload: &UpdateWhatsappCommerce,
    ) -> Result<WhatsappCommerceSettings> {
        let path = format!("/accounts/{account_id}/whatsapp/commerce");
        let body: Envelope<WhatsappCommerceSettings> = self
            .http
            .send(Method::PATCH, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// Points the number at a catalog the customer already owns.
    pub async fn link_catalog(
        &self,
        account_id: &str,
        catalog_id: &str,
    ) -> Result<WhatsappCommerceSettings> {
        let path = format!("/accounts/{account_id}/whatsapp/commerce/catalog");
        let payload = json!({ "catalog_id": catalog_id });
        let body: Envelope<WhatsappCommerceSettings> = self
            .http
            .send(Method::POST, &path, None, Some(&payload))
            .await?;
        Ok(body.data)
    }

    // ─── Flows ──────────────────────────────────────────────────────────

    /// The in-chat forms on this account, with their validation errors.
    pub async fn flows(&self, account_id: &str) -> Result<Vec<WhatsappFlow>> {
        let path = format!("/accounts/{account_id}/whatsapp/flows");
        let body: Envelope<Vec<WhatsappFlow>> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// One flow and what the platform found wrong with it.
    pub async fn flow(&self, account_id: &str, flow_id: &str) -> Result<WhatsappFlow> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/{flow_id}");
        let body: Envelope<WhatsappFlow> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Creates a draft flow; its screens are uploaded separately.
    pub async fn create_flow(
        &self,
        account_id: &str,
        payload: &CreateWhatsappFlow,
    ) -> Result<WhatsappFlow> {
        let path = format!("/accounts/{account_id}/whatsapp/flows");
        let body: Envelope<WhatsappFlow> = self
            .http
            .send(Method::POST, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// Changes a flow's name, categories or endpoint.
    pub async fn update_flow(
        &self,
        account_id: &str,
        flow_id: &str,
        payload: &UpdateWhatsappFlow,
    ) -> Result<WhatsappFlow> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/{flow_id}");
        let body: Envelope<WhatsappFlow> = self
            .http
            .send(Method::PATCH, &path, None, Some(payload))
            .await?;
        Ok(body.data)
    }

    /// Drafts only; a published flow is deprecated instead.
    pub async fn delete_flow(&self, account_id: &str, flow_id: &str) -> Result<()> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/{flow_id}");
        let _: Envelope<Value> = self
            .http
            .send::<_, ()>(Method::DELETE, &path, None, None)
            .await?;
        Ok(())
    }

    /// Replaces the flow's screens. The platform answers with its validation
    /// errors rather than refusing, so they come back as data.
    pub async fn upload_flow_json(
        &self,
        account_id: &str,
        flow_id: &str,
        flow_json: &Value,
    ) -> Result<WhatsappFlowJsonResult> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/{flow_id}/json");
        let payload = json!({ "flow_json": flow_json });
        let body: Envelope<WhatsappFlowJsonResult> = self
            .http
            .send(Method::PUT, &path, None, Some(&payload))
            .await?;
        Ok(body.data)
    }

    /// Makes the flow sendable. A published flow can no longer be deleted.
    pub async fn publish_flow(&self, account_id: &str, flow_id: &str) -> Result<WhatsappFlow> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/{flow_id}/publish");
        let body: Envelope<WhatsappFlow> = self
            .http
            .send::<_, ()>(Method::POST, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Retires a published flow.
    pub async fn deprecate_flow(&self, account_id: &str, flow_id: &str) -> Result<WhatsappFlow> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/{flow_id}/deprecate");
        let body: Envelope<WhatsappFlow> = self
            .http
            .send::<_, ()>(Method::POST, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// What people submitted through this account's flows.
    pub async fn flow_responses(&self, account_id: &str) -> Result<Vec<WhatsappFlowResponse>> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/responses");
        let body: Envelope<Vec<WhatsappFlowResponse>> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Whether a business public key is registered, and how the platform judged it.
    pub async fn encryption_key_status(
        &self,
        account_id: &str,
    ) -> Result<WhatsappEncryptionKeyStatus> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/encryption-key");
        let body: Envelope<WhatsappEncryptionKeyStatus> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Registers the public half of the key the platform encrypts a flow
    /// endpoint's payloads with. The private half stays with the customer.
    pub async fn set_encryption_key(
        &self,
        account_id: &str,
        business_public_key: &str,
    ) -> Result<WhatsappEncryptionKeyStatus> {
        let path = format!("/accounts/{account_id}/whatsapp/flows/encryption-key");
        let payload = json!({ "business_public_key": business_public_key });
        let body: Envelope<WhatsappEncryptionKeyStatus> = self
            .http
            .send(Method::PUT, &path, None, Some(&payload))
            .await?;
        Ok(body.data)
    }

    // ─── Account state and sandbox ──────────────────────────────────────

    /// The account review state and the number's quality and limit tier.
    pub async fn account_events(&self, account_id: &str) -> Result<Value> {
        let path = format!("/accounts/{account_id}/whatsapp/events");
        let body: Envelope<Value> = self
            .http
            .send::<_, ()>(Method::GET, &path, None, None)
            .await?;
        Ok(body.data)
    }

    /// Sandbox invitations for a workspace.
    pub async fn sandbox_sessions(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<WhatsappSandboxSession>> {
        let query: Query = vec![("workspaceId", workspace_id.to_string())];
        let body: Envelope<Vec<WhatsappSandboxSession>> = self
            .http
            .send::<_, ()>(Method::GET, "/whatsapp/sandbox/sessions", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Invites one tester to the platform-owned test number. Inviting sends a
    /// template, so it needs the `publish` scope.
    pub async fn create_sandbox_session(
        &self,
        workspace_id: &str,
        phone_number: &str,
    ) -> Result<WhatsappSandboxSession> {
        let payload = json!({ "workspaceId": workspace_id, "phoneNumber": phone_number });
        let body: Envelope<WhatsappSandboxSession> = self
            .http
            .send(
                Method::POST,
                "/whatsapp/sandbox/sessions",
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }
}

fn invite_link_of(value: &Value) -> Option<String> {
    value
        .get("inviteLink")
        .and_then(Value::as_str)
        .map(str::to_string)
}

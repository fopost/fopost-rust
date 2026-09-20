//! `client.contacts()` — the people behind the inbox, and the fields a
//! workspace keeps about them.
//!
//! A contact is one person however many handles they write from. An inbound
//! inbox item files its author, a reply files whoever you answered, and both
//! fold into whatever is already on file. Every method needs the `inbox`
//! scope, except [`Contacts::conversation_analytics`], which needs `analytics`.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    Contact, ContactConversation, ContactDeleted, ContactField, ContactImportResult, ContactPage,
    ConversationAnalytics, ConversationAnalyticsQuery, CreateContact, CreateContactField,
    ImportContacts, ListContacts, UpdateContact, UpdateContactField,
};

/// Contacts.
#[derive(Debug, Clone)]
pub struct Contacts<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Contacts<'_> {
    /// One page of contacts, most recently active first.
    ///
    /// Leave the workspace unset to span every workspace the key reaches; each
    /// contact then carries `workspace_id`.
    pub async fn list(&self, params: &ListContacts) -> Result<ContactPage> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "search", params.search.as_ref());
        push_opt(&mut query, "platform", params.platform.as_ref());
        push_opt(&mut query, "source", params.source.as_ref());
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        self.http
            .send::<ContactPage, ()>(Method::GET, "/contacts", Some(query), None)
            .await
    }

    /// One contact. A contact in a workspace the key cannot reach answers 404,
    /// exactly as an id that never existed does.
    pub async fn get(&self, id: &str) -> Result<Contact> {
        let body: Envelope<Contact> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/contacts/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// File a contact.
    ///
    /// It folds into the contact that already holds the first channel, so this
    /// cannot duplicate someone the inbox has already met.
    pub async fn create(&self, contact: &CreateContact) -> Result<Contact> {
        let body: Envelope<Contact> = self
            .http
            .send(Method::POST, "/contacts", None, Some(contact))
            .await?;
        Ok(body.data)
    }

    /// Patch a contact. Only what is set is sent.
    pub async fn update(&self, id: &str, changes: &UpdateContact) -> Result<Contact> {
        let body: Envelope<Contact> = self
            .http
            .send(
                Method::PATCH,
                &format!("/contacts/{id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove a contact and its field values. The messages they sent stay in
    /// the inbox, so a later message files them again.
    pub async fn delete(&self, id: &str) -> Result<ContactDeleted> {
        let body: Envelope<ContactDeleted> = self
            .http
            .send::<_, ()>(Method::DELETE, &format!("/contacts/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// The threads one contact appears in, newest first.
    ///
    /// Matched on their channels, so a contact merged from two handles brings
    /// both threads with it.
    pub async fn conversations(
        &self,
        id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<ContactConversation>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "limit", limit);
        let body: Envelope<Vec<ContactConversation>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/contacts/{id}/conversations"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Import contacts from CSV text.
    ///
    /// `platform` and `handle` are required columns; any other column is read
    /// as a custom field key, and one matching no field comes back in
    /// `unknown_columns` rather than being stored.
    pub async fn import(&self, input: &ImportContacts) -> Result<ContactImportResult> {
        let body: Envelope<ContactImportResult> = self
            .http
            .send(Method::POST, "/contacts/import", None, Some(input))
            .await?;
        Ok(body.data)
    }

    /// The columns this workspace keeps about its contacts, in display order.
    pub async fn list_fields(&self, workspace_id: &str) -> Result<Vec<ContactField>> {
        let query: Query = vec![("workspace_id", workspace_id.to_string())];
        let body: Envelope<Vec<ContactField>> = self
            .http
            .send::<_, ()>(Method::GET, "/contacts/fields", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Add a custom field. A duplicate key answers 409.
    pub async fn create_field(
        &self,
        workspace_id: &str,
        field: &CreateContactField,
    ) -> Result<ContactField> {
        let query: Query = vec![("workspace_id", workspace_id.to_string())];
        let body: Envelope<ContactField> = self
            .http
            .send(Method::POST, "/contacts/fields", Some(query), Some(field))
            .await?;
        Ok(body.data)
    }

    /// Rename a field, or change its options or position.
    pub async fn update_field(
        &self,
        id: &str,
        changes: &UpdateContactField,
    ) -> Result<ContactField> {
        let body: Envelope<ContactField> = self
            .http
            .send(
                Method::PATCH,
                &format!("/contacts/fields/{id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove the field and every answer to it.
    pub async fn delete_field(&self, id: &str) -> Result<ContactDeleted> {
        let body: Envelope<ContactDeleted> = self
            .http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/contacts/fields/{id}"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Volume and median reply time per thread.
    ///
    /// Counts and timings only: no message text and no author. Needs the
    /// `analytics` scope rather than `inbox`.
    pub async fn conversation_analytics(
        &self,
        params: &ConversationAnalyticsQuery,
    ) -> Result<ConversationAnalytics> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "accountId", params.account_id.as_ref());
        push_opt(&mut query, "days", params.days);
        push_opt(&mut query, "sort", params.sort.as_ref());
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        let body: Envelope<ConversationAnalytics> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/analytics/inbox/conversations",
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }
}

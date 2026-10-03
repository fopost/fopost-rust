//! `client.broadcasts()` and `client.sequences()` — one message into many
//! conversations, and a series of messages on a delay.
//!
//! Neither opens a cold DM: every message lands in a direct-message thread the
//! contact already started. Both honour each network's messaging window
//! server-side. Messenger and Instagram take a business-initiated message only
//! within 24 hours of the contact's last one, so a recipient outside it comes
//! back skipped with `window_closed` and nothing is attempted — which is why
//! the number sent is often lower than the audience. Telegram, Slack, Bluesky
//! and Reddit have no window.
//!
//! Reading needs the `inbox` scope; sending, cancelling, enrolling and
//! unenrolling also need `publish`.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    Broadcast, BroadcastCancelled, BroadcastPage, BroadcastSent, ContactDeleted, CreateBroadcast,
    CreateSequence, Enroll, Enrolled, EnrollmentPage, ListBroadcasts, ListEnrollments,
    ListRecipients, ListSequences, RecipientPage, Sequence, SequencePage, Unenrolled,
    UpdateBroadcast, UpdateSequence,
};

/// Broadcasts.
#[derive(Debug, Clone)]
pub struct Broadcasts<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Broadcasts<'_> {
    /// One page of broadcasts, newest first.
    ///
    /// Leave the workspace unset to span every workspace the key reaches; each
    /// broadcast then carries `workspace_id`.
    pub async fn list(&self, params: &ListBroadcasts) -> Result<BroadcastPage> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(
            &mut query,
            "status",
            params.status.as_ref().map(|s| s.to_string()).as_ref(),
        );
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        self.http
            .send::<BroadcastPage, ()>(Method::GET, "/broadcasts", Some(query), None)
            .await
    }

    /// One broadcast. A broadcast in a workspace the key cannot reach answers
    /// 404, exactly as an id that never existed does.
    pub async fn get(&self, id: &str) -> Result<Broadcast> {
        let body: Envelope<Broadcast> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/broadcasts/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Write a broadcast without sending it.
    ///
    /// Set `scheduled_at` to have it go out on its own at that time;
    /// otherwise call [`Broadcasts::send`].
    pub async fn create(&self, broadcast: &CreateBroadcast) -> Result<Broadcast> {
        let body: Envelope<Broadcast> = self
            .http
            .send(Method::POST, "/broadcasts", None, Some(broadcast))
            .await?;
        Ok(body.data)
    }

    /// Patch a broadcast. Only what is set is sent, and only a draft or
    /// scheduled broadcast can be edited.
    pub async fn update(&self, id: &str, changes: &UpdateBroadcast) -> Result<Broadcast> {
        let body: Envelope<Broadcast> = self
            .http
            .send(
                Method::PATCH,
                &format!("/broadcasts/{id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Freeze the audience into a recipient list and start sending.
    ///
    /// The returned `recipients` is how many contacts matched, not how many
    /// will be messaged — the messaging window decides that. Needs the
    /// `publish` scope as well as `inbox`.
    pub async fn send(&self, id: &str) -> Result<BroadcastSent> {
        let body: Envelope<BroadcastSent> = self
            .http
            .send(
                Method::POST,
                &format!("/broadcasts/{id}/send"),
                None,
                Some(&serde_json::json!({})),
            )
            .await?;
        Ok(body.data)
    }

    /// Stop a broadcast where it stands.
    ///
    /// Anyone not yet written to stays unsent; messages already delivered are
    /// not recalled. Needs the `publish` scope.
    pub async fn cancel(&self, id: &str) -> Result<BroadcastCancelled> {
        let body: Envelope<BroadcastCancelled> = self
            .http
            .send(
                Method::POST,
                &format!("/broadcasts/{id}/cancel"),
                None,
                Some(&serde_json::json!({})),
            )
            .await?;
        Ok(body.data)
    }

    /// One row per contact, with what became of their message. A skipped row
    /// carries `skip_reason`.
    pub async fn recipients(&self, id: &str, params: &ListRecipients) -> Result<RecipientPage> {
        let mut query: Query = Vec::new();
        push_opt(
            &mut query,
            "status",
            params.status.as_ref().map(|s| s.to_string()).as_ref(),
        );
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        self.http
            .send::<RecipientPage, ()>(
                Method::GET,
                &format!("/broadcasts/{id}/recipients"),
                Some(query),
                None,
            )
            .await
    }

    /// Remove a broadcast and its recipient records. Messages already sent
    /// stay in the conversations they went to.
    pub async fn delete(&self, id: &str) -> Result<ContactDeleted> {
        let body: Envelope<ContactDeleted> = self
            .http
            .send::<_, ()>(Method::DELETE, &format!("/broadcasts/{id}"), None, None)
            .await?;
        Ok(body.data)
    }
}

/// Drip sequences.
#[derive(Debug, Clone)]
pub struct Sequences<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Sequences<'_> {
    /// One page of sequences.
    pub async fn list(&self, params: &ListSequences) -> Result<SequencePage> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspace_id", params.workspace_id.as_ref());
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        self.http
            .send::<SequencePage, ()>(Method::GET, "/sequences", Some(query), None)
            .await
    }

    /// One sequence.
    pub async fn get(&self, id: &str) -> Result<Sequence> {
        let body: Envelope<Sequence> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/sequences/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Write a sequence. Creating one enrolls nobody.
    pub async fn create(&self, sequence: &CreateSequence) -> Result<Sequence> {
        let body: Envelope<Sequence> = self
            .http
            .send(Method::POST, "/sequences", None, Some(sequence))
            .await?;
        Ok(body.data)
    }

    /// Patch a sequence.
    ///
    /// Pausing stops every enrollment from firing without ending any of them;
    /// resuming picks them up where they stood.
    pub async fn update(&self, id: &str, changes: &UpdateSequence) -> Result<Sequence> {
        let body: Envelope<Sequence> = self
            .http
            .send(
                Method::PATCH,
                &format!("/sequences/{id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Put contacts on the sequence, by id or by audience.
    ///
    /// Re-enrolling someone restarts their walk from the first step rather
    /// than running two in parallel. Needs `publish` as well as `inbox`.
    pub async fn enroll(&self, id: &str, who: &Enroll) -> Result<Enrolled> {
        let body: Envelope<Enrolled> = self
            .http
            .send(
                Method::POST,
                &format!("/sequences/{id}/enroll"),
                None,
                Some(who),
            )
            .await?;
        Ok(body.data)
    }

    /// Take contacts off the sequence. Nothing further fires for them. Needs
    /// the `publish` scope.
    pub async fn unenroll(&self, id: &str, contact_ids: &[String]) -> Result<Unenrolled> {
        let body: Envelope<Unenrolled> = self
            .http
            .send(
                Method::POST,
                &format!("/sequences/{id}/unenroll"),
                None,
                Some(&serde_json::json!({ "contact_ids": contact_ids })),
            )
            .await?;
        Ok(body.data)
    }

    /// Who is on the sequence, what step they are at, and when the next one
    /// is due.
    pub async fn enrollments(&self, id: &str, params: &ListEnrollments) -> Result<EnrollmentPage> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "page", params.page);
        push_opt(&mut query, "per_page", params.per_page);
        self.http
            .send::<EnrollmentPage, ()>(
                Method::GET,
                &format!("/sequences/{id}/enrollments"),
                Some(query),
                None,
            )
            .await
    }

    /// Remove a sequence and every enrollment on it.
    pub async fn delete(&self, id: &str) -> Result<ContactDeleted> {
        let body: Envelope<ContactDeleted> = self
            .http
            .send::<_, ()>(Method::DELETE, &format!("/sequences/{id}"), None, None)
            .await?;
        Ok(body.data)
    }
}

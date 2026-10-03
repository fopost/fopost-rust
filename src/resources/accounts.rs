//! `client.accounts()` — the social accounts a workspace publishes through.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    Account, AccountAnalyticsHistory, AccountCreated, AccountDetail, AccountHealth, AccountMoved,
    AccountPlatformMetrics, AccountRenamed, AccountsHealthSummary, BlueskyLanguages, Community,
    CommunitySearchResult, CreateAccount, CreatePinterestBoard, CreateYouTubePlaylist,
    CredentialCheck, DiscordAck, DiscordChannel, DiscordEventInput, DiscordIdentity, DiscordMember,
    DiscordMessage, DiscordMessageRef, DiscordRole, DiscordRoleInput, DiscordScheduledEvent,
    DiscordThread, DiscordThreadInput, InstagramAudio, InstagramPublishingLimit, InstagramStory,
    InstagramStoryInsights, LinkedInMention, ListAccounts, Message, MetaGreeting, MetaGreetingText,
    MetaIceBreaker, MetaIceBreakers, MetaPersistentMenu, MetaPersistentMenuEntry, PinterestBoard,
    PrimaryToggled, SlackChannel, SlackIdentity, SlackMember, TelegramBotCommand,
    TelegramBotCommands, TelegramConnectCode, TelegramConnectStatus, TikTokCreatorInfo,
    TikTokMusic, TikTokPlace, TikTokVideoSource, TokenRefreshed, UpdateDiscordIdentity,
    UpdateSlackIdentity, UploadYouTubeCaptions, WebhookSubscription, YouTubeCaptionTrack,
    YouTubePlaylist, YouTubeTranscript,
};

/// Connected accounts, their health, and their X communities.
#[derive(Debug, Clone)]
pub struct Accounts<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Accounts<'_> {
    /// Connected accounts, across every workspace the key reaches unless one is named.
    pub async fn list(&self, workspace_id: Option<&str>) -> Result<Vec<Account>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspaceId", workspace_id);
        let body: Envelope<Vec<Account>> = self
            .http
            .send::<_, ()>(Method::GET, "/accounts", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Connected accounts matching the filters, e.g. one account group's.
    pub async fn list_with(&self, filters: &ListAccounts) -> Result<Vec<Account>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspaceId", filters.workspace_id.as_deref());
        push_opt(&mut query, "group_id", filters.group_id.as_deref());
        let body: Envelope<Vec<Account>> = self
            .http
            .send::<_, ()>(Method::GET, "/accounts", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// One account in full, with the workspace it belongs to.
    pub async fn get(&self, id: &str) -> Result<AccountDetail> {
        let body: Envelope<AccountDetail> = self
            .http
            .send::<_, ()>(Method::GET, &format!("/accounts/{id}"), None, None)
            .await?;
        Ok(body.data)
    }

    /// The numbers only this account's network reports, in its own vocabulary:
    /// ad-break earnings, story taps, a retention curve, the search terms behind
    /// a listing. Keyed by the platform's own metric names, read from the newest
    /// collected snapshot rather than fetched live. Needs the `analytics` scope.
    ///
    /// A network whose metric access has not been granted yet answers `503`
    /// (`platform_metrics_unavailable`) rather than an empty set.
    pub async fn platform_metrics(&self, id: &str) -> Result<AccountPlatformMetrics> {
        let query: Query = vec![("raw", "true".into())];
        let body: Envelope<AccountPlatformMetrics> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/insights"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Connect an account with credentials you already hold. The dashboard's
    /// OAuth flow is the other way in, and the usual one.
    pub async fn create(&self, account: &CreateAccount) -> Result<AccountCreated> {
        let body: Envelope<AccountCreated> = self
            .http
            .send(Method::POST, "/accounts", None, Some(account))
            .await?;
        Ok(body.data)
    }

    /// Set the name shown instead of the platform name. `None` restores it.
    pub async fn rename(&self, id: &str, display_name: Option<&str>) -> Result<AccountRenamed> {
        let payload = serde_json::json!({ "display_name": display_name });
        let body: Envelope<AccountRenamed> = self
            .http
            .send(
                Method::PATCH,
                &format!("/accounts/{id}"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Move an account to another workspace the caller owns. It leaves its account
    /// groups. A `409` with code `move_blocked` lists the blockers in the body's
    /// `blocking_tables`.
    pub async fn move_to(&self, id: &str, workspace_id: &str) -> Result<AccountMoved> {
        let payload = serde_json::json!({ "workspace_id": workspace_id });
        let body: Envelope<AccountMoved> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/move"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Mint a one-time code, valid for 15 minutes. Sending its `command` to the bot
    /// in a chat connects that chat. `workspace_id` may be `None` for a key bound
    /// to one workspace.
    pub async fn create_telegram_connect_code(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<TelegramConnectCode> {
        let payload = match workspace_id {
            Some(id) => serde_json::json!({ "workspaceId": id }),
            None => serde_json::json!({}),
        };
        let body: Envelope<TelegramConnectCode> = self
            .http
            .send(
                Method::POST,
                "/accounts/telegram/connect-code",
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Whether a connect code is still pending, connected a chat, failed, or expired.
    pub async fn telegram_connect_status(&self, code: &str) -> Result<TelegramConnectStatus> {
        let query: Query = vec![("code", code.to_string())];
        let body: Envelope<TelegramConnectStatus> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/accounts/telegram/connect-code/status",
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The command menu the bot shows in a connected chat.
    pub async fn telegram_bot_commands(&self, id: &str) -> Result<TelegramBotCommands> {
        let body: Envelope<TelegramBotCommands> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/telegram/commands"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Replace the command menu for a connected chat with 1-100 commands.
    pub async fn set_telegram_bot_commands(
        &self,
        id: &str,
        commands: &[TelegramBotCommand],
    ) -> Result<TelegramBotCommands> {
        let payload = serde_json::json!({ "commands": commands });
        let body: Envelope<TelegramBotCommands> = self
            .http
            .send(
                Method::PUT,
                &format!("/accounts/{id}/telegram/commands"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Clear the command menu for a connected chat.
    pub async fn delete_telegram_bot_commands(&self, id: &str) -> Result<TelegramBotCommands> {
        let body: Envelope<TelegramBotCommands> = self
            .http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/accounts/{id}/telegram/commands"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Channels a Slack account can post to: every public channel, and private ones the
    /// app was invited to. A `409` with code `webhook_connection` means the account posts
    /// through a webhook.
    pub async fn slack_channels(&self, id: &str) -> Result<Vec<SlackChannel>> {
        let body: Envelope<Vec<SlackChannel>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/slack/channels"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// People in a Slack account's workspace, for addressing a DM.
    pub async fn slack_members(&self, id: &str) -> Result<Vec<SlackMember>> {
        let body: Envelope<Vec<SlackMember>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/slack/members"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The name and icon a Slack account posts under.
    pub async fn slack_identity(&self, id: &str) -> Result<SlackIdentity> {
        let body: Envelope<SlackIdentity> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/slack/identity"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Set the name and icon a Slack account posts under.
    pub async fn update_slack_identity(
        &self,
        id: &str,
        identity: &UpdateSlackIdentity,
    ) -> Result<SlackIdentity> {
        let body: Envelope<SlackIdentity> = self
            .http
            .send(
                Method::PATCH,
                &format!("/accounts/{id}/slack/identity"),
                None,
                Some(identity),
            )
            .await?;
        Ok(body.data)
    }

    // ─── Meta messaging settings (Facebook Pages, Instagram) ─────

    /// The prompts shown before the first message. A network without them answers `400`.
    pub async fn ice_breakers(&self, id: &str) -> Result<MetaIceBreakers> {
        let body: Envelope<MetaIceBreakers> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/messaging/ice-breakers"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Replace the ice breakers, up to four.
    pub async fn set_ice_breakers(
        &self,
        id: &str,
        ice_breakers: &[MetaIceBreaker],
    ) -> Result<MetaIceBreakers> {
        let payload = serde_json::json!({ "ice_breakers": ice_breakers });
        let body: Envelope<MetaIceBreakers> = self
            .http
            .send(
                Method::PUT,
                &format!("/accounts/{id}/messaging/ice-breakers"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Clear the ice breakers.
    pub async fn delete_ice_breakers(&self, id: &str) -> Result<MetaIceBreakers> {
        let body: Envelope<MetaIceBreakers> = self
            .http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/accounts/{id}/messaging/ice-breakers"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The always-visible Messenger menu. Facebook Pages only.
    pub async fn persistent_menu(&self, id: &str) -> Result<MetaPersistentMenu> {
        let body: Envelope<MetaPersistentMenu> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/messaging/persistent-menu"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Replace the menu, one entry per locale, up to three items each.
    pub async fn set_persistent_menu(
        &self,
        id: &str,
        menu: &[MetaPersistentMenuEntry],
    ) -> Result<MetaPersistentMenu> {
        let payload = serde_json::json!({ "persistent_menu": menu });
        let body: Envelope<MetaPersistentMenu> = self
            .http
            .send(
                Method::PUT,
                &format!("/accounts/{id}/messaging/persistent-menu"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Clear the menu.
    pub async fn delete_persistent_menu(&self, id: &str) -> Result<MetaPersistentMenu> {
        let body: Envelope<MetaPersistentMenu> = self
            .http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/accounts/{id}/messaging/persistent-menu"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The text shown before a Messenger conversation starts. Facebook Pages only.
    pub async fn greeting(&self, id: &str) -> Result<MetaGreeting> {
        let body: Envelope<MetaGreeting> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/messaging/greeting"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Replace the greeting, one entry per locale, each up to 160 characters.
    pub async fn set_greeting(
        &self,
        id: &str,
        greeting: &[MetaGreetingText],
    ) -> Result<MetaGreeting> {
        let payload = serde_json::json!({ "greeting": greeting });
        let body: Envelope<MetaGreeting> = self
            .http
            .send(
                Method::PUT,
                &format!("/accounts/{id}/messaging/greeting"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Clear the greeting.
    pub async fn delete_greeting(&self, id: &str) -> Result<MetaGreeting> {
        let body: Envelope<MetaGreeting> = self
            .http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/accounts/{id}/messaging/greeting"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// What the network is delivering to the FoPost webhook for this account.
    pub async fn webhook_subscription(&self, id: &str) -> Result<WebhookSubscription> {
        let body: Envelope<WebhookSubscription> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/webhook-subscription"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Subscribe to every field this account needs, lapsed or not.
    pub async fn resubscribe_webhook(&self, id: &str) -> Result<WebhookSubscription> {
        let body: Envelope<WebhookSubscription> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/accounts/{id}/webhook-subscription"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    // ── Discord (bot connections) ───────────────────────────────────────────
    //
    // A webhook connection answers `409` with code `webhook_connection` on each
    // of these; there is no bot on it to act as.

    /// Text channels the bot can post to in the connected server.
    pub async fn discord_channels(&self, id: &str) -> Result<Vec<DiscordChannel>> {
        self.discord_get(id, "channels".to_string()).await
    }

    /// Move the account to another channel in the same server.
    pub async fn switch_discord_channel(
        &self,
        id: &str,
        channel_id: &str,
    ) -> Result<DiscordChannel> {
        let body: Envelope<DiscordChannel> = self
            .http
            .send(
                Method::PATCH,
                &format!("/accounts/{id}/discord/channels/current"),
                None,
                Some(&serde_json::json!({ "channel_id": channel_id })),
            )
            .await?;
        Ok(body.data)
    }

    /// The nickname and avatar the bot wears in the server.
    pub async fn discord_identity(&self, id: &str) -> Result<DiscordIdentity> {
        self.discord_get(id, "identity".to_string()).await
    }

    /// Set the nickname and avatar the bot wears in the server.
    pub async fn update_discord_identity(
        &self,
        id: &str,
        identity: &UpdateDiscordIdentity,
    ) -> Result<DiscordIdentity> {
        let body: Envelope<DiscordIdentity> = self
            .http
            .send(
                Method::PATCH,
                &format!("/accounts/{id}/discord/identity"),
                None,
                Some(identity),
            )
            .await?;
        Ok(body.data)
    }

    /// Pinned messages in the account's channel.
    pub async fn discord_pins(&self, id: &str) -> Result<Vec<DiscordMessage>> {
        self.discord_get(id, "messages/pinned".to_string()).await
    }

    /// Remove a message from the account's channel.
    pub async fn delete_discord_message(&self, id: &str, message_id: &str) -> Result<DiscordAck> {
        self.discord_send(Method::DELETE, id, format!("messages/{message_id}"))
            .await
    }

    /// Pin a message in the account's channel.
    pub async fn pin_discord_message(&self, id: &str, message_id: &str) -> Result<DiscordAck> {
        self.discord_send(Method::POST, id, format!("messages/{message_id}/pin"))
            .await
    }

    /// Unpin a message in the account's channel.
    pub async fn unpin_discord_message(&self, id: &str, message_id: &str) -> Result<DiscordAck> {
        self.discord_send(Method::DELETE, id, format!("messages/{message_id}/pin"))
            .await
    }

    /// Publish an announcement-channel message to every server following the channel.
    pub async fn crosspost_discord_message(
        &self,
        id: &str,
        message_id: &str,
    ) -> Result<DiscordMessageRef> {
        self.discord_send(Method::POST, id, format!("messages/{message_id}/crosspost"))
            .await
    }

    /// Start a thread on a message.
    pub async fn create_discord_thread(
        &self,
        id: &str,
        message_id: &str,
        input: &DiscordThreadInput,
    ) -> Result<DiscordThread> {
        let body: Envelope<DiscordThread> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/discord/messages/{message_id}/thread"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Send one message to a member of the server.
    pub async fn send_discord_dm(
        &self,
        id: &str,
        member_id: &str,
        content: &str,
    ) -> Result<DiscordMessageRef> {
        let body: Envelope<DiscordMessageRef> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/discord/dm"),
                None,
                Some(&serde_json::json!({ "member_id": member_id, "content": content })),
            )
            .await?;
        Ok(body.data)
    }

    /// The server's scheduled events.
    pub async fn discord_events(&self, id: &str) -> Result<Vec<DiscordScheduledEvent>> {
        self.discord_get(id, "events".to_string()).await
    }

    /// One scheduled event.
    pub async fn discord_event(&self, id: &str, event_id: &str) -> Result<DiscordScheduledEvent> {
        self.discord_get(id, format!("events/{event_id}")).await
    }

    /// Add an event to the server's calendar.
    pub async fn create_discord_event(
        &self,
        id: &str,
        input: &DiscordEventInput,
    ) -> Result<DiscordScheduledEvent> {
        let body: Envelope<DiscordScheduledEvent> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/discord/events"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Change a scheduled event.
    pub async fn update_discord_event(
        &self,
        id: &str,
        event_id: &str,
        input: &DiscordEventInput,
    ) -> Result<DiscordScheduledEvent> {
        let body: Envelope<DiscordScheduledEvent> = self
            .http
            .send(
                Method::PATCH,
                &format!("/accounts/{id}/discord/events/{event_id}"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove a scheduled event.
    pub async fn delete_discord_event(&self, id: &str, event_id: &str) -> Result<DiscordAck> {
        self.discord_send(Method::DELETE, id, format!("events/{event_id}"))
            .await
    }

    /// The server's roster, or the members whose name starts with `query`.
    pub async fn discord_members(
        &self,
        id: &str,
        query: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<DiscordMember>> {
        let mut params: Query = Vec::new();
        push_opt(&mut params, "q", query);
        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        let body: Envelope<Vec<DiscordMember>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/discord/members"),
                Some(params),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// One member of the server.
    pub async fn discord_member(&self, id: &str, member_id: &str) -> Result<DiscordMember> {
        self.discord_get(id, format!("members/{member_id}")).await
    }

    /// The server's roles, highest first.
    pub async fn discord_roles(&self, id: &str) -> Result<Vec<DiscordRole>> {
        self.discord_get(id, "roles".to_string()).await
    }

    /// Add a role to the server.
    pub async fn create_discord_role(
        &self,
        id: &str,
        input: &DiscordRoleInput,
    ) -> Result<DiscordRole> {
        let body: Envelope<DiscordRole> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/discord/roles"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Change a role on the server.
    pub async fn update_discord_role(
        &self,
        id: &str,
        role_id: &str,
        input: &DiscordRoleInput,
    ) -> Result<DiscordRole> {
        let body: Envelope<DiscordRole> = self
            .http
            .send(
                Method::PATCH,
                &format!("/accounts/{id}/discord/roles/{role_id}"),
                None,
                Some(input),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove a role from the server.
    pub async fn delete_discord_role(&self, id: &str, role_id: &str) -> Result<DiscordAck> {
        self.discord_send(Method::DELETE, id, format!("roles/{role_id}"))
            .await
    }

    /// Give a member a role.
    pub async fn add_discord_member_role(
        &self,
        id: &str,
        role_id: &str,
        member_id: &str,
    ) -> Result<DiscordAck> {
        self.discord_send(
            Method::PUT,
            id,
            format!("roles/{role_id}/members/{member_id}"),
        )
        .await
    }

    /// Take a role from a member.
    pub async fn remove_discord_member_role(
        &self,
        id: &str,
        role_id: &str,
        member_id: &str,
    ) -> Result<DiscordAck> {
        self.discord_send(
            Method::DELETE,
            id,
            format!("roles/{role_id}/members/{member_id}"),
        )
        .await
    }

    async fn discord_get<T: serde::de::DeserializeOwned>(
        &self,
        id: &str,
        suffix: String,
    ) -> Result<T> {
        let body: Envelope<T> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/discord/{suffix}"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    async fn discord_send<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        id: &str,
        suffix: String,
    ) -> Result<T> {
        let body: Envelope<T> = self
            .http
            .send::<_, ()>(
                method,
                &format!("/accounts/{id}/discord/{suffix}"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Disconnect an account. Scheduled posts targeting it will fail.
    pub async fn delete(&self, id: &str) -> Result<Message> {
        self.http
            .send::<Message, ()>(Method::DELETE, &format!("/accounts/{id}"), None, None)
            .await
    }

    /// Make this the primary account for its platform, or clear the flag.
    pub async fn toggle_primary(&self, id: &str) -> Result<PrimaryToggled> {
        let body: Envelope<PrimaryToggled> = self
            .http
            .send::<_, ()>(Method::POST, &format!("/accounts/{id}/primary"), None, None)
            .await?;
        Ok(body.data)
    }

    /// Ask the platform whether the stored credentials still work.
    pub async fn validate(&self, id: &str) -> Result<CredentialCheck> {
        let body: Envelope<CredentialCheck> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/accounts/{id}/validate"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The stored health verdict. Set `refresh` to re-check with the platform first.
    pub async fn health(&self, id: &str, refresh: bool) -> Result<AccountHealth> {
        let mut query: Query = Vec::new();
        if refresh {
            query.push(("refresh", "true".into()));
        }
        let body: Envelope<AccountHealth> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/health"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Health for every account the key can see, with the counts.
    pub async fn health_summary(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<AccountsHealthSummary> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "workspaceId", workspace_id);
        let body: Envelope<AccountsHealthSummary> = self
            .http
            .send::<_, ()>(Method::GET, "/accounts/health", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Force an OAuth token refresh instead of waiting for the scheduled one.
    pub async fn refresh_token(&self, id: &str) -> Result<TokenRefreshed> {
        let body: Envelope<TokenRefreshed> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/accounts/{id}/refresh-token"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The account's audience over time, newest first.
    pub async fn analytics(&self, id: &str, limit: Option<u32>) -> Result<AccountAnalyticsHistory> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "limit", limit);
        let body: Envelope<AccountAnalyticsHistory> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/analytics"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    // ─── X communities ─────────────────────────────────────────────

    /// The X communities this account can post into.
    pub async fn communities(&self, account_id: &str) -> Result<Vec<Community>> {
        let body: Envelope<Vec<Community>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{account_id}/communities"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Re-read the account's communities from X.
    pub async fn sync_communities(&self, account_id: &str) -> Result<Vec<Community>> {
        let body: Envelope<Vec<Community>> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/accounts/{account_id}/communities/sync"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Search X's communities by name.
    pub async fn search_communities(
        &self,
        account_id: &str,
        query: &str,
    ) -> Result<Vec<CommunitySearchResult>> {
        let params: Query = vec![("q", query.to_string())];
        let body: Envelope<Vec<CommunitySearchResult>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{account_id}/communities/search"),
                Some(params),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Add a community by the id X uses, for one search cannot reach.
    pub async fn add_community(
        &self,
        account_id: &str,
        community_id: &str,
        name: Option<&str>,
    ) -> Result<Community> {
        let payload = serde_json::json!({ "communityId": community_id, "name": name });
        let body: Envelope<Community> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{account_id}/communities/manual"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove a community. `community_id` is the row id, not X's own id.
    pub async fn remove_community(&self, account_id: &str, community_id: i64) -> Result<bool> {
        #[derive(serde::Deserialize)]
        struct Removed {
            success: bool,
        }
        let body: Removed = self
            .http
            .send::<_, ()>(
                Method::DELETE,
                &format!("/accounts/{account_id}/communities/{community_id}"),
                None,
                None,
            )
            .await?;
        Ok(body.success)
    }

    // ─── Per-network extras ──────────────────────────────────────

    /// Boards this Pinterest connection can pin to.
    pub async fn pinterest_boards(&self, id: &str) -> Result<Vec<PinterestBoard>> {
        let body: Envelope<Vec<PinterestBoard>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/pinterest/boards"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Create a board on the connected Pinterest account.
    pub async fn create_pinterest_board(
        &self,
        id: &str,
        board: &CreatePinterestBoard,
    ) -> Result<PinterestBoard> {
        let body: Envelope<PinterestBoard> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/pinterest/boards"),
                None,
                Some(board),
            )
            .await?;
        Ok(body.data)
    }

    /// The channel's own playlists, with the stored default marked.
    pub async fn youtube_playlists(&self, id: &str) -> Result<Vec<YouTubePlaylist>> {
        let body: Envelope<Vec<YouTubePlaylist>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/youtube/playlists"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Create a playlist on the connected channel.
    pub async fn create_youtube_playlist(
        &self,
        id: &str,
        playlist: &CreateYouTubePlaylist,
    ) -> Result<YouTubePlaylist> {
        let body: Envelope<YouTubePlaylist> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/youtube/playlists"),
                None,
                Some(playlist),
            )
            .await?;
        Ok(body.data)
    }

    /// The playlist a new video joins when the post picks none. `None` clears it.
    pub async fn set_default_youtube_playlist(
        &self,
        id: &str,
        playlist_id: Option<&str>,
    ) -> Result<Option<String>> {
        #[derive(serde::Deserialize)]
        struct Stored {
            playlist_id: Option<String>,
        }
        let payload = serde_json::json!({ "playlist_id": playlist_id });
        let body: Envelope<Stored> = self
            .http
            .send(
                Method::PUT,
                &format!("/accounts/{id}/youtube/playlists/default"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data.playlist_id)
    }

    /// Caption tracks on one of the channel's videos.
    pub async fn youtube_captions(
        &self,
        id: &str,
        video_id: &str,
    ) -> Result<Vec<YouTubeCaptionTrack>> {
        let body: Envelope<Vec<YouTubeCaptionTrack>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/youtube/videos/{video_id}/captions"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Upload a caption track to a video.
    pub async fn upload_youtube_captions(
        &self,
        id: &str,
        video_id: &str,
        captions: &UploadYouTubeCaptions,
    ) -> Result<YouTubeCaptionTrack> {
        let body: Envelope<YouTubeCaptionTrack> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/youtube/videos/{video_id}/captions"),
                None,
                Some(captions),
            )
            .await?;
        Ok(body.data)
    }

    /// One caption track read back as text.
    pub async fn youtube_transcript(
        &self,
        id: &str,
        caption_id: &str,
    ) -> Result<YouTubeTranscript> {
        let body: Envelope<YouTubeTranscript> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/youtube/captions/{caption_id}"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// What a post from this Bluesky connection is written in when it does not say.
    pub async fn bluesky_languages(&self, id: &str) -> Result<BlueskyLanguages> {
        let body: Envelope<BlueskyLanguages> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/bluesky/languages"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Store up to three BCP-47 tags. An empty slice clears the default.
    pub async fn set_bluesky_languages(
        &self,
        id: &str,
        languages: &[String],
    ) -> Result<BlueskyLanguages> {
        let payload = serde_json::json!({ "languages": languages });
        let body: Envelope<BlueskyLanguages> = self
            .http
            .send(
                Method::PUT,
                &format!("/accounts/{id}/bluesky/languages"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// The switches TikTok enforces at publish time, changed in the TikTok app.
    pub async fn tiktok_creator_info(&self, id: &str) -> Result<TikTokCreatorInfo> {
        let body: Envelope<TikTokCreatorInfo> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/tiktok/creator-info"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// TikTok's Commercial Music Library. Needs the Marketing API product on the
    /// TikTok app; without it the call fails with 403 rather than answering empty.
    pub async fn tiktok_music(
        &self,
        id: &str,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<TikTokMusic>> {
        let mut params: Query = vec![("q", query.to_string())];
        push_opt(&mut params, "limit", limit);
        let body: Envelope<Vec<TikTokMusic>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/tiktok/music"),
                Some(params),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Places a post can be tagged with. Same TikTok product as the music library.
    pub async fn tiktok_locations(
        &self,
        id: &str,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<TikTokPlace>> {
        let mut params: Query = vec![("q", query.to_string())];
        push_opt(&mut params, "limit", limit);
        let body: Envelope<Vec<TikTokPlace>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/tiktok/locations"),
                Some(params),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Resolve a share link to one of this account's own videos, for repurposing.
    pub async fn tiktok_video_lookup(&self, id: &str, url: &str) -> Result<TikTokVideoSource> {
        let payload = serde_json::json!({ "url": url });
        let body: Envelope<TikTokVideoSource> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{id}/tiktok/video-download"),
                None,
                Some(&payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Tracks a Reel can carry. With no query Instagram answers with what is trending.
    pub async fn instagram_audio(
        &self,
        id: &str,
        query: Option<&str>,
        audio_type: Option<&str>,
    ) -> Result<Vec<InstagramAudio>> {
        let mut params: Query = Vec::new();
        push_opt(&mut params, "q", query);
        push_opt(&mut params, "audio_type", audio_type);
        let body: Envelope<Vec<InstagramAudio>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/instagram/audio"),
                Some(params),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// How many posts are left before Instagram refuses the next one.
    pub async fn instagram_publishing_limit(&self, id: &str) -> Result<InstagramPublishingLimit> {
        let body: Envelope<InstagramPublishingLimit> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/instagram/publishing-limit"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Stories still inside their 24 hours, posted through FoPost or not.
    /// Asking for insights costs one extra call per story.
    pub async fn instagram_stories(&self, id: &str, insights: bool) -> Result<Vec<InstagramStory>> {
        let mut params: Query = Vec::new();
        if insights {
            params.push(("insights", "true".to_string()));
        }
        let body: Envelope<Vec<InstagramStory>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/instagram/stories"),
                Some(params),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// The insight set for one story.
    pub async fn instagram_story_insights(
        &self,
        id: &str,
        story_id: &str,
    ) -> Result<InstagramStoryInsights> {
        let body: Envelope<InstagramStoryInsights> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/instagram/stories/{story_id}/insights"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Organizations a LinkedIn post can mention. People are not searchable:
    /// LinkedIn has no public person search.
    pub async fn linkedin_mentions(&self, id: &str, query: &str) -> Result<Vec<LinkedInMention>> {
        let params: Query = vec![("q", query.to_string())];
        let body: Envelope<Vec<LinkedInMention>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{id}/linkedin/mentions"),
                Some(params),
                None,
            )
            .await?;
        Ok(body.data)
    }
}

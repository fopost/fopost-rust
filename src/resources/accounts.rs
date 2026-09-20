//! `client.accounts()` — the social accounts a workspace publishes through.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    Account, AccountAnalyticsHistory, AccountCreated, AccountDetail, AccountHealth, AccountMoved,
    AccountRenamed, AccountsHealthSummary, Community, CommunitySearchResult, CreateAccount,
    CredentialCheck, ListAccounts, Message, MetaGreeting, MetaGreetingText, MetaIceBreaker,
    MetaIceBreakers, MetaPersistentMenu, MetaPersistentMenuEntry, PrimaryToggled, SlackChannel,
    SlackIdentity, SlackMember, TelegramBotCommand, TelegramBotCommands, TelegramConnectCode,
    TelegramConnectStatus, TokenRefreshed, UpdateSlackIdentity, WebhookSubscription,
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
}

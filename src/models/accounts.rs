//! Connected social accounts, their health, and X communities.

use serde::{Deserialize, Serialize};

use super::common::{HealthStatus, Platform};

/// A connected account as the list endpoint returns it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub platform: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// The name on the platform; `name` is the display override when one is set.
    #[serde(default)]
    pub platform_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub is_primary: Option<bool>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub health_status: Option<HealthStatus>,
    #[serde(default)]
    pub last_health_check: Option<String>,
    /// True when the account was connected before a permission it now needs was
    /// asked for. Reconnecting it is the fix.
    #[serde(default)]
    pub reconnect_required: bool,
}

/// The workspace an account belongs to, as `GET /accounts/{id}` reports it.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountWorkspaceRef {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(rename = "type", default)]
    pub workspace_type: Option<String>,
}

/// One account in full, with the workspace it belongs to.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountDetail {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub platform: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    /// The name on the platform; `name` is the display override when one is set.
    #[serde(default)]
    pub platform_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub workspace: Option<AccountWorkspaceRef>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Filters for `GET /accounts`.
#[derive(Debug, Clone, Default)]
pub struct ListAccounts {
    pub workspace_id: Option<String>,
    /// Only the accounts in this account group.
    pub group_id: Option<String>,
}

impl ListAccounts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn group(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = Some(group_id.into());
        self
    }
}

/// An account's names after a rename.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountRenamed {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub platform_name: Option<String>,
}

/// Where an account lives after a move.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountMoved {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// A one-time code that connects a Telegram chat. Send `command` to the bot there.
#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConnectCode {
    pub code: String,
    /// What to send in the chat: `/connect <code>`.
    pub command: String,
    #[serde(default)]
    pub bot_username: Option<String>,
    #[serde(default)]
    pub deep_link: Option<String>,
    #[serde(default)]
    pub group_link: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Where a connect code stands.
#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConnectStatus {
    /// `pending`, `connected`, `failed` or `expired`.
    pub status: String,
    /// The connected account, once `connected`.
    #[serde(default)]
    pub account_id: Option<String>,
    /// `card_required`, `slot_taken` or `workspace_unavailable`, once `failed`.
    #[serde(default)]
    pub reason: Option<String>,
}

/// One entry in the bot's command menu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelegramBotCommand {
    /// 1-32 lowercase letters, digits or underscores, without the slash.
    pub command: String,
    pub description: String,
}

impl TelegramBotCommand {
    pub fn new(command: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            description: description.into(),
        }
    }
}

/// The command menu the bot shows in a connected chat.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TelegramBotCommands {
    #[serde(default)]
    pub commands: Vec<TelegramBotCommand>,
}

/// A channel a Slack account can post to.
#[derive(Debug, Clone, Deserialize)]
pub struct SlackChannel {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_private: bool,
    /// Whether the bot is in the channel.
    #[serde(default)]
    pub is_member: bool,
    /// The channel this account posts to.
    #[serde(default)]
    pub is_current: bool,
}

/// A person in the connected Slack workspace. Pass `id` as the handle to start a DM.
#[derive(Debug, Clone, Deserialize)]
pub struct SlackMember {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub real_name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub is_bot: bool,
}

/// The name and icon a Slack account posts under. `None` falls back to the app's own.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SlackIdentity {
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    /// An emoji code such as `:rocket:`.
    #[serde(default)]
    pub icon_emoji: Option<String>,
}

/// The body of `PATCH /accounts/{id}/slack/identity`. For each field `None` keeps the
/// value and `Some(None)` clears it. Set `icon_url` or `icon_emoji`, not both; setting
/// one clears the other.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateSlackIdentity {
    /// 1-80 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<Option<String>>,
    /// An http(s) image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<Option<String>>,
    /// An emoji code such as `:rocket:`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_emoji: Option<Option<String>>,
}

/// A newly connected account.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountCreated {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub platform: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Whether one account's credentials still work.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountHealth {
    pub id: String,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub health_status: Option<HealthStatus>,
    #[serde(default)]
    pub last_health_check: Option<String>,
}

/// One row of the account health sweep.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountHealthRow {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub health_status: Option<HealthStatus>,
    #[serde(default)]
    pub last_health_check: Option<String>,
}

/// How many accounts sit in each health state.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountHealthCounts {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub healthy: u32,
    #[serde(default)]
    pub degraded: u32,
    #[serde(default)]
    pub expired: u32,
    #[serde(default)]
    pub revoked: u32,
    #[serde(default)]
    pub unknown: u32,
}

/// The whole account health picture, per account and in total.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountsHealthSummary {
    #[serde(default)]
    pub accounts: Vec<AccountHealthRow>,
    pub summary: AccountHealthCounts,
}

/// The result of toggling the primary flag.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryToggled {
    pub id: String,
    #[serde(default)]
    pub is_primary: bool,
}

/// The verdict of a credential check.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialCheck {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub valid: bool,
    #[serde(default)]
    pub health_status: Option<HealthStatus>,
}

/// The result of refreshing an OAuth token.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenRefreshed {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// One snapshot of an account's audience, oldest first.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountHistoryPoint {
    #[serde(default)]
    pub followers: Option<i64>,
    #[serde(default)]
    pub following: Option<i64>,
    #[serde(default)]
    pub total_posts: Option<i64>,
    #[serde(default)]
    pub reach: Option<i64>,
    #[serde(default)]
    pub profile_views: Option<i64>,
    #[serde(default)]
    pub fetched_at: Option<String>,
}

/// An account's audience over time.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAnalyticsHistory {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub history: Vec<AccountHistoryPoint>,
}

/// An X community an account can post into.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Community {
    /// The row id, which is what removing one takes.
    pub id: i64,
    #[serde(default)]
    pub account_id: Option<String>,
    /// The id X itself uses.
    #[serde(default)]
    pub community_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub member_count: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub last_synced_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A community search hit, straight from X.
#[derive(Debug, Clone, Deserialize)]
pub struct CommunitySearchResult {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub member_count: Option<i64>,
}

/// The body of `POST /accounts` — connecting an account with credentials you
/// already hold, instead of running the OAuth flow in the dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct CreateAccount {
    #[serde(rename = "workspaceId")]
    pub workspace_id: String,
    pub platform: Platform,
    pub username: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// Platform credentials. What belongs here differs per platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl CreateAccount {
    pub fn new(
        workspace_id: impl Into<String>,
        platform: Platform,
        username: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            platform,
            username: username.into(),
            name: name.into(),
            avatar: None,
            credentials: None,
        }
    }

    pub fn credentials(
        mut self,
        credentials: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn avatar(mut self, avatar: impl Into<String>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }
}

/// A subreddit a Reddit account is in, or its own profile page.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedditSubreddit {
    /// The name, without the `r/` prefix.
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subscribers: Option<u64>,
    #[serde(default)]
    pub over18: bool,
    /// False where the account may read but not submit.
    #[serde(default)]
    pub can_post: bool,
    /// Whether the subreddit offers post flairs at all.
    #[serde(default)]
    pub flair_enabled: bool,
    #[serde(default)]
    pub icon_url: Option<String>,
    /// Where posts go when a post names no subreddit.
    #[serde(default)]
    pub is_default: bool,
}

/// One rule a subreddit publishes. `applies_to` is `link`, `comment` or `all`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedditSubredditRule {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub applies_to: Option<String>,
}

/// A subreddit's rules, in its own order.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedditSubredditRules {
    #[serde(default)]
    pub subreddit: String,
    #[serde(default)]
    pub rules: Vec<RedditSubredditRule>,
}

/// A post flair, valid only in the subreddit it came from.
#[derive(Debug, Clone, Deserialize)]
pub struct RedditFlair {
    pub id: String,
    #[serde(default)]
    pub text: String,
    /// Whether the label may be replaced with your own text.
    #[serde(default)]
    pub editable: bool,
}

/// The post flairs one subreddit offers.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedditFlairs {
    #[serde(default)]
    pub subreddit: String,
    #[serde(default)]
    pub flairs: Vec<RedditFlair>,
}

/// Where posts go when a post names none. `None` means the account's own profile page.
#[derive(Debug, Clone, Deserialize)]
pub struct RedditDefaultSubreddit {
    #[serde(default)]
    pub subreddit: Option<String>,
}

/// The body of `PUT /accounts/{id}/reddit/default-subreddit`.
#[derive(Debug, Clone, Serialize)]
pub struct SetRedditDefaultSubreddit {
    /// `None` falls back to the account's own profile page.
    pub subreddit: Option<String>,
}

impl SetRedditDefaultSubreddit {
    pub fn new(subreddit: Option<String>) -> Self {
        Self { subreddit }
    }
}

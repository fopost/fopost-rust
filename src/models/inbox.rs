//! Inbox — comments, mentions, reviews and direct messages on connected accounts.

use serde::{Deserialize, Serialize};

use super::common::{string_enum, Platform};

string_enum! {
    /// What an inbox item is.
    pub enum InboxItemType {
        Comment => "comment",
        Mention => "mention",
        Dm => "dm",
        /// A rating left on the business itself: a Google Business review or a
        /// Facebook Page recommendation.
        Review => "review",
    }
}

string_enum! {
    /// Where an inbox item sits in triage.
    pub enum InboxItemState {
        Unread => "unread",
        Read => "read",
        Resolved => "resolved",
        Snoozed => "snoozed",
    }
}

string_enum! {
    /// Whether an item came in or went out as the connected account.
    pub enum InboxDirection {
        Inbound => "inbound",
        Outbound => "outbound",
    }
}

string_enum! {
    /// Order of an inbox listing.
    pub enum InboxSort {
        Newest => "newest",
        Oldest => "oldest",
        Unanswered => "unanswered",
    }
}

string_enum! {
    /// Which threads `GET /inbox/posts` lists.
    pub enum InboxThreadKind {
        /// Threads under posts on our accounts. The default.
        Comments => "comments",
        /// Posts our accounts were tagged in.
        Mentions => "mentions",
        /// Reviews left on the business, one row each.
        Reviews => "reviews",
    }
}

string_enum! {
    /// Whether a platform's comments or DMs can be read.
    pub enum InboxSupport {
        Live => "live",
        Soon => "soon",
        Unsupported => "none",
    }
}

/// The connected account an inbox row belongs to.
#[derive(Debug, Clone, Deserialize)]
pub struct InboxAccountRef {
    pub id: String,
    pub platform: Platform,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
}

/// A file or link on an inbox item.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxAttachment {
    /// `image`, `video`, `audio`, `file`, `link` or `share`.
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    /// The target of a `link` attachment.
    #[serde(default)]
    pub link: Option<String>,
    /// Served by the API, never a platform URL.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub preview_url: Option<String>,
}

/// The FoPost post a platform post was published from.
#[derive(Debug, Clone, Deserialize)]
pub struct PublishedPostRef {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
}

/// The platform post a comment thread hangs off.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxPostContext {
    #[serde(default)]
    pub external_id: Option<String>,
    /// Published from one of our accounts.
    #[serde(default)]
    pub is_own: bool,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub author_name: Option<String>,
    #[serde(default)]
    pub author_handle: Option<String>,
    #[serde(default)]
    pub author_avatar_url: Option<String>,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub permalink: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    /// Set when the post went out through FoPost.
    #[serde(default)]
    pub published: Option<PublishedPostRef>,
}

/// One comment, mention, review or DM.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxItem {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub platform: Platform,
    #[serde(rename = "type")]
    pub item_type: InboxItemType,
    pub state: InboxItemState,
    #[serde(default)]
    pub direction: Option<InboxDirection>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub author_name: Option<String>,
    #[serde(default)]
    pub author_handle: Option<String>,
    #[serde(default)]
    pub author_avatar_url: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    /// Stars on a review, 1-5. `None` on every other type.
    #[serde(default)]
    pub rating: Option<u8>,
    #[serde(default)]
    pub attachments: Vec<InboxAttachment>,
    #[serde(default)]
    pub permalink: Option<String>,
    #[serde(default)]
    pub post_external_id: Option<String>,
    #[serde(default)]
    pub parent_external_id: Option<String>,
    #[serde(default)]
    pub platform_created_at: Option<String>,
    #[serde(default)]
    pub snoozed_until: Option<String>,
    #[serde(default)]
    pub replied_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub can_reply: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub liked: bool,
    #[serde(default)]
    pub pinned: bool,
    /// Our reaction on a DM.
    #[serde(default)]
    pub reaction: Option<String>,
    #[serde(default)]
    pub edited_at: Option<String>,
    #[serde(default)]
    pub can_hide: bool,
    /// A comment someone left, or our own reply.
    #[serde(default)]
    pub can_delete: bool,
    #[serde(default)]
    pub can_like: bool,
    /// Our own comment only.
    #[serde(default)]
    pub can_pin: bool,
    /// Our own comment only.
    #[serde(default)]
    pub can_edit: bool,
    #[serde(default)]
    pub can_react: bool,
    #[serde(default)]
    pub can_send_media: bool,
    #[serde(default)]
    pub can_quick_reply: bool,
    /// A DM can be opened with `start_conversation` and a `comment_id`.
    #[serde(default)]
    pub can_private_reply: bool,
    /// The FoPost post this sits under, when there is one.
    #[serde(default)]
    pub post: Option<PublishedPostRef>,
    #[serde(default)]
    pub post_context: Option<InboxPostContext>,
    #[serde(default)]
    pub account: Option<InboxAccountRef>,
}

/// One post with its comments, as `GET /inbox/posts` lists them.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxThread {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub account_id: String,
    #[serde(default)]
    pub post_external_id: Option<String>,
    #[serde(default)]
    pub comment_count: u64,
    #[serde(default)]
    pub unread_count: u64,
    #[serde(default)]
    pub last_comment_at: Option<String>,
    #[serde(default)]
    pub last_comment_text: Option<String>,
    #[serde(default)]
    pub last_comment_author: Option<String>,
    /// Stars, on a review thread. `None` on comments and mentions.
    #[serde(default)]
    pub rating: Option<u8>,
    #[serde(default)]
    pub post: Option<InboxPostContext>,
    #[serde(default)]
    pub account: Option<InboxAccountRef>,
}

/// The other side of a DM thread.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxParticipant {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub handle: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

/// One DM thread, as `GET /inbox/conversations` lists them.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxConversation {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub account_id: String,
    pub conversation_id: String,
    #[serde(default)]
    pub message_count: u64,
    #[serde(default)]
    pub unread_count: u64,
    #[serde(default)]
    pub last_message_at: Option<String>,
    #[serde(default)]
    pub last_message_text: Option<String>,
    #[serde(default)]
    pub last_message_outbound: bool,
    #[serde(default)]
    pub participant: InboxParticipant,
    #[serde(default)]
    pub account: Option<InboxAccountRef>,
}

/// A connected account, flagged with what the inbox can read for it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxAccount {
    pub id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub platform: Platform,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub inbox_supported: bool,
    #[serde(default)]
    pub pending_reason: Option<String>,
    #[serde(default)]
    pub dm_supported: bool,
    #[serde(default)]
    pub dm_pending_reason: Option<String>,
    /// A new DM can be opened from this account by handle.
    #[serde(default)]
    pub can_start_conversation: bool,
}

/// What the inbox can read on a platform. Not tenant data.
#[derive(Debug, Clone, Deserialize)]
pub struct InboxPlatform {
    pub platform: Platform,
    pub comments: InboxSupport,
    pub dms: InboxSupport,
}

/// The item a drafted reply answers.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxApprovalItem {
    pub id: String,
    pub platform: Platform,
    #[serde(rename = "type", default)]
    pub item_type: Option<InboxItemType>,
    #[serde(default)]
    pub state: Option<InboxItemState>,
    #[serde(default)]
    pub author_name: Option<String>,
    #[serde(default)]
    pub author_handle: Option<String>,
    #[serde(default)]
    pub author_avatar_url: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub permalink: Option<String>,
    #[serde(default)]
    pub platform_created_at: Option<String>,
}

/// A reply an automation or the agent drafted that a person still has to send.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxApproval {
    /// Passed to `approve_reply` and `reject_reply`.
    pub id: u64,
    #[serde(default)]
    pub workspace_id: Option<String>,
    /// What drafted the reply.
    #[serde(default)]
    pub source: Option<String>,
    /// The drafted text.
    #[serde(default)]
    pub reply: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub item: Option<InboxApprovalItem>,
}

/// The answer to approving or rejecting a drafted reply.
#[derive(Debug, Clone, Deserialize)]
pub struct ApprovalDecision {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub outcome: Option<String>,
}

/// Pagination footer on the inbox list endpoints.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxPageMeta {
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u64,
}

/// One page of an inbox listing: its rows plus the pagination footer.
#[derive(Debug, Clone, Deserialize)]
pub struct InboxPage<T> {
    #[serde(rename = "data", default = "Vec::new")]
    pub items: Vec<T>,
    #[serde(default)]
    pub meta: InboxPageMeta,
}

impl<T> InboxPage<T> {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when another page exists after this one.
    pub fn has_next(&self) -> bool {
        u64::from(self.meta.page) * u64::from(self.meta.per_page) < self.meta.total
    }
}

impl<T> IntoIterator for InboxPage<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a InboxPage<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// Filters for `GET /inbox`. Everything is optional.
#[derive(Debug, Clone, Default)]
pub struct ListInbox {
    pub workspace_id: Option<String>,
    pub item_type: Option<InboxItemType>,
    pub state: Option<InboxItemState>,
    pub platform: Option<String>,
    pub account_id: Option<String>,
    /// Comments under one FoPost post.
    pub post_id: Option<String>,
    /// Comments under one platform post, including posts not published through FoPost.
    pub post_external_id: Option<String>,
    /// One DM thread.
    pub conversation_id: Option<String>,
    pub direction: Option<InboxDirection>,
    pub q: Option<String>,
    pub sort: Option<InboxSort>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListInbox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn item_type(mut self, item_type: InboxItemType) -> Self {
        self.item_type = Some(item_type);
        self
    }

    pub fn state(mut self, state: InboxItemState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn post(mut self, post_id: impl Into<String>) -> Self {
        self.post_id = Some(post_id.into());
        self
    }

    pub fn post_external_id(mut self, post_external_id: impl Into<String>) -> Self {
        self.post_external_id = Some(post_external_id.into());
        self
    }

    pub fn conversation(mut self, conversation_id: impl Into<String>) -> Self {
        self.conversation_id = Some(conversation_id.into());
        self
    }

    pub fn direction(mut self, direction: InboxDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    pub fn search(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }

    pub fn sort(mut self, sort: InboxSort) -> Self {
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

/// Filters for `GET /inbox/posts`. Everything is optional.
#[derive(Debug, Clone, Default)]
pub struct ListInboxThreads {
    pub workspace_id: Option<String>,
    /// Comment threads by default; `Mentions` for posts we were tagged in.
    pub kind: Option<InboxThreadKind>,
    pub platform: Option<String>,
    pub account_id: Option<String>,
    pub state: Option<InboxItemState>,
    pub q: Option<String>,
    pub sort: Option<InboxSort>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListInboxThreads {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn kind(mut self, kind: InboxThreadKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn state(mut self, state: InboxItemState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn search(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }

    pub fn sort(mut self, sort: InboxSort) -> Self {
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

/// Filters for `GET /inbox/conversations`. Everything is optional.
#[derive(Debug, Clone, Default)]
pub struct ListInboxConversations {
    pub workspace_id: Option<String>,
    pub platform: Option<String>,
    pub account_id: Option<String>,
    pub state: Option<InboxItemState>,
    pub q: Option<String>,
    pub sort: Option<InboxSort>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

impl ListInboxConversations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    pub fn account(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn state(mut self, state: InboxItemState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn search(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }

    pub fn sort(mut self, sort: InboxSort) -> Self {
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

/// The body of `POST /inbox/read`: one comment thread or one DM thread.
#[derive(Debug, Clone, Serialize)]
pub struct MarkThreadRead {
    pub workspace_id: String,
    pub account_id: String,
    /// The platform post whose thread to settle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_external_id: Option<String>,
    /// The DM thread to settle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

impl MarkThreadRead {
    /// Every comment under one platform post.
    pub fn post(
        workspace_id: impl Into<String>,
        account_id: impl Into<String>,
        post_external_id: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            account_id: account_id.into(),
            post_external_id: Some(post_external_id.into()),
            conversation_id: None,
        }
    }

    /// Every message in one DM thread.
    pub fn conversation(
        workspace_id: impl Into<String>,
        account_id: impl Into<String>,
        conversation_id: impl Into<String>,
    ) -> Self {
        Self {
            workspace_id: workspace_id.into(),
            account_id: account_id.into(),
            post_external_id: None,
            conversation_id: Some(conversation_id.into()),
        }
    }
}

/// The body of `PATCH /inbox/{id}`.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateInboxItem {
    pub state: InboxItemState,
    /// Required when `state` is `Snoozed`. ISO 8601, in the future.
    #[serde(rename = "snoozedUntil", skip_serializing_if = "Option::is_none")]
    pub snoozed_until: Option<String>,
}

impl UpdateInboxItem {
    pub fn new(state: InboxItemState) -> Self {
        Self {
            state,
            snoozed_until: None,
        }
    }

    /// Snooze the item until an ISO 8601 instant.
    pub fn snooze_until(until: impl Into<String>) -> Self {
        Self {
            state: InboxItemState::Snoozed,
            snoozed_until: Some(until.into()),
        }
    }
}

/// The body of `POST /inbox/{id}/reply`, for a reply carrying media or quick replies.
#[derive(Debug, Clone, Default, Serialize)]
pub struct InboxReply {
    /// Required unless `media_ids` is given.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Media library ids to attach to a DM, at most 10. Only where `can_send_media` is true.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub media_ids: Vec<String>,
    /// Answer buttons under a DM, at most 13 of up to 20 characters. Only where
    /// `can_quick_reply` is true.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quick_replies: Vec<String>,
}

impl InboxReply {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn media_ids<I, S>(mut self, media_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.media_ids = media_ids.into_iter().map(Into::into).collect();
        self
    }

    pub fn quick_replies<I, S>(mut self, quick_replies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.quick_replies = quick_replies.into_iter().map(Into::into).collect();
        self
    }
}

/// The body of `POST /inbox/conversations`: a DM by handle, or a private reply to a comment.
#[derive(Debug, Clone, Serialize)]
pub struct StartInboxConversation {
    /// The account to send from, with `handle`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    /// Who to message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    /// An inbox comment to answer privately instead. Only where `can_private_reply` is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_id: Option<String>,
    pub text: String,
    /// Media library ids to attach, at most 10.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub media_ids: Vec<String>,
}

impl StartInboxConversation {
    /// Message `handle` from `account_id`. Only where the account's
    /// `can_start_conversation` is true.
    pub fn handle(
        account_id: impl Into<String>,
        handle: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            account_id: Some(account_id.into()),
            handle: Some(handle.into()),
            comment_id: None,
            text: text.into(),
            media_ids: Vec::new(),
        }
    }

    /// Answer an inbox comment privately, by DM (Facebook, Instagram).
    pub fn private_reply(comment_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            account_id: None,
            handle: None,
            comment_id: Some(comment_id.into()),
            text: text.into(),
            media_ids: Vec::new(),
        }
    }

    pub fn media_ids<I, S>(mut self, media_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.media_ids = media_ids.into_iter().map(Into::into).collect();
        self
    }
}

/// The answer to `POST /inbox/conversations`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxConversationStarted {
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub item: Option<InboxItem>,
}

/// Where a sent reply landed on the platform.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxReplyRef {
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub external_url: Option<String>,
}

/// The answer to `POST /inbox/{id}/reply`.
#[derive(Debug, Clone, Deserialize)]
pub struct InboxReplyResult {
    pub item: InboxItem,
    #[serde(default)]
    pub reply: InboxReplyRef,
}

/// A DM account whose grant has to be renewed before its messages can be read.
#[derive(Debug, Clone, Deserialize)]
pub struct InboxDmReconnect {
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub account: Option<String>,
}

/// The answer to `POST /inbox/refresh`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxRefreshResult {
    #[serde(default)]
    pub accounts_polled: u64,
    #[serde(default)]
    pub new_items: u64,
    /// Accounts the platform rate-limited during this poll.
    #[serde(default)]
    pub rate_limited: u64,
    #[serde(default)]
    pub dm_reconnect: Vec<InboxDmReconnect>,
}

/// The outcome of a Messenger thread hand-over.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct InboxHandover {
    /// The app control went to, or `None` when it was taken back.
    #[serde(default)]
    pub app_id: Option<String>,
    /// `passed` or `taken`.
    #[serde(default)]
    pub control: String,
}

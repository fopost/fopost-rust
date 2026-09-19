//! One call per resource group, pinning the path, the query casing, and the
//! envelope each endpoint actually uses. The API is not consistent about
//! snake_case versus camelCase, so these are the tests that catch a drift.

mod common;

use common::{account_fixture, client};
use fopost::models::{
    AdBudget, AdGoal, AdInsightsQuery, AdKind, AdObjectLevel, AdObjectQuery, AdObjectRef, AdStatus,
    AdTargeting, AdTargetingItem, AnalyticsQuery, AudienceFilter, AudienceSpec, AuthorizeGoogleAds,
    BoostPost, BroadcastStatus, ContactChannel, ContactFieldType, ConversationAnalyticsQuery,
    ConversationSort, CreateAccountGroup, CreateAudience, CreateAutomation, CreateBroadcast,
    CreateContact, CreateContactField, CreateGoogleKeyword, CreatePinterestBoard, CreateSequence,
    CreateWebhook, DiscordEventInput, DiscordRoleInput, Enroll, GoogleAdScheduleInput,
    GoogleDayOfWeek, GoogleMatchType, GoogleQuery, GoogleScope, ImportContacts, InboxItemState,
    InboxItemType, InboxReply, InboxSort, InsightsBreakdown, InsightsQuery, LeadsFeedQuery,
    LeadsQuery, ListAccounts, ListBroadcasts, ListContacts, ListInbox, ListRecipients,
    MarkThreadRead, Platform, RecipientStatus, SequenceStep, SetAdStatuses, SetGoogleAdSchedule,
    SignalLevel, SkipReason, StartInboxConversation, TelegramBotCommand, TriggerType,
    UpdateContact, UpdateDiscordIdentity, UpdateInboxItem, UpdateSlackIdentity, ValidateLength,
    ValidateMedia, ValidateMediaItem, ValidatePost, WebhookEvent,
};
use wiremock::matchers::{body_bytes, body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn accounts_list_sends_the_camel_case_workspace_param() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts"))
        .and(query_param("workspaceId", "ws_1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [account_fixture()]})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accounts = client.accounts().list(Some("ws_1")).await.unwrap();

    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].workspace_id.as_deref(), Some("ws_1"));
    assert_eq!(accounts[0].is_primary, Some(true));
}

#[tokio::test]
async fn accounts_list_with_sends_the_group_filter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts"))
        .and(query_param("group_id", "grp_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "acc_1", "platform": "twitter", "name": "Client A", "platformName": "fopost"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accounts = client
        .accounts()
        .list_with(&ListAccounts::new().group("grp_1"))
        .await
        .unwrap();
    assert_eq!(accounts[0].platform_name.as_deref(), Some("fopost"));
}

#[tokio::test]
async fn renaming_an_account_to_none_sends_null() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/accounts/acc_1"))
        .and(body_json(serde_json::json!({"display_name": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "acc_1", "name": "fopost", "platform_name": "fopost"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let renamed = client.accounts().rename("acc_1", None).await.unwrap();
    assert_eq!(renamed.platform_name.as_deref(), Some("fopost"));
}

#[tokio::test]
async fn a_blocked_move_surfaces_the_conflict_and_its_tables() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/move"))
        .and(body_json(serde_json::json!({"workspace_id": "ws_2"})))
        .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "error": "move_blocked",
            "message": "blocked",
            "blocking_tables": ["posts"]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client
        .accounts()
        .move_to("acc_1", "ws_2")
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(409));
    assert_eq!(err.code(), Some("move_blocked"));
    match err {
        fopost::Error::Api(api) => {
            assert_eq!(api.body.unwrap()["blocking_tables"][0], "posts")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn a_telegram_connect_code_sends_the_workspace() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/telegram/connect-code"))
        .and(body_json(serde_json::json!({"workspaceId": "ws_1"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "code": "abc123",
                "command": "/connect abc123",
                "bot_username": "fopost_bot",
                "deep_link": null,
                "group_link": null,
                "expires_at": "2026-09-19T12:15:00Z"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let code = client
        .accounts()
        .create_telegram_connect_code(Some("ws_1"))
        .await
        .unwrap();
    assert_eq!(code.code, "abc123");
    assert_eq!(code.bot_username.as_deref(), Some("fopost_bot"));
    assert!(code.deep_link.is_none());
}

#[tokio::test]
async fn a_telegram_connect_status_sends_the_code() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/telegram/connect-code/status"))
        .and(query_param("code", "abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"status": "failed", "account_id": null, "reason": "card_required"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let status = client
        .accounts()
        .telegram_connect_status("abc123")
        .await
        .unwrap();
    assert_eq!(status.status, "failed");
    assert_eq!(status.reason.as_deref(), Some("card_required"));
}

#[tokio::test]
async fn telegram_bot_commands_are_read_replaced_and_cleared() {
    let server = MockServer::start().await;
    let menu = serde_json::json!({
        "data": {"commands": [{"command": "start", "description": "Start"}]}
    });
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/telegram/commands"))
        .respond_with(ResponseTemplate::new(200).set_body_json(menu.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/accounts/acc_1/telegram/commands"))
        .and(body_json(serde_json::json!({
            "commands": [{"command": "start", "description": "Start"}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(menu))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/accounts/acc_1/telegram/commands"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"commands": []}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accounts = client.accounts();
    let got = accounts.telegram_bot_commands("acc_1").await.unwrap();
    assert_eq!(got.commands[0].command, "start");
    let set = accounts
        .set_telegram_bot_commands("acc_1", &[TelegramBotCommand::new("start", "Start")])
        .await
        .unwrap();
    assert_eq!(set.commands[0].description, "Start");
    let cleared = accounts
        .delete_telegram_bot_commands("acc_1")
        .await
        .unwrap();
    assert!(cleared.commands.is_empty());
}

#[tokio::test]
async fn slack_channels_members_and_identity_unwrap_the_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/slack/channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "C1", "name": "general", "is_private": false, "is_member": true, "is_current": true}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/slack/members"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "U1", "name": "sam", "real_name": "Sam Doe", "display_name": null, "avatar": null, "is_bot": false}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/slack/identity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"username": "Launch Bot", "icon_url": null, "icon_emoji": ":rocket:"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accounts = client.accounts();
    let channels = accounts.slack_channels("acc_1").await.unwrap();
    assert!(channels[0].is_current);
    let members = accounts.slack_members("acc_1").await.unwrap();
    assert_eq!(members[0].real_name.as_deref(), Some("Sam Doe"));
    assert!(members[0].display_name.is_none());
    let identity = accounts.slack_identity("acc_1").await.unwrap();
    assert_eq!(identity.icon_emoji.as_deref(), Some(":rocket:"));
}

#[tokio::test]
async fn updating_the_slack_identity_omits_kept_fields_and_nulls_cleared_ones() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/accounts/acc_1/slack/identity"))
        .and(body_json(
            serde_json::json!({"username": null, "icon_emoji": ":rocket:"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"username": null, "icon_url": null, "icon_emoji": ":rocket:"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let update = UpdateSlackIdentity {
        username: Some(None),
        icon_emoji: Some(Some(":rocket:".into())),
        ..Default::default()
    };
    let identity = client
        .accounts()
        .update_slack_identity("acc_1", &update)
        .await
        .unwrap();
    assert!(identity.username.is_none());
}

#[tokio::test]
async fn a_slack_webhook_connection_is_a_conflict() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/slack/channels"))
        .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "error": "webhook_connection",
            "message": "Reconnect with the Slack app"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client.accounts().slack_channels("acc_1").await.unwrap_err();
    assert_eq!(err.status(), Some(409));
    assert_eq!(err.code(), Some("webhook_connection"));
}

#[tokio::test]
async fn creating_an_account_group_unwraps_the_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/account-groups"))
        .and(body_json(serde_json::json!({
            "workspace_id": "ws_1",
            "name": "Clients",
            "account_ids": ["acc_1"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": "grp_1", "name": "Clients", "account_ids": ["acc_1"]}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let group = client
        .account_groups()
        .create(&CreateAccountGroup::new("ws_1", "Clients").account_ids(["acc_1"]))
        .await
        .unwrap();
    assert_eq!(group.account_ids, vec!["acc_1".to_string()]);
}

#[tokio::test]
async fn setting_no_members_sends_an_empty_list() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/account-groups/grp_1/members"))
        .and(body_json(serde_json::json!({"account_ids": []})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "grp_1", "name": "Clients", "account_ids": []}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let group = client
        .account_groups()
        .set_members("grp_1", Vec::<String>::new())
        .await
        .unwrap();
    assert!(group.account_ids.is_empty());
}

#[tokio::test]
async fn labels_list_sends_the_snake_case_workspace_param() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/labels"))
        .and(query_param("workspace_id", "ws_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "lbl_1", "name": "Launch", "color": "#4F46E5", "workspace": null}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let labels = client.labels().list(Some("ws_1")).await.unwrap();
    assert_eq!(labels[0].name, "Launch");
}

#[tokio::test]
async fn workspaces_list_unwraps_the_envelope_and_keeps_the_accounts() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "ws_1",
                "name": "Acme",
                "slug": "acme",
                "type": "BRAND",
                "timezone": "UTC",
                "language": "en",
                "created_at": "2026-01-01T00:00:00.000Z",
                "accounts": [{"id": "acc_1", "workspaceId": "ws_1", "platform": "twitter"}]
            }]
        })))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let workspaces = client.workspaces().list().await.unwrap();

    assert_eq!(workspaces[0].id, "ws_1");
    assert_eq!(workspaces[0].accounts.len(), 1);
}

#[tokio::test]
async fn a_created_webhook_hands_back_the_signing_secret() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/webhooks"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "url": "https://example.com/hooks/fopost",
            "events": ["post.published", "delivery.failed"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": "wh_1",
                "workspaceId": "ws_1",
                "url": "https://example.com/hooks/fopost",
                "secret": "whsec_abc",
                "events": ["post.published", "delivery.failed"],
                "active": true
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let created = client
        .webhooks()
        .create(&CreateWebhook::new(
            "ws_1",
            "https://example.com/hooks/fopost",
            [WebhookEvent::PostPublished, WebhookEvent::DeliveryFailed],
        ))
        .await
        .unwrap();

    assert_eq!(created.secret, "whsec_abc");
    assert_eq!(created.events.len(), 2);
}

#[tokio::test]
async fn an_automation_serializes_its_steps_in_camel_case() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/automations"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "name": "Blog to social",
            "triggerType": "rss_feed",
            "steps": [{"actionType": "publish"}]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": "auto_1",
                "workspaceId": "ws_1",
                "name": "Blog to social",
                "triggerType": "rss_feed",
                "active": true,
                "secret": "asec_abc",
                "steps": [{"id": 1, "position": 0, "actionType": "publish"}]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let created = client
        .automations()
        .create(&CreateAutomation::new(
            "ws_1",
            "Blog to social",
            TriggerType::RssFeed,
            [fopost::models::AutomationStep::new(
                fopost::models::ActionType::Publish,
            )],
        ))
        .await
        .unwrap();

    assert_eq!(created.secret.as_deref(), Some("asec_abc"));
    assert_eq!(created.steps.len(), 1);
}

#[tokio::test]
async fn the_analytics_overview_mixes_query_casing_the_way_the_api_does() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/analytics/overview"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("accountId", "acc_1"))
        .and(query_param("days", "30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "totalAccounts": 3,
                "totalFollowers": 1200,
                "engagementRate": 0.042,
                "platforms": [{"platform": "twitter", "accounts": 1, "followers": 900}],
                "accounts": []
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let overview = client
        .analytics()
        .overview(
            &AnalyticsQuery::new()
                .workspace("ws_1")
                .account("acc_1")
                .days(30),
        )
        .await
        .unwrap();

    assert_eq!(overview.total_followers, 1200);
    assert_eq!(overview.platforms[0].followers, 900);
}

#[tokio::test]
async fn deleting_media_reads_the_bare_success_flag() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/media/med_1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"success": true})),
        )
        .mount(&server)
        .await;

    let client = client(&server).await;
    assert!(client.media().delete("med_1").await.unwrap());
}

#[cfg(feature = "multipart")]
#[tokio::test]
async fn uploading_media_posts_a_multipart_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/media/upload"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": [{
                "id": "med_1",
                "type": "image",
                "name": "card.png",
                "url": "https://api.fopost.com/v1/media/med_1/file",
                "size": 4
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let uploaded = client
        .media()
        .upload(
            "ws_1",
            [fopost::models::MediaUpload::new(
                "card.png",
                "image/png",
                vec![1, 2, 3, 4],
            )],
        )
        .await
        .unwrap();

    assert_eq!(uploaded[0].name, "card.png");

    let sent = &server.received_requests().await.unwrap()[0];
    let content_type = sent.headers.get("content-type").unwrap().to_str().unwrap();
    assert!(
        content_type.starts_with("multipart/form-data"),
        "{content_type}"
    );
}

#[tokio::test]
async fn inbox_list_sends_snake_case_filters_and_reads_the_page_meta() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/inbox"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("type", "comment"))
        .and(query_param("state", "unread"))
        .and(query_param("post_external_id", "ext_9"))
        .and(query_param("sort", "unanswered"))
        .and(query_param("per_page", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "ib_1",
                "workspaceId": "ws_1",
                "platform": "instagram",
                "type": "comment",
                "state": "unread",
                "direction": "inbound",
                "authorHandle": "morgan.lee",
                "text": "Love this",
                "attachments": [{"kind": "image", "url": "https://api.fopost.com/v1/inbox/ib_1/attachments/0"}],
                "createdAt": "2026-09-01T10:00:00.000Z",
                "canReply": true,
                "hidden": false,
                "postContext": {"externalId": "ext_9", "isOwn": true, "published": {"id": "post_1", "title": null}},
                "account": {"id": "acc_1", "platform": "instagram", "username": "yourbrand", "name": "Your Brand", "avatar": null}
            }],
            "meta": {"page": 1, "perPage": 10, "total": 23}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .inbox()
        .list(
            &ListInbox::new()
                .workspace("ws_1")
                .item_type(InboxItemType::Comment)
                .state(InboxItemState::Unread)
                .post_external_id("ext_9")
                .sort(InboxSort::Unanswered)
                .per_page(10),
        )
        .await
        .unwrap();

    assert_eq!(page.len(), 1);
    assert_eq!(page.meta.total, 23);
    assert!(page.has_next());
    let item = &page.items[0];
    assert_eq!(item.item_type, InboxItemType::Comment);
    assert_eq!(item.attachments[0].kind, "image");
    assert_eq!(
        item.post_context
            .as_ref()
            .unwrap()
            .published
            .as_ref()
            .unwrap()
            .id,
        "post_1"
    );
    assert_eq!(item.account.as_ref().unwrap().platform, Platform::Instagram);
}

#[tokio::test]
async fn snoozing_an_inbox_item_patches_a_camel_case_body() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/inbox/ib_1"))
        .and(body_json(serde_json::json!({
            "state": "snoozed",
            "snoozedUntil": "2026-09-02T09:00:00Z"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "ib_1",
                "platform": "instagram",
                "type": "comment",
                "state": "snoozed",
                "snoozedUntil": "2026-09-02T09:00:00.000Z"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let item = client
        .inbox()
        .update(
            "ib_1",
            &UpdateInboxItem::snooze_until("2026-09-02T09:00:00Z"),
        )
        .await
        .unwrap();

    assert_eq!(item.state, InboxItemState::Snoozed);
    assert_eq!(
        item.snoozed_until.as_deref(),
        Some("2026-09-02T09:00:00.000Z")
    );
}

#[tokio::test]
async fn marking_a_thread_read_posts_snake_case_and_reads_the_count() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/inbox/read"))
        .and(body_json(serde_json::json!({
            "workspace_id": "ws_1",
            "account_id": "acc_1",
            "conversation_id": "conv_7"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"updated": 4}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let updated = client
        .inbox()
        .mark_thread_read(&MarkThreadRead::conversation("ws_1", "acc_1", "conv_7"))
        .await
        .unwrap();
    assert_eq!(updated, 4);
}

#[tokio::test]
async fn liking_an_inbox_item_posts_without_a_body_and_reads_the_flags() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/inbox/ib_1/like"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "ib_1",
                "platform": "instagram",
                "type": "comment",
                "state": "read",
                "liked": true,
                "pinned": false,
                "reaction": null,
                "editedAt": "2026-09-02T09:00:00.000Z",
                "canLike": true,
                "canPin": false,
                "canEdit": true,
                "canReact": false,
                "canSendMedia": false,
                "canQuickReply": false,
                "canPrivateReply": true
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let item = client.inbox().like("ib_1").await.unwrap();

    assert!(item.liked);
    assert!(!item.pinned);
    assert!(item.can_like && item.can_edit && item.can_private_reply);
    assert!(!item.can_send_media);
    assert_eq!(item.edited_at.as_deref(), Some("2026-09-02T09:00:00.000Z"));
}

#[tokio::test]
async fn reacting_with_none_sends_a_null_reaction() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/inbox/ib_1/react"))
        .and(body_json(serde_json::json!({"reaction": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "ib_1", "platform": "instagram", "type": "dm", "state": "read", "reaction": null}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let item = client.inbox().react("ib_1", None).await.unwrap();
    assert_eq!(item.reaction, None);
}

#[tokio::test]
async fn editing_a_comment_patches_only_the_text() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/inbox/ib_1"))
        .and(body_json(serde_json::json!({"text": "Fixed the typo"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "ib_1", "platform": "facebook", "type": "comment", "state": "read", "text": "Fixed the typo"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let item = client
        .inbox()
        .edit_comment("ib_1", "Fixed the typo")
        .await
        .unwrap();
    assert_eq!(item.text.as_deref(), Some("Fixed the typo"));
}

#[tokio::test]
async fn a_reply_with_media_sends_snake_case_ids_and_quick_replies() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/inbox/ib_1/reply"))
        .and(body_json(serde_json::json!({
            "media_ids": ["med_1"],
            "quick_replies": ["Yes", "No"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "item": {"id": "ib_1", "platform": "instagram", "type": "dm", "state": "resolved"},
                "reply": {"externalId": "m_1"}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let sent = client
        .inbox()
        .reply_with(
            "ib_1",
            &InboxReply::new()
                .media_ids(["med_1"])
                .quick_replies(["Yes", "No"]),
        )
        .await
        .unwrap();
    assert_eq!(sent.reply.external_id.as_deref(), Some("m_1"));
}

#[tokio::test]
async fn a_private_reply_starts_a_conversation_from_a_comment() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/inbox/conversations"))
        .and(body_json(serde_json::json!({
            "comment_id": "ib_1",
            "text": "Sent you the details"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "conversationId": "conv_7",
                "item": {"id": "ib_2", "platform": "instagram", "type": "dm", "state": "read", "direction": "outbound"}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let started = client
        .inbox()
        .start_conversation(&StartInboxConversation::private_reply(
            "ib_1",
            "Sent you the details",
        ))
        .await
        .unwrap();
    assert_eq!(started.conversation_id.as_deref(), Some("conv_7"));
    assert_eq!(started.item.unwrap().id, "ib_2");
}

#[tokio::test]
async fn typing_posts_the_account_and_reads_the_state() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/inbox/conversations/conv_7/typing"))
        .and(body_json(
            serde_json::json!({"account_id": "acc_1", "on": false}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"typing": false}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let typing = client
        .inbox()
        .set_typing("conv_7", "acc_1", false)
        .await
        .unwrap();
    assert!(!typing);
}

#[tokio::test]
async fn boosting_a_post_serializes_the_camel_case_body_and_parses_the_ad() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/boost"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "connectionId": "conn_1",
            "adAccountId": "act_123",
            "postId": "post_1",
            "accountId": "acc_1",
            "name": "Autumn drop boost",
            "goal": "engagement",
            "budget": {"minor": 2000, "type": "daily"},
            "targeting": {
                "countries": ["US", "CA"],
                "ageMin": 21,
                "ageMax": 45,
                "gender": "all",
                "interests": [{"id": "6003", "name": "Coffee"}]
            },
            "paused": false
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": "ad_1",
                "workspaceId": "ws_1",
                "kind": "boost",
                "name": "Autumn drop boost",
                "goal": "engagement",
                "status": "active",
                "effectiveStatus": "PENDING_REVIEW",
                "adAccountId": "act_123",
                "sourcePostId": "post_1",
                "budgetMinor": 2000,
                "budgetType": "daily",
                "currency": "USD",
                "targeting": {"countries": ["US", "CA"], "ageMin": 21, "ageMax": 45, "gender": "all"},
                "insights": {"impressions": 0, "reach": 0, "clicks": 0, "spendMinor": 0},
                "createdAt": "2026-09-01T10:00:00.000Z"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let ad = client
        .ads()
        .boost(
            &BoostPost::new(
                "ws_1",
                "conn_1",
                "act_123",
                "post_1",
                "acc_1",
                "Autumn drop boost",
                AdGoal::Engagement,
                AdBudget::daily(2000),
                AdTargeting::new(["US", "CA"], 21, 45)
                    .interests([AdTargetingItem::new("6003", "Coffee")]),
            )
            .paused(false),
        )
        .await
        .unwrap();

    assert_eq!(ad.kind, AdKind::Boost);
    assert_eq!(ad.status, Some(AdStatus::Active));
    assert_eq!(ad.budget_minor, 2000);
    assert_eq!(ad.targeting.countries, ["US", "CA"]);
    assert_eq!(ad.insights.as_ref().unwrap().spend_minor, 0);
}

#[tokio::test]
async fn pausing_an_ad_patches_with_the_workspace_in_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/ads/ad_1"))
        .and(query_param("workspace_id", "ws_1"))
        .and(body_json(serde_json::json!({"status": "paused"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "ad_1", "kind": "ad", "name": "Launch", "status": "paused"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let ad = client
        .ads()
        .set_status("ad_1", "ws_1", AdStatus::Paused)
        .await
        .unwrap();
    assert_eq!(ad.status, Some(AdStatus::Paused));
}

#[tokio::test]
async fn a_lookalike_audience_carries_its_subtype_tag() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/audiences"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "connectionId": "conn_1",
            "adAccountId": "act_123",
            "name": "Like our buyers",
            "spec": {"subtype": "LOOKALIKE", "originAudienceId": "aud_1", "country": "US"}
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": "aud_2", "added": 0}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let created = client
        .ads()
        .create_audience(&CreateAudience::new(
            "ws_1",
            "conn_1",
            "act_123",
            "Like our buyers",
            AudienceSpec::Lookalike {
                origin_audience_id: "aud_1".into(),
                country: "US".into(),
                ratio: None,
            },
        ))
        .await
        .unwrap();
    assert_eq!(created.id, "aud_2");
}

#[tokio::test]
async fn leads_send_the_cursor_and_read_the_next_one() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/lead-forms/form_1/leads"))
        .and(query_param("connection_id", "conn_1"))
        .and(query_param("page_id", "page_1"))
        .and(query_param("after", "cur_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "leads": [{
                    "id": "lead_1",
                    "fields": [{"name": "full_name", "values": ["Morgan Lee"]}],
                    "isOrganic": false
                }],
                "nextCursor": "cur_2"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .ads()
        .leads(
            "form_1",
            &LeadsQuery::new("conn_1", "page_1").after("cur_1"),
        )
        .await
        .unwrap();
    assert_eq!(page.leads[0].fields[0].values[0], "Morgan Lee");
    assert_eq!(page.next_cursor.as_deref(), Some("cur_2"));
}

#[tokio::test]
async fn the_account_tree_nests_ad_sets_and_ads_under_campaigns() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/accounts/act_123/tree"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("connection_id", "conn_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "adAccountId": "act_123",
                "currency": "USD",
                "campaigns": [{
                    "id": "c_1",
                    "name": "Launch",
                    "status": "PAUSED",
                    "budgetMinor": null,
                    "adSets": [{
                        "id": "s_1",
                        "name": "US",
                        "campaignId": "c_1",
                        "status": "ACTIVE",
                        "budgetMinor": 5000,
                        "budgetType": "daily",
                        "ads": [{"id": "a_1", "name": "Hero", "creativeId": "cr_1", "status": "ACTIVE"}]
                    }]
                }]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let tree = client
        .ads()
        .tree("act_123", &AdObjectQuery::new("conn_1").workspace("ws_1"))
        .await
        .unwrap();
    let campaign = &tree.campaigns[0];
    assert_eq!(campaign.campaign.id, "c_1");
    assert_eq!(campaign.campaign.budget_minor, None);
    assert_eq!(campaign.ad_sets[0].ad_set.budget_minor, Some(5000));
    assert_eq!(
        campaign.ad_sets[0].ads[0].creative_id.as_deref(),
        Some("cr_1")
    );
}

#[tokio::test]
async fn bulk_status_posts_each_object_and_reads_each_outcome() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/status"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "connectionId": "conn_1",
            "status": "paused",
            "objects": [{"id": "c_1", "level": "campaign"}, {"id": "s_1", "level": "ad_set"}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {"id": "c_1", "level": "campaign", "ok": true, "error": null},
                {"id": "s_1", "level": "ad_set", "ok": false, "error": "Not found"}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let results = client
        .ads()
        .set_statuses(&SetAdStatuses::new(
            "ws_1",
            "conn_1",
            AdStatus::Paused,
            [
                AdObjectRef::new("c_1", AdObjectLevel::Campaign),
                AdObjectRef::new("s_1", AdObjectLevel::AdSet),
            ],
        ))
        .await
        .unwrap();
    assert!(results[0].ok);
    assert_eq!(results[1].error.as_deref(), Some("Not found"));
}

#[tokio::test]
async fn insights_send_the_range_breakdown_and_daily_flag() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/insights"))
        .and(query_param("connection_id", "conn_1"))
        .and(query_param("object_id", "c_1"))
        .and(query_param("since", "2026-09-01"))
        .and(query_param("until", "2026-09-07"))
        .and(query_param("breakdown", "age"))
        .and(query_param("daily", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "objectId": "c_1",
                "since": "2026-09-01",
                "until": "2026-09-07",
                "breakdownBy": "age",
                "totals": {"impressions": 100, "clicks": 4, "ctr": 4.0, "spendMinor": 250},
                "breakdown": [{"key": "18-24", "metrics": {"impressions": 60}}],
                "timeline": [{"date": "2026-09-01", "metrics": {"impressions": 10}}]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/ad_1/insights"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("since", "2026-09-01"))
        .and(query_param("until", "2026-09-07"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"objectId": "ad_1", "since": "2026-09-01", "until": "2026-09-07", "totals": null}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let report = client
        .ads()
        .insights(
            &InsightsQuery::new("conn_1", "c_1", "2026-09-01", "2026-09-07")
                .breakdown(InsightsBreakdown::Age)
                .daily(true),
        )
        .await
        .unwrap();
    assert_eq!(report.breakdown_by, Some(InsightsBreakdown::Age));
    assert_eq!(report.totals.as_ref().unwrap().spend_minor, 250);
    assert_eq!(report.breakdown[0].metrics.impressions, 60);
    assert_eq!(report.timeline[0].date, "2026-09-01");

    let ad = client
        .ads()
        .ad_insights(
            "ad_1",
            &AdInsightsQuery::new("ws_1", "2026-09-01", "2026-09-07"),
        )
        .await
        .unwrap();
    assert!(ad.totals.is_none());
}

#[tokio::test]
async fn the_leads_feed_sends_the_cursor_and_reads_the_next_one() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/leads"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("page_id", "page_1"))
        .and(query_param("cursor", "cur_1"))
        .and(query_param("limit", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "leads": [{
                    "id": "lead_1",
                    "leadId": "m_1",
                    "pageId": "page_1",
                    "formId": null,
                    "isOrganic": true,
                    "fields": [{"name": "full_name", "values": ["Morgan Lee"]}],
                    "submittedAt": "2026-09-19T10:00:00.000Z"
                }],
                "nextCursor": "cur_2"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .ads()
        .leads_feed(
            &LeadsFeedQuery::new()
                .workspace("ws_1")
                .page("page_1")
                .cursor("cur_1")
                .limit(50),
        )
        .await
        .unwrap();
    assert_eq!(page.leads[0].lead_id, "m_1");
    assert!(page.leads[0].form_id.is_none());
    assert_eq!(page.next_cursor.as_deref(), Some("cur_2"));
}

#[tokio::test]
async fn upload_direct_presigns_puts_the_bytes_and_completes() {
    let server = MockServer::start().await;
    let upload_url = format!("{}/storage/up_1", server.uri());
    Mock::given(method("POST"))
        .and(path("/v1/media/presign"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "filename": "card.png",
            "mimeType": "image/png",
            "size": 4
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "uploadId": "up_1",
                "uploadUrl": upload_url,
                "method": "PUT",
                "headers": { "Content-Type": "image/png" },
                "expiresAt": "2026-09-19T12:00:00Z"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/up_1"))
        .and(header("content-type", "image/png"))
        .and(header("content-length", "4"))
        .and(body_bytes(vec![1, 2, 3, 4]))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/media/presign/up_1/complete"))
        .and(header("x-api-key", common::API_KEY))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": "med_1",
                "type": "image",
                "name": "card.png",
                "url": "https://api.fopost.com/v1/media/med_1/file",
                "previewUrl": "https://api.fopost.com/v1/media/med_1/file",
                "size": 4
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let uploaded = client
        .media()
        .upload_direct("ws_1", "card.png", "image/png", vec![1, 2, 3, 4])
        .await
        .unwrap();

    assert_eq!(uploaded.id.as_deref(), Some("med_1"));
    assert_eq!(uploaded.name, "card.png");

    let requests = server.received_requests().await.unwrap();
    let put = requests.iter().find(|r| r.method == "PUT").unwrap();
    assert!(
        put.headers.get("x-api-key").is_none(),
        "the upload carries no key"
    );
}

#[tokio::test]
async fn upload_direct_fails_on_a_rejected_put_without_completing() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/media/presign"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "uploadId": "up_1",
                "uploadUrl": format!("{}/storage/up_1", server.uri()),
                "method": "PUT",
                "headers": { "Content-Type": "image/png" },
                "expiresAt": "2026-09-19T12:00:00Z"
            }
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/storage/up_1"))
        .respond_with(ResponseTemplate::new(403).set_body_string("AccessDenied"))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/media/presign/up_1/complete"))
        .respond_with(ResponseTemplate::new(201))
        .expect(0)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client
        .media()
        .upload_direct("ws_1", "card.png", "image/png", vec![1, 2, 3, 4])
        .await
        .unwrap_err();

    match err {
        fopost::Error::Api(err) => {
            assert_eq!(err.status, 403);
            assert_eq!(err.message, "AccessDenied");
        }
        other => panic!("expected an api error, got {other:?}"),
    }
}

#[tokio::test]
async fn validating_a_post_sends_the_platforms_and_media_and_reads_each_platform() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/validate/post"))
        .and(body_json(serde_json::json!({
            "platforms": ["twitter", "bluesky"],
            "content": "Hello from Rust",
            "media": [{"url": "https://cdn.yourbrand.com/a.png", "mime_type": "image/png", "size": 1024}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "ready": false,
                "platforms": [
                    {"platform": "twitter", "ready": true, "issues": [], "score": 82,
                     "signals": [{"level": "info", "code": "has_media", "message": "Has media"}]},
                    {"platform": "bluesky", "ready": false, "issues": ["media_too_large"], "signals": []}
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client
        .validate()
        .post(
            &ValidatePost::new(["twitter", "bluesky"])
                .content("Hello from Rust")
                .media([
                    ValidateMediaItem::new("https://cdn.yourbrand.com/a.png", "image/png")
                        .size(1024),
                ]),
        )
        .await
        .unwrap();

    assert!(!result.ready);
    assert_eq!(result.platforms.len(), 2);
    assert_eq!(result.platforms[0].score, Some(82.0));
    assert_eq!(result.platforms[0].signals[0].level, SignalLevel::Info);
    assert_eq!(result.platforms[1].issues, ["media_too_large"]);
    assert_eq!(result.platforms[1].score, None);
}

#[tokio::test]
async fn validating_length_sends_the_text_and_reads_a_null_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/validate/length"))
        .and(body_json(serde_json::json!({
            "text": "Hello from Rust",
            "platforms": ["twitter", "telegram"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "ok": true,
                "platforms": [
                    {"platform": "twitter", "length": 15, "limit": 280, "unit": "chars", "ok": true, "signals": []},
                    {"platform": "telegram", "length": 15, "limit": null, "unit": "bytes", "ok": true, "signals": []}
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client
        .validate()
        .length(&ValidateLength::new(
            "Hello from Rust",
            ["twitter", "telegram"],
        ))
        .await
        .unwrap();

    assert!(result.ok);
    assert_eq!(result.platforms[0].limit, Some(280));
    assert_eq!(result.platforms[0].unit, "chars");
    assert_eq!(result.platforms[1].limit, None);
    assert_eq!(result.platforms[1].length, 15);
}

#[tokio::test]
async fn validating_media_sends_the_url_and_reads_the_type_field() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/validate/media"))
        .and(body_json(
            serde_json::json!({"url": "https://cdn.yourbrand.com/a.png"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "ok": true,
                "issues": [],
                "name": "a.png",
                "size": 1024,
                "mime_type": "image/png",
                "type": "image"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client
        .validate()
        .media(&ValidateMedia::new("https://cdn.yourbrand.com/a.png"))
        .await
        .unwrap();

    assert!(result.ok);
    assert_eq!(result.name, "a.png");
    assert_eq!(result.size, 1024);
    assert_eq!(result.mime_type.as_deref(), Some("image/png"));
    assert_eq!(result.media_type.as_deref(), Some("image"));
}

// ─── Contacts ──────────────────────────────────────────────────────

fn contact_fixture() -> serde_json::Value {
    serde_json::json!({
        "id": "con_1",
        "display_name": "Ada Okafor",
        "channels": [
            {"platform": "instagram", "handle": "adaokafor", "externalId": "178414"},
            {"platform": "x", "handle": "ada_writes", "externalId": null}
        ],
        "source": "inbox",
        "note": null,
        "first_seen_at": "2026-04-02T09:14:00.000Z",
        "last_seen_at": "2026-09-18T14:30:00.000Z",
        "fields": {"plan_tier": "Pro"},
        "labels": [{"id": "lbl_1", "name": "VIP", "color": "#0070f3"}]
    })
}

#[tokio::test]
async fn contacts_list_reads_the_pagination_block_not_meta() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/contacts"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("search", "ada"))
        .and(query_param("per_page", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [contact_fixture()],
            "pagination": {"page": 2, "per_page": 10, "total": 11}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .contacts()
        .list(
            &ListContacts::new()
                .workspace("ws_1")
                .search("ada")
                .page(2)
                .per_page(10),
        )
        .await
        .unwrap();

    assert_eq!(page.len(), 1);
    assert_eq!(page.items[0].display_name.as_deref(), Some("Ada Okafor"));
    assert_eq!(
        page.items[0].channels[0].external_id.as_deref(),
        Some("178414")
    );
    assert_eq!(
        page.items[0].fields.get("plan_tier").map(String::as_str),
        Some("Pro")
    );
    assert_eq!(page.pagination.total, 11);
    assert_eq!(page.pagination.page, 2);
}

#[tokio::test]
async fn contacts_create_omits_an_absent_external_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/contacts"))
        .and(body_json(serde_json::json!({
            "workspace_id": "ws_1",
            "channels": [{"platform": "x", "handle": "ada_writes"}],
            "display_name": "Ada Okafor",
            "fields": {"plan_tier": "Pro"}
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": contact_fixture()})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let contact = client
        .contacts()
        .create(
            &CreateContact::new("ws_1", vec![ContactChannel::new("x", "ada_writes")])
                .display_name("Ada Okafor")
                .field("plan_tier", "Pro"),
        )
        .await
        .unwrap();

    assert_eq!(contact.id, "con_1");
}

#[tokio::test]
async fn contacts_update_clears_a_field_with_null_and_sends_nothing_else() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/contacts/con_1"))
        .and(body_json(serde_json::json!({"fields": {"region": null}})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": contact_fixture()})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .contacts()
        .update("con_1", &UpdateContact::new().clear_field("region"))
        .await
        .unwrap();
}

#[tokio::test]
async fn contacts_conversations_reads_the_threads_a_contact_appears_in() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/contacts/con_1/conversations"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "key": "t_182736",
                "account_id": "acc_1",
                "account_username": "yourbrand",
                "platform": "instagram",
                "messages": 14,
                "received": 9,
                "sent": 5,
                "last_message_at": "2026-09-18T14:30:00.000Z",
                "last_item_id": "inb_1"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let rows = client
        .contacts()
        .conversations("con_1", Some(10))
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].key, "t_182736");
    assert_eq!(rows[0].received, 9);
}

#[tokio::test]
async fn contacts_import_reports_what_merged_and_what_was_skipped() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/contacts/import"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "created": 1,
                "merged": 2,
                "skipped": [{"row": 4, "reason": "platform and handle are both required"}],
                "unknownColumns": ["lifetime_value"]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client
        .contacts()
        .import(&ImportContacts::new(
            "ws_1",
            "platform,handle\nx,ada_writes",
        ))
        .await
        .unwrap();

    assert_eq!(result.created, 1);
    assert_eq!(result.merged, 2);
    assert_eq!(result.skipped[0].row, 4);
    assert_eq!(result.unknown_columns, vec!["lifetime_value".to_string()]);
}

#[tokio::test]
async fn contacts_create_field_puts_the_workspace_on_the_query() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/contacts/fields"))
        .and(query_param("workspace_id", "ws_1"))
        .and(body_json(serde_json::json!({
            "key": "plan_tier",
            "name": "Plan Tier",
            "type": "select",
            "options": ["Free", "Pro"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": "fld_1", "key": "plan_tier", "name": "Plan Tier",
                "type": "select", "options": ["Free", "Pro"], "position": 0
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let field = client
        .contacts()
        .create_field(
            "ws_1",
            &CreateContactField::new("plan_tier", "Plan Tier")
                .field_type(ContactFieldType::Select)
                .options(vec!["Free".into(), "Pro".into()]),
        )
        .await
        .unwrap();

    assert_eq!(field.key, "plan_tier");
    assert_eq!(field.field_type, ContactFieldType::Select);
}

#[tokio::test]
async fn contacts_conversation_analytics_reads_the_analytics_route() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/analytics/inbox/conversations"))
        .and(query_param("days", "30"))
        .and(query_param("sort", "slowest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "conversations": [{
                    "key": "t_1", "accountId": "acc_1", "platform": "instagram",
                    "received": 9, "sent": 5, "answered": 5, "open": 1,
                    "medianResponseMinutes": 47, "firstMessageAt": null, "lastMessageAt": null
                }],
                "total": 128, "page": 1, "perPage": 25
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let report = client
        .contacts()
        .conversation_analytics(
            &ConversationAnalyticsQuery::new()
                .days(30)
                .sort(ConversationSort::Slowest),
        )
        .await
        .unwrap();

    assert_eq!(report.total, 128);
    assert_eq!(report.conversations[0].median_response_minutes, Some(47.0));
}

fn broadcast_fixture() -> serde_json::Value {
    serde_json::json!({
        "id": "bc_1",
        "name": "September check-in",
        "text": "New colours just landed.",
        "account_id": "acc_1",
        "audience": {"platforms": ["instagram"]},
        "status": "sent",
        "scheduled_at": null,
        "sent_at": "2026-09-19T10:04:00.000Z",
        "created_at": "2026-09-19T09:58:00.000Z",
        "counts": {"total": 3, "sent": 2, "skipped": 1, "failed": 0, "pending": 0}
    })
}

#[tokio::test]
async fn broadcasts_list_reads_the_pagination_block_not_meta() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/broadcasts"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("status", "sent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [broadcast_fixture()],
            "pagination": {"page": 2, "per_page": 10, "total": 11}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .broadcasts()
        .list(
            &ListBroadcasts::new()
                .workspace("ws_1")
                .status(BroadcastStatus::Sent)
                .page(2)
                .per_page(10),
        )
        .await
        .unwrap();

    assert_eq!(page.len(), 1);
    assert_eq!(page.items[0].name, "September check-in");
    let counts = page.items[0].counts.as_ref().unwrap();
    assert_eq!(counts.sent, 2);
    assert_eq!(counts.skipped, 1);
    assert_eq!(page.pagination.total, 11);
}

#[tokio::test]
async fn broadcasts_create_sends_the_snake_case_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/broadcasts"))
        .and(body_json(serde_json::json!({
            "workspace_id": "ws_1",
            "account_id": "acc_1",
            "name": "September check-in",
            "text": "New colours just landed.",
            "audience": {"platforms": ["instagram"]},
            "scheduled_at": "2026-10-01T09:00:00.000Z"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": broadcast_fixture()
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .broadcasts()
        .create(
            &CreateBroadcast::new(
                "ws_1",
                "acc_1",
                "September check-in",
                "New colours just landed.",
            )
            .audience(AudienceFilter::new().platforms(["instagram"]))
            .scheduled_at("2026-10-01T09:00:00.000Z"),
        )
        .await
        .unwrap();
}

/// A closed messaging window has to be readable, or a non-send is a mystery.
#[tokio::test]
async fn a_skipped_recipient_keeps_its_reason() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/broadcasts/bc_1/recipients"))
        .and(query_param("status", "skipped"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "contact_id": "con_1",
                "display_name": "Sam Rivera",
                "status": "skipped",
                "skip_reason": "window_closed",
                "sent_at": null,
                "error": null
            }],
            "pagination": {"page": 1, "per_page": 50, "total": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .broadcasts()
        .recipients(
            "bc_1",
            &ListRecipients::new().status(RecipientStatus::Skipped),
        )
        .await
        .unwrap();

    assert_eq!(page.items[0].status, RecipientStatus::Skipped);
    assert_eq!(page.items[0].skip_reason, Some(SkipReason::WindowClosed));
}

#[tokio::test]
async fn broadcast_send_reports_how_many_matched() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/broadcasts/bc_1/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "bc_1", "status": "sending", "recipients": 3}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let sent = client.broadcasts().send("bc_1").await.unwrap();

    assert_eq!(sent.recipients, 3);
    assert_eq!(sent.status, "sending");
}

#[tokio::test]
async fn sequence_steps_travel_as_given() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/sequences"))
        .and(body_json(serde_json::json!({
            "workspace_id": "ws_1",
            "account_id": "acc_1",
            "name": "Welcome",
            "steps": [{"delay_hours": 0.0, "text": "Hi"}]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": "seq_1",
                "name": "Welcome",
                "account_id": "acc_1",
                "steps": [
                    {"delay_hours": 0, "text": "Hi"},
                    {"delay_hours": 48, "text": "Still here?"}
                ],
                "status": "active",
                "created_at": "2026-09-12T08:00:00.000Z"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let sequence = client
        .sequences()
        .create(&CreateSequence::new(
            "ws_1",
            "acc_1",
            "Welcome",
            vec![SequenceStep::new(0.0, "Hi")],
        ))
        .await
        .unwrap();

    assert_eq!(sequence.steps[1].delay_hours, 48.0);
}

#[tokio::test]
async fn enroll_takes_ids_or_an_audience() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/sequences/seq_1/enroll"))
        .and(body_json(
            serde_json::json!({"contact_ids": ["con_1", "con_2"]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "seq_1", "enrolled": 2}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let enrolled = client
        .sequences()
        .enroll("seq_1", &Enroll::contacts(["con_1", "con_2"]))
        .await
        .unwrap();

    assert_eq!(enrolled.enrolled, 2);
}

#[tokio::test]
async fn unenroll_names_the_contacts_it_stops() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/sequences/seq_1/unenroll"))
        .and(body_json(serde_json::json!({"contact_ids": ["con_1"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "seq_1", "stopped": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let stopped = client
        .sequences()
        .unenroll("seq_1", &["con_1".to_string()])
        .await
        .unwrap();

    assert_eq!(stopped.stopped, 1);
}

#[tokio::test]
async fn discord_channels_and_the_channel_switch() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/discord/channels"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "c2", "name": "launches", "type": 0, "parent_id": null, "nsfw": false, "can_post": true, "is_current": true}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/accounts/acc_1/discord/channels/current"))
        .and(body_json(serde_json::json!({"channel_id": "c2"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "c2", "name": "launches", "is_current": true}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let channels = client.accounts().discord_channels("acc_1").await.unwrap();
    assert!(channels[0].is_current);
    let switched = client
        .accounts()
        .switch_discord_channel("acc_1", "c2")
        .await
        .unwrap();
    assert_eq!(switched.name, "launches");
}

#[tokio::test]
async fn updating_the_discord_identity_omits_kept_fields_and_nulls_cleared_ones() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/accounts/acc_1/discord/identity"))
        .and(body_json(serde_json::json!({"username": "Release Bot"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"username": "Release Bot", "avatar_url": null}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let update = UpdateDiscordIdentity {
        username: Some(Some("Release Bot".into())),
        ..Default::default()
    };
    let identity = client
        .accounts()
        .update_discord_identity("acc_1", &update)
        .await
        .unwrap();
    assert_eq!(identity.username.as_deref(), Some("Release Bot"));
}

#[tokio::test]
async fn a_discord_scheduled_event_round_trips() {
    let server = MockServer::start().await;
    let event = serde_json::json!({
        "id": "e1",
        "name": "Launch stream",
        "description": null,
        "channel_id": null,
        "location": "https://example.com/live",
        "start_time": "2026-10-01T18:00:00.000Z",
        "end_time": "2026-10-01T19:00:00.000Z",
        "status": "scheduled",
        "user_count": 0
    });
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/discord/events"))
        .and(body_json(serde_json::json!({
            "name": "Launch stream",
            "start_time": "2026-10-01T18:00:00.000Z",
            "end_time": "2026-10-01T19:00:00.000Z",
            "location": "https://example.com/live"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"data": event})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/discord/events"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": [event]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/accounts/acc_1/discord/events/e1"))
        .and(body_json(serde_json::json!({"status": "canceled"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"id": "e1", "name": "Launch stream", "start_time": "2026-10-01T18:00:00.000Z", "status": "canceled"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/accounts/acc_1/discord/events/e1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"deleted": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accounts = client.accounts();

    let created = accounts
        .create_discord_event(
            "acc_1",
            &DiscordEventInput {
                name: Some("Launch stream".into()),
                start_time: Some("2026-10-01T18:00:00.000Z".into()),
                end_time: Some("2026-10-01T19:00:00.000Z".into()),
                location: Some("https://example.com/live".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(created.id, "e1");

    let listed = accounts.discord_events("acc_1").await.unwrap();
    assert_eq!(listed.len(), 1);

    let updated = accounts
        .update_discord_event(
            "acc_1",
            "e1",
            &DiscordEventInput {
                status: Some("canceled".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.status, "canceled");

    let ack = accounts.delete_discord_event("acc_1", "e1").await.unwrap();
    assert_eq!(ack.deleted, Some(true));
}

#[tokio::test]
async fn discord_members_roles_and_dms() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/discord/members"))
        .and(query_param("q", "ada"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "u7", "username": "ada", "is_bot": false, "roles": ["r1"]}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/discord/roles"))
        .and(body_json(serde_json::json!({"name": "Beta"})))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": {"id": "r2", "name": "Beta"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/accounts/acc_1/discord/roles/r2/members/u7"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"assigned": true}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/discord/dm"))
        .and(body_json(
            serde_json::json!({"member_id": "u7", "content": "hi"}),
        ))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": {"id": "m1", "channel_id": "dm1"}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accounts = client.accounts();

    let members = accounts
        .discord_members("acc_1", Some("ada"), None)
        .await
        .unwrap();
    assert_eq!(members[0].roles, vec!["r1".to_string()]);

    let role = accounts
        .create_discord_role(
            "acc_1",
            &DiscordRoleInput {
                name: Some("Beta".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let assigned = accounts
        .add_discord_member_role("acc_1", &role.id, "u7")
        .await
        .unwrap();
    assert_eq!(assigned.assigned, Some(true));

    let sent = accounts.send_discord_dm("acc_1", "u7", "hi").await.unwrap();
    assert_eq!(sent.channel_id, "dm1");
}

#[tokio::test]
async fn a_discord_webhook_connection_is_a_conflict() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/discord/channels"))
        .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "error": "webhook_connection",
            "message": "Upgrade it to the bot first"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client
        .accounts()
        .discord_channels("acc_1")
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(409));
    assert_eq!(err.code(), Some("webhook_connection"));
}

// ─── Google Ads ────────────────────────────────────────────────────

#[tokio::test]
async fn google_keywords_name_the_connection_and_the_customer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/google/keywords"))
        .and(query_param("connection_id", "conn_1"))
        .and(query_param("customer_id", "1234567890"))
        .and(query_param("ad_group_id", "1234567890~adGroup~77"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "1234567890~keyword~77~99",
                "adGroupId": "1234567890~adGroup~77",
                "text": "running shoes",
                "matchType": "EXACT",
                "status": "ENABLED",
                "cpcBidMinor": 180,
                "negative": false
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let scope = GoogleScope::new("conn_1", "1234567890");
    let keywords = client
        .google_ads()
        .keywords(&scope, Some("1234567890~adGroup~77"))
        .await
        .expect("keywords");

    assert_eq!(keywords[0].text, "running shoes");
    assert_eq!(keywords[0].cpc_bid_minor, Some(180));
}

#[tokio::test]
async fn google_create_keyword_sends_the_scope_in_the_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/google/keywords"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "connectionId": "conn_1",
            "customerId": "1234567890",
            "adGroupId": "1234567890~adGroup~77",
            "text": "running shoes",
            "matchType": "EXACT"
        })))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": {"id": "1234567890~keyword~77~99"}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let id = client
        .google_ads()
        .create_keyword(&CreateGoogleKeyword {
            scope: GoogleScope::new("conn_1", "1234567890").in_workspace("ws_1"),
            ad_group_id: "1234567890~adGroup~77".into(),
            text: "running shoes".into(),
            match_type: GoogleMatchType::Exact,
            cpc_bid_minor: None,
        })
        .await
        .expect("create_keyword");

    assert_eq!(id, "1234567890~keyword~77~99");
}

#[tokio::test]
async fn google_ad_schedule_is_replaced_with_put() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/ads/google/ad-schedule"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"slots": 2}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let slots = client
        .google_ads()
        .set_ad_schedule(&SetGoogleAdSchedule {
            scope: GoogleScope::new("conn_1", "1234567890").in_workspace("ws_1"),
            campaign_id: "1234567890~campaign~55".into(),
            slots: vec![GoogleAdScheduleInput {
                day_of_week: GoogleDayOfWeek::Monday,
                start_hour: 9,
                end_hour: 18,
                bid_modifier: None,
            }],
        })
        .await
        .expect("set_ad_schedule");

    assert_eq!(slots, 2);
}

#[tokio::test]
async fn google_query_returns_rows_as_google_sends_them() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/insights/query"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"rows": [{"campaign": {"id": "55"}}]}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client
        .google_ads()
        .query(&GoogleQuery {
            scope: GoogleScope::new("conn_1", "1234567890"),
            query: "SELECT campaign.id FROM campaign".into(),
        })
        .await
        .expect("query");

    assert_eq!(result.rows.len(), 1);
}

#[tokio::test]
async fn authorize_google_has_its_own_route() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/connections/google/authorize"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"url": "https://accounts.google.com/o/x"}}),
            ),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let url = client
        .ads()
        .authorize_google(&AuthorizeGoogleAds::new("ws_1"))
        .await
        .expect("authorize_google");

    assert_eq!(url, "https://accounts.google.com/o/x");
}

#[tokio::test]
async fn creating_a_pinterest_board_omits_the_optionals_it_was_not_given() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/pinterest/boards"))
        .and(body_json(serde_json::json!({"name": "Recipes"})))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {"id": "b1", "name": "Recipes", "privacy": "PUBLIC", "description": null, "image": null}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let board = client
        .accounts()
        .create_pinterest_board(
            "acc_1",
            &CreatePinterestBoard {
                name: "Recipes".to_string(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(board.id, "b1");
}

#[tokio::test]
async fn setting_the_default_youtube_playlist_sends_null_to_clear_it() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/accounts/acc_1/youtube/playlists/default"))
        .and(body_json(serde_json::json!({"playlist_id": null})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"playlist_id": null}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let stored = client
        .accounts()
        .set_default_youtube_playlist("acc_1", None)
        .await
        .unwrap();

    assert!(stored.is_none());
}

#[tokio::test]
async fn bluesky_languages_round_trip_through_the_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/accounts/acc_1/bluesky/languages"))
        .and(body_json(serde_json::json!({"languages": ["en", "pt-BR"]})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"languages": ["en", "pt-BR"]}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client
        .accounts()
        .set_bluesky_languages("acc_1", &["en".to_string(), "pt-BR".to_string()])
        .await
        .unwrap();

    assert_eq!(result.languages, vec!["en", "pt-BR"]);
}

#[tokio::test]
async fn tiktok_creator_info_reports_the_accounts_own_switches() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/tiktok/creator-info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "privacy_level_options": ["PUBLIC_TO_EVERYONE"],
                "duet_disabled": true,
                "max_video_post_duration_sec": 600
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let info = client
        .accounts()
        .tiktok_creator_info("acc_1")
        .await
        .unwrap();

    assert!(info.duet_disabled);
    assert!(!info.stitch_disabled);
    assert_eq!(info.max_video_post_duration_sec, Some(600));
}

#[tokio::test]
async fn tiktok_music_search_passes_the_query_through() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/tiktok/music"))
        .and(query_param("q", "sunrise"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{ "id": "m1", "title": "Sunrise", "author": "Kite" }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let tracks = client
        .accounts()
        .tiktok_music("acc_1", "sunrise", Some(5))
        .await
        .unwrap();

    assert_eq!(tracks[0].id, "m1");
    assert_eq!(tracks[0].author.as_deref(), Some("Kite"));
}

#[tokio::test]
async fn tiktok_video_lookup_returns_the_address_a_repurpose_run_reads() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/tiktok/video-download"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "video_id": "7300000000000000000",
                "download_url": "https://www.tiktok.com/@a/video/7300000000000000000"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let video = client
        .accounts()
        .tiktok_video_lookup(
            "acc_1",
            "https://www.tiktok.com/@a/video/7300000000000000000",
        )
        .await
        .unwrap();

    assert_eq!(video.video_id, "7300000000000000000");
    assert!(video.download_url.is_some());
}

#[tokio::test]
async fn instagram_stories_ask_for_insights_only_when_requested() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/instagram/stories"))
        .and(query_param("insights", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "s1", "media_type": "IMAGE", "insights": {"views": 40}}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let stories = client
        .accounts()
        .instagram_stories("acc_1", true)
        .await
        .unwrap();

    assert_eq!(stories[0].insights.as_ref().unwrap()["views"], 40);
}

#[tokio::test]
async fn linkedin_mentions_carry_the_annotation_to_paste() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/linkedin/mentions"))
        .and(query_param("q", "devtestco"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "urn": "urn:li:organization:2414183",
                "name": "Devtestco",
                "annotation": "@[Devtestco](urn:li:organization:2414183)"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let mentions = client
        .accounts()
        .linkedin_mentions("acc_1", "devtestco")
        .await
        .unwrap();

    assert_eq!(
        mentions[0].annotation,
        "@[Devtestco](urn:li:organization:2414183)"
    );
}

#[tokio::test]
async fn accounts_platform_metrics_asks_for_raw_and_decodes_the_set() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/insights"))
        .and(query_param("raw", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "platform": "facebook",
                "account": {
                    "fetched_at": "2026-09-20T02:00:00.000Z",
                    "metrics": [
                        {
                            "key": "page_daily_video_ad_break_earnings",
                            "label": "Ad Break Earnings",
                            "kind": "currency_usd",
                            "value": 42.15
                        },
                        {
                            "key": "page_impressions_paid",
                            "label": "Paid Impressions",
                            "kind": "count",
                            "value": 1500
                        }
                    ]
                },
                "post": {
                    "external_post_id": "123_456",
                    "fetched_at": "2026-09-20T02:00:00.000Z",
                    "metrics": []
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let metrics = client.accounts().platform_metrics("acc_1").await.unwrap();

    assert_eq!(metrics.platform.as_deref(), Some("facebook"));
    assert_eq!(
        metrics.account.fetched_at.as_deref(),
        Some("2026-09-20T02:00:00.000Z")
    );
    assert_eq!(
        metrics
            .account
            .metrics
            .iter()
            .map(|m| m.key.as_str())
            .collect::<Vec<_>>(),
        vec![
            "page_daily_video_ad_break_earnings",
            "page_impressions_paid"
        ]
    );
    assert_eq!(metrics.account.metrics[0].as_number(), Some(42.15));
    assert_eq!(metrics.post.external_post_id.as_deref(), Some("123_456"));
    assert!(metrics.post.metrics.is_empty());
}

#[tokio::test]
async fn accounts_platform_metrics_keeps_a_series_value_as_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/insights"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "platform": "youtube",
                "account": {
                    "fetched_at": null,
                    "metrics": [{
                        "key": "daily_views",
                        "label": "Views by Day",
                        "kind": "series",
                        "value": [{"day": "2026-09-19", "views": 600}]
                    }]
                },
                "post": {"external_post_id": null, "fetched_at": null, "metrics": []}
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let metrics = client.accounts().platform_metrics("acc_1").await.unwrap();
    let row = &metrics.account.metrics[0];

    assert_eq!(row.as_number(), None);
    assert_eq!(row.value[0]["views"], 600);
    assert!(metrics.account.fetched_at.is_none());
}

#[tokio::test]
async fn accounts_platform_metrics_surfaces_a_pending_grant() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/insights"))
        .respond_with(ResponseTemplate::new(503).set_body_json(serde_json::json!({
            "error": "platform_metrics_unavailable",
            "message": "google-business metrics are not available on this deployment yet."
        })))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client
        .accounts()
        .platform_metrics("acc_1")
        .await
        .expect_err("a pending grant must be an error");

    assert_eq!(err.status(), Some(503));
    assert_eq!(err.code(), Some("platform_metrics_unavailable"));
}

//! One call per resource group, pinning the path, the query casing, and the
//! envelope each endpoint actually uses. The API is not consistent about
//! snake_case versus camelCase, so these are the tests that catch a drift.

mod common;

use common::{account_fixture, client};
use fopost::models::{
    AdBudget, AdGoal, AdKind, AdStatus, AdTargeting, AdTargetingItem, AnalyticsQuery, AudienceSpec,
    BoostPost, CreateAccountGroup, CreateAudience, CreateAutomation, CreateWebhook, InboxItemState,
    InboxItemType, InboxReply, InboxSort, LeadsQuery, ListAccounts, ListInbox, MarkThreadRead,
    Platform, SignalLevel, StartInboxConversation, TelegramBotCommand, TriggerType,
    UpdateInboxItem, UpdateSlackIdentity, ValidateLength, ValidateMedia, ValidateMediaItem,
    ValidatePost, WebhookEvent,
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

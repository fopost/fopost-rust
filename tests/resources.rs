//! One call per resource group, pinning the path, the query casing, and the
//! envelope each endpoint actually uses. The API is not consistent about
//! snake_case versus camelCase, so these are the tests that catch a drift.

mod common;

use common::{account_fixture, client};
use fopost::models::{
    AdBudget, AdGoal, AdKind, AdStatus, AdTargeting, AdTargetingItem, AnalyticsQuery, AudienceSpec,
    BoostPost, CreateAudience, CreateAutomation, CreateWebhook, InboxItemState, InboxItemType,
    InboxSort, LeadsQuery, ListInbox, MarkThreadRead, Platform, TriggerType, UpdateInboxItem,
    WebhookEvent,
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

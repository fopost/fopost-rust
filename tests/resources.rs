//! One call per resource group, pinning the path, the query casing, and the
//! envelope each endpoint actually uses. The API is not consistent about
//! snake_case versus camelCase, so these are the tests that catch a drift.

mod common;

use common::{account_fixture, client};
use fopost::models::{AnalyticsQuery, CreateAutomation, CreateWebhook, TriggerType, WebhookEvent};
use wiremock::matchers::{body_json, method, path, query_param};
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

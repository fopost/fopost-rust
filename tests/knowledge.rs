//! The knowledge base: path, query casing, the partial-update body, and that
//! a camelCase response deserializes into the snake_case model.

mod common;

use common::client;
use fopost::models::{CreateKnowledgeSource, UpdateKnowledgeSource};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn source_fixture() -> serde_json::Value {
    serde_json::json!({
        "id": "know_1",
        "kind": "url",
        "title": "Refund policy",
        "status": "ready",
        "statusMessage": null,
        "url": "https://yourbrand.com/help/refunds",
        "mediaId": null,
        "brandVoiceId": null,
        "chunkCount": 3,
        "content": null,
        "lastSyncedAt": "2026-09-20T00:00:00.000Z",
        "createdAt": "2026-09-19T00:00:00.000Z",
        "updatedAt": "2026-09-20T00:00:00.000Z"
    })
}

#[tokio::test]
async fn list_sends_the_workspace_filter_and_reads_camel_case_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/knowledge/sources"))
        .and(query_param("workspace_id", "ws_1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [source_fixture()]})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let sources = client.knowledge().list(Some("ws_1")).await.unwrap();

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].status, "ready");
    assert_eq!(sources[0].chunk_count, 3);
    assert!(sources[0].status_message.is_none());
}

#[tokio::test]
async fn create_sends_a_snake_case_body_and_omits_what_was_never_set() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/knowledge/sources"))
        .and(body_json(serde_json::json!({
            "kind": "file",
            "title": "Price list",
            "media_id": "media_1",
            "workspace_id": "ws_1"
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": source_fixture()})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .knowledge()
        .create(
            &CreateKnowledgeSource::new("file", "Price list")
                .media_id("media_1")
                .workspace_id("ws_1"),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn update_patches_only_the_fields_that_were_set() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/knowledge/sources/know_1"))
        .and(body_json(serde_json::json!({"title": "Refunds"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": source_fixture()})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .knowledge()
        .update("know_1", &UpdateKnowledgeSource::new().title("Refunds"))
        .await
        .unwrap();
}

#[tokio::test]
async fn search_sends_top_k_and_reads_the_matches() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/knowledge/search"))
        .and(query_param("q", "refunds"))
        .and(query_param("top_k", "3"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": [{
                "sourceId": "know_1",
                "sourceTitle": "Refund policy",
                "sourceKind": "url",
                "sourceUrl": "https://yourbrand.com/help/refunds",
                "text": "We refund within 30 days.",
                "score": 0.82
            }]})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let matches = client
        .knowledge()
        .search("refunds", Some(3), None, None)
        .await
        .unwrap();

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].source_title, "Refund policy");
    assert!((matches[0].score - 0.82).abs() < f64::EPSILON);
}

#[tokio::test]
async fn sync_posts_to_the_sources_sync_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/knowledge/sources/know_1/sync"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"id": "know_1", "status": "pending"}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let queued = client.knowledge().sync("know_1").await.unwrap();
    assert_eq!(queued.status, "pending");
}

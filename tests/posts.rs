mod common;

use common::{client, post_fixture};
use fopost::models::{
    BulkPostAction, ContentBlock, CreatePost, LabelMode, ListPosts, PostStatus, PublishOptions,
    PublishOutcome,
};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn list_returns_a_page_with_its_meta() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/posts"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("status", "draft"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [post_fixture()],
            "meta": {"current_page": 1, "per_page": 30, "total": 1, "last_page": 1, "from": 1, "to": 1}
        })))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .posts()
        .list(&ListPosts::new().workspace("ws_1").status(PostStatus::Draft))
        .await
        .unwrap();

    assert_eq!(page.len(), 1);
    assert_eq!(page.meta.total, 1);
    assert!(!page.has_next());
}

#[tokio::test]
async fn list_all_walks_every_page() {
    let server = MockServer::start().await;
    let page = |current: u32| {
        serde_json::json!({
            "data": [post_fixture()],
            "meta": {"current_page": current, "per_page": 1, "total": 2, "last_page": 2, "from": current, "to": current}
        })
    };
    Mock::given(method("GET"))
        .and(path("/v1/posts"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(1)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/posts"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(page(2)))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let posts = client
        .posts()
        .list_all(&ListPosts::new().per_page(1))
        .await
        .unwrap();
    assert_eq!(posts.len(), 2);
}

#[tokio::test]
async fn create_sends_the_body_the_api_documents() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/posts"))
        .and(body_json(serde_json::json!({
            "workspace_id": "ws_1",
            "accounts": ["acc_1"],
            "content": [{"text": "Hello from Rust", "media": []}],
            "status": "scheduled",
            "schedule_at": "2026-09-01T10:00:00Z"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(post_fixture()))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let body = CreatePost::text("ws_1", "Hello from Rust")
        .accounts(["acc_1"])
        .schedule_at("2026-09-01T10:00:00Z");
    let post = client.posts().create(&body).await.unwrap();
    assert_eq!(post.id, "post_1");
}

#[tokio::test]
async fn a_thread_is_several_content_blocks() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/posts"))
        .and(body_json(serde_json::json!({
            "workspace_id": "ws_1",
            "accounts": [],
            "content": [
                {"text": "one", "media": []},
                {"text": "two", "media": []}
            ]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(post_fixture()))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let body = CreatePost::new(
        "ws_1",
        [ContentBlock::text("one"), ContentBlock::text("two")],
    );
    client.posts().create(&body).await.unwrap();
}

#[tokio::test]
async fn publish_unwraps_the_envelope_and_reads_the_deliveries() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/posts/post_1/publish"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "data": {
                "post_status": "publishing",
                "deliveries": [{"id": "d_1", "accountId": "acc_1", "status": "queued", "attempts": 1}],
                "healthWarnings": []
            }
        })))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let outcome = client
        .posts()
        .publish("post_1", &PublishOptions::new())
        .await
        .unwrap();

    assert!(matches!(outcome, PublishOutcome::Queued(_)));
    assert_eq!(outcome.deliveries().len(), 1);
}

#[tokio::test]
async fn a_dry_run_publishes_nothing_and_decodes_as_a_dry_run() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/posts/post_1/publish"))
        .and(body_json(serde_json::json!({"options": {"dryRun": true}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "dryRun": true,
                "post": {"id": "post_1", "status": "draft"},
                "accounts": [{"accountId": "acc_1", "platform": "twitter"}],
                "healthWarnings": []
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let outcome = client
        .posts()
        .publish("post_1", &PublishOptions::new().dry_run())
        .await
        .unwrap();

    assert!(outcome.deliveries().is_empty());
    match outcome {
        PublishOutcome::DryRun(result) => {
            assert!(result.dry_run);
            assert_eq!(result.accounts.len(), 1);
        }
        PublishOutcome::Queued(_) => panic!("a dry run must not queue deliveries"),
    }
}

#[tokio::test]
async fn a_bulk_action_serializes_as_the_discriminated_union() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/posts/bulk"))
        .and(body_json(serde_json::json!({
            "action": "label",
            "workspace_id": "ws_1",
            "post_ids": ["post_1", "post_2"],
            "label_ids": ["lbl_1"],
            "mode": "add"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"updated": 2, "action": "label", "mode": "add"})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let action = BulkPostAction::label("ws_1", ["post_1", "post_2"], ["lbl_1"], LabelMode::Add);
    let result = client.posts().bulk(&action).await.unwrap();
    assert_eq!(result.updated, 2);
}

#[tokio::test]
async fn cancel_always_sends_the_account_ids_field() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/posts/post_1/cancel"))
        .and(body_json(serde_json::json!({"accountIds": null})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"post_status": "cancelled", "deliveries": []}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client.posts().cancel("post_1", None).await.unwrap();
    assert_eq!(result.post_status.unwrap().as_str(), "cancelled");
}

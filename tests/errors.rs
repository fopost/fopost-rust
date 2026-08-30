mod common;

use common::{client, post_fixture};
use fopost::models::ListPosts;
use fopost::Error;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_404_carries_the_status_the_code_and_the_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/posts/missing"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(
                serde_json::json!({"error": "not_found", "message": "Post not found"}),
            ),
        )
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client.posts().get("missing").await.unwrap_err();

    assert_eq!(err.status(), Some(404));
    assert_eq!(err.code(), Some("not_found"));
    match err {
        Error::Api(api) => {
            assert!(api.is_not_found());
            assert_eq!(api.message, "Post not found");
            assert_eq!(api.to_string(), "[404 not_found] Post not found");
        }
        other => panic!("expected an api error, got {other}"),
    }
}

#[tokio::test]
async fn a_402_exposes_the_upgrade_url_the_api_suggests() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/posts"))
        .respond_with(ResponseTemplate::new(402).set_body_json(serde_json::json!({
            "error": "subscription_required",
            "message": "This workspace has no active subscription",
            "upgrade_url": "https://app.fopost.com/settings/billing"
        })))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let body = fopost::models::CreatePost::text("ws_1", "hi");
    let err = client.posts().create(&body).await.unwrap_err();

    match err {
        Error::Api(api) => {
            assert!(api.is_payment_required());
            assert_eq!(
                api.upgrade_url(),
                Some("https://app.fopost.com/settings/billing")
            );
        }
        other => panic!("expected an api error, got {other}"),
    }
}

#[tokio::test]
async fn a_403_from_a_key_bound_to_another_workspace_is_reported_as_such() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/posts"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "error": "workspace_scope_denied",
            "message": "This API key is bound to another workspace"
        })))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client
        .posts()
        .list(&ListPosts::new().workspace("ws_2"))
        .await
        .unwrap_err();

    assert_eq!(err.code(), Some("workspace_scope_denied"));
    assert!(matches!(&err, Error::Api(api) if api.is_forbidden()));
}

#[tokio::test]
async fn a_429_is_retried_after_the_interval_the_api_asks_for() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/posts/post_1"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "0")
                .set_body_json(serde_json::json!({"error": "too_many_requests"})),
        )
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/posts/post_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(post_fixture()))
        .expect(1)
        .mount(&server)
        .await;

    let client = fopost::Client::builder()
        .api_key(common::API_KEY)
        .base_url(format!("{}/v1", server.uri()))
        .max_retries(3)
        .build()
        .unwrap();

    let post = client.posts().get("post_1").await.unwrap();
    assert_eq!(post.id, "post_1");
}

#[tokio::test]
async fn the_retry_budget_is_finite_and_the_429_surfaces() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/posts/post_1"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("retry-after", "0")
                .set_body_json(serde_json::json!({"error": "too_many_requests"})),
        )
        .expect(2)
        .mount(&server)
        .await;

    let client = fopost::Client::builder()
        .api_key(common::API_KEY)
        .base_url(format!("{}/v1", server.uri()))
        .max_retries(2)
        .build()
        .unwrap();

    let err = client.posts().get("post_1").await.unwrap_err();
    assert!(err.is_rate_limited());
    assert_eq!(err.retry_after(), Some(0.0));
}

#[tokio::test]
async fn an_error_body_that_is_not_json_still_becomes_an_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/posts/post_1"))
        .respond_with(ResponseTemplate::new(502).set_body_string("<html>bad gateway</html>"))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client.posts().get("post_1").await.unwrap_err();

    assert_eq!(err.status(), Some(502));
    assert!(err.to_string().contains("bad gateway"), "{err}");
}

#[tokio::test]
async fn a_success_body_of_the_wrong_shape_is_a_decode_error_that_keeps_the_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/posts/post_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": 12})))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let err = client.posts().get("post_1").await.unwrap_err();

    match err {
        Error::Decode { body, .. } => assert!(body.contains("12"), "{body}"),
        other => panic!("expected a decode error, got {other}"),
    }
}

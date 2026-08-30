mod common;

use common::{client, post_fixture, API_KEY};
use fopost::{Client, DEFAULT_BASE_URL};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn an_api_key_is_required() {
    let err = Client::new("").unwrap_err();
    assert!(err.to_string().contains("api key is required"), "{err}");
}

#[test]
fn the_default_base_url_carries_the_version_path() {
    assert_eq!(DEFAULT_BASE_URL, "https://api.fopost.com/v1");
    let client = Client::new(API_KEY).unwrap();
    assert_eq!(client.base_url(), DEFAULT_BASE_URL);
}

#[test]
fn a_trailing_slash_on_the_base_url_does_not_double_up() {
    let client = Client::builder()
        .api_key(API_KEY)
        .base_url("https://api.example.com/v1/")
        .build()
        .unwrap();
    assert_eq!(client.base_url(), "https://api.example.com/v1");
}

#[test]
fn max_retries_must_be_at_least_one() {
    let err = Client::builder()
        .api_key(API_KEY)
        .max_retries(0)
        .build()
        .unwrap_err();
    assert!(err.to_string().contains("max_retries"), "{err}");
}

#[tokio::test]
async fn every_request_carries_the_api_key_and_user_agent() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/posts/post_1"))
        .and(header("x-api-key", API_KEY))
        .and(header("accept", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(post_fixture()))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let post = client.posts().get("post_1").await.unwrap();
    assert_eq!(post.id, "post_1");

    let sent = &server.received_requests().await.unwrap()[0];
    let agent = sent.headers.get("user-agent").unwrap().to_str().unwrap();
    assert!(agent.starts_with("fopost-rust/"), "{agent}");
}

#[tokio::test]
async fn request_reaches_an_endpoint_the_sdk_does_not_wrap() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/platforms"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let body = client
        .request(reqwest::Method::GET, "/platforms", None, None::<&()>)
        .await
        .unwrap();
    assert_eq!(body, serde_json::json!({"data": []}));
}

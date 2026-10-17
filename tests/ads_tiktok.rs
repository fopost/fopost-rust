//! TikTok Ads runs on the shared ads resource. These pin the paths, the query
//! casing and the two fields a Spark ad and a Smart+ campaign add to bodies
//! that already existed.

mod common;

use common::client;
use fopost::models::{
    AdBudget, AdCommentWrite, AdCommentsQuery, AdGoal, AdObjectQuery, AdTargeting, ConversionEvent,
    CreateAd, CreateCampaign, SparkPostsQuery, UploadConversions,
};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn targeting() -> AdTargeting {
    AdTargeting::new(vec!["US"], 18, 44)
}

#[tokio::test]
async fn spark_posts_are_read_for_one_identity() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/spark-posts"))
        .and(query_param("connection_id", "conn_1"))
        .and(query_param("ad_account_id", "7011"))
        .and(query_param("identity_id", "idt_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "item_99", "identityId": "idt_1", "views": 48213}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let posts = client
        .ads()
        .spark_posts(&SparkPostsQuery::new("conn_1", "7011", "idt_1"))
        .await
        .unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].views, Some(48213));
}

#[tokio::test]
async fn business_centers_and_identities_use_the_tiktok_named_paths() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/tiktok/business-centers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "bc1", "name": "Brand HQ", "role": "ADMIN"}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let centers = client
        .ads()
        .tiktok_business_centers(&AdObjectQuery::new("conn_1"))
        .await
        .unwrap();

    assert_eq!(centers[0].name, "Brand HQ");
}

#[tokio::test]
async fn a_spark_ad_and_a_smart_plus_campaign_carry_their_own_field() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "connectionId": "conn_1",
            "adAccountId": "7011",
            "pageId": "idt_1",
            "name": "Spark",
            "goal": "traffic",
            "budget": {"minor": 2000, "type": "daily"},
            "targeting": {"countries": ["US"], "ageMin": 18, "ageMax": 44, "gender": "all"},
            "text": "",
            "sparkPostId": "item_99"
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"data": {
                "id": "ad_1",
                "workspaceId": "ws_1",
                "kind": "ad",
                "name": "Spark",
                "goal": "traffic",
                "status": "paused"
            }})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/campaigns"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "connectionId": "conn_1",
            "adAccountId": "7011",
            "name": "Smart",
            "goal": "traffic",
            "smartPlus": true
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({"data": {
                "id": "c1", "name": "Smart", "status": "PAUSED"
            }})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .ads()
        .create(
            &CreateAd::new(
                "ws_1",
                "conn_1",
                "7011",
                "idt_1",
                "Spark",
                AdGoal::Traffic,
                AdBudget::daily(2000),
                targeting(),
                "",
            )
            .spark_post_id("item_99"),
        )
        .await
        .unwrap();

    client
        .ads()
        .create_campaign(
            &CreateCampaign::new("ws_1", "conn_1", "7011", "Smart", AdGoal::Traffic)
                .smart_plus(true),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn conversions_report_what_the_network_accepted() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/conversions"))
        .respond_with(
            ResponseTemplate::new(202).set_body_json(serde_json::json!({"data": {"accepted": 2}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accepted = client
        .ads()
        .upload_conversions(&UploadConversions {
            workspace_id: "ws_1".into(),
            connection_id: "conn_1".into(),
            ad_account_id: "7011".into(),
            pixel_id: "px_1".into(),
            events: vec![
                ConversionEvent::new("CompletePayment", "2026-09-18T10:04:00Z")
                    .email("buyer@example.com")
                    .value(4999, "USD"),
            ],
        })
        .await
        .unwrap();

    assert_eq!(accepted, 2);
}

#[tokio::test]
async fn comments_page_and_the_three_writes() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/comments"))
        .and(query_param("ad_id", "ad_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "comments": [{"id": "cm_1", "text": "nice", "likes": 3, "hidden": true}],
                "nextCursor": "2"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/comments/cm_1/reply"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(serde_json::json!({"data": {"replyId": "cm_2"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/comments/cm_1/hide"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"message": "Comment hidden"}})),
        )
        .expect(1)
        .mount(&server)
        .await;
    // The ad travels in the body, because the path already carries the comment.
    Mock::given(method("DELETE"))
        .and(path("/v1/ads/comments/cm_1"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1", "connectionId": "conn_1", "adId": "ad_1"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"message": "Comment deleted"}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .ads()
        .comments(&AdCommentsQuery::new("conn_1", "ad_1"))
        .await
        .unwrap();
    assert_eq!(page.next_cursor.as_deref(), Some("2"));
    assert!(page.comments[0].hidden);
    assert_eq!(page.comments[0].likes, 3);

    let scope = AdCommentWrite::new("ws_1", "conn_1", "ad_1");
    let reply_id = client
        .ads()
        .reply_to_comment("cm_1", &scope.clone().text("Friday!"))
        .await
        .unwrap();
    assert_eq!(reply_id, "cm_2");

    client
        .ads()
        .set_comment_hidden("cm_1", &scope.clone().hidden(true))
        .await
        .unwrap();
    client.ads().delete_comment("cm_1", &scope).await.unwrap();
}

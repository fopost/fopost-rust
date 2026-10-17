//! Meta messaging settings, the webhook subscription, and Messenger hand-over.
//! Each test pins the path and the exact body the API takes.

mod common;

use common::client;
use fopost::models::{MetaGreetingText, MetaIceBreaker, MetaMenuItem, MetaPersistentMenuEntry};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn ice_breakers_are_read_replaced_and_cleared() {
    let server = MockServer::start().await;
    let breakers = serde_json::json!({
        "data": {"ice_breakers": [{"question": "What are your hours?", "payload": "HOURS"}]}
    });
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/messaging/ice-breakers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(breakers.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/accounts/acc_1/messaging/ice-breakers"))
        .and(body_json(serde_json::json!({
            "ice_breakers": [{"question": "What are your hours?", "payload": "HOURS"}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(breakers))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/accounts/acc_1/messaging/ice-breakers"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"ice_breakers": []}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accounts = client.accounts();
    let got = accounts.ice_breakers("acc_1").await.unwrap();
    assert_eq!(got.ice_breakers[0].payload, "HOURS");
    let set = accounts
        .set_ice_breakers(
            "acc_1",
            &[MetaIceBreaker::new("What are your hours?", "HOURS")],
        )
        .await
        .unwrap();
    assert_eq!(set.ice_breakers[0].question, "What are your hours?");
    let cleared = accounts.delete_ice_breakers("acc_1").await.unwrap();
    assert!(cleared.ice_breakers.is_empty());
}

#[tokio::test]
async fn a_link_menu_item_omits_the_payload_key() {
    let server = MockServer::start().await;
    let menu = serde_json::json!({
        "data": {"persistent_menu": [{
            "locale": "default",
            "call_to_actions": [{"type": "web_url", "title": "Shop", "url": "https://example.com/shop"}]
        }]}
    });
    Mock::given(method("PUT"))
        .and(path("/v1/accounts/acc_1/messaging/persistent-menu"))
        .and(body_json(serde_json::json!({
            "persistent_menu": [{
                "locale": "default",
                "call_to_actions": [{"type": "web_url", "title": "Shop", "url": "https://example.com/shop"}]
            }]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(menu))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let set = client
        .accounts()
        .set_persistent_menu(
            "acc_1",
            &[MetaPersistentMenuEntry::default_locale(vec![
                MetaMenuItem::link("Shop", "https://example.com/shop"),
            ])],
        )
        .await
        .unwrap();
    assert_eq!(
        set.persistent_menu[0].call_to_actions[0].url.as_deref(),
        Some("https://example.com/shop")
    );
}

#[tokio::test]
async fn the_greeting_defaults_its_locale() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/accounts/acc_1/messaging/greeting"))
        .and(body_json(serde_json::json!({
            "greeting": [{"locale": "default", "text": "Hi! Ask us anything."}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"greeting": [{"locale": "default", "text": "Hi! Ask us anything."}]}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let saved = client
        .accounts()
        .set_greeting("acc_1", &[MetaGreetingText::new("Hi! Ask us anything.")])
        .await
        .unwrap();
    assert_eq!(saved.greeting[0].locale, "default");
}

#[tokio::test]
async fn a_lapsed_subscription_is_reported_and_resubscribed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/webhook-subscription"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"subscribed": false, "fields": ["feed"], "missing_fields": ["messages"]}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/webhook-subscription"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"subscribed": true, "fields": ["feed", "messages"], "missing_fields": []}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accounts = client.accounts();
    let lapsed = accounts.webhook_subscription("acc_1").await.unwrap();
    assert!(!lapsed.subscribed);
    assert_eq!(lapsed.missing_fields, vec!["messages".to_string()]);
    let fixed = accounts.resubscribe_webhook("acc_1").await.unwrap();
    assert!(fixed.subscribed);
}

#[tokio::test]
async fn handover_passes_to_an_app_and_takes_control_back() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/inbox/conversations/t_1/handover"))
        .and(body_json(serde_json::json!({
            "account_id": "acc_1", "app_id": "263902037430900"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"app_id": "263902037430900", "control": "passed"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/inbox/conversations/t_2/handover"))
        .and(body_json(serde_json::json!({"account_id": "acc_1"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {"app_id": null, "control": "taken"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let inbox = client.inbox();
    let passed = inbox
        .handover("t_1", "acc_1", Some("263902037430900"), None)
        .await
        .unwrap();
    assert_eq!(passed.control, "passed");
    let taken = inbox.handover("t_2", "acc_1", None, None).await.unwrap();
    assert_eq!(taken.control, "taken");
    assert!(taken.app_id.is_none());
}

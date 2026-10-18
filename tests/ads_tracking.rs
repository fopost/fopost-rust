//! Pixels, the Conversions API and custom conversions on the shared ads
//! resource. These pin the idempotency header, the account-scoped reads and
//! the create bodies.

mod common;

use common::client;
use fopost::models::{
    ActionSource, AdAccountQuery, ConversionEvent, CreateCustomConversion,
    CustomConversionEventType, CustomConversionRule, PixelDataUseSetting, UploadConversions,
};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn upload() -> UploadConversions {
    UploadConversions {
        workspace_id: "ws_1".into(),
        connection_id: "conn_1".into(),
        ad_account_id: "act_1".into(),
        pixel_id: "px_1".into(),
        events: vec![ConversionEvent::new("Purchase", "2026-10-18T10:04:00Z")
            .event_id("order_77")
            .action_source(ActionSource::Website)
            .email("buyer@example.com")
            .client("203.0.113.9", "Mozilla/5.0")
            .value(4999, "USD")
            .order_id("77")],
    }
}

#[tokio::test]
async fn send_conversions_carries_the_idempotency_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/conversions/events"))
        .and(header("idempotency-key", "order-77-purchase"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "connectionId": "conn_1",
            "adAccountId": "act_1",
            "pixelId": "px_1",
            "events": [{
                "eventName": "Purchase",
                "occurredAt": "2026-10-18T10:04:00Z",
                "eventId": "order_77",
                "actionSource": "website",
                "email": "buyer@example.com",
                "clientIpAddress": "203.0.113.9",
                "clientUserAgent": "Mozilla/5.0",
                "valueMinor": 4999,
                "currency": "USD",
                "orderId": "77"
            }]
        })))
        .respond_with(
            ResponseTemplate::new(202)
                .insert_header("idempotency-replayed", "true")
                .set_body_json(serde_json::json!({
                    "data": {"id": "batch_1", "accepted": 1, "eventCount": 1, "replayed": true}
                })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client
        .ads()
        .send_conversions(&upload(), "order-77-purchase")
        .await
        .unwrap();

    assert_eq!(result.id, "batch_1");
    assert_eq!(result.accepted, 1);
    assert!(result.replayed);
}

#[tokio::test]
async fn pixel_is_read_with_its_install_script() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/pixels/px_1"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("connection_id", "conn_1"))
        .and(query_param("ad_account_id", "act_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "px_1",
                "name": "Site",
                "code": "<script>fbq('init', 'px_1')</script>",
                "createdAt": "2026-10-01T00:00:00Z",
                "lastFiredAt": null,
                "isUnavailable": false,
                "dataUseSetting": "analytics_only"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let pixel = client
        .ads()
        .pixel(
            "px_1",
            &AdAccountQuery::new("conn_1", "act_1").workspace("ws_1"),
        )
        .await
        .unwrap();

    assert_eq!(pixel.name, "Site");
    assert_eq!(
        pixel.code.as_deref(),
        Some("<script>fbq('init', 'px_1')</script>")
    );
    assert_eq!(pixel.last_fired_at, None);
    assert_eq!(
        pixel.data_use_setting,
        Some(PixelDataUseSetting::AnalyticsOnly)
    );
}

#[tokio::test]
async fn custom_conversion_is_created_from_a_url_rule() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/custom-conversions"))
        .and(body_json(serde_json::json!({
            "workspaceId": "ws_1",
            "connectionId": "conn_1",
            "adAccountId": "act_1",
            "pixelId": "px_1",
            "name": "Checkout done",
            "eventType": "PURCHASE",
            "rule": {"urlContains": "/thank-you"},
            "defaultValueMinor": 2500
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "data": {
                "id": "cc_1",
                "name": "Checkout done",
                "pixelId": "px_1",
                "eventType": "PURCHASE",
                "rule": {"eventName": "PageView", "urlContains": "/thank-you"},
                "description": null,
                "defaultValueMinor": 2500,
                "isArchived": false,
                "isUnavailable": false,
                "createdAt": "2026-10-18T10:00:00Z",
                "lastFiredAt": null
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let conversion = client
        .ads()
        .create_custom_conversion(&CreateCustomConversion {
            workspace_id: "ws_1".into(),
            connection_id: "conn_1".into(),
            ad_account_id: "act_1".into(),
            pixel_id: "px_1".into(),
            name: "Checkout done".into(),
            event_type: CustomConversionEventType::Purchase,
            rule: CustomConversionRule::url_contains("/thank-you"),
            description: None,
            default_value_minor: Some(2500),
        })
        .await
        .unwrap();

    assert_eq!(conversion.id, "cc_1");
    assert_eq!(
        conversion.event_type,
        Some(CustomConversionEventType::Purchase)
    );
    let rule = conversion.rule.unwrap();
    assert_eq!(rule.event_name.as_deref(), Some("PageView"));
    assert_eq!(rule.url_contains.as_deref(), Some("/thank-you"));
    assert!(!conversion.is_archived);
}

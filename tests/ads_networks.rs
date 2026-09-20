//! A second ad network behind the same endpoints: the registry read, the
//! per-provider login, a company-list upload and a conversion event.

mod common;

use common::client;
use fopost::models::{AdCompany, AdObjectQuery, AuthorizeMetaAds, ConversionEvent};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn authorize_reaches_whichever_network_the_registry_named() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/connections/linkedin/authorize"))
        .and(body_json(serde_json::json!({ "workspaceId": "ws_1" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": { "url": "https://www.linkedin.com/oauth" }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let url = client
        .ads()
        .authorize("linkedin", &AuthorizeMetaAds::new("ws_1"))
        .await
        .unwrap();
    assert_eq!(url, "https://www.linkedin.com/oauth");
}

#[tokio::test]
async fn providers_carry_what_each_network_supports() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/ads/providers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "linkedin",
                "name": "LinkedIn Ads",
                "configured": false,
                "capabilities": { "conversions": true },
                "targetingFacets": ["country", "job_title"]
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let providers = client.ads().providers().await.unwrap();
    assert_eq!(providers[0].id, "linkedin");
    assert!(!providers[0].configured);
    assert_eq!(providers[0].capabilities.get("conversions"), Some(&true));
    assert_eq!(providers[0].targeting_facets, ["country", "job_title"]);
}

#[tokio::test]
async fn company_rows_travel_with_the_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/ads/audiences/urn:li:adSegment:44/companies"))
        .and(body_json(serde_json::json!({
            "companies": [{ "domain": "northwind.example" }, { "name": "Contoso" }]
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": { "added": 2 } })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let added = client
        .ads()
        .add_audience_companies(
            "urn:li:adSegment:44",
            &AdObjectQuery::new("conn_1").workspace("ws_1"),
            &[
                AdCompany {
                    domain: Some("northwind.example".into()),
                    ..Default::default()
                },
                AdCompany {
                    name: Some("Contoso".into()),
                    ..Default::default()
                },
            ],
        )
        .await
        .unwrap();
    assert_eq!(added, 2);
}

#[tokio::test]
async fn conversion_events_send_the_identity_the_api_hashes() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/v1/ads/linkedin/conversion-rules/urn:li:conversion:9/events",
        ))
        .and(query_param("connection_id", "conn_1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({ "data": { "accepted": 1 } })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let accepted = client
        .ads()
        .send_conversion_events(
            "urn:li:conversion:9",
            &AdObjectQuery::new("conn_1").workspace("ws_1"),
            &[ConversionEvent {
                happened_at: 1_758_326_400_000,
                email: Some("buyer@example.test".into()),
                ..Default::default()
            }],
        )
        .await
        .unwrap();
    assert_eq!(accepted, 1);
}

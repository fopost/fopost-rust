//! Business Profile management: one call per route, pinning the path, the
//! query casing and the body each endpoint actually receives.

mod common;

use common::client;
use fopost::resources::google_business::{CreatePlaceAction, StartVerification, UpdateLocation};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn ok(server: &MockServer, verb: &str, route: &str) {
    Mock::given(method(verb))
        .and(path(route))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "ok": true } })))
        .expect(1)
        .mount(server)
        .await;
}

#[tokio::test]
async fn every_method_maps_onto_its_route() {
    let server = MockServer::start().await;
    let base = "/v1/accounts/acc_1/gbp";

    for (verb, route) in [
        ("GET", "/location"),
        ("PATCH", "/location"),
        ("GET", "/attributes"),
        ("PATCH", "/attributes"),
        ("GET", "/menus"),
        ("PUT", "/menus"),
        ("GET", "/services"),
        ("PUT", "/services"),
        ("GET", "/media"),
        ("POST", "/media"),
        ("DELETE", "/media/CAoSL"),
        ("GET", "/place-actions"),
        ("POST", "/place-actions"),
        ("PATCH", "/place-actions/links-1"),
        ("DELETE", "/place-actions/links-1"),
        ("GET", "/verification"),
        ("POST", "/verification/start"),
        ("POST", "/verification/complete"),
        ("GET", "/performance"),
    ] {
        ok(&server, verb, &format!("{base}{route}")).await;
    }

    let client = client(&server).await;
    let gb = client.google_business();

    gb.get_location("acc_1").await.unwrap();
    gb.update_location("acc_1", &UpdateLocation::default())
        .await
        .unwrap();
    gb.get_attributes("acc_1", false, None, None, None)
        .await
        .unwrap();
    gb.update_attributes("acc_1", &[]).await.unwrap();
    gb.get_menus("acc_1").await.unwrap();
    gb.replace_menus("acc_1", &[]).await.unwrap();
    gb.get_services("acc_1").await.unwrap();
    gb.replace_services("acc_1", &[]).await.unwrap();
    gb.list_media("acc_1", None, None).await.unwrap();
    gb.add_media("acc_1", "m_1", None, None).await.unwrap();
    gb.delete_media("acc_1", "CAoSL").await.unwrap();
    gb.list_place_actions("acc_1").await.unwrap();
    gb.create_place_action(
        "acc_1",
        &CreatePlaceAction {
            uri: "https://example.com/book".into(),
            place_action_type: "APPOINTMENT".into(),
            is_preferred: None,
        },
    )
    .await
    .unwrap();
    gb.update_place_action("acc_1", "links-1", None, Some(true))
        .await
        .unwrap();
    gb.delete_place_action("acc_1", "links-1").await.unwrap();
    gb.get_verification_options("acc_1", None).await.unwrap();
    gb.start_verification(
        "acc_1",
        &StartVerification {
            method: "SMS".into(),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    gb.complete_verification("acc_1", "v1", "123456")
        .await
        .unwrap();
    gb.get_performance("acc_1", "2026-09-01", "2026-09-07", &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn update_location_sends_only_the_fields_the_caller_set() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/accounts/acc_1/gbp/location"))
        .and(body_json(
            json!({ "title": "Corner Bakery", "description": null }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .google_business()
        .update_location(
            "acc_1",
            &UpdateLocation {
                title: Some("Corner Bakery".into()),
                description: Some(None),
                ..Default::default()
            },
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn a_photo_is_named_by_its_library_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/gbp/media"))
        .and(body_json(
            json!({ "media_id": "m_1", "category": "INTERIOR" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .google_business()
        .add_media("acc_1", "m_1", Some("INTERIOR"), None)
        .await
        .unwrap();
}

#[tokio::test]
async fn performance_repeats_the_metric_parameter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/gbp/performance"))
        .and(query_param("start_date", "2026-09-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .google_business()
        .get_performance(
            "acc_1",
            "2026-09-01",
            "2026-09-07",
            &["CALL_CLICKS", "WEBSITE_CLICKS"],
        )
        .await
        .unwrap();

    let request = &server.received_requests().await.unwrap()[0];
    let metrics: Vec<String> = request
        .url
        .query_pairs()
        .filter(|(key, _)| key == "daily_metrics")
        .map(|(_, value)| value.into_owned())
        .collect();
    assert_eq!(metrics, ["CALL_CLICKS", "WEBSITE_CLICKS"]);
}

#[tokio::test]
async fn search_keywords_asks_the_same_route_for_the_monthly_terms() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/gbp/performance"))
        .and(query_param("keywords", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {} })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .google_business()
        .get_search_keywords("acc_1", "2026-08-01", "2026-09-01", None)
        .await
        .unwrap();
}

#[tokio::test]
async fn assign_hands_the_location_to_another_workspace() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/accounts/acc_1/gbp/assign"))
        .and(body_json(json!({ "workspace_id": "ws_2" })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "data": { "id": "acc_1", "workspace_id": "ws_2" } })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let moved = client
        .google_business()
        .assign("acc_1", "ws_2")
        .await
        .unwrap();
    assert_eq!(moved.id, "acc_1");
}

#[tokio::test]
async fn a_pending_api_grant_surfaces_as_unavailable() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/gbp/location"))
        .respond_with(ResponseTemplate::new(503).set_body_json(
            json!({ "error": "configuration_error", "message": "Not available yet" }),
        ))
        .mount(&server)
        .await;

    let client = client(&server).await;
    let error = client
        .google_business()
        .get_location("acc_1")
        .await
        .unwrap_err();
    assert!(
        format!("{error}").contains("configuration_error"),
        "error = {error}"
    );
}

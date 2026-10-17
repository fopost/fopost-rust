//! The activity log, and the security audit rows inside it.

mod common;

use common::client;
use fopost::models::ListActivity;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn reads_the_audit_log_and_keeps_the_cursor() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/activity"))
        .and(query_param("kind", "security"))
        .and(query_param("workspace_id", "ws_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "id": "evt_1",
                "workspace_id": "ws_1",
                "kind": "security",
                "ref_type": "member_removed",
                "ref_id": "usr_2",
                "summary": "Removed sam@example.com",
                "actor": {"type": "user", "name": "Ada"},
                "time": "2026-09-20T10:00:00Z"
            }],
            "meta": {"next_cursor": "42"}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .activity()
        .list(&ListActivity {
            workspace_id: Some("ws_1"),
            kind: Some("security"),
            limit: Some(1),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].ref_type.as_deref(), Some("member_removed"));
    assert_eq!(page.data[0].actor.name.as_deref(), Some("Ada"));
    assert_eq!(page.meta.next_cursor.as_deref(), Some("42"));
}

#[tokio::test]
async fn the_end_of_the_list_is_no_cursor() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/activity"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [], "meta": {"next_cursor": null}})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .activity()
        .list(&ListActivity::default())
        .await
        .unwrap();

    assert!(page.data.is_empty());
    assert!(page.meta.next_cursor.is_none());
}

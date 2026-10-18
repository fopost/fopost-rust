//! Request logs: every filter lands on the query string, and the page carries
//! its cursor and the window's totals beside the rows.

mod common;

use common::client;
use fopost::models::ListLogs;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn logs_page_with_a_cursor_and_a_summary() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/logs"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("days", "3"))
        .and(query_param("status", "failed"))
        .and(query_param("status_code", "404"))
        .and(query_param("route", "/v1/posts/{id}"))
        .and(query_param("q", "not found"))
        .and(query_param("limit", "1"))
        .and(query_param("before", "900"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "id": 812,
                "workspace_id": "ws_1",
                "time": "2026-10-18T10:00:00.000Z",
                "auth_method": "api_key",
                "method": "GET",
                "route": "/v1/posts/{id}",
                "path": "/v1/posts/post_9",
                "status_code": 404,
                "duration_ms": 12,
                "platform": null,
                "message": "Post not found"
            }],
            "meta": { "next_cursor": "812", "has_more": true },
            "summary": {
                "since": "2026-10-15T10:00:00.000Z",
                "total": 40,
                "errors": 3,
                "buckets": [{ "at": "2026-10-18T00:00:00.000Z", "total": 10, "errors": 1 }]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .logs()
        .list(&ListLogs {
            workspace_id: Some("ws_1"),
            days: Some(3),
            status: Some("failed"),
            status_code: Some(404),
            route: Some("/v1/posts/{id}"),
            q: Some("not found"),
            limit: Some(1),
            before: Some("900"),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].id, 812);
    assert_eq!(page.data[0].status_code, Some(404));
    assert_eq!(page.data[0].message.as_deref(), Some("Post not found"));
    assert_eq!(page.meta.next_cursor.as_deref(), Some("812"));
    assert!(page.meta.has_more);
    assert_eq!(page.summary.errors, 3);
    assert_eq!(page.summary.buckets.len(), 1);
}

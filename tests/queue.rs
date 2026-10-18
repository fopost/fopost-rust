//! The posting queue: slots are a snake_case body, a changed slot reports the
//! posts it moved, and the next and preview reads carry the workspace on the
//! query string.

mod common;

use common::client;
use fopost::models::{CreateQueueSlot, UpdateQueueSlot};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn slot() -> serde_json::Value {
    json!({
        "id": "slot_1",
        "workspace_id": "ws_1",
        "account_id": null,
        "weekday": 1,
        "time": "09:30",
        "timezone": "Europe/Zagreb",
        "created_at": "2026-10-18T10:00:00.000Z"
    })
}

#[tokio::test]
async fn slots_are_listed_created_updated_and_deleted() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/queue/slots"))
        .and(query_param("workspace_id", "ws_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [slot()] })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/queue/slots"))
        .and(body_json(json!({
            "workspace_id": "ws_1",
            "account_id": "acc_1",
            "weekday": 1,
            "time": "09:30",
            "timezone": "Europe/Zagreb"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(slot()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/v1/queue/slots/slot_1"))
        .and(body_json(json!({ "account_id": null, "time": "10:00" })))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut updated = slot();
            updated["time"] = json!("10:00");
            updated["moved"] = json!([{
                "id": "post_1",
                "from": "2026-10-19T07:30:00.000Z",
                "to": "2026-10-19T08:00:00.000Z"
            }]);
            updated
        }))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/queue/slots/slot_1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "deleted": true, "moved": [] })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let queue = client.queue();

    let slots = queue.slots("ws_1").await.unwrap();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].weekday, 1);
    assert!(slots[0].account_id.is_none());

    let created = queue
        .create_slot(&CreateQueueSlot::new("ws_1", 1, "09:30", "Europe/Zagreb").account("acc_1"))
        .await
        .unwrap();
    assert_eq!(created.id, "slot_1");

    let updated = queue
        .update_slot(
            "slot_1",
            &UpdateQueueSlot::new().account(None).time("10:00"),
        )
        .await
        .unwrap();
    assert_eq!(updated.time, "10:00");
    assert_eq!(updated.moved.len(), 1);
    assert_eq!(updated.moved[0].id, "post_1");

    let deleted = queue.delete_slot("slot_1").await.unwrap();
    assert!(deleted.deleted);
    assert!(deleted.moved.is_empty());
}

#[tokio::test]
async fn next_and_preview_read_the_free_times() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/queue/next"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("account_id", "acc_1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "next": "2026-10-19T07:30:00.000Z" })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/queue/preview"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("count", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": ["2026-10-19T07:30:00.000Z", "2026-10-20T07:30:00.000Z"]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let next = client.queue().next("ws_1", Some("acc_1")).await.unwrap();
    assert_eq!(next.as_deref(), Some("2026-10-19T07:30:00.000Z"));

    let preview = client.queue().preview("ws_1", None, Some(2)).await.unwrap();
    assert_eq!(preview.len(), 2);
    assert_eq!(preview[1], "2026-10-20T07:30:00.000Z");
}

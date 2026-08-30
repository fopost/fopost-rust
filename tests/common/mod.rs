#![allow(dead_code)]

use fopost::Client;
use wiremock::MockServer;

pub const API_KEY: &str = "fp_test_key";

/// A client pointed at a mock server, with retries off unless a test wants them.
pub async fn client(server: &MockServer) -> Client {
    Client::builder()
        .api_key(API_KEY)
        .base_url(format!("{}/v1", server.uri()))
        .max_retries(1)
        .build()
        .expect("client")
}

pub fn post_fixture() -> serde_json::Value {
    serde_json::json!({
        "id": "post_1",
        "workspace_id": "ws_1",
        "status": "draft",
        "content_type": "post",
        "schedule_at": null,
        "repeatable": false,
        "content": [{ "id": 1, "text": "Hello from Rust", "media": [], "position": 0 }],
        "accounts": [{
            "id": "acc_1",
            "platform": "twitter",
            "username": "fopost",
            "name": "FoPost",
            "publish_status": "pending",
            "attempts": 0,
            "max_attempts": 3
        }],
        "labels": [],
        "settings": {},
        "created_at": "2026-08-12T10:00:00.000Z",
        "updated_at": "2026-08-12T10:00:00.000Z"
    })
}

pub fn account_fixture() -> serde_json::Value {
    serde_json::json!({
        "id": "acc_1",
        "workspaceId": "ws_1",
        "platform": "twitter",
        "username": "fopost",
        "name": "FoPost",
        "avatar": null,
        "isPrimary": true,
        "active": true,
        "healthStatus": "healthy",
        "lastHealthCheck": "2026-08-12T09:00:00.000Z"
    })
}

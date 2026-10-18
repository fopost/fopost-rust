//! Hosted connect links: the body is camelCase, the answer snake_case, and
//! the hosted `url` rides only on creation.

mod common;

use common::client;
use fopost::models::{ConnectBranding, ConnectSessionStatus, CreateConnectSession, Platform};
use serde_json::json;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn a_session_is_created_and_read_back() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/connect-sessions"))
        .and(body_json(json!({
            "workspaceId": "ws_1",
            "externalUserId": "user-42",
            "platforms": ["twitter", "linkedin"],
            "redirectUrl": "https://yourbrand.com/connected",
            "branding": { "name": "Your Brand", "logoUrl": "https://yourbrand.com/logo.png" }
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "cs_1",
            "status": "pending",
            "external_user_id": "user-42",
            "platforms": ["twitter", "linkedin"],
            "redirect_url": "https://yourbrand.com/connected",
            "branding": { "name": "Your Brand", "logo_url": "https://yourbrand.com/logo.png" },
            "account_id": null,
            "url": "https://app.fopost.com/connect/tok_abc",
            "expires_at": "2026-10-18T11:00:00.000Z",
            "used_at": null,
            "created_at": "2026-10-18T10:00:00.000Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/connect-sessions/cs_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "cs_1",
            "status": "used",
            "external_user_id": "user-42",
            "platforms": ["twitter", "linkedin"],
            "redirect_url": "https://yourbrand.com/connected",
            "branding": { "name": "Your Brand", "logo_url": null },
            "account_id": "acc_1",
            "expires_at": "2026-10-18T11:00:00.000Z",
            "used_at": "2026-10-18T10:05:00.000Z",
            "created_at": "2026-10-18T10:00:00.000Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let created = client
        .connect_sessions()
        .create(
            &CreateConnectSession::new("user-42", [Platform::Twitter, Platform::Linkedin])
                .workspace("ws_1")
                .redirect_url("https://yourbrand.com/connected")
                .branding(ConnectBranding {
                    name: Some("Your Brand".into()),
                    logo_url: Some("https://yourbrand.com/logo.png".into()),
                }),
        )
        .await
        .unwrap();
    assert_eq!(created.status, ConnectSessionStatus::Pending);
    assert_eq!(
        created.url.as_deref(),
        Some("https://app.fopost.com/connect/tok_abc")
    );
    assert_eq!(created.branding.name.as_deref(), Some("Your Brand"));

    let used = client.connect_sessions().get("cs_1").await.unwrap();
    assert_eq!(used.status, ConnectSessionStatus::Used);
    assert_eq!(used.account_id.as_deref(), Some("acc_1"));
    assert!(used.url.is_none());
    assert_eq!(used.platforms, vec![Platform::Twitter, Platform::Linkedin]);
}

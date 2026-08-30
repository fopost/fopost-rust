//! The response models have to survive the API growing new values.

use fopost::models::{Account, HealthStatus, Platform, Post, PostStatus, Workspace};

#[test]
fn a_platform_this_version_does_not_know_still_parses() {
    let json = serde_json::json!({
        "id": "acc_1",
        "workspaceId": "ws_1",
        "platform": "some-network-shipped-next-year",
        "healthStatus": "quarantined"
    });

    let account: Account = serde_json::from_value(json).unwrap();
    assert_eq!(account.platform, "some-network-shipped-next-year");
    assert_eq!(
        account.health_status,
        Some(HealthStatus::Other("quarantined".into()))
    );
}

#[test]
fn a_status_this_version_does_not_know_still_parses() {
    let json = serde_json::json!({
        "id": "p1",
        "workspace_id": "ws_1",
        "status": "awaiting_legal_review",
        "content_type": "post",
        "content": [],
        "accounts": [],
        "labels": []
    });

    let post: Post = serde_json::from_value(json).unwrap();
    assert_eq!(
        post.status,
        PostStatus::Other("awaiting_legal_review".into())
    );
    assert_eq!(post.status.to_string(), "awaiting_legal_review");
}

#[test]
fn a_field_the_sdk_does_not_model_is_ignored_rather_than_fatal() {
    let json = serde_json::json!({
        "id": "ws_1",
        "name": "Acme",
        "slug": "acme",
        "type": "BRAND",
        "somethingAddedLater": {"nested": true}
    });

    let workspace: Workspace = serde_json::from_value(json).unwrap();
    assert_eq!(workspace.name, "Acme");
}

#[test]
fn known_values_round_trip_through_their_wire_spelling() {
    assert_eq!(Platform::InstagramBusiness.as_str(), "instagram-business");
    assert_eq!(Platform::from("google-business"), Platform::GoogleBusiness);
    assert_eq!(
        serde_json::to_string(&PostStatus::PartiallyFailed).unwrap(),
        "\"partially_failed\""
    );
    assert_eq!(
        Platform::from("brand-new"),
        Platform::Other("brand-new".into())
    );
}

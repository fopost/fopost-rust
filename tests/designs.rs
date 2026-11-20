//! Designs: snake_case bodies, `{data}` envelopes, and the `{data, version}`
//! answer of update and restore read as one value.

mod common;

use std::collections::BTreeMap;

use common::client;
use fopost::models::{
    BulkCreateDesigns, CreateDesign, DesignExportFormat, DesignFromPost, DesignKind,
    DesignPostStyle, ExportDesign, ListDesigns, Platform, PostStatus, TextToSlides, UpdateDesign,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn design() -> serde_json::Value {
    json!({
        "id": "des_1",
        "name": "Launch",
        "kind": "story",
        "width": 1080,
        "height": 1920,
        "document": { "pages": [{ "layers": [] }] },
        "thumbnail_media_id": null,
        "thumbnail_url": null,
        "created_at": "2026-11-20T10:00:00.000Z",
        "updated_at": "2026-11-20T10:00:00.000Z"
    })
}

fn version() -> serde_json::Value {
    json!({ "id": "ver_1", "version": 2, "created_at": "2026-11-20T11:00:00.000Z" })
}

#[tokio::test]
async fn designs_are_listed_created_read_and_deleted() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/designs"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("kind", "story"))
        .and(query_param("limit", "10"))
        .and(query_param("offset", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [design()] })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/designs"))
        .and(body_json(json!({
            "workspace_id": "ws_1",
            "kind": "story",
            "name": "Launch",
            "document": { "pages": [] },
            "brand_voice_id": "bv_1"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "data": design() })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/designs/des_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": design() })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/designs/des_1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "data": { "id": "des_1", "deleted": true } })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let designs = client
        .designs()
        .list(&ListDesigns {
            workspace_id: "ws_1",
            kind: Some(DesignKind::Story),
            limit: Some(10),
            offset: Some(20),
        })
        .await
        .unwrap();
    assert_eq!(designs.len(), 1);
    assert_eq!(designs[0].kind, DesignKind::Story);
    assert_eq!(designs[0].height, 1920);

    let created = client
        .designs()
        .create(
            &CreateDesign::new("ws_1", DesignKind::Story)
                .name("Launch")
                .document(json!({ "pages": [] }))
                .brand_voice("bv_1"),
        )
        .await
        .unwrap();
    assert_eq!(created.id, "des_1");

    let got = client.designs().get("des_1").await.unwrap();
    assert_eq!(got.document["pages"][0]["layers"], json!([]));

    let deleted = client.designs().delete("des_1").await.unwrap();
    assert!(deleted.deleted);
    assert_eq!(deleted.id, "des_1");
}

#[tokio::test]
async fn update_and_restore_carry_the_saved_version() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/designs/des_1"))
        .and(body_json(json!({
            "name": "Launch v2",
            "document": { "pages": [] },
            "create_version": true
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "data": design(), "version": version() })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/designs/des_1/versions/ver_1/restore"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "data": design(), "version": null })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let saved = client
        .designs()
        .update(
            "des_1",
            &UpdateDesign::new()
                .name("Launch v2")
                .document(json!({ "pages": [] }))
                .create_version(true),
        )
        .await
        .unwrap();
    assert_eq!(saved.id, "des_1");
    assert_eq!(saved.version.as_ref().unwrap().version, 2);

    let restored = client
        .designs()
        .restore_version("des_1", "ver_1")
        .await
        .unwrap();
    assert_eq!(restored.design.name, "Launch");
    assert!(restored.version.is_none());
}

#[tokio::test]
async fn versions_ask_for_documents_only_when_told() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/designs/des_1/versions"))
        .and(query_param("include", "document"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{ "id": "ver_1", "version": 2, "document": { "pages": [] },
                       "created_at": "2026-11-20T11:00:00.000Z" }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/designs/des_2/versions"))
        .and(query_param_is_missing("include"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [version()] })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let full = client.designs().versions("des_1", true).await.unwrap();
    assert_eq!(full[0].document.as_ref().unwrap()["pages"], json!([]));
    let bare = client.designs().versions("des_2", false).await.unwrap();
    assert!(bare[0].document.is_none());
}

#[tokio::test]
async fn duplicate_export_bulk_and_posts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/designs/des_1/duplicate"))
        .and(body_json(json!({ "name": "Copy" })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "data": design() })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/designs/des_2/duplicate"))
        .and(body_json(json!({})))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "data": design() })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/designs/des_1/export"))
        .and(body_json(
            json!({ "format": "mp4", "page_indexes": [0, 2], "duration_sec": 6 }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "format": "mp4",
                "files": [{
                    "media_id": "med_1",
                    "page_index": 0,
                    "mime_type": "video/mp4",
                    "preview_url": "https://api.fopost.com/v1/media/med_1/file"
                }]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/designs/des_1/bulk"))
        .and(body_json(json!({
            "rows": [{ "headline": "Hello", "title": "One" }],
            "name_column": "title"
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "data": [design()] })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/designs/des_1/posts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{
                "id": "post_1",
                "status": "published",
                "title": null,
                "text": "Out now",
                "schedule_at": null,
                "published_at": "2026-11-20T12:00:00.000Z",
                "platforms": ["instagram"],
                "engagement": 12,
                "views": 340
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .designs()
        .duplicate("des_1", Some("Copy"))
        .await
        .unwrap();
    client.designs().duplicate("des_2", None).await.unwrap();

    let export = client
        .designs()
        .export(
            "des_1",
            &ExportDesign::new(DesignExportFormat::Mp4)
                .pages([0, 2])
                .duration_sec(6),
        )
        .await
        .unwrap();
    assert_eq!(export.format, DesignExportFormat::Mp4);
    assert_eq!(export.files[0].media_id, "med_1");
    assert_eq!(export.files[0].page_index, Some(0));

    let row = BTreeMap::from([
        ("headline".to_string(), "Hello".to_string()),
        ("title".to_string(), "One".to_string()),
    ]);
    let created = client
        .designs()
        .bulk("des_1", &BulkCreateDesigns::new([row]).name_column("title"))
        .await
        .unwrap();
    assert_eq!(created.len(), 1);

    let posts = client.designs().posts("des_1").await.unwrap();
    assert_eq!(posts[0].status, PostStatus::Published);
    assert_eq!(posts[0].platforms, vec![Platform::Instagram]);
    assert_eq!(posts[0].views, Some(340));
}

#[tokio::test]
async fn text_to_slides_and_from_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/designs/text-to-slides"))
        .and(body_json(
            json!({ "workspace_id": "ws_1", "text": "Five tips", "max_slides": 5 }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "cover": { "title": "Five tips", "subtitle": "For creators" },
                "slides": [{ "heading": "One", "body": "Post daily" }],
                "cta": { "heading": "Follow", "body": "For more" },
                "brand": { "colors": ["#111111"], "fonts": { "heading": "Geist" },
                           "logo_media_id": null }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/designs/from-post"))
        .and(body_json(json!({
            "workspace_id": "ws_1",
            "post_id": "post_1",
            "style": "quote",
            "account_id": "acc_1",
            "carousel": false
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "data": design() })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let outline = client
        .designs()
        .text_to_slides(&TextToSlides::new("ws_1").text("Five tips").max_slides(5))
        .await
        .unwrap();
    assert_eq!(outline.cover.title, "Five tips");
    assert_eq!(outline.slides[0].body, "Post daily");
    assert_eq!(outline.brand.fonts.heading.as_deref(), Some("Geist"));

    let design = client
        .designs()
        .from_post(
            &DesignFromPost::new("ws_1")
                .post("post_1")
                .style(DesignPostStyle::Quote)
                .account("acc_1")
                .carousel(false),
        )
        .await
        .unwrap();
    assert_eq!(design.id, "des_1");
}

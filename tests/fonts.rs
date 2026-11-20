//! Fonts: the font list and upload come back bare, the
//! upload is a multipart form, and delete answers 204.

mod common;

use common::client;
use fopost::models::FontStyle;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn uploaded() -> serde_json::Value {
    json!({
        "id": "fnt_1",
        "family": "Brand Sans",
        "style": "italic",
        "weight": 700,
        "format": "woff2",
        "file_url": "https://api.fopost.com/v1/fonts/fnt_1/file",
        "created_at": "2026-11-20T10:00:00.000Z"
    })
}

#[tokio::test]
async fn fonts_are_listed_and_deleted() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/fonts"))
        .and(query_param("workspace_id", "ws_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "library": [{
                "slug": "geist",
                "family": "Geist",
                "category": "sans-serif",
                "scripts": ["latin"],
                "weights": [400, 700],
                "files": [{ "path": "/fonts/geist.woff2", "subset": "latin", "weight": 400,
                            "style": "normal", "unicode_range": "U+0000-00FF" }]
            }],
            "uploaded": [uploaded()]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/fonts/fnt_1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let fonts = client.fonts().list("ws_1").await.unwrap();
    assert_eq!(fonts.library[0].weights, vec![400, 700]);
    assert_eq!(fonts.library[0].files[0].style, Some(FontStyle::Normal));
    assert_eq!(fonts.uploaded[0].style, FontStyle::Italic);
    assert_eq!(fonts.uploaded[0].weight, 700);

    client.fonts().delete("fnt_1").await.unwrap();
}

#[cfg(feature = "multipart")]
#[tokio::test]
async fn a_font_uploads_as_a_multipart_form() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/fonts"))
        .respond_with(ResponseTemplate::new(201).set_body_json(uploaded()))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let font = client
        .fonts()
        .upload(
            "ws_1",
            fopost::models::MediaFile::new("brand.woff2", "font/woff2", b"wOF2data".to_vec()),
            &fopost::models::FontUpload::new()
                .family("Brand Sans")
                .weight(700)
                .style(FontStyle::Italic),
        )
        .await
        .unwrap();
    assert_eq!(font.id, "fnt_1");
    assert_eq!(font.format.as_deref(), Some("woff2"));

    let sent = &server.received_requests().await.unwrap()[0];
    let content_type = sent.headers.get("content-type").unwrap().to_str().unwrap();
    assert!(
        content_type.starts_with("multipart/form-data"),
        "{content_type}"
    );
    let body = String::from_utf8_lossy(&sent.body);
    assert!(body.contains("name=\"workspace_id\"\r\n\r\nws_1"), "{body}");
    assert!(
        body.contains("name=\"file\"; filename=\"brand.woff2\""),
        "{body}"
    );
    assert!(body.contains("wOF2data"), "{body}");
    assert!(body.contains("name=\"family\"\r\n\r\nBrand Sans"), "{body}");
    assert!(body.contains("name=\"weight\"\r\n\r\n700"), "{body}");
    assert!(body.contains("name=\"style\"\r\n\r\nitalic"), "{body}");
}

#[cfg(feature = "multipart")]
#[tokio::test]
async fn a_font_uploads_from_a_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/fonts"))
        .respond_with(ResponseTemplate::new(201).set_body_json(uploaded()))
        .expect(1)
        .mount(&server)
        .await;

    let dir = std::env::temp_dir().join(format!("fopost-font-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("brand.ttf");
    std::fs::write(&file, b"ttfbytes").unwrap();

    let client = client(&server).await;
    client
        .fonts()
        .upload(
            "ws_1",
            fopost::models::MediaFile::from_path(&file, "font/ttf"),
            &fopost::models::FontUpload::new(),
        )
        .await
        .unwrap();
    std::fs::remove_dir_all(&dir).ok();

    let sent = &server.received_requests().await.unwrap()[0];
    let body = String::from_utf8_lossy(&sent.body);
    assert!(body.contains("filename=\"brand.ttf\""), "{body}");
    assert!(body.contains("ttfbytes"), "{body}");
    assert!(!body.contains("name=\"family\""), "{body}");
}

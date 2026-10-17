//! Blog: one call per route group, pinning the path, the camelCase workspace
//! key on the query and in the body, and the envelope each endpoint uses.

mod common;

use common::client;
use fopost::models::{
    AddBlogKeyword, BlogArticleStatus, BlogKeywordStatus, BlogTargetKind, ListBlogArticles,
    ListBlogKeywords, PlanBlogKeywords, RequestBlogArticle, UpdateBlogArticle, UpdateBlogKeyword,
    UpdateBlogSettings,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn settings_fixture() -> serde_json::Value {
    json!({
        "siteUrl": "https://yourbrand.com",
        "topic": "running shoes",
        "audience": null,
        "language": "en",
        "locationCode": 2840,
        "cadencePerWeek": 2,
        "reviewHoldDays": 3,
        "autoPublish": false,
        "imageTier": "standard",
        "inBodyImages": 1,
        "minWords": 800,
        "maxWords": 1500,
        "brandVoiceId": null,
        "targetKind": "website",
        "targetAccountId": null,
        "targetPlatform": null,
        "websiteUrl": "https://yourbrand.com/api/fopost",
        "hasWebsiteSecret": true,
        "keywordDataAvailable": true,
        "updatedAt": "2026-10-01T10:00:00.000Z"
    })
}

fn keyword_fixture() -> serde_json::Value {
    json!({
        "id": "kw_1",
        "keyword": "trail running shoes",
        "searchVolume": 1200,
        "difficulty": 34,
        "cpc": 1.25,
        "intent": "commercial",
        "source": "provider",
        "status": "planned",
        "plannedFor": "2026-10-20",
        "createdAt": "2026-10-01T10:00:00.000Z"
    })
}

fn article_fixture(status: &str) -> serde_json::Value {
    json!({
        "id": "art_1",
        "title": "The best trail running shoes",
        "slug": "best-trail-running-shoes",
        "keyword": "trail running shoes",
        "excerpt": null,
        "metaTitle": null,
        "metaDescription": null,
        "featuredImage": { "mediaId": "m_1", "url": "https://cdn.example/f.jpg", "alt": "Shoes" },
        "tags": ["running"],
        "wordCount": 1200,
        "status": status,
        "origin": "user",
        "reviewUntil": null,
        "publishAt": null,
        "publishedAt": null,
        "publishedUrl": null,
        "error": null,
        "createdAt": "2026-10-01T10:00:00.000Z",
        "updatedAt": "2026-10-01T10:00:00.000Z"
    })
}

#[tokio::test]
async fn settings_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/blog/settings"))
        .and(query_param("workspaceId", "ws_1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "data": settings_fixture() })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/blog/settings"))
        .and(body_json(json!({
            "workspaceId": "ws_1",
            "websiteUrl": "https://yourbrand.com/api/fopost",
            "targetKind": "website",
            "brandVoiceId": null
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": settings_fixture(),
            "websiteSecret": "whsec_abc"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/settings/website-secret"))
        .and(body_json(json!({ "workspaceId": "ws_1" })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "data": { "websiteSecret": "whsec_new" } })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/settings/website-test"))
        .and(body_json(json!({ "workspaceId": "ws_1" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": { "ok": true, "status": 200, "error": null }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let blog = client.blog();

    let settings = blog.get_settings("ws_1").await.unwrap();
    assert_eq!(settings.target_kind, Some(BlogTargetKind::Website));
    assert_eq!(settings.cadence_per_week, 2);
    assert!(settings.has_website_secret);

    let updated = blog
        .update_settings(
            "ws_1",
            &UpdateBlogSettings::new()
                .website_url(Some("https://yourbrand.com/api/fopost"))
                .target_kind(Some(BlogTargetKind::Website))
                .brand_voice_id(None::<String>),
        )
        .await
        .unwrap();
    assert_eq!(updated.website_secret.as_deref(), Some("whsec_abc"));
    assert_eq!(updated.settings.language, "en");

    let rotated = blog.rotate_website_secret("ws_1").await.unwrap();
    assert_eq!(rotated.website_secret, "whsec_new");

    let test = blog.test_website("ws_1").await.unwrap();
    assert!(test.ok);
    assert_eq!(test.status, Some(200));
}

#[tokio::test]
async fn keywords_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/blog/keywords"))
        .and(query_param("workspaceId", "ws_1"))
        .and(query_param("status", "suggested,planned"))
        .and(query_param("limit", "50"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "data": [keyword_fixture()] })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/keywords/plan"))
        .and(body_json(
            json!({ "workspaceId": "ws_1", "seeds": ["running"] }),
        ))
        .respond_with(
            ResponseTemplate::new(202).set_body_json(json!({ "data": { "status": "queued" } })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/keywords"))
        .and(body_json(json!({
            "workspaceId": "ws_1",
            "keyword": "trail running shoes",
            "plannedFor": "2026-10-20"
        })))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(json!({ "data": keyword_fixture() })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/blog/keywords/kw_1"))
        .and(query_param("workspaceId", "ws_1"))
        .and(body_json(
            json!({ "status": "dismissed", "plannedFor": null }),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "data": keyword_fixture() })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/blog/keywords/kw_1"))
        .and(query_param("workspaceId", "ws_1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let blog = client.blog();

    let keywords = blog
        .list_keywords(
            "ws_1",
            &ListBlogKeywords::new()
                .status(BlogKeywordStatus::Suggested)
                .status(BlogKeywordStatus::Planned)
                .limit(50),
        )
        .await
        .unwrap();
    assert_eq!(keywords.len(), 1);
    assert_eq!(keywords[0].status, BlogKeywordStatus::Planned);
    assert_eq!(keywords[0].search_volume, Some(1200));

    blog.plan_keywords("ws_1", &PlanBlogKeywords::new().seeds(["running"]))
        .await
        .unwrap();

    let added = blog
        .add_keyword(
            "ws_1",
            &AddBlogKeyword::new("trail running shoes").planned_for("2026-10-20"),
        )
        .await
        .unwrap();
    assert_eq!(added.id, "kw_1");

    blog.update_keyword(
        "ws_1",
        "kw_1",
        &UpdateBlogKeyword::new()
            .status(BlogKeywordStatus::Dismissed)
            .planned_for(None::<String>),
    )
    .await
    .unwrap();

    blog.delete_keyword("ws_1", "kw_1").await.unwrap();
}

#[tokio::test]
async fn articles_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/blog/articles"))
        .and(query_param("workspaceId", "ws_1"))
        .and(query_param("status", "draft"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "data": [article_fixture("draft")] })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/articles"))
        .and(body_json(json!({
            "workspaceId": "ws_1",
            "keywordId": "kw_1",
            "instructions": "Compare three models"
        })))
        .respond_with(
            ResponseTemplate::new(202).set_body_json(json!({ "data": article_fixture("queued") })),
        )
        .expect(1)
        .mount(&server)
        .await;
    let mut full = article_fixture("draft");
    full["bodyMarkdown"] = json!("# Shoes");
    full["bodyHtml"] = json!("<h1>Shoes</h1>");
    full["outline"] = json!(["Intro", "Picks"]);
    full["images"] = json!([{ "mediaId": "m_2", "url": "https://cdn.example/1.jpg", "alt": "A", "position": 1 }]);
    Mock::given(method("GET"))
        .and(path("/v1/blog/articles/art_1"))
        .and(query_param("workspaceId", "ws_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": full })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/blog/articles/art_1"))
        .and(query_param("workspaceId", "ws_1"))
        .and(body_json(
            json!({ "title": "New title", "publishAt": null }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": full })))
        .expect(1)
        .mount(&server)
        .await;
    for action in ["regenerate", "publish", "reject"] {
        Mock::given(method("POST"))
            .and(path(format!("/v1/blog/articles/art_1/{action}")))
            .and(query_param("workspaceId", "ws_1"))
            .respond_with(
                ResponseTemplate::new(202)
                    .set_body_json(json!({ "data": article_fixture("queued") })),
            )
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("DELETE"))
        .and(path("/v1/blog/articles/art_1"))
        .and(query_param("workspaceId", "ws_1"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let blog = client.blog();

    let articles = blog
        .list_articles(
            "ws_1",
            &ListBlogArticles::new().status(BlogArticleStatus::Draft),
        )
        .await
        .unwrap();
    assert_eq!(articles[0].status, BlogArticleStatus::Draft);
    assert_eq!(articles[0].featured_image.as_ref().unwrap().media_id, "m_1");

    let requested = blog
        .request_article(
            "ws_1",
            &RequestBlogArticle::keyword_id("kw_1").instructions("Compare three models"),
        )
        .await
        .unwrap();
    assert_eq!(requested.status, BlogArticleStatus::Queued);

    let article = blog.get_article("ws_1", "art_1").await.unwrap();
    assert_eq!(article.body_markdown, "# Shoes");
    assert_eq!(article.outline, vec!["Intro", "Picks"]);
    assert_eq!(article.images[0].position, Some(1));
    assert_eq!(article.title, "The best trail running shoes");

    blog.update_article(
        "ws_1",
        "art_1",
        &UpdateBlogArticle::new()
            .title("New title")
            .publish_at(None::<String>),
    )
    .await
    .unwrap();

    blog.regenerate_article("ws_1", "art_1").await.unwrap();
    blog.publish_article("ws_1", "art_1").await.unwrap();
    blog.reject_article("ws_1", "art_1").await.unwrap();
    blog.delete_article("ws_1", "art_1").await.unwrap();
}

#[tokio::test]
async fn credit_and_state_errors_map_onto_api_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/articles"))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "insufficient_credits",
            "message": "Balance is empty",
            "upgrade_url": "https://app.example/billing"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/articles/art_1/publish"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "no_target",
            "message": "Pick where articles publish first"
        })))
        .mount(&server)
        .await;

    let client = client(&server).await;

    let err = client
        .blog()
        .request_article("ws_1", &RequestBlogArticle::title("Shoes"))
        .await
        .unwrap_err();
    match err {
        fopost::Error::Api(api) => {
            assert!(api.is_payment_required());
            assert_eq!(api.code.as_deref(), Some("insufficient_credits"));
            assert_eq!(api.upgrade_url(), Some("https://app.example/billing"));
        }
        other => panic!("expected an api error, got {other:?}"),
    }

    let err = client
        .blog()
        .publish_article("ws_1", "art_1")
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(409));
    assert_eq!(err.code(), Some("no_target"));
}

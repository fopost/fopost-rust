//! Blog: one call per route group, pinning the path, `workspace_id` on the
//! query, `workspaceId` in the body, and the envelope each endpoint uses.

mod common;

use common::client;
use fopost::models::{
    AddBlogKeyword, BlogArticleStatus, BlogGoal, BlogKeywordStatus, BlogPlanRunStatus,
    BlogTargetKind, ListBlogArticles, ListBlogKeywords, PlanBlogKeywords, RequestBlogArticle,
    SuggestBlogSeeds, UpdateBlogArticle, UpdateBlogKeyword, UpdateBlogSettings,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn settings_fixture() -> serde_json::Value {
    json!({
        "site_url": "https://yourbrand.com",
        "topic": "running shoes",
        "audience": null,
        "goal": "leads",
        "focus_topics": "trail running gear",
        "avoid_topics": "competitor brands",
        "language": "en",
        "location_code": 2840,
        "cadence_per_week": 2,
        "review_hold_days": 3,
        "image_tier": "standard",
        "in_body_images": 1,
        "min_words": 800,
        "max_words": 1500,
        "brand_voice_id": null,
        "target_kind": "website",
        "target_account_id": null,
        "target_platform": null,
        "website_url": "https://yourbrand.com/api/fopost",
        "has_website_secret": true,
        "plan_run": {
            "status": "done",
            "error": null,
            "added": 12,
            "finished_at": "2026-10-01T10:05:00.000Z"
        },
        "keyword_data_available": true,
        "updated_at": "2026-10-01T10:00:00.000Z"
    })
}

fn keyword_fixture() -> serde_json::Value {
    json!({
        "id": "kw_1",
        "keyword": "trail running shoes",
        "search_volume": 1200,
        "difficulty": 34,
        "cpc": 1.25,
        "intent": "commercial",
        "source": "provider",
        "status": "planned",
        "planned_for": "2026-10-20",
        "fit_reason": "Buyers compare shoes before they sign up.",
        "created_at": "2026-10-01T10:00:00.000Z"
    })
}

fn article_fixture(status: &str) -> serde_json::Value {
    json!({
        "id": "art_1",
        "title": "The best trail running shoes",
        "slug": "best-trail-running-shoes",
        "keyword": "trail running shoes",
        "excerpt": null,
        "meta_title": null,
        "meta_description": null,
        "featured_image": { "media_id": "m_1", "url": "https://cdn.example/f.jpg", "alt": "Shoes" },
        "tags": ["running"],
        "word_count": 1200,
        "status": status,
        "origin": "user",
        "review_until": null,
        "publish_at": null,
        "published_at": null,
        "published_url": null,
        "error": null,
        "created_at": "2026-10-01T10:00:00.000Z",
        "updated_at": "2026-10-01T10:00:00.000Z"
    })
}

#[tokio::test]
async fn settings_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/blog/settings"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param_is_missing("workspaceId"))
        .respond_with(ResponseTemplate::new(200).set_body_json(settings_fixture()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/blog/settings"))
        .and(body_json(json!({
            "workspaceId": "ws_1",
            "websiteUrl": "https://yourbrand.com/api/fopost",
            "targetKind": "website",
            "brandVoiceId": null,
            "goal": "leads",
            "focusTopics": "trail running gear",
            "avoidTopics": null
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut body = settings_fixture();
            body["website_secret"] = json!("whsec_abc");
            body
        }))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/settings/website-secret"))
        .and(body_json(json!({ "workspaceId": "ws_1" })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "website_secret": "whsec_new" })),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/settings/website-test"))
        .and(body_json(json!({ "workspaceId": "ws_1" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "http_status": 200 })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let blog = client.blog();

    let settings = blog.get_settings("ws_1").await.unwrap();
    assert_eq!(settings.target_kind, Some(BlogTargetKind::Website));
    assert_eq!(settings.cadence_per_week, 2);
    assert!(settings.has_website_secret);
    assert_eq!(settings.goal, Some(BlogGoal::Leads));
    assert_eq!(settings.focus_topics.as_deref(), Some("trail running gear"));
    assert_eq!(settings.avoid_topics.as_deref(), Some("competitor brands"));
    let run = settings.plan_run.as_ref().unwrap();
    assert_eq!(run.status, BlogPlanRunStatus::Done);
    assert_eq!(run.added, Some(12));

    let updated = blog
        .update_settings(
            "ws_1",
            &UpdateBlogSettings::new()
                .website_url(Some("https://yourbrand.com/api/fopost"))
                .target_kind(Some(BlogTargetKind::Website))
                .brand_voice_id(None::<String>)
                .goal(Some(BlogGoal::Leads))
                .focus_topics(Some("trail running gear"))
                .avoid_topics(None::<String>),
        )
        .await
        .unwrap();
    assert_eq!(updated.website_secret.as_deref(), Some("whsec_abc"));
    assert_eq!(updated.settings.language, "en");

    let rotated = blog.rotate_website_secret("ws_1").await.unwrap();
    assert_eq!(rotated.website_secret, "whsec_new");

    let test = blog.test_website("ws_1").await.unwrap();
    assert_eq!(test.http_status, Some(200));
}

#[tokio::test]
async fn keywords_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/blog/keywords"))
        .and(query_param("workspace_id", "ws_1"))
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
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({ "status": "queued" })))
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
        .respond_with(ResponseTemplate::new(201).set_body_json(keyword_fixture()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/blog/keywords/kw_1"))
        .and(query_param("workspace_id", "ws_1"))
        .and(body_json(
            json!({ "status": "dismissed", "plannedFor": null }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(keyword_fixture()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/blog/keywords/kw_1"))
        .and(query_param("workspace_id", "ws_1"))
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
    assert_eq!(
        keywords[0].fit_reason.as_deref(),
        Some("Buyers compare shoes before they sign up.")
    );

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
async fn suggest_seeds_sends_seeds_and_prompt() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/keywords/suggest"))
        .and(body_json(json!({
            "workspaceId": "ws_1",
            "seeds": ["trail running"],
            "prompt": "Beginners, not racers"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "seeds": ["trail running shoes", "waterproof trail shoes"]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/keywords/suggest"))
        .and(body_json(json!({ "workspaceId": "ws_2" })))
        .respond_with(ResponseTemplate::new(402).set_body_json(json!({
            "error": "insufficient_credits",
            "message": "Balance is empty"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;

    let seeds = client
        .blog()
        .suggest_seeds(
            "ws_1",
            &SuggestBlogSeeds::new()
                .seeds(["trail running"])
                .prompt("Beginners, not racers"),
        )
        .await
        .unwrap();
    assert_eq!(seeds, vec!["trail running shoes", "waterproof trail shoes"]);

    let err = client
        .blog()
        .suggest_seeds("ws_2", &SuggestBlogSeeds::new())
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(402));
}

#[tokio::test]
async fn submit_article_moves_it_to_review() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/articles/art_1/submit"))
        .and(query_param("workspace_id", "ws_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(article_fixture("in_review")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/blog/articles/art_2/submit"))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({
            "error": "invalid_state",
            "message": "Only a draft can be submitted for approval"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;

    let article = client.blog().submit_article("ws_1", "art_1").await.unwrap();
    assert_eq!(article.status, BlogArticleStatus::InReview);

    let err = client
        .blog()
        .submit_article("ws_1", "art_2")
        .await
        .unwrap_err();
    assert_eq!(err.status(), Some(409));
    assert_eq!(err.code(), Some("invalid_state"));
}

#[tokio::test]
async fn articles_round_trip() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/blog/articles"))
        .and(query_param("workspace_id", "ws_1"))
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
        .respond_with(ResponseTemplate::new(202).set_body_json(article_fixture("queued")))
        .expect(1)
        .mount(&server)
        .await;
    let mut full = article_fixture("draft");
    full["body_markdown"] = json!("# Shoes");
    full["body_html"] = json!("<h1>Shoes</h1>");
    full["outline"] = json!(["Intro", "Picks"]);
    full["images"] = json!([{ "media_id": "m_2", "url": "https://cdn.example/1.jpg", "alt": "A", "position": 1 }]);
    Mock::given(method("GET"))
        .and(path("/v1/blog/articles/art_1"))
        .and(query_param("workspace_id", "ws_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&full))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v1/blog/articles/art_1"))
        .and(query_param("workspace_id", "ws_1"))
        .and(body_json(
            json!({ "title": "New title", "publishAt": null }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&full))
        .expect(1)
        .mount(&server)
        .await;
    for action in ["regenerate", "publish", "reject"] {
        Mock::given(method("POST"))
            .and(path(format!("/v1/blog/articles/art_1/{action}")))
            .and(query_param("workspace_id", "ws_1"))
            .respond_with(ResponseTemplate::new(202).set_body_json(article_fixture("queued")))
            .expect(1)
            .mount(&server)
            .await;
    }
    Mock::given(method("DELETE"))
        .and(path("/v1/blog/articles/art_1"))
        .and(query_param("workspace_id", "ws_1"))
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

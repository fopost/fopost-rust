//! Articles and products on a connected site: the path each call uses, the
//! filters it sends, and the property that an update never forks the article
//! into a second post.

mod common;

use common::client;
use fopost::models::{ListArticles, ListProducts, UpdateArticle, UpdateProduct};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn article_fixture() -> serde_json::Value {
    serde_json::json!({
        "id": "99",
        "blog_id": "11",
        "title": "Spring drop",
        "body_html": "<p>Hello</p>",
        "excerpt": "A short summary",
        "status": "published",
        "author_name": "Store Owner",
        "tags": ["news"],
        "image_url": null,
        "url": "https://demo.myshopify.com/blogs/article/spring-drop",
        "published_at": "2026-09-01T10:00:00.000Z",
        "updated_at": null
    })
}

#[tokio::test]
async fn list_blogs_reads_every_blog_on_the_site() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/blogs"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "11", "title": "News", "handle": "news", "url": null}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let blogs = client.blogs().list_blogs("acc_1").await.unwrap();

    assert_eq!(blogs.len(), 1);
    assert_eq!(blogs[0].id, "11");
    assert_eq!(blogs[0].title, "News");
}

#[tokio::test]
async fn list_articles_sends_the_filters() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/blogs/11/articles"))
        .and(query_param("limit", "5"))
        .and(query_param("status", "draft"))
        .and(query_param("q", "spring"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": [article_fixture()]})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let filters = ListArticles::new().limit(5).status("draft").q("spring");
    let articles = client
        .blogs()
        .list_articles("acc_1", "11", &filters)
        .await
        .unwrap();

    assert_eq!(articles[0].id, "99");
    assert_eq!(articles[0].tags, vec!["news".to_string()]);
    assert_eq!(articles[0].author_name.as_deref(), Some("Store Owner"));
}

/// The article id is in the path and only what the caller set is sent, which is
/// what stops an edit from creating a second post on the site.
#[tokio::test]
async fn update_article_changes_it_in_place() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/accounts/acc_1/blogs/11/articles/99"))
        .and(body_json(
            serde_json::json!({"title": "Spring drop, restocked"}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": article_fixture()})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let changes = UpdateArticle::new().title("Spring drop, restocked");
    let article = client
        .blogs()
        .update_article("acc_1", "11", "99", &changes)
        .await
        .unwrap();

    assert_eq!(article.id, "99");
}

#[tokio::test]
async fn delete_article_hits_the_article_route() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/accounts/acc_1/blogs/11/articles/99"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    client
        .blogs()
        .delete_article("acc_1", "11", "99")
        .await
        .unwrap();
}

#[tokio::test]
async fn update_product_sends_only_what_changed() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/v1/accounts/acc_1/products/7"))
        .and(body_json(
            serde_json::json!({"title": "Mug XL", "product_type": "Drinkware"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "id": "7", "title": "Mug XL", "handle": "mug", "status": "draft",
                "description": null, "vendor": null, "product_type": "Drinkware",
                "tags": [], "image_url": null, "url": null, "price": "12.00",
                "currency": "USD", "updated_at": null
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let changes = UpdateProduct::new()
        .title("Mug XL")
        .product_type("Drinkware");
    let product = client
        .blogs()
        .update_product("acc_1", "7", &changes)
        .await
        .unwrap();

    assert_eq!(product.price.as_deref(), Some("12.00"));
}

#[tokio::test]
async fn list_products_sends_the_status_filter() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/products"))
        .and(query_param("status", "active"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": []})))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let filters = ListProducts::new().status("active");
    let products = client
        .blogs()
        .list_products("acc_1", &filters)
        .await
        .unwrap();

    assert!(products.is_empty());
}

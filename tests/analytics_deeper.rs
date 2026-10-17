//! The deeper analytics endpoints: the age bands, the cadence bands, a
//! per-post timeline addressed by permalink, the changes cursor, the
//! on-demand refresh, and the posts that never went out through FoPost.

mod common;

use common::client;
use fopost::models::{AnalyticsQuery, MetricChangesQuery, NativePostsQuery};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn decay_reads_the_bands_and_the_half_life() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/analytics/decay"))
        .and(query_param("days", "30"))
        .and(query_param("accountId", "acc_1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {
                "days": 30,
                "postsMeasured": 2,
                "halfLifeBucket": "1h_3h",
                "bands": [
                    {"bucket": "under_1h", "label": "First hour", "posts": 2,
                     "avgEngagements": 25.0, "avgImpressions": 300.0, "shareOfFinal": 0.3},
                    {"bucket": "6h_12h", "label": "6-12 hours", "posts": 0,
                     "avgEngagements": 0.0, "avgImpressions": 0.0, "shareOfFinal": null}
                ]
            }})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let decay = client
        .analytics()
        .decay(&AnalyticsQuery::new().days(30).account("acc_1"))
        .await
        .unwrap();

    assert_eq!(decay.half_life_bucket.as_deref(), Some("1h_3h"));
    assert_eq!(decay.posts_measured, 2);
    assert_eq!(decay.bands[0].share_of_final, Some(0.3));
    // A band nothing was measured in reports no share rather than zero
    assert_eq!(decay.bands[1].share_of_final, None);
}

#[tokio::test]
async fn frequency_reads_the_weeks_and_the_best_cadence() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/analytics/frequency"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {
                "days": 90,
                "weeks": [{"weekStart": "2026-03-02", "posts": 2, "engagements": 240,
                           "avgEngagementsPerPost": 120.0}],
                "bands": [{"band": "under_3", "label": "1-2 a week", "weeks": 1, "posts": 2,
                           "avgPostsPerWeek": 2.0, "avgEngagementsPerPost": 120.0,
                           "engagementRate": 0.12}],
                "best": {"band": "under_3", "label": "1-2 a week", "avgEngagementsPerPost": 120.0}
            }})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let frequency = client
        .analytics()
        .frequency(&AnalyticsQuery::new().days(90))
        .await
        .unwrap();

    assert_eq!(frequency.weeks[0].week_start, "2026-03-02");
    assert_eq!(frequency.bands[0].engagement_rate, Some(0.12));
    assert_eq!(frequency.best.as_ref().unwrap().label, "1-2 a week");
}

#[tokio::test]
async fn a_timeline_can_be_addressed_by_permalink() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v1/analytics/posts/https%3A%2F%2Fx.com%2Facme%2Fstatus%2F1/timeline",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {
                "postId": null,
                "deliveries": [{
                    "accountId": "acc_1", "platform": "twitter", "username": "acme",
                    "externalPostId": "1", "postedAt": "2026-03-02T00:00:00.000Z",
                    "points": [{
                        "at": "2026-03-02T00:30:00.000Z", "ageMinutes": 30,
                        "engagements": 40, "impressions": 400, "reach": null, "likes": 30,
                        "comments": null, "shares": null, "videoViews": null,
                        "delta": {"impressions": 400, "reach": 0, "engagements": 40,
                                  "likes": 30, "comments": 0, "shares": 0}
                    }]
                }]
            }})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let timeline = client
        .analytics()
        .timeline("https://x.com/acme/status/1")
        .await
        .unwrap();

    // A post made on the network has no FoPost id
    assert_eq!(timeline.post_id, None);
    let point = &timeline.deliveries[0].points[0];
    assert_eq!(point.age_minutes, Some(30));
    assert_eq!(point.delta.engagements, 40);
}

#[tokio::test]
async fn changes_carries_the_cursor() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/analytics/changes"))
        .and(query_param("since", "2026-03-02T00:00:00Z"))
        .and(query_param("limit", "100"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {
                "since": "2026-03-02T00:00:00.000Z",
                "cursor": "2026-03-02T06:00:00.000Z",
                "hasMore": true,
                "changes": [{"accountId": "acc_1", "platform": "twitter", "externalPostId": "1",
                             "postId": "post_1", "postedAt": "2026-03-02T00:00:00.000Z",
                             "fetchedAt": "2026-03-02T06:00:00.000Z", "impressions": 900,
                             "reach": null, "engagements": 90, "likes": 70, "comments": 10,
                             "shares": 10}]
            }})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .analytics()
        .changes(
            &MetricChangesQuery::new()
                .since("2026-03-02T00:00:00Z")
                .limit(100),
        )
        .await
        .unwrap();

    assert!(page.has_more);
    assert_eq!(page.cursor.as_deref(), Some("2026-03-02T06:00:00.000Z"));
    assert_eq!(page.changes[0].post_id.as_deref(), Some("post_1"));
}

#[tokio::test]
async fn collect_post_reports_each_delivery() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/posts/post_1/analytics/collect"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {
                "collected": 1,
                "deliveries": [{"accountId": "acc_1", "platform": "twitter", "externalPostId": "1",
                                "collected": true, "fetchedAt": "2026-03-02T00:30:00.000Z",
                                "message": null}]
            }})),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let result = client.analytics().collect_post("post_1").await.unwrap();

    assert_eq!(result.collected, 1);
    assert!(result.deliveries[0].collected);
}

#[tokio::test]
async fn native_posts_keeps_the_meta_envelope() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc_1/native-posts"))
        .and(query_param("page", "1"))
        .and(query_param("per_page", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{
                "externalPostId": "1", "text": "Posted by hand",
                "permalink": "https://x.com/acme/status/1", "thumbnailUrl": null,
                "mediaType": null, "postedAt": "2026-03-02T00:00:00.000Z",
                "fetchedAt": "2026-03-02T06:00:00.000Z",
                "metrics": {"impressions": 900, "reach": null, "engagements": 90,
                            "likes": 70, "comments": 10, "shares": 10, "videoViews": null}
            }],
            "meta": {"page": 1, "perPage": 20, "total": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let page = client
        .analytics()
        .native_posts("acc_1", &NativePostsQuery::new().page(1).per_page(20))
        .await
        .unwrap();

    assert_eq!(page.len(), 1);
    assert_eq!(page.meta.total, 1);
    assert_eq!(
        page.items[0].permalink.as_deref(),
        Some("https://x.com/acme/status/1")
    );
    assert_eq!(page.items[0].metrics.engagements, Some(90));
}

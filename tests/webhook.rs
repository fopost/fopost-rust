//! The delivery verifier: the timestamped signature wins, the body-only
//! signature is a fallback, and a stale or tampered delivery is refused.

use fopost::models::{BlogArticlePublished, WebhookEvent};
use fopost::{
    sign_webhook, sign_webhook_at, verify_webhook_at, WebhookError, WebhookSignature,
    WEBHOOK_TOLERANCE,
};
use reqwest::header::{HeaderMap, HeaderValue};

const SECRET: &str = "whsec_test";
const NOW: u64 = 1_760_000_000;

fn body() -> Vec<u8> {
    serde_json::json!({
        "event": "article.published",
        "data": {
            "id": "art_1",
            "title": "The best trail running shoes",
            "slug": "best-trail-running-shoes",
            "bodyMarkdown": "# Shoes",
            "bodyHtml": "<h1>Shoes</h1>",
            "featuredImage": null,
            "images": [],
            "tags": ["running"],
            "keyword": "trail running shoes",
            "wordCount": 1200,
            "publishedAt": "2026-10-09T10:00:00.000Z",
            "publishedUrl": "https://yourbrand.com/blog/best-trail-running-shoes"
        },
        "timestamp": "2026-10-09T10:00:00.000Z"
    })
    .to_string()
    .into_bytes()
}

fn timestamped(body: &[u8], at: u64) -> WebhookSignature {
    WebhookSignature {
        signature: Some(sign_webhook(body, SECRET)),
        signature_256: Some(sign_webhook_at(body, SECRET, at)),
        timestamp: Some(at.to_string()),
    }
}

#[test]
fn a_timestamped_signature_passes_and_the_payload_decodes() {
    let body = body();
    let envelope = verify_webhook_at(&body, &timestamped(&body, NOW - 10), SECRET, NOW).unwrap();

    assert_eq!(envelope.event, WebhookEvent::ArticlePublished);
    assert_eq!(envelope.timestamp, "2026-10-09T10:00:00.000Z");
    let article: BlogArticlePublished = envelope.data_as().unwrap();
    assert_eq!(article.slug, "best-trail-running-shoes");
    assert_eq!(
        article.published_url.as_deref(),
        Some("https://yourbrand.com/blog/best-trail-running-shoes")
    );
}

#[test]
fn a_tampered_body_fails() {
    let body = body();
    let signature = timestamped(&body, NOW);
    let mut tampered = body.clone();
    tampered[20] ^= 1;

    assert_eq!(
        verify_webhook_at(&tampered, &signature, SECRET, NOW).unwrap_err(),
        WebhookError::InvalidSignature
    );
    assert_eq!(
        verify_webhook_at(&body, &signature, "whsec_other", NOW).unwrap_err(),
        WebhookError::InvalidSignature
    );
}

#[test]
fn a_stale_timestamp_fails_even_with_a_valid_signature() {
    let body = body();
    let sent = NOW - WEBHOOK_TOLERANCE.as_secs() - 1;

    assert_eq!(
        verify_webhook_at(&body, &timestamped(&body, sent), SECRET, NOW).unwrap_err(),
        WebhookError::StaleTimestamp
    );
    assert!(verify_webhook_at(&body, &timestamped(&body, NOW - 299), SECRET, NOW).is_ok());
}

#[test]
fn the_body_only_signature_passes_when_no_timestamp_header_is_present() {
    let body = body();
    let legacy = WebhookSignature {
        signature: Some(sign_webhook(&body, SECRET)),
        signature_256: None,
        timestamp: None,
    };
    assert!(verify_webhook_at(&body, &legacy, SECRET, NOW).is_ok());

    // With a timestamp present the body-only signature is no longer enough.
    let downgraded = WebhookSignature {
        timestamp: Some(NOW.to_string()),
        ..legacy
    };
    assert_eq!(
        verify_webhook_at(&body, &downgraded, SECRET, NOW).unwrap_err(),
        WebhookError::MissingSignature
    );
    assert_eq!(
        verify_webhook_at(&body, &WebhookSignature::default(), SECRET, NOW).unwrap_err(),
        WebhookError::MissingSignature
    );
}

#[test]
fn signatures_are_read_from_a_header_map() {
    let body = body();
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-FoPost-Signature",
        HeaderValue::from_str(&sign_webhook(&body, SECRET)).unwrap(),
    );
    headers.insert(
        "X-FoPost-Signature-256",
        HeaderValue::from_str(&sign_webhook_at(&body, SECRET, NOW)).unwrap(),
    );
    headers.insert(
        "X-FoPost-Timestamp",
        HeaderValue::from_str(&NOW.to_string()).unwrap(),
    );

    let signature = WebhookSignature::from_headers(&headers);
    assert_eq!(signature, timestamped(&body, NOW));
    assert!(verify_webhook_at(&body, &signature, SECRET, NOW).is_ok());
}

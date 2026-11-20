//! Video edits: the ops list is enveloped, a single job comes back bare, and
//! operations go out as `{op, params}`.

mod common;

use common::client;
use fopost::models::{
    CreateVideoEdit, ListVideoEdits, VideoEditOp, VideoEditOperation, VideoEditStatus,
};
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn job(status: &str) -> serde_json::Value {
    json!({
        "id": "ve_1",
        "status": status,
        "progress": 0.5,
        "source_media_id": "med_1",
        "operations": [{ "op": "trim", "params": { "start": 1, "end": 4 } }],
        "result_media_id": null,
        "result_media": null,
        "extra_outputs": [{ "media_id": "med_9", "label": "Subtitles", "mime": "text/vtt",
                            "preview_url": null }],
        "result": null,
        "error": null,
        "created_at": "2026-11-20T10:00:00.000Z",
        "updated_at": "2026-11-20T10:00:05.000Z",
        "completed_at": null
    })
}

#[tokio::test]
async fn ops_are_listed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/video-edits/ops"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "name": "trim", "available": true, "uses_ai": false },
                { "name": "subtitles", "available": false, "uses_ai": true },
                { "name": "brand_new_op", "available": true, "uses_ai": false }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let ops = client.video_edits().ops().await.unwrap();
    assert_eq!(ops[0].name, VideoEditOp::Trim);
    assert!(ops[1].uses_ai && !ops[1].available);
    assert_eq!(ops[2].name, VideoEditOp::Other("brand_new_op".into()));
}

#[tokio::test]
async fn an_edit_is_created_listed_and_read() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/video-edits"))
        .and(body_json(json!({
            "workspace_id": "ws_1",
            "media_id": "med_1",
            "operations": [
                { "op": "trim", "params": { "start": 1, "end": 4 } },
                { "op": "denoise" }
            ]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(job("queued")))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/video-edits"))
        .and(query_param("workspace_id", "ws_1"))
        .and(query_param("media_id", "med_1"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [job("running")] })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/video-edits/ve_1"))
        .and(query_param("workspace_id", "ws_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut done = job("succeeded");
            done["result_media_id"] = json!("med_2");
            done["result_media"] = json!({ "id": "med_2", "preview_url": null,
                                           "mime_type": "video/mp4" });
            done
        }))
        .expect(1)
        .mount(&server)
        .await;

    let client = client(&server).await;
    let created = client
        .video_edits()
        .create(
            &CreateVideoEdit::new(
                "med_1",
                [
                    VideoEditOperation::new(VideoEditOp::Trim)
                        .params(json!({ "start": 1, "end": 4 })),
                    VideoEditOperation::new(VideoEditOp::Denoise),
                ],
            )
            .workspace("ws_1"),
        )
        .await
        .unwrap();
    assert_eq!(created.status, VideoEditStatus::Queued);
    assert_eq!(created.operations[0].params["end"], json!(4));
    assert_eq!(created.extra_outputs[0].label, "Subtitles");

    let jobs = client
        .video_edits()
        .list(&ListVideoEdits {
            workspace_id: Some("ws_1"),
            media_id: Some("med_1"),
            limit: Some(5),
        })
        .await
        .unwrap();
    assert_eq!(jobs[0].status, VideoEditStatus::Running);
    assert_eq!(jobs[0].progress, 0.5);

    let done = client
        .video_edits()
        .get("ve_1", Some("ws_1"))
        .await
        .unwrap();
    assert_eq!(done.status, VideoEditStatus::Succeeded);
    assert_eq!(
        done.result_media.unwrap().mime_type.as_deref(),
        Some("video/mp4")
    );
}

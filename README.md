# fopost

[![crates.io](https://img.shields.io/crates/v/fopost.svg)](https://crates.io/crates/fopost)
[![docs.rs](https://img.shields.io/docsrs/fopost)](https://docs.rs/fopost)
[![license](https://img.shields.io/crates/l/fopost.svg)](https://github.com/fopost/fopost-rust/blob/main/LICENSE)
[![ci](https://img.shields.io/github/actions/workflow/status/fopost/fopost-rust/ci.yml?label=ci)](https://github.com/fopost/fopost-rust/actions/workflows/ci.yml)

Official Rust SDK for the [FoPost](https://fopost.com) API. Schedule and publish to +30 social
platforms from your code.

```toml
[dependencies]
fopost = "0.3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Async, built on `reqwest`. Requires Rust 1.88 or newer.

> **0.x release.** The public API is still settling and minor versions may contain breaking
> changes. Pin an exact version if that matters to you.

## Quick start

```rust,no_run
use fopost::{models::CreatePost, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?; // reads FOPOST_API_KEY

    // Everything is scoped to a workspace.
    let workspaces = client.workspaces().list().await?;
    let workspace = &workspaces[0];

    let accounts = client.accounts().list(Some(&workspace.id)).await?;
    let ids: Vec<_> = accounts.iter().map(|a| a.id.clone()).collect();

    // Create a post, then publish it immediately.
    let post = client
        .posts()
        .create(&CreatePost::text(&workspace.id, "Hello from Rust").accounts(ids.clone()))
        .await?;
    client.posts().publish(&post.id, &Default::default()).await?;

    // Or schedule it for later.
    client
        .posts()
        .create(
            &CreatePost::text(&workspace.id, "Scheduled with the SDK")
                .accounts(ids)
                .schedule_at("2026-09-01T10:00:00Z"),
        )
        .await?;

    Ok(())
}
```

## Configuration

```rust,no_run
use std::time::Duration;

let client = fopost::Client::builder()
    .api_key(std::env::var("FOPOST_API_KEY")?)
    .base_url("https://api.fopost.com/v1") // override for another deployment
    .timeout(Duration::from_secs(30))
    .max_retries(3)                        // 1 disables retrying
    .build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

| Env var | Used for |
| --- | --- |
| `FOPOST_API_KEY` | The API key `Client::from_env()` reads |

Keys are created in the dashboard under **Settings → API Keys**. A key carries only the scopes
granted at creation and may be bound to a single workspace, in which case naming any other
workspace answers `403`.

## Resources

| Namespace | Methods |
| --- | --- |
| `posts()` | `list`, `list_all`, `get`, `create`, `update`, `delete`, `duplicate`, `publish`, `retry`, `cancel`, `preflight`, `deliveries`, `publish_runs`, `analytics`, `bulk`, `validate_import`, `commit_import`, `rollback_import` |
| `accounts()` | `list`, `list_with`, `get`, `create`, `rename`, `move_to`, `delete`, `toggle_primary`, `validate`, `health`, `health_summary`, `refresh_token`, `analytics`, `communities`, `sync_communities`, `search_communities`, `add_community`, `remove_community`, `create_telegram_connect_code`, `telegram_connect_status`, `telegram_bot_commands`, `set_telegram_bot_commands`, `delete_telegram_bot_commands`, `slack_channels`, `slack_members`, `slack_identity`, `update_slack_identity`, `ice_breakers`, `set_ice_breakers`, `delete_ice_breakers`, `persistent_menu`, `set_persistent_menu`, `delete_persistent_menu`, `greeting`, `set_greeting`, `delete_greeting`, `webhook_subscription`, `resubscribe_webhook`, `discord_channels`, `switch_discord_channel`, `discord_identity`, `update_discord_identity`, `discord_pins`, `delete_discord_message`, `pin_discord_message`, `unpin_discord_message`, `crosspost_discord_message`, `create_discord_thread`, `send_discord_dm`, `discord_events`, `discord_event`, `create_discord_event`, `update_discord_event`, `delete_discord_event`, `discord_members`, `discord_member`, `discord_roles`, `create_discord_role`, `update_discord_role`, `delete_discord_role`, `add_discord_member_role`, `remove_discord_member_role` |
| `account_groups()` | `list`, `get`, `create`, `update`, `delete`, `set_members` |
| `workspaces()` | `list`, `get`, `create`, `update`, `delete`, `analytics` |
| `labels()` | `list`, `get`, `create`, `update`, `delete` |
| `webhooks()` | `list`, `create`, `update`, `delete`, `test` |
| `automations()` | `list`, `get`, `create`, `update`, `delete`, `toggle`, `runs`, `get_run`, `stats`, `trigger` |
| `analytics()` | `overview`, `time_series`, `top_posts`, `labels`, `posts_table`, `posting_streak`, `demographics`, `collect` |
| `media()` | `list`, `upload`, `presign`, `complete`, `upload_direct`, `delete` |
| `contacts()` | `list`, `get`, `create`, `update`, `delete`, `conversations`, `import`, `list_fields`, `create_field`, `update_field`, `delete_field`, `conversation_analytics` |
| `broadcasts()` | `list`, `get`, `create`, `update`, `delete`, `send`, `cancel`, `recipients` |
| `sequences()` | `list`, `get`, `create`, `update`, `delete`, `enroll`, `unenroll`, `enrollments` |
| `inbox()` | `list`, `threads`, `conversations`, `unread_count`, `accounts`, `platforms`, `mark_thread_read`, `refresh`, `update`, `edit_comment`, `reply`, `reply_with`, `hide`, `unhide`, `delete`, `like`, `unlike`, `pin`, `unpin`, `react`, `start_conversation`, `set_typing`, `handover`, `approvals`, `approve_reply`, `reject_reply` |
| `ads()` | `list`, `external`, `boostable`, `connections`, `sources`, `authorize_meta`, `delete_connection`, `boost`, `create`, `refresh`, `set_status`, `delete`, `audiences`, `create_audience`, `search_targeting`, `lead_forms`, `create_lead_form`, `leads`, `tree`, `create_campaign`, `campaign`, `update_campaign`, `delete_campaign`, `duplicate_campaign`, `create_ad_set`, `ad_set`, `update_ad_set`, `delete_ad_set`, `duplicate_ad_set`, `create_network_ad`, `network_ad`, `update_network_ad`, `delete_network_ad`, `duplicate_network_ad`, `set_statuses`, `creatives`, `create_creative`, `creative`, `delete_creative`, `audience`, `update_audience`, `delete_audience`, `add_audience_users`, `estimate_reach`, `insights`, `ad_insights`, `lead_form`, `archive_lead_form`, `leads_feed`, `lead_pages`, `subscribe_lead_page`, `unsubscribe_lead_page`, `goals`, `catalogs`, `create_catalog`, `catalog`, `update_catalog`, `delete_catalog`, `catalog_products`, `write_catalog_products`, `product_feeds`, `create_product_feed`, `delete_product_feed`, `feed_uploads`, `start_feed_upload`, `product_sets`, `create_product_set`, `update_product_set`, `delete_product_set`, `reach_frequency`, `create_reach_frequency`, `reach_frequency_prediction`, `reserve_reach_frequency`, `cancel_reach_frequency`, `library`, `partnership_creators`, `request_partnership`, `revoke_partnership`, `account_activity`, `labels`, `create_label`, `update_label`, `delete_label`, `apply_label`, `studies`, `create_study`, `study`, `delete_study`, `ios_campaign_limits`, `high_demand_periods`, `create_high_demand_period`, `delete_high_demand_period`, `value_rule_sets`, `create_value_rule_set`, `delete_value_rule_set` |
| `knowledge()` | `list`, `create`, `update`, `delete`, `sync`, `search` |
| `validate()` | `post`, `length`, `media` |
| `activity()` | `list` |

`account_groups()` needs the `accounts` scope. `inbox()`, `knowledge()`, `contacts()`, `broadcasts()` and `sequences()` need the `inbox` scope (and `publish` on `broadcasts().send`/`cancel` and `sequences().enroll`/`unenroll`, which reach a platform), except `contacts().conversation_analytics`, which needs `analytics` and `ads()` the `ads` scope; `boost`, `create`, `set_status`,
`delete`, `set_statuses` and every create, update, delete and duplicate on campaigns, ad sets and network ads
on `ads()` spend money and also need `publish`. On `inbox()`, `edit_comment`, `like`, `unlike`, `pin`,
`unpin`, `react`, `start_conversation`, `set_typing`, `handover`, a `reply_with` carrying media or quick replies,
and deleting our own reply also need `publish`. A boost or ad starts paused unless you pass
`.paused(false)`. `validate()` needs the `posts` scope and stores nothing.

That is every endpoint the API documents. Anything not yet wrapped is reachable through
`client.request(method, path, query, body)`, which gets the same auth, retries, and error handling.

## Error handling

Every call returns `Result<T, fopost::Error>`. A non-2xx response becomes `Error::Api`, carrying the
status, the API's machine-readable code, and the whole body.

```rust,no_run
use fopost::Error;

# async fn run(client: fopost::Client) {
match client.posts().publish("9b2f6c1e-…", &Default::default()).await {
    Ok(outcome) => println!("{} deliveries queued", outcome.deliveries().len()),
    Err(Error::Api(err)) if err.is_payment_required() => {
        println!("Out of room on this plan — upgrade at {:?}", err.upgrade_url());
    }
    Err(Error::Api(err)) if err.is_rate_limited() => {
        println!("Rate limited, retry in {:?}s", err.retry_after);
    }
    Err(err) => eprintln!("{err}"),
}
# }
```

Requests are rate limited per key, per minute. A `429` is retried automatically, waiting for the
interval the API asks for in `Retry-After`, up to `max_retries` attempts.

## Publishing safely

`publish` in dry-run mode, and `preflight`, both report what would happen without sending anything:

```rust,no_run
use fopost::models::{PublishOptions, PublishOutcome};

# async fn run(client: fopost::Client, post_id: &str) -> Result<(), fopost::Error> {
// Hard blockers per account, plus advisory content signals.
let check = client.posts().preflight(post_id).await?;
if !check.ready {
    for account in &check.accounts {
        for issue in &account.issues {
            println!("{}: {issue}", account.platform.as_deref().unwrap_or("?"));
        }
    }
}

// Or run the publish path itself without anything leaving the building.
let outcome = client.posts().publish(post_id, &PublishOptions::new().dry_run()).await?;
assert!(matches!(outcome, PublishOutcome::DryRun(_)));
# Ok(()) }
```

## Media

Media upload and bulk CSV import post multipart bodies, which the `multipart` feature covers. It is
on by default.

```rust,no_run
use fopost::models::{ContentBlock, CreatePost, MediaItem, MediaType, MediaUpload};

# async fn run(client: fopost::Client, workspace_id: &str) -> Result<(), fopost::Error> {
let bytes = std::fs::read("card.png").unwrap();
let uploaded = client
    .media()
    .upload(workspace_id, [MediaUpload::new("card.png", "image/png", bytes)])
    .await?;

let block = ContentBlock::text("Ship it").with_media([MediaItem::new(
    MediaType::Image,
    &uploaded[0].name,
    &uploaded[0].url,
)
.alt("A product screenshot")]);

client.posts().create(&CreatePost::new(workspace_id, [block])).await?;
# Ok(()) }
```

Larger files can skip the API and go straight to storage. `upload_direct` reserves a slot with
`presign`, PUTs the bytes to the returned url with the returned headers and no API key, then calls
`complete` to land the file in the library. It needs no Cargo feature.

```rust,no_run
# async fn run(client: fopost::Client, workspace_id: &str) -> Result<(), fopost::Error> {
let bytes = std::fs::read("clip.mp4").unwrap();
let uploaded = client
    .media()
    .upload_direct(workspace_id, "clip.mp4", "video/mp4", bytes)
    .await?;
println!("{}", uploaded.url);
# Ok(()) }
```

## Cargo features

| Feature | Default | What it does |
| --- | --- | --- |
| `rustls-tls` | yes | TLS through rustls, needing no system OpenSSL |
| `native-tls` | no | TLS through the platform's own stack |
| `multipart` | yes | Media upload and bulk CSV import |

To use `native-tls` instead, turn the defaults off and name what you want back:

```toml
fopost = { version = "0.3", default-features = false, features = ["native-tls", "multipart"] }
```

## Forward compatibility

Response types ignore fields they do not know, and every enum keeps an `Other(String)` variant, so a
platform or status added server-side parses on an older SDK instead of failing the whole response.

```rust
use fopost::models::Platform;

assert_eq!(Platform::InstagramBusiness.as_str(), "instagram-business");
assert_eq!(Platform::from("brand-new"), Platform::Other("brand-new".into()));
```

## Examples

```sh
export FOPOST_API_KEY=fp_...
cargo run --example create_post -- "Hello from the Rust SDK"
cargo run --example preflight -- <post-id>
```

## Contributing

Issues and pull requests are welcome at [fopost/fopost-rust](https://github.com/fopost/fopost-rust).

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
```

## Other SDKs

TypeScript ([`@fopost/sdk`](https://github.com/fopost/fopost-js)), Python
([`fopost`](https://github.com/fopost/fopost-python)), PHP, Laravel, and WordPress, plus ready-to-run
[API collections](https://github.com/fopost/fopost-api-collections) for Postman and Bruno.

## License

MIT. See [LICENSE](LICENSE).

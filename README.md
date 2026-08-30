# fopost

[![crates.io](https://img.shields.io/crates/v/fopost.svg)](https://crates.io/crates/fopost)
[![docs.rs](https://img.shields.io/docsrs/fopost)](https://docs.rs/fopost)
[![license](https://img.shields.io/crates/l/fopost.svg)](https://github.com/fopost/fopost-rust/blob/main/LICENSE)
[![ci](https://img.shields.io/github/actions/workflow/status/fopost/fopost-rust/ci.yml?label=ci)](https://github.com/fopost/fopost-rust/actions/workflows/ci.yml)

Official Rust SDK for the [FoPost](https://fopost.com) API. Schedule and publish to +30 social
platforms from your code.

```toml
[dependencies]
fopost = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Async, built on `reqwest`. Requires Rust 1.85 or newer.

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
| `accounts()` | `list`, `get`, `create`, `delete`, `toggle_primary`, `validate`, `health`, `health_summary`, `refresh_token`, `analytics`, `communities`, `sync_communities`, `search_communities`, `add_community`, `remove_community` |
| `workspaces()` | `list`, `get`, `create`, `update`, `delete`, `analytics` |
| `labels()` | `list`, `get`, `create`, `update`, `delete` |
| `webhooks()` | `list`, `create`, `update`, `delete`, `test` |
| `automations()` | `list`, `get`, `create`, `update`, `delete`, `toggle`, `runs`, `get_run`, `stats`, `trigger` |
| `analytics()` | `overview`, `time_series`, `top_posts`, `labels`, `posts_table`, `posting_streak`, `demographics`, `collect` |
| `media()` | `list`, `upload`, `delete` |

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

## Cargo features

| Feature | Default | What it does |
| --- | --- | --- |
| `rustls-tls` | yes | TLS through rustls, needing no system OpenSSL |
| `native-tls` | no | TLS through the platform's own stack |
| `multipart` | yes | Media upload and bulk CSV import |

To use `native-tls` instead, turn the defaults off and name what you want back:

```toml
fopost = { version = "0.1", default-features = false, features = ["native-tls", "multipart"] }
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

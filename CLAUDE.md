# CLAUDE.md

Guidance for Claude Code (claude.ai/code) when working in this repository.

## What This Is

Crate `fopost` on crates.io — the official Rust client for the FoPost REST API
(`fopost.com`). Version `0.1.0`. It wraps the API's HTTP surface in resource accessors on
`Client`: `posts()`, `accounts()`, `workspaces()`, `labels()`, `webhooks()`,
`automations()`, `analytics()`, `media()`.

Edition 2021, `rust-version = "1.85"` (the MSRV CI builds against). Async on `reqwest`
0.12 + `tokio`; models are `serde`; errors are `thiserror`. Docs at docs.rs/fopost.

This crate calls the REST API directly and has no FoPost code dependency — and nothing
wraps it, so there is no downstream lockstep obligation. It does still break when the API
contract changes.

## Brand Rules

- The product is **FoPost** (`fopost.com`). Never write "OwlStack" — retired Aug 2026.
- Never write an email address. Support is https://fopost.com/contact and GitHub issues.
- Never name AI providers/models, infrastructure vendors, or any person.
- Never type a platform count. The crate description and `lib.rs` say "+30 social
  platforms"; keep it that way.

## Architecture

```
src/
  lib.rs         crate docs, lints, re-exports (Client, ClientBuilder, Error, ApiError,
                 Result, DEFAULT_BASE_URL, API_KEY_ENV, FoPost alias)
  client.rs      Client + ClientBuilder, resource accessors, request() escape hatch
  http.rs        HttpClient: headers, retry loop, Envelope<T>, Query/push_opt, decode
  error.rs       Error enum, ApiError
  models/        common posts accounts workspaces labels webhooks automations analytics media
  resources/     one module per group, each a borrowed struct holding &HttpClient
```

Request flow: `client.posts()` returns `Posts<'_> { http: &HttpClient }`; a method builds a
`Query` (via `push_opt`, which drops `None`) and a serializable body, then calls
`http.send::<T, B>(method, path, query, body)` → retry loop → `finish()` (status split,
`ApiError` on non-2xx) → `decode()`. Resource modules deserialize into `Envelope<T>` where
the endpoint wraps its payload, so **the `{"data": …}` unwrap lives in the resources, not
the transport**. `send_value` returns raw `serde_json::Value` for the escape hatch;
`raw_request`/`send_request` (feature `multipart`) carry the same auth headers for uploads.

`Client` is `Clone` and cheap to clone — the `reqwest` connection pool is shared.

## API Contract

- Base URL: `DEFAULT_BASE_URL = "https://api.fopost.com/v1"`, version path included.
  Override with `ClientBuilder::base_url`. **The library reads no `FOPOST_BASE_URL`** —
  only `examples/create_post.rs` looks the variable up and passes it to the builder.
- Auth: header `X-API-Key`. `Client::new(key)` requires the key; `Client::from_env()` reads
  `FOPOST_API_KEY` (`API_KEY_ENV`). An empty key is an `Error::Config`.
- Headers sent: `X-API-Key`, `Accept: application/json`, `User-Agent: fopost-rust/<crate
  version>` (also set on the built `reqwest::Client`). `Content-Type: application/json` is
  set by `reqwest`'s `.json()`, not by hand.
- Timeout: 30s default on the `reqwest::Client`. `ClientBuilder::http_client` replaces it,
  and then the timeout is the caller's to configure.
- **Retries: 3 total attempts by default (`ClientBuilder::max_retries`, minimum 1).**
  Retried only on **429, 502, 503, 504** (`is_retryable`) — **500 and 501 are not retried,
  and transport errors are not retried either.** The wait is `Retry-After` when present,
  otherwise a flat 1 second, capped at 60s (`MAX_RETRY_WAIT`); there is **no exponential
  backoff**, and `Retry-After` is parsed as **delta-seconds only**, not as an HTTP date.
  Both are divergences from the shared SDK brief — document the code, not the brief.
- Success envelope: `{"data": …}` peeled by the resource module via `Envelope<T>`.
  Pagination meta lives in `models::common`. An empty body or 204 decodes as
  `serde_json::Value::Null`.
- Error envelope: `{"error": "<code>", "message": "<text>"}` → `ApiError { status, code,
  message, body, retry_after }`, reached as `Error::Api`. A non-JSON error body is wrapped
  as `{"message": <text>}` so nothing is lost. `ApiError::upgrade_url()` reads a 402's
  `upgrade_url`; predicates are `is_unauthorized`, `is_payment_required`, `is_forbidden`,
  `is_not_found`, `is_rate_limited`. `Error` also has `status()`, `code()`,
  `is_rate_limited()`, `retry_after()`.
- The `Error` enum is `#[non_exhaustive]`: `Api`, `Transport` (reqwest — DNS, TLS,
  connection, timeout), `Decode { source, body }`, `Config(String)`. There is no per-status
  error type.
- Rate-limit headers (`X-RateLimit-*`) are **not read or surfaced anywhere** in this crate.
  Only the Go SDK exposes them.
- Escape hatch: `client.request(method, path, query, body) -> Result<serde_json::Value>`,
  which goes through the same auth, retries, and error handling.

Resource coverage is broad but **`communities` is not wrapped** (the Go SDK has it) —
reach `GET /accounts/{id}/communities` and friends through `client.request` until it is.

## Commands

```bash
cargo build
cargo test --all-features
cargo test --no-default-features --features rustls-tls --lib --tests
cargo fmt --all --check                                  # `cargo fmt --all` to fix
cargo clippy --all-features --all-targets -- -D warnings
cargo build --examples --all-features
cargo doc --all-features --no-deps                       # RUSTDOCFLAGS="--cfg docsrs -D warnings"
cargo package --all-features
cargo run --example create_post                          # needs FOPOST_API_KEY
cargo run --example preflight
```

`.github/workflows/ci.yml` runs with `RUSTFLAGS: -D warnings` on `1.85` and `stable`:
fmt check and clippy on stable only, `cargo test --all-features` on both, the
no-default-features lib/tests build (so a `default-features = false` consumer stays
unbroken), and an examples build. A second `docs` job runs nightly `cargo doc` with the
docs.rs invocation so a broken intra-doc link fails here first.

Cargo features: `rustls-tls` (default) / `native-tls` — pick one TLS backend; `multipart`
(default) gates media upload and bulk CSV import. Anything behind `multipart` must be
`#[cfg(feature = "multipart")]`, or the no-default-features CI job breaks.

## Conventions

- `rustfmt.toml`: `max_width = 100`, edition 2021. Clippy at `-D warnings` with
  `--all-targets`, so tests and examples must be clean too.
- `#![forbid(unsafe_code)]` and `#![warn(missing_docs)]` in `lib.rs`. Every public item
  outside `models` carries a doc comment; `models/mod.rs` opts out with
  `#![allow(missing_docs)]` because field names mirror the API's own.
- Response types are permissive on purpose: unknown fields are ignored, missing fields
  default, and enums keep an `Other(String)` variant, so a server-side addition never
  breaks a client. Preserve that when adding a model.
- Resource structs borrow (`Posts<'_>`), so they are created per call and never stored.
- Query values are stringified into `Query = Vec<(&'static str, String)>` through
  `push_opt`, which drops `None`.
- Doc examples use `no_run` (they would otherwise need a live key) and must compile —
  the nightly `docs` job catches breakage.
- `FoPost` is a type alias for `Client`, for people arriving from the TypeScript SDK.

## Testing

Integration tests in `tests/` using `wiremock`. **Tests never hit the live API** — every
request goes to a `wiremock::MockServer`.

`tests/common/mod.rs` provides `client(&server)`, which builds a `Client` at
`{server.uri()}/v1` with `max_retries(1)` so nothing waits on a real backoff; retry tests
build their own client with a larger budget. It also holds JSON fixtures
(`post_fixture()`, `account_fixture()`, …) that deliberately mix snake_case and camelCase,
matching what the API actually sends.

Files: `client.rs` (key required, base-url handling, auth headers, escape hatch),
`errors.rs` (404/402/403 mapping, `upgrade_url`, 429 retry and budget exhaustion, non-JSON
error bodies), `posts.rs`, `resources.rs`, `models.rs` (unknown platform/status/field still
parses). Add to the matching file rather than a new one.

## Releasing

Tag `v<version>` matching `Cargo.toml`; `.github/workflows/release.yml` publishes to
crates.io. It verifies the tag equals the crate version via `cargo metadata`, runs fmt,
clippy, and `cargo test --all-features`, then `cargo package --all-features` (this catches
a `.crate` missing a file that would not build for anyone else) before
`cargo publish --all-features`.

Requires repo secret **`CARGO_REGISTRY_TOKEN`**, on the GitHub environment named
`crates-io` — the environment gate means commit access alone does not grant the ability to
publish. Note `Cargo.toml` has `exclude = [".github/", "tests/"]`, so the packaged crate
carries neither.

## Git

Conventional Commits (`<type>(<scope>): <description>`), atomic — one logical change per
commit. Branch `feature/<description>` off a fresh `main`, merge via PR.
Never `gh pr create` — push the branch and hand over the compare link
(`https://github.com/fopost/fopost-rust/compare/main...<branch>`).

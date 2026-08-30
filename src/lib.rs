//! Official Rust SDK for the [FoPost](https://fopost.com) API — schedule and
//! publish to +30 social platforms from your code.
//!
//! ```no_run
//! use fopost::{Client, models::CreatePost};
//!
//! # async fn run() -> Result<(), fopost::Error> {
//! let client = Client::from_env()?;
//!
//! let workspaces = client.workspaces().list().await?;
//! let workspace = &workspaces[0];
//!
//! let accounts = client.accounts().list(Some(&workspace.id)).await?;
//! let ids: Vec<_> = accounts.iter().map(|a| a.id.clone()).collect();
//!
//! // Create a draft, then send it out.
//! let post = client
//!     .posts()
//!     .create(&CreatePost::text(&workspace.id, "Hello from Rust").accounts(ids))
//!     .await?;
//! client.posts().publish(&post.id, &Default::default()).await?;
//! # Ok(()) }
//! ```
//!
//! # Authentication
//!
//! Every request sends an API key in `X-API-Key`. Keys are created in the
//! dashboard under **Settings → API Keys** and carry only the scopes granted at
//! creation: `posts` (which covers publishing, deliveries, and media),
//! `workspaces`, `accounts`, `labels`, `webhooks`, `analytics`, `automations`.
//! A key may also be bound to one workspace, in which case naming any other
//! workspace answers `403`.
//!
//! # Errors
//!
//! Everything returns [`Result<T>`](Result). A non-2xx response becomes
//! [`Error::Api`], carrying the status, the API's machine-readable `code`, and
//! the whole body:
//!
//! ```no_run
//! # async fn run(client: fopost::Client) {
//! match client.posts().publish("9b2f6c1e-…", &Default::default()).await {
//!     Ok(result) => println!("{} deliveries queued", result.deliveries().len()),
//!     Err(fopost::Error::Api(err)) if err.is_payment_required() => {
//!         println!("upgrade at {:?}", err.upgrade_url());
//!     }
//!     Err(err) => eprintln!("{err}"),
//! }
//! # }
//! ```
//!
//! # Rate limiting
//!
//! Requests are limited per key, per minute. A `429` is retried automatically,
//! waiting for the interval the API asks for in `Retry-After`, up to
//! [`ClientBuilder::max_retries`] attempts. Set it to `1` to handle backoff
//! yourself.
//!
//! # Cargo features
//!
//! | Feature | Default | What it does |
//! | --- | --- | --- |
//! | `rustls-tls` | yes | TLS through rustls, needing no system OpenSSL |
//! | `native-tls` | no | TLS through the platform's own stack |
//! | `multipart` | yes | Media upload and bulk CSV import |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub mod http;
pub mod models;
pub mod resources;

mod client;

pub use client::{Client, ClientBuilder, API_KEY_ENV};
pub use error::{ApiError, Error, Result};
pub use http::DEFAULT_BASE_URL;

/// Alias for people arriving from the TypeScript SDK, where the class is `FoPost`.
pub type FoPost = Client;

/// The README's examples, compiled as doctests so they cannot rot.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct Readme;

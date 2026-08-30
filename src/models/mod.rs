//! Types for every request body and response the SDK models.
//!
//! Response structs are permissive on purpose: unknown fields are ignored and
//! missing ones default, so a server-side addition never breaks a client. Enums
//! keep an `Other(String)` variant for the same reason.
//!
//! Field names mirror the API's own, so they carry their own documentation;
//! anything non-obvious is commented individually.

#![allow(missing_docs)]

pub mod accounts;
pub mod analytics;
pub mod automations;
pub mod common;
pub mod labels;
pub mod media;
pub mod posts;
pub mod webhooks;
pub mod workspaces;

pub use accounts::*;
pub use analytics::*;
pub use automations::*;
pub use common::*;
pub use labels::*;
pub use media::*;
pub use posts::*;
pub use webhooks::*;
pub use workspaces::*;

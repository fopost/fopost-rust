//! Types for every request body and response the SDK models.
//!
//! Response structs are permissive on purpose: unknown fields are ignored and
//! missing ones default, so a server-side addition never breaks a client. Enums
//! keep an `Other(String)` variant for the same reason.
//!
//! Field names mirror the API's own, so they carry their own documentation;
//! anything non-obvious is commented individually.

#![allow(missing_docs)]

pub mod account_groups;
pub mod accounts;
pub mod activity;
pub mod ads;
pub mod analytics;
pub mod automations;
pub mod broadcasts;
pub mod common;
pub mod contacts;
pub mod google_ads;
pub mod inbox;
pub mod knowledge;
pub mod labels;
pub mod media;
pub mod posts;
pub mod validate;
pub mod webhooks;
pub mod workspaces;

pub use account_groups::*;
pub use accounts::*;
pub use activity::*;
pub use ads::*;
pub use analytics::*;
pub use automations::*;
pub use broadcasts::*;
pub use common::*;
pub use contacts::*;
pub use google_ads::*;
pub use inbox::*;
pub use knowledge::*;
pub use labels::*;
pub use media::*;
pub use posts::*;
pub use validate::*;
pub use webhooks::*;
pub use workspaces::*;

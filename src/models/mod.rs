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
pub mod blog;
pub mod blogs;
pub mod broadcasts;
pub mod common;
pub mod connect_sessions;
pub mod contacts;
pub mod google_ads;
pub mod inbox;
pub mod knowledge;
pub mod labels;
pub mod logs;
pub mod media;
pub mod posts;
pub mod queue;
pub mod validate;
pub mod webhooks;
pub mod whatsapp;
pub mod workspaces;

pub use account_groups::*;
pub use accounts::*;
pub use activity::*;
pub use ads::*;
pub use analytics::*;
pub use automations::*;
pub use blog::*;
pub use blogs::*;
pub use broadcasts::*;
pub use common::*;
pub use connect_sessions::*;
pub use contacts::*;
pub use google_ads::*;
pub use inbox::*;
pub use knowledge::*;
pub use labels::*;
pub use logs::*;
pub use media::*;
pub use posts::*;
pub use queue::*;
pub use validate::*;
pub use webhooks::*;
pub use whatsapp::*;
pub use workspaces::*;

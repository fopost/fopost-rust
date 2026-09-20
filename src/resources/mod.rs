//! One module per API group. Reach them through the [`crate::Client`].

pub mod account_groups;
pub mod accounts;
pub mod activity;
pub mod ads;
pub mod analytics;
pub mod automations;
pub mod broadcasts;
pub mod contacts;
pub mod google_business;
pub mod inbox;
pub mod knowledge;
pub mod labels;
pub mod media;
pub mod posts;
pub mod validate;
pub mod webhooks;
pub mod workspaces;

pub use account_groups::AccountGroups;
pub use accounts::Accounts;
pub use activity::Activity;
pub use ads::Ads;
pub use analytics::Analytics;
pub use automations::Automations;
pub use broadcasts::{Broadcasts, Sequences};
pub use contacts::Contacts;
pub use google_business::GoogleBusiness;
pub use inbox::Inbox;
pub use knowledge::Knowledge;
pub use labels::Labels;
pub use media::Media;
pub use posts::Posts;
pub use validate::Validate;
pub use webhooks::Webhooks;
pub use workspaces::Workspaces;

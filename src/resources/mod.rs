//! One module per API group. Reach them through the [`crate::Client`].

pub mod accounts;
pub mod ads;
pub mod analytics;
pub mod automations;
pub mod inbox;
pub mod labels;
pub mod media;
pub mod posts;
pub mod validate;
pub mod webhooks;
pub mod workspaces;

pub use accounts::Accounts;
pub use ads::Ads;
pub use analytics::Analytics;
pub use automations::Automations;
pub use inbox::Inbox;
pub use labels::Labels;
pub use media::Media;
pub use posts::Posts;
pub use validate::Validate;
pub use webhooks::Webhooks;
pub use workspaces::Workspaces;

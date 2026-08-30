//! One module per API group. Reach them through the [`crate::Client`].

pub mod accounts;
pub mod analytics;
pub mod automations;
pub mod labels;
pub mod media;
pub mod posts;
pub mod webhooks;
pub mod workspaces;

pub use accounts::Accounts;
pub use analytics::Analytics;
pub use automations::Automations;
pub use labels::Labels;
pub use media::Media;
pub use posts::Posts;
pub use webhooks::Webhooks;
pub use workspaces::Workspaces;

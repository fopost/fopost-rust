//! Shared enums and small value types.

use serde::{Deserialize, Serialize};

/// Declare a wire-string enum that keeps parsing when the API grows a new value.
///
/// Unknown strings land in `Other(String)` instead of failing the whole
/// response, so a platform added server-side never breaks an older client.
macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident { $($(#[$vmeta:meta])* $variant:ident => $wire:literal),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, ::serde::Serialize, ::serde::Deserialize)]
        pub enum $name {
            $($(#[$vmeta])* #[serde(rename = $wire)] $variant,)+
            /// A value this SDK version does not know about yet.
            #[serde(untagged)]
            Other(String),
        }

        impl $name {
            /// The value as the API spells it.
            pub fn as_str(&self) -> &str {
                match self {
                    $($name::$variant => $wire,)+
                    $name::Other(value) => value.as_str(),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::convert::Infallible;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Ok(match value {
                    $($wire => $name::$variant,)+
                    other => $name::Other(other.to_string()),
                })
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                value.parse().unwrap_or_else(|_| unreachable!())
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                $name::from(value.as_str())
            }
        }
    };
}

pub(crate) use string_enum;

string_enum! {
    /// Every network the API can publish to.
    pub enum Platform {
        Twitter => "twitter",
        Linkedin => "linkedin",
        Facebook => "facebook",
        Instagram => "instagram",
        InstagramBusiness => "instagram-business",
        Telegram => "telegram",
        Twitch => "twitch",
        Discord => "discord",
        Slack => "slack",
        Reddit => "reddit",
        Pinterest => "pinterest",
        Tumblr => "tumblr",
        Dribbble => "dribbble",
        Mewe => "mewe",
        Tiktok => "tiktok",
        Youtube => "youtube",
        Bluesky => "bluesky",
        Threads => "threads",
        Mastodon => "mastodon",
        Lemmy => "lemmy",
        Devto => "devto",
        Hashnode => "hashnode",
        Medium => "medium",
        Substack => "substack",
        GoogleBusiness => "google-business",
        Kick => "kick",
        Listmonk => "listmonk",
        Wordpress => "wordpress",
        Nostr => "nostr",
        Whop => "whop",
        Skool => "skool",
    }
}

string_enum! {
    /// Where a post sits in its lifecycle.
    pub enum PostStatus {
        Draft => "draft",
        Scheduled => "scheduled",
        Publishing => "publishing",
        Published => "published",
        PartiallyFailed => "partially_failed",
        Failed => "failed",
        Cancelled => "cancelled",
    }
}

string_enum! {
    /// The state of one post-to-account delivery.
    pub enum DeliveryStatus {
        Pending => "pending",
        Queued => "queued",
        Delayed => "delayed",
        Publishing => "publishing",
        Published => "published",
        Failed => "failed",
        Cancelled => "cancelled",
    }
}

string_enum! {
    /// Shape of a post's body.
    pub enum ContentType {
        Post => "post",
        Thread => "thread",
        Reel => "reel",
    }
}

string_enum! {
    /// What kind of artifact the composer produced.
    pub enum ArtifactType {
        TextPost => "text_post",
        Thread => "thread",
        Article => "article",
        Carousel => "carousel",
        ShortVideo => "short_video",
        LinkShare => "link_share",
    }
}

string_enum! {
    /// What an attachment is.
    pub enum MediaType {
        Image => "image",
        Video => "video",
        Gif => "gif",
        Document => "document",
    }
}

string_enum! {
    /// Whether an account's credentials still work.
    pub enum HealthStatus {
        Healthy => "healthy",
        Degraded => "degraded",
        Expired => "expired",
        Revoked => "revoked",
        Unknown => "unknown",
    }
}

string_enum! {
    /// The unit a repeating post's gap is measured in.
    pub enum GapUnit {
        Hours => "hours",
        Days => "days",
        Weeks => "weeks",
        Months => "months",
    }
}

string_enum! {
    /// What a workspace represents.
    pub enum WorkspaceType {
        Personal => "PERSONAL",
        Team => "TEAM",
        Organization => "ORGANIZATION",
        Client => "CLIENT",
        Project => "PROJECT",
        Department => "DEPARTMENT",
        Event => "EVENT",
        Temporary => "TEMPORARY",
        Community => "COMMUNITY",
        Brand => "BRAND",
        Agency => "AGENCY",
    }
}

/// A file attached to a post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    #[serde(rename = "type")]
    pub media_type: MediaType,
    #[serde(default)]
    pub name: Option<String>,
    pub url: String,
    /// File size in bytes.
    #[serde(default)]
    pub size: Option<f64>,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default)]
    pub thumbnail: Option<String>,
}

impl MediaItem {
    /// The minimum the API accepts on a create or update: kind, name, and url.
    pub fn new(media_type: MediaType, name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            media_type,
            name: Some(name.into()),
            url: url.into(),
            size: None,
            alt: None,
            thumbnail: None,
        }
    }

    /// Alt text, which every image should carry.
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = Some(alt.into());
        self
    }
}

/// Pagination footer on the list endpoints that page.
#[derive(Debug, Clone, Deserialize)]
pub struct PageMeta {
    #[serde(default)]
    pub current_page: u32,
    #[serde(default)]
    pub per_page: u32,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub last_page: u32,
    /// 1-based index of the first item on this page (0 when empty).
    #[serde(default)]
    pub from: u64,
    /// 1-based index of the last item on this page.
    #[serde(default)]
    pub to: u64,
}

/// One page of a list endpoint: its items plus the pagination footer.
#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    #[serde(rename = "data", default = "Vec::new")]
    pub items: Vec<T>,
    pub meta: PageMeta,
}

impl<T> Page<T> {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when another page exists after this one.
    pub fn has_next(&self) -> bool {
        self.meta.current_page < self.meta.last_page
    }
}

impl<T> IntoIterator for Page<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Page<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// The `{"message": …}` body the delete endpoints answer with.
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub message: String,
}

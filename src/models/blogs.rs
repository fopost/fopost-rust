//! Blogs, articles and products that already live on a connected site.
//!
//! Every id here is the platform's own, never a FoPost id. WordPress and a
//! Shopify store both answer the article shapes; products are Shopify only.

use serde::{Deserialize, Serialize};

/// A blog on a connected site.
///
/// A Shopify store reports every blog it has; WordPress has one implicit blog
/// and reports it under the id `default`, so both answer the same shape.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteBlog {
    pub id: String,
    pub title: String,
    pub handle: Option<String>,
    pub url: Option<String>,
}

/// An article that already lives on a connected site.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteArticle {
    pub id: String,
    pub blog_id: Option<String>,
    pub title: String,
    pub body_html: Option<String>,
    pub excerpt: Option<String>,
    /// One of `published`, `draft`, `pending`, `scheduled`.
    pub status: String,
    pub author_name: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub image_url: Option<String>,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub updated_at: Option<String>,
}

/// A product on a connected store.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteProduct {
    pub id: String,
    pub title: String,
    pub handle: Option<String>,
    /// One of `active`, `draft`, `archived`.
    pub status: String,
    pub description: Option<String>,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub image_url: Option<String>,
    pub url: Option<String>,
    /// Lowest variant price, as a decimal string.
    pub price: Option<String>,
    pub currency: Option<String>,
    pub updated_at: Option<String>,
}

/// Filters for listing a blog's articles. Unset fields are not sent.
#[derive(Debug, Clone, Default)]
pub struct ListArticles {
    /// 1 to 50; the API defaults to 20.
    pub limit: Option<u32>,
    /// `published`, `draft`, `pending` or `scheduled`.
    pub status: Option<String>,
    /// Matches the article title.
    pub q: Option<String>,
}

impl ListArticles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn q(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }
}

/// Filters for listing a store's products. Unset fields are not sent.
#[derive(Debug, Clone, Default)]
pub struct ListProducts {
    pub limit: Option<u32>,
    /// `active`, `draft` or `archived`.
    pub status: Option<String>,
    /// Matches the product title.
    pub q: Option<String>,
}

impl ListProducts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn q(mut self, q: impl Into<String>) -> Self {
        self.q = Some(q.into());
        self
    }
}

/// The body of `POST .../articles`. `title` and `body` are required.
#[derive(Debug, Clone, Serialize)]
pub struct CreateArticle {
    pub title: String,
    /// FoPost body markup; the site's own format is rendered from it.
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

impl CreateArticle {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            excerpt: None,
            status: None,
            tags: None,
            author_name: None,
            image_url: None,
        }
    }

    pub fn excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn author_name(mut self, name: impl Into<String>) -> Self {
        self.author_name = Some(name.into());
        self
    }

    pub fn image_url(mut self, url: impl Into<String>) -> Self {
        self.image_url = Some(url.into());
        self
    }
}

/// The body of `PATCH .../articles/{id}`.
///
/// Every field is optional and only the ones set are sent, so an omitted field
/// keeps whatever the site already had. Set at least one.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateArticle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

impl UpdateArticle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn author_name(mut self, name: impl Into<String>) -> Self {
        self.author_name = Some(name.into());
        self
    }

    pub fn image_url(mut self, url: impl Into<String>) -> Self {
        self.image_url = Some(url.into());
        self
    }
}

/// The body of `PATCH .../products/{id}`. Only what is set travels.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
}

impl UpdateProduct {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn product_type(mut self, product_type: impl Into<String>) -> Self {
        self.product_type = Some(product_type.into());
        self
    }

    pub fn vendor(mut self, vendor: impl Into<String>) -> Self {
        self.vendor = Some(vendor.into());
        self
    }
}

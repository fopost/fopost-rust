//! Blog — keyword planning, article drafting and publishing to a website.
//!
//! An article goes to a website; a post goes to a social network. Every wire
//! field here is camelCase, and `workspaceId` travels as a query parameter on
//! the `{id}` routes and as a body field on the collection routes.

use serde::{Deserialize, Serialize};

use super::common::string_enum;

string_enum! {
    /// Where an article sits in its lifecycle.
    pub enum BlogArticleStatus {
        Queued => "queued",
        Writing => "writing",
        Draft => "draft",
        Scheduled => "scheduled",
        Publishing => "publishing",
        Published => "published",
        Failed => "failed",
        Rejected => "rejected",
    }
}

string_enum! {
    /// Where a planned keyword stands.
    pub enum BlogKeywordStatus {
        Suggested => "suggested",
        Planned => "planned",
        Written => "written",
        Dismissed => "dismissed",
    }
}

string_enum! {
    /// Who put a keyword on the plan.
    pub enum BlogKeywordSource {
        Provider => "provider",
        User => "user",
        Agent => "agent",
    }
}

string_enum! {
    /// Who asked for an article.
    pub enum BlogArticleOrigin {
        Plan => "plan",
        User => "user",
        Agent => "agent",
    }
}

string_enum! {
    /// Where published articles go.
    pub enum BlogTargetKind {
        ProductBlog => "product_blog",
        Account => "account",
        Website => "website",
    }
}

string_enum! {
    /// The image quality tier used for article imagery.
    pub enum BlogImageTier {
        Draft => "draft",
        Standard => "standard",
        Best => "best",
    }
}

/// A workspace's blog settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogSettings {
    #[serde(default)]
    pub site_url: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub audience: Option<String>,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub location_code: Option<i64>,
    /// Articles planned per week; 0 turns planning off.
    #[serde(default)]
    pub cadence_per_week: u32,
    /// Days a draft waits for review before it may auto-publish.
    #[serde(default)]
    pub review_hold_days: u32,
    #[serde(default)]
    pub auto_publish: bool,
    #[serde(default)]
    pub image_tier: Option<BlogImageTier>,
    #[serde(default)]
    pub in_body_images: Option<u32>,
    #[serde(default)]
    pub min_words: Option<u32>,
    #[serde(default)]
    pub max_words: Option<u32>,
    #[serde(default)]
    pub brand_voice_id: Option<String>,
    #[serde(default)]
    pub target_kind: Option<BlogTargetKind>,
    #[serde(default)]
    pub target_account_id: Option<String>,
    #[serde(default)]
    pub target_platform: Option<String>,
    /// Where published articles are POSTed as a signed webhook.
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub has_website_secret: bool,
    /// Whether search volume and difficulty can be fetched on this deployment.
    #[serde(default)]
    pub keyword_data_available: bool,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// The answer to `PATCH /blog/settings`: the settings, plus the website
/// secret when this patch set `website_url` for the first time. It is shown
/// once; store it, it signs every delivery to the website.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogSettingsUpdated {
    #[serde(rename = "data")]
    pub settings: BlogSettings,
    #[serde(default)]
    pub website_secret: Option<String>,
}

/// The answer to `POST /blog/settings/website-secret`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogWebsiteSecret {
    pub website_secret: String,
}

/// The answer to `POST /blog/settings/website-test`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogWebsiteTest {
    #[serde(default)]
    pub ok: bool,
    /// The HTTP status the website answered, when it answered at all.
    #[serde(default)]
    pub status: Option<u16>,
    #[serde(default)]
    pub error: Option<String>,
}

/// The body of `PATCH /blog/settings`. Only the fields you set are sent;
/// `Some(None)` on a nullable field clears it.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBlogSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_url: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_code: Option<i64>,
    /// 0 to 7.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadence_per_week: Option<u32>,
    /// 0 to 14.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_hold_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_publish: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_tier: Option<BlogImageTier>,
    /// 0 to 3.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_body_images: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_words: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_words: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_voice_id: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_kind: Option<Option<BlogTargetKind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_account_id: Option<Option<String>>,
    /// Setting it for the first time issues a website secret, returned once.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<Option<String>>,
}

impl UpdateBlogSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn site_url(mut self, url: Option<impl Into<String>>) -> Self {
        self.site_url = Some(url.map(Into::into));
        self
    }

    pub fn topic(mut self, topic: Option<impl Into<String>>) -> Self {
        self.topic = Some(topic.map(Into::into));
        self
    }

    pub fn audience(mut self, audience: Option<impl Into<String>>) -> Self {
        self.audience = Some(audience.map(Into::into));
        self
    }

    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn location_code(mut self, code: i64) -> Self {
        self.location_code = Some(code);
        self
    }

    pub fn cadence_per_week(mut self, cadence: u32) -> Self {
        self.cadence_per_week = Some(cadence);
        self
    }

    pub fn review_hold_days(mut self, days: u32) -> Self {
        self.review_hold_days = Some(days);
        self
    }

    pub fn auto_publish(mut self, auto_publish: bool) -> Self {
        self.auto_publish = Some(auto_publish);
        self
    }

    pub fn image_tier(mut self, tier: BlogImageTier) -> Self {
        self.image_tier = Some(tier);
        self
    }

    pub fn in_body_images(mut self, count: u32) -> Self {
        self.in_body_images = Some(count);
        self
    }

    pub fn words(mut self, min: u32, max: u32) -> Self {
        self.min_words = Some(min);
        self.max_words = Some(max);
        self
    }

    pub fn brand_voice_id(mut self, id: Option<impl Into<String>>) -> Self {
        self.brand_voice_id = Some(id.map(Into::into));
        self
    }

    pub fn target_kind(mut self, kind: Option<BlogTargetKind>) -> Self {
        self.target_kind = Some(kind);
        self
    }

    pub fn target_account_id(mut self, id: Option<impl Into<String>>) -> Self {
        self.target_account_id = Some(id.map(Into::into));
        self
    }

    pub fn website_url(mut self, url: Option<impl Into<String>>) -> Self {
        self.website_url = Some(url.map(Into::into));
        self
    }
}

/// A keyword on the plan.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogKeyword {
    pub id: String,
    pub keyword: String,
    #[serde(default)]
    pub search_volume: Option<i64>,
    /// 0 to 100.
    #[serde(default)]
    pub difficulty: Option<i64>,
    #[serde(default)]
    pub cpc: Option<f64>,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default = "BlogKeywordSource::user")]
    pub source: BlogKeywordSource,
    #[serde(default = "BlogKeywordStatus::suggested")]
    pub status: BlogKeywordStatus,
    /// `YYYY-MM-DD`.
    #[serde(default)]
    pub planned_for: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl BlogKeywordSource {
    fn user() -> Self {
        Self::User
    }
}

impl BlogKeywordStatus {
    fn suggested() -> Self {
        Self::Suggested
    }
}

/// Query for `GET /blog/keywords`.
#[derive(Debug, Clone, Default)]
pub struct ListBlogKeywords {
    /// Keep only these statuses.
    pub status: Vec<BlogKeywordStatus>,
    /// At most 500.
    pub limit: Option<u32>,
}

impl ListBlogKeywords {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: BlogKeywordStatus) -> Self {
        self.status.push(status);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// The body of `POST /blog/keywords/plan`, minus the workspace.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PlanBlogKeywords {
    /// Terms to grow the plan from. Unset plans from the settings' topic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seeds: Option<Vec<String>>,
}

impl PlanBlogKeywords {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seeds<I, S>(mut self, seeds: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.seeds = Some(seeds.into_iter().map(Into::into).collect());
        self
    }
}

/// The body of `POST /blog/keywords`, minus the workspace.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddBlogKeyword {
    pub keyword: String,
    /// `YYYY-MM-DD`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_for: Option<String>,
}

impl AddBlogKeyword {
    pub fn new(keyword: impl Into<String>) -> Self {
        Self {
            keyword: keyword.into(),
            planned_for: None,
        }
    }

    pub fn planned_for(mut self, date: impl Into<String>) -> Self {
        self.planned_for = Some(date.into());
        self
    }
}

/// The body of `PATCH /blog/keywords/{id}`. Only what is set is sent;
/// `planned_for(None)` takes the keyword off the calendar.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBlogKeyword {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<BlogKeywordStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_for: Option<Option<String>>,
}

impl UpdateBlogKeyword {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: BlogKeywordStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn planned_for(mut self, date: Option<impl Into<String>>) -> Self {
        self.planned_for = Some(date.map(Into::into));
        self
    }
}

/// One image on an article.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogArticleImage {
    pub media_id: String,
    pub url: String,
    #[serde(default)]
    pub alt: String,
    /// Index of the outline heading the image sits under; absent on the
    /// featured image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
}

/// An article as the list endpoint returns it: metadata without the body.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogArticle {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub meta_title: Option<String>,
    #[serde(default)]
    pub meta_description: Option<String>,
    #[serde(default)]
    pub featured_image: Option<BlogArticleImage>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub word_count: u32,
    #[serde(default = "BlogArticleStatus::queued")]
    pub status: BlogArticleStatus,
    #[serde(default = "BlogArticleOrigin::user")]
    pub origin: BlogArticleOrigin,
    /// When the review hold ends and the draft may auto-publish.
    #[serde(default)]
    pub review_until: Option<String>,
    #[serde(default)]
    pub publish_at: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub published_url: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl BlogArticleStatus {
    fn queued() -> Self {
        Self::Queued
    }
}

impl BlogArticleOrigin {
    fn user() -> Self {
        Self::User
    }
}

/// An article with its body, outline, brief and images.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogArticleFull {
    #[serde(flatten)]
    pub article: BlogArticle,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub outline: Vec<String>,
    /// The research the writer worked from.
    #[serde(default)]
    pub brief: Option<serde_json::Value>,
    #[serde(default)]
    pub body_markdown: String,
    #[serde(default)]
    pub body_html: String,
    #[serde(default)]
    pub images: Vec<BlogArticleImage>,
}

impl std::ops::Deref for BlogArticleFull {
    type Target = BlogArticle;

    fn deref(&self) -> &BlogArticle {
        &self.article
    }
}

/// Query for `GET /blog/articles`.
#[derive(Debug, Clone, Default)]
pub struct ListBlogArticles {
    /// Keep only these statuses.
    pub status: Vec<BlogArticleStatus>,
    /// At most 200.
    pub limit: Option<u32>,
}

impl ListBlogArticles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: BlogArticleStatus) -> Self {
        self.status.push(status);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// The body of `POST /blog/articles`, minus the workspace. Give one of a
/// title, a keyword, or the id of a keyword on the plan.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBlogArticle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// A keyword from the plan; its term becomes the target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword_id: Option<String>,
    /// Up to 4000 characters of direction for the writer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

impl RequestBlogArticle {
    pub fn title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Default::default()
        }
    }

    pub fn keyword(keyword: impl Into<String>) -> Self {
        Self {
            keyword: Some(keyword.into()),
            ..Default::default()
        }
    }

    pub fn keyword_id(id: impl Into<String>) -> Self {
        Self {
            keyword_id: Some(id.into()),
            ..Default::default()
        }
    }

    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }
}

/// The body of `PATCH /blog/articles/{id}`. Only what is set is sent;
/// `Some(None)` on a nullable field clears it.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBlogArticle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Lowercase letters, digits and hyphens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_title: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_markdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Puts the draft on the calendar; `Some(None)` returns it to review.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_at: Option<Option<String>>,
}

impl UpdateBlogArticle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = Some(slug.into());
        self
    }

    pub fn excerpt(mut self, excerpt: Option<impl Into<String>>) -> Self {
        self.excerpt = Some(excerpt.map(Into::into));
        self
    }

    pub fn meta_title(mut self, title: Option<impl Into<String>>) -> Self {
        self.meta_title = Some(title.map(Into::into));
        self
    }

    pub fn meta_description(mut self, description: Option<impl Into<String>>) -> Self {
        self.meta_description = Some(description.map(Into::into));
        self
    }

    pub fn body_markdown(mut self, markdown: impl Into<String>) -> Self {
        self.body_markdown = Some(markdown.into());
        self
    }

    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = Some(tags.into_iter().map(Into::into).collect());
        self
    }

    pub fn publish_at(mut self, at: Option<impl Into<String>>) -> Self {
        self.publish_at = Some(at.map(Into::into));
        self
    }
}

/// The `data` of an `article.published` webhook, and of the signed delivery
/// a Website target receives. Image URLs are fetchable for six hours; download
/// and store your own copy.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlogArticlePublished {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub meta_title: Option<String>,
    #[serde(default)]
    pub meta_description: Option<String>,
    #[serde(default)]
    pub body_markdown: String,
    #[serde(default)]
    pub body_html: String,
    #[serde(default)]
    pub featured_image: Option<BlogArticleImage>,
    #[serde(default)]
    pub images: Vec<BlogArticleImage>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub word_count: u32,
    #[serde(default)]
    pub published_at: Option<String>,
    /// Where the article now lives. Absent on an `article.test` delivery.
    #[serde(default)]
    pub published_url: Option<String>,
}

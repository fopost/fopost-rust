//! `client.blogs()` — content a connected site already owns.
//!
//! Most of this SDK creates content. These calls reach what is already there:
//! the articles on a WordPress site or a Shopify store's blog, and a Shopify
//! store's products. Every id here is the platform's own, never a FoPost id.
//!
//! Reads need the `posts` scope. Anything that changes the site needs `posts`
//! and `publish`, because a change here is visible to the site's own readers.

use reqwest::Method;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    CreateArticle, ListArticles, ListProducts, RemoteArticle, RemoteBlog, RemoteProduct,
    UpdateArticle, UpdateProduct,
};

/// Blogs, articles and products on a connected site.
#[derive(Debug, Clone)]
pub struct Blogs<'a> {
    pub(crate) http: &'a HttpClient,
}

impl Blogs<'_> {
    /// Blogs the account can write to.
    ///
    /// A Shopify store reports every blog it has; WordPress reports its one
    /// implicit blog, under the id `default`.
    pub async fn list_blogs(&self, account_id: &str) -> Result<Vec<RemoteBlog>> {
        let body: Envelope<Vec<RemoteBlog>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{account_id}/blogs"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Articles on the blog, newest first, drafts included.
    pub async fn list_articles(
        &self,
        account_id: &str,
        blog_id: &str,
        filters: &ListArticles,
    ) -> Result<Vec<RemoteArticle>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "limit", filters.limit);
        push_opt(&mut query, "status", filters.status.as_deref());
        push_opt(&mut query, "q", filters.q.as_deref());
        let body: Envelope<Vec<RemoteArticle>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{account_id}/blogs/{blog_id}/articles"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// One article in full.
    pub async fn get_article(
        &self,
        account_id: &str,
        blog_id: &str,
        article_id: &str,
    ) -> Result<RemoteArticle> {
        let body: Envelope<RemoteArticle> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{account_id}/blogs/{blog_id}/articles/{article_id}"),
                None,
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Write a new article to the blog. Needs the `publish` scope.
    pub async fn create_article(
        &self,
        account_id: &str,
        blog_id: &str,
        payload: &CreateArticle,
    ) -> Result<RemoteArticle> {
        let body: Envelope<RemoteArticle> = self
            .http
            .send(
                Method::POST,
                &format!("/accounts/{account_id}/blogs/{blog_id}/articles"),
                None,
                Some(payload),
            )
            .await?;
        Ok(body.data)
    }

    /// Change the live article in place. Needs the `publish` scope.
    ///
    /// The article is addressed by its own id and only the fields set on
    /// `changes` are sent, so an edit never creates a second post on the site.
    pub async fn update_article(
        &self,
        account_id: &str,
        blog_id: &str,
        article_id: &str,
        changes: &UpdateArticle,
    ) -> Result<RemoteArticle> {
        let body: Envelope<RemoteArticle> = self
            .http
            .send(
                Method::PATCH,
                &format!("/accounts/{account_id}/blogs/{blog_id}/articles/{article_id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove the article from the site. Needs `publish`; cannot be undone.
    pub async fn delete_article(
        &self,
        account_id: &str,
        blog_id: &str,
        article_id: &str,
    ) -> Result<()> {
        self.http
            .send_value::<()>(
                Method::DELETE,
                &format!("/accounts/{account_id}/blogs/{blog_id}/articles/{article_id}"),
                None,
                None,
            )
            .await?;
        Ok(())
    }

    /// The store's products.
    pub async fn list_products(
        &self,
        account_id: &str,
        filters: &ListProducts,
    ) -> Result<Vec<RemoteProduct>> {
        let mut query: Query = Vec::new();
        push_opt(&mut query, "limit", filters.limit);
        push_opt(&mut query, "status", filters.status.as_deref());
        push_opt(&mut query, "q", filters.q.as_deref());
        let body: Envelope<Vec<RemoteProduct>> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/accounts/{account_id}/products"),
                Some(query),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Change a product on the store. Needs `publish`; only what is set changes.
    pub async fn update_product(
        &self,
        account_id: &str,
        product_id: &str,
        changes: &UpdateProduct,
    ) -> Result<RemoteProduct> {
        let body: Envelope<RemoteProduct> = self
            .http
            .send(
                Method::PATCH,
                &format!("/accounts/{account_id}/products/{product_id}"),
                None,
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }
}

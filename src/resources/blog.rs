//! `client.blog()` — keyword planning, article drafting and publishing to a
//! website.
//!
//! An article goes to a website; a post goes to a social network. Reading,
//! settings, keyword edits and submitting an article need the `posts` scope.
//! Suggesting search terms, planning keywords, requesting an article and
//! regenerating one spend credits and need `ai`; a spent balance answers 402
//! with an `upgrade_url`. Publishing needs `publish` and answers 409
//! `no_target` when the settings name no target; a patch, regenerate, submit,
//! publish or reject on an article in the wrong state answers 409
//! `invalid_state`.

use reqwest::Method;
use serde::Serialize;

use crate::error::Result;
use crate::http::{push_opt, Envelope, HttpClient, Query};
use crate::models::{
    AddBlogKeyword, BlogArticle, BlogArticleFull, BlogKeyword, BlogSeedSuggestions, BlogSettings,
    BlogSettingsUpdated, BlogWebsiteSecret, BlogWebsiteTest, ListBlogArticles, ListBlogKeywords,
    PlanBlogKeywords, RequestBlogArticle, SuggestBlogSeeds, UpdateBlogArticle, UpdateBlogKeyword,
    UpdateBlogSettings,
};

/// The blog.
#[derive(Debug, Clone)]
pub struct Blog<'a> {
    pub(crate) http: &'a HttpClient,
}

/// A body the collection routes take: the workspace beside the payload.
#[derive(Serialize)]
struct InWorkspace<'a, T: Serialize> {
    #[serde(rename = "workspaceId")]
    workspace_id: &'a str,
    #[serde(flatten)]
    body: &'a T,
}

fn workspace_query(workspace_id: &str) -> Query {
    vec![("workspaceId", workspace_id.to_string())]
}

fn status_list<S: ToString>(statuses: &[S]) -> Option<String> {
    (!statuses.is_empty()).then(|| {
        statuses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    })
}

impl Blog<'_> {
    /// Where articles publish, what the blog is for, how often it is planned
    /// and the last keyword research run.
    pub async fn get_settings(&self, workspace_id: &str) -> Result<BlogSettings> {
        let body: Envelope<BlogSettings> = self
            .http
            .send::<_, ()>(
                Method::GET,
                "/blog/settings",
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Patch the settings. The answer carries `website_secret` only when this
    /// patch set `website_url` for the first time; store it then, it is never
    /// shown again.
    pub async fn update_settings(
        &self,
        workspace_id: &str,
        changes: &UpdateBlogSettings,
    ) -> Result<BlogSettingsUpdated> {
        self.http
            .send(
                Method::PATCH,
                "/blog/settings",
                None,
                Some(&InWorkspace {
                    workspace_id,
                    body: changes,
                }),
            )
            .await
    }

    /// Issue a new website secret. The old one stops verifying at once.
    pub async fn rotate_website_secret(&self, workspace_id: &str) -> Result<BlogWebsiteSecret> {
        let body: Envelope<BlogWebsiteSecret> = self
            .http
            .send(
                Method::POST,
                "/blog/settings/website-secret",
                None,
                Some(&serde_json::json!({ "workspaceId": workspace_id })),
            )
            .await?;
        Ok(body.data)
    }

    /// Send a signed `article.test` delivery to `website_url`. A website that
    /// does not answer 2xx comes back as an `Error::Api` with status 502
    /// and the same `{ok, status, error}` shape in its body.
    pub async fn test_website(&self, workspace_id: &str) -> Result<BlogWebsiteTest> {
        let body: Envelope<BlogWebsiteTest> = self
            .http
            .send(
                Method::POST,
                "/blog/settings/website-test",
                None,
                Some(&serde_json::json!({ "workspaceId": workspace_id })),
            )
            .await?;
        Ok(body.data)
    }

    /// The keyword plan, optionally narrowed by status.
    pub async fn list_keywords(
        &self,
        workspace_id: &str,
        params: &ListBlogKeywords,
    ) -> Result<Vec<BlogKeyword>> {
        let mut query = workspace_query(workspace_id);
        push_opt(&mut query, "status", status_list(&params.status));
        push_opt(&mut query, "limit", params.limit);
        let body: Envelope<Vec<BlogKeyword>> = self
            .http
            .send::<_, ()>(Method::GET, "/blog/keywords", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// About 10 search terms from the blog goal and topics, to edit before
    /// [`Blog::plan_keywords`]. Pass the current seeds to refine them and a
    /// prompt to steer. Needs the `ai` scope; a spent balance answers 402.
    pub async fn suggest_seeds(
        &self,
        workspace_id: &str,
        params: &SuggestBlogSeeds,
    ) -> Result<Vec<String>> {
        let body: Envelope<BlogSeedSuggestions> = self
            .http
            .send(
                Method::POST,
                "/blog/keywords/suggest",
                None,
                Some(&InWorkspace {
                    workspace_id,
                    body: params,
                }),
            )
            .await?;
        Ok(body.data.seeds)
    }

    /// Queue a research run. Results land as `suggested` keywords and never
    /// fill the calendar; set one to `planned` with [`Blog::update_keyword`]
    /// to pick it. Follow the run in `plan_run` on [`Blog::get_settings`].
    /// Needs the `ai` scope; a spent balance answers 402.
    pub async fn plan_keywords(&self, workspace_id: &str, params: &PlanBlogKeywords) -> Result<()> {
        self.http
            .send::<serde_json::Value, _>(
                Method::POST,
                "/blog/keywords/plan",
                None,
                Some(&InWorkspace {
                    workspace_id,
                    body: params,
                }),
            )
            .await?;
        Ok(())
    }

    /// Put a keyword of your own on the plan.
    pub async fn add_keyword(
        &self,
        workspace_id: &str,
        keyword: &AddBlogKeyword,
    ) -> Result<BlogKeyword> {
        let body: Envelope<BlogKeyword> = self
            .http
            .send(
                Method::POST,
                "/blog/keywords",
                None,
                Some(&InWorkspace {
                    workspace_id,
                    body: keyword,
                }),
            )
            .await?;
        Ok(body.data)
    }

    /// Change a keyword's status or planned date. Setting it to `planned`
    /// picks it and gives it the next free date.
    pub async fn update_keyword(
        &self,
        workspace_id: &str,
        id: &str,
        changes: &UpdateBlogKeyword,
    ) -> Result<BlogKeyword> {
        let body: Envelope<BlogKeyword> = self
            .http
            .send(
                Method::PATCH,
                &format!("/blog/keywords/{id}"),
                Some(workspace_query(workspace_id)),
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Remove a keyword from the plan.
    pub async fn delete_keyword(&self, workspace_id: &str, id: &str) -> Result<()> {
        self.http
            .send::<serde_json::Value, ()>(
                Method::DELETE,
                &format!("/blog/keywords/{id}"),
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(())
    }

    /// Articles without their bodies, optionally narrowed by status.
    pub async fn list_articles(
        &self,
        workspace_id: &str,
        params: &ListBlogArticles,
    ) -> Result<Vec<BlogArticle>> {
        let mut query = workspace_query(workspace_id);
        push_opt(&mut query, "status", status_list(&params.status));
        push_opt(&mut query, "limit", params.limit);
        let body: Envelope<Vec<BlogArticle>> = self
            .http
            .send::<_, ()>(Method::GET, "/blog/articles", Some(query), None)
            .await?;
        Ok(body.data)
    }

    /// Ask for an article. It comes back `queued` and is written in the
    /// background; poll [`Blog::get_article`] or subscribe to
    /// `article.published`. Needs the `ai` scope; a spent balance answers 402.
    pub async fn request_article(
        &self,
        workspace_id: &str,
        request: &RequestBlogArticle,
    ) -> Result<BlogArticle> {
        let body: Envelope<BlogArticle> = self
            .http
            .send(
                Method::POST,
                "/blog/articles",
                None,
                Some(&InWorkspace {
                    workspace_id,
                    body: request,
                }),
            )
            .await?;
        Ok(body.data)
    }

    /// One article with its body, outline, brief and images.
    pub async fn get_article(&self, workspace_id: &str, id: &str) -> Result<BlogArticleFull> {
        let body: Envelope<BlogArticleFull> = self
            .http
            .send::<_, ()>(
                Method::GET,
                &format!("/blog/articles/{id}"),
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }

    /// Edit an article. One still being written or already publishing answers
    /// 409 `invalid_state`, as does a `publish_at` on a draft that was never
    /// submitted with [`Blog::submit_article`].
    pub async fn update_article(
        &self,
        workspace_id: &str,
        id: &str,
        changes: &UpdateBlogArticle,
    ) -> Result<BlogArticleFull> {
        let body: Envelope<BlogArticleFull> = self
            .http
            .send(
                Method::PATCH,
                &format!("/blog/articles/{id}"),
                Some(workspace_query(workspace_id)),
                Some(changes),
            )
            .await?;
        Ok(body.data)
    }

    /// Write the article again from its brief. Needs the `ai` scope.
    pub async fn regenerate_article(&self, workspace_id: &str, id: &str) -> Result<BlogArticle> {
        self.article_action(workspace_id, id, "regenerate").await
    }

    /// Publish an article in review to the configured target now. Needs the
    /// `publish` scope. A draft answers 409 until it is submitted with
    /// [`Blog::submit_article`]; 409 `no_target` means the settings name no
    /// target.
    pub async fn publish_article(&self, workspace_id: &str, id: &str) -> Result<BlogArticle> {
        self.article_action(workspace_id, id, "publish").await
    }

    /// Move a draft, failed or rejected article to `in_review`. Any other
    /// state answers 409.
    pub async fn submit_article(&self, workspace_id: &str, id: &str) -> Result<BlogArticle> {
        self.article_action(workspace_id, id, "submit").await
    }

    /// Reject an article so it never publishes.
    pub async fn reject_article(&self, workspace_id: &str, id: &str) -> Result<BlogArticle> {
        self.article_action(workspace_id, id, "reject").await
    }

    /// Delete an article. One already published stays on the website.
    pub async fn delete_article(&self, workspace_id: &str, id: &str) -> Result<()> {
        self.http
            .send::<serde_json::Value, ()>(
                Method::DELETE,
                &format!("/blog/articles/{id}"),
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(())
    }

    async fn article_action(
        &self,
        workspace_id: &str,
        id: &str,
        action: &str,
    ) -> Result<BlogArticle> {
        let body: Envelope<BlogArticle> = self
            .http
            .send::<_, ()>(
                Method::POST,
                &format!("/blog/articles/{id}/{action}"),
                Some(workspace_query(workspace_id)),
                None,
            )
            .await?;
        Ok(body.data)
    }
}

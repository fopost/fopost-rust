//! The [`Client`] and its builder.

use std::time::Duration;

use crate::error::{Error, Result};
use crate::http::{HttpClient, DEFAULT_BASE_URL, USER_AGENT};
use crate::resources::{
    Accounts, Analytics, Automations, Labels, Media, Posts, Webhooks, Workspaces,
};

/// The environment variable the key is read from by [`Client::from_env`].
pub const API_KEY_ENV: &str = "FOPOST_API_KEY";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_RETRIES: u32 = 3;

/// A client for the FoPost API.
///
/// ```no_run
/// # async fn run() -> Result<(), fopost::Error> {
/// let client = fopost::Client::new("fp_…")?;
/// let workspaces = client.workspaces().list().await?;
/// # Ok(()) }
/// ```
///
/// Cloning is cheap — the underlying connection pool is shared — so pass a
/// `Client` around rather than building one per request.
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
}

impl Client {
    /// A client with the defaults: the production API, a 30 second timeout, and
    /// up to three attempts when the rate limiter or a gateway says to back off.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Client::builder().api_key(api_key).build()
    }

    /// A client reading its key from `FOPOST_API_KEY`.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var(API_KEY_ENV).map_err(|_| {
            Error::Config(format!(
                "fopost: {API_KEY_ENV} is not set — pass the key explicitly"
            ))
        })?;
        Client::new(api_key)
    }

    /// Start configuring a client: base url, timeout, retries, or your own
    /// `reqwest::Client`.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// The base url this client sends to.
    pub fn base_url(&self) -> &str {
        self.http.base_url()
    }

    /// The key this client authenticates with.
    pub fn api_key(&self) -> &str {
        self.http.api_key()
    }

    /// Posts, publishing, and bulk import.
    pub fn posts(&self) -> Posts<'_> {
        Posts { http: &self.http }
    }

    /// Connected social accounts and their health.
    pub fn accounts(&self) -> Accounts<'_> {
        Accounts { http: &self.http }
    }

    /// Workspaces.
    pub fn workspaces(&self) -> Workspaces<'_> {
        Workspaces { http: &self.http }
    }

    /// Labels.
    pub fn labels(&self) -> Labels<'_> {
        Labels { http: &self.http }
    }

    /// Webhook subscriptions.
    pub fn webhooks(&self) -> Webhooks<'_> {
        Webhooks { http: &self.http }
    }

    /// Automations and their runs.
    pub fn automations(&self) -> Automations<'_> {
        Automations { http: &self.http }
    }

    /// Reporting.
    pub fn analytics(&self) -> Analytics<'_> {
        Analytics { http: &self.http }
    }

    /// The media library.
    pub fn media(&self) -> Media<'_> {
        Media { http: &self.http }
    }

    /// Call an endpoint the SDK does not wrap yet, with the auth, retries, and
    /// error handling every other call gets. Returns the raw JSON body.
    ///
    /// ```no_run
    /// # async fn run(client: fopost::Client) -> Result<(), fopost::Error> {
    /// let spec = client
    ///     .request(reqwest::Method::GET, "/openapi.json", None, None::<&()>)
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub async fn request<B: serde::Serialize + ?Sized>(
        &self,
        method: reqwest::Method,
        path: &str,
        query: Option<Vec<(&'static str, String)>>,
        body: Option<&B>,
    ) -> Result<serde_json::Value> {
        self.http.send_value(method, path, query, body).await
    }
}

/// Builds a [`Client`].
#[derive(Debug, Default)]
pub struct ClientBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    max_retries: Option<u32>,
    http_client: Option<reqwest::Client>,
}

impl ClientBuilder {
    /// The API key, from **Settings → API Keys** in the dashboard. Required.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Point at another deployment. Include the version path, as the default
    /// `https://api.fopost.com/v1` does.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Per-request timeout. Defaults to 30 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// How many attempts a request gets when the API asks it to back off.
    /// Defaults to 3; 1 disables retrying.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Bring your own `reqwest::Client` — a shared pool, a proxy, custom TLS.
    /// The timeout set here is then yours to configure, not the builder's.
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Finish, failing if the key is missing or the base url will not parse.
    pub fn build(self) -> Result<Client> {
        let api_key = self
            .api_key
            .filter(|key| !key.is_empty())
            .ok_or_else(|| Error::Config("fopost: an api key is required".into()))?;

        let http_client = match self.http_client {
            Some(client) => client,
            None => reqwest::Client::builder()
                .timeout(self.timeout.unwrap_or(DEFAULT_TIMEOUT))
                .user_agent(USER_AGENT)
                .build()?,
        };

        let http = HttpClient::new(
            http_client,
            api_key,
            self.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL),
            self.max_retries.unwrap_or(DEFAULT_MAX_RETRIES),
        )?;

        Ok(Client { http })
    }
}

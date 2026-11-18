use std::fmt::{self, Debug, Formatter};
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use url::Url;

use crate::error::{DeadlockApiError, Result};
use crate::limiter::{Bucket, Limiter, Priority};
use crate::retry::retry_after;

pub const DEFAULT_BASE_URL: &str = "https://api.deadlock-api.com";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const API_KEY_HEADER: &str = "X-API-KEY";

pub(crate) struct ApiRequest {
    pub(crate) path: &'static str,
    pub(crate) query: Vec<(&'static str, String)>,
    pub(crate) bucket: Bucket,
}

#[must_use]
pub struct DeadlockClientBuilder {
    http: Option<reqwest::Client>,
    api_key: Option<String>,
    base_url: String,
    limiter: Option<Limiter>,
}

impl DeadlockClientBuilder {
    pub fn new() -> Self {
        Self {
            http: None,
            api_key: None,
            base_url: DEFAULT_BASE_URL.to_owned(),
            limiter: None,
        }
    }

    pub fn http(mut self, client: reqwest::Client) -> Self {
        self.http = Some(client);
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        self.api_key = (!key.is_empty()).then_some(key);
        self
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        url.trim_end_matches('/').clone_into(&mut self.base_url);
        self
    }

    pub fn limiter(mut self, limiter: Limiter) -> Self {
        self.limiter = Some(limiter);
        self
    }

    pub fn build(self) -> Result<DeadlockClient> {
        let has_key = self.api_key.is_some();

        let api_key = match self.api_key {
            Some(key) => {
                let mut value = HeaderValue::from_str(&key)?;
                value.set_sensitive(true);
                Some(value)
            },
            None => None,
        };

        let invalid = |source| DeadlockApiError::InvalidBaseUrl {
            url: self.base_url.clone(),
            source,
        };

        let mut base_url =
            Url::parse(&self.base_url).map_err(|e| invalid(Some(e)))?;

        if !matches!(base_url.scheme(), "http" | "https")
            || base_url.cannot_be_a_base()
            || base_url.query().is_some()
            || base_url.fragment().is_some()
        {
            return Err(invalid(None));
        }

        let path = base_url.path().trim_end_matches('/').to_owned();
        base_url.set_path(&path);

        Ok(DeadlockClient {
            http: self.http.unwrap_or_default(),
            base_url,
            api_key,
            limiter: self.limiter.unwrap_or_else(|| Limiter::new(has_key)),
        })
    }
}

impl Default for DeadlockClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for DeadlockClientBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeadlockClientBuilder")
            .field("http", &self.http)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .field("base_url", &self.base_url)
            .field("limiter", &self.limiter)
            .finish()
    }
}

#[derive(Debug)]
pub struct DeadlockClient {
    http: reqwest::Client,
    base_url: Url,
    api_key: Option<HeaderValue>,
    limiter: Limiter,
}

impl DeadlockClient {
    pub fn builder() -> DeadlockClientBuilder {
        DeadlockClientBuilder::new()
    }

    #[must_use]
    pub const fn limiter(&self) -> &Limiter {
        &self.limiter
    }

    /// `path_and_query` must start with `/`; its query is sent as given, so it must
    /// already be canonical and percent-encoded.
    pub async fn get_raw(
        &self,
        path_and_query: &str,
        bucket: Bucket,
        priority: Priority,
    ) -> Result<String> {
        if !path_and_query.starts_with('/') {
            return Err(DeadlockApiError::InvalidPath {
                path: path_and_query.to_owned(),
            });
        }

        let (path, query) = match path_and_query.split_once('?') {
            Some((path, query)) => (path, Some(query)),
            None => (path_and_query, None),
        };

        self.execute(self.http.get(self.endpoint(path, query)), bucket, priority)
            .await
    }

    fn endpoint(&self, path: &str, query: Option<&str>) -> Url {
        let mut url = self.base_url.clone();
        url.set_path(&format!(
            "{}{path}",
            self.base_url.path().trim_end_matches('/')
        ));
        url.set_query(query.filter(|q| !q.is_empty()));
        url
    }

    async fn send(&self, req: &ApiRequest, priority: Priority) -> Result<String> {
        let mut url = self.endpoint(req.path, None);

        if !req.query.is_empty() {
            url.query_pairs_mut().extend_pairs(&req.query);
        }

        self.execute(self.http.get(url), req.bucket, priority).await
    }

    async fn execute(
        &self,
        request: reqwest::RequestBuilder,
        bucket: Bucket,
        priority: Priority,
    ) -> Result<String> {
        self.limiter.acquire(bucket, priority).await?;

        let mut request = request.timeout(REQUEST_TIMEOUT);

        if let Some(key) = &self.api_key {
            request = request.header(API_KEY_HEADER, key.clone());
        }

        let response = request.send().await?;
        let status = response.status();

        if status.is_success() {
            return Ok(response.text().await?);
        }

        match status {
            StatusCode::NOT_FOUND => Err(DeadlockApiError::NotFound),
            StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE => {
                let delay = retry_after(response.headers());
                self.limiter.block(bucket, delay);
                Err(DeadlockApiError::Busy { retry_after: delay })
            },
            _ => Err(DeadlockApiError::Http { status }),
        }
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        req: &ApiRequest,
        priority: Priority,
    ) -> Result<T> {
        let body = self.send(req, priority).await?;

        serde_json::from_str(&body).map_err(|source| DeadlockApiError::Decode {
            path: req.path.to_owned(),
            source,
        })
    }
}

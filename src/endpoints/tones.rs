use crate::client::Client;
use crate::error::Result;
use crate::http::{check_status, json};
use crate::models::{SearchParams, SearchResults, Tone};

impl Client {
    /// Search & filter the public tone library. Heavily rate-limited.
    pub async fn search(&self, params: SearchParams) -> Result<SearchResults> {
        self.maybe_proactive_refresh().await;
        let mut req = self
            .http
            .get(format!("{}/tones/search", self.base_url))
            .headers(self.headers().await);
        if let Some(q) = &params.query {
            req = req.query(&[("query", q)]);
        }
        for gear in &params.gears {
            req = req.query(&[("gears[]", gear)]);
        }
        if let Some(sort) = params.sort {
            req = req.query(&[("sort", sort.as_str())]);
        }
        if let Some(page) = params.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(per_page) = params.per_page {
            req = req.query(&[("per_page", per_page)]);
        }
        let resp = check_status(req.send().await?).await?;
        json(resp).await
    }

    /// Fetch a single tone by id.
    pub async fn tone(&self, id: &str) -> Result<Tone> {
        self.maybe_proactive_refresh().await;
        let req = self
            .http
            .get(format!("{}/tones/{id}", self.base_url))
            .headers(self.headers().await);
        let resp = check_status(req.send().await?).await?;
        json(resp).await
    }

    /// The authenticated user's created tones. Requires an access token.
    pub async fn created(&self, params: SearchParams) -> Result<SearchResults> {
        self.user_tones("created", params).await
    }

    /// The authenticated user's favorited tones. Requires an access token.
    pub async fn favorited(&self, params: SearchParams) -> Result<SearchResults> {
        self.user_tones("favorited", params).await
    }

    /// Shared implementation for `created`/`favorited`.
    async fn user_tones(&self, kind: &str, params: SearchParams) -> Result<SearchResults> {
        if !self.has_access_token().await {
            return Err(crate::Error::Unauthenticated);
        }
        self.maybe_proactive_refresh().await;
        let mut req = self
            .http
            .get(format!("{}/tones/{kind}", self.base_url))
            .headers(self.headers().await);
        if let Some(sort) = params.sort {
            req = req.query(&[("sort", sort.as_str())]);
        }
        if let Some(page) = params.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(per_page) = params.per_page {
            req = req.query(&[("per_page", per_page)]);
        }
        let resp = check_status(req.send().await?).await?;
        json(resp).await
    }
}

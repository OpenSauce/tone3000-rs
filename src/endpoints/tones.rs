use crate::client::Client;
use crate::error::Result;
use crate::http::json;
use crate::models::{ListParams, Page, SearchParams, Tone, ToneId};

impl Client {
    /// Search & filter the public tone library. Heavily rate-limited.
    pub async fn search(&self, params: SearchParams) -> Result<Page<Tone>> {
        let mut req = self.http.get(format!("{}/tones/search", self.base_url));
        if let Some(q) = &params.query {
            req = req.query(&[("query", q)]);
        }
        if !params.gears.is_empty() {
            let joined = params
                .gears
                .iter()
                .map(|g| g.as_str())
                .collect::<Vec<_>>()
                .join("_");
            req = req.query(&[("gears", joined)]);
        }
        if !params.sizes.is_empty() {
            let joined = params
                .sizes
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("_");
            req = req.query(&[("sizes", joined)]);
        }
        if let Some(sort) = params.sort {
            req = req.query(&[("sort", sort.as_str())]);
        }
        if let Some(page) = params.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(page_size) = params.page_size {
            req = req.query(&[("page_size", page_size)]);
        }
        if let Some(arch) = params.architecture {
            req = req.query(&[("architecture", arch)]);
        }
        let resp = self.send(req).await?;
        json(resp).await
    }

    /// Fetch a single tone by id.
    pub async fn tone(&self, id: ToneId) -> Result<Tone> {
        let req = self.http.get(format!("{}/tones/{id}", self.base_url));
        let resp = self.send(req).await?;
        json(resp).await
    }

    /// The authenticated user's created tones.
    pub async fn created(&self, params: ListParams) -> Result<Page<Tone>> {
        self.tone_list("created", params).await
    }

    /// The authenticated user's favorited tones.
    pub async fn favorited(&self, params: ListParams) -> Result<Page<Tone>> {
        self.tone_list("favorited", params).await
    }

    async fn tone_list(&self, kind: &str, params: ListParams) -> Result<Page<Tone>> {
        let mut req = self.http.get(format!("{}/tones/{kind}", self.base_url));
        if let Some(page) = params.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(page_size) = params.page_size {
            req = req.query(&[("page_size", page_size)]);
        }
        let resp = self.send(req).await?;
        json(resp).await
    }
}

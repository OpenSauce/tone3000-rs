use std::future::{Future, IntoFuture};
use std::pin::Pin;

use bytes::Bytes;
use futures_util::StreamExt;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::client::Client;
use crate::error::Result;
use crate::http::json;
use crate::models::{ArchitectureVersion, Model, ModelId, Page, ToneId};

impl Client {
    /// List the models belonging to a tone.
    ///
    /// A tone's detail response does not embed its models, so a detail screen is this
    /// call plus [`Client::tone`].
    pub fn models(&self, tone_id: ToneId) -> ModelList<'_> {
        ModelList::new(self, tone_id)
    }

    /// Fetch a single model by id.
    pub async fn model(&self, id: ModelId) -> Result<Model> {
        let req = self.http.get(format!("{}/models/{id}", self.base_url));
        let resp = self.send(req).await?;
        json(resp).await
    }

    /// Download a model's file into memory.
    pub async fn download_model(&self, model: &Model) -> Result<Bytes> {
        let req = self.http.get(&model.model_url);
        let resp = self.send(req).await?;
        Ok(resp.bytes().await?)
    }

    /// Download a model's `.nam` file as a JSON string.
    ///
    /// Convenience over [`Client::download_model`] for the in-memory path: the
    /// returned `String` can be handed straight to a NAM loader, e.g.
    /// `nam_rs::NamModel::from_json_str(&client.download_model_json(&model).await?)`.
    /// For the on-disk path, prefer [`Client::download_model_to`] into a file and
    /// load it with `nam_rs::NamModel::from_file(path)`.
    pub async fn download_model_json(&self, model: &Model) -> Result<String> {
        let bytes = self.download_model(model).await?;
        Ok(String::from_utf8(bytes.to_vec())?)
    }

    /// Stream a model's file to `writer`, returning the number of bytes written.
    pub async fn download_model_to<W>(&self, model: &Model, writer: &mut W) -> Result<u64>
    where
        W: AsyncWrite + Unpin,
    {
        let req = self.http.get(&model.model_url);
        let resp = self.send(req).await?;
        let mut stream = resp.bytes_stream();
        let mut written: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            writer.write_all(&chunk).await?;
            written += chunk.len() as u64;
        }
        writer.flush().await?;
        Ok(written)
    }
}

/// A pending listing of a tone's models, built by [`Client::models`].
///
/// Await it directly to send the request; every method is optional.
pub struct ModelList<'a> {
    client: &'a Client,
    tone_id: ToneId,
    page: Option<u32>,
    page_size: Option<u32>,
    architecture: Option<ArchitectureVersion>,
}

impl<'a> ModelList<'a> {
    fn new(client: &'a Client, tone_id: ToneId) -> Self {
        Self {
            client,
            tone_id,
            page: None,
            page_size: None,
            architecture: None,
        }
    }

    /// Request a specific 1-based page.
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set how many models a page holds.
    pub fn page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Restrict to models of a given neural architecture version.
    pub fn architecture(mut self, architecture: ArchitectureVersion) -> Self {
        self.architecture = Some(architecture);
        self
    }

    async fn send(self) -> Result<Page<Model>> {
        let mut req = self
            .client
            .http
            .get(format!("{}/models", self.client.base_url))
            .query(&[("tone_id", self.tone_id.to_string())]);
        if let Some(page) = self.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(page_size) = self.page_size {
            req = req.query(&[("page_size", page_size)]);
        }
        if let Some(arch) = &self.architecture {
            req = req.query(&[("architecture", arch.as_str())]);
        }
        let resp = self.client.send(req).await?;
        json(resp).await
    }
}

impl<'a> IntoFuture for ModelList<'a> {
    type Output = Result<Page<Model>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}

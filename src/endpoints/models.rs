use std::future::{Future, IntoFuture};
use std::pin::Pin;

use bytes::Bytes;
use futures_util::StreamExt;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::client::Client;
use crate::error::{Result, TransportResultExt};
use crate::http::json;
use crate::models::{ArchitectureVersion, Model, ModelId, Page, ToneId};

impl Client {
    /// List the models belonging to a tone.
    ///
    /// A tone's detail response does not embed its models, so a detail screen is this
    /// call plus [`Client::tone`].
    ///
    /// # This returns one architecture at a time
    ///
    /// The API defaults to architecture 1 and there is no "all architectures" value, so
    /// a bare call returns only the v1 models. Tone 6298 has 112 on v1 and 111 on v2; this
    /// returns 112 of them.
    ///
    /// This bites hardest alongside [`Tone::models_count`], which reports the
    /// cross-architecture total when the tone came from search — 223 for that tone. A list
    /// screen saying "223 models" whose detail view lists 112 is the same bug seen twice.
    ///
    /// To show everything, request each architecture the tone reports:
    ///
    /// ```no_run
    /// # async fn run(client: tone3000::Client, tone: tone3000::Tone) -> tone3000::Result<()> {
    /// use tone3000::ArchitectureVersion::{Custom, V1, V2};
    ///
    /// let mut models = Vec::new();
    /// for (count, arch) in [
    ///     (tone.a1_models_count, V1),
    ///     (tone.a2_models_count, V2),
    ///     (tone.custom_models_count, Custom),
    /// ] {
    ///     if count > 0 {
    ///         models.extend(client.models(tone.id).architecture(arch).await?.data);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Tone::models_count`]: crate::Tone::models_count
    pub fn models(&self, tone_id: ToneId) -> ModelList {
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
        self.download_url(&model.model_url).await
    }

    /// Download a model file from a `model_url` directly.
    ///
    /// For callers that persisted a URL rather than the whole [`Model`].
    pub async fn download_url(&self, model_url: &str) -> Result<Bytes> {
        let req = self.http.get(model_url);
        let resp = self.send(req).await?;
        resp.bytes().await.transport()
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
        self.download_url_to(&model.model_url, writer).await
    }

    /// Stream a model file from a `model_url` to `writer`, returning bytes written.
    pub async fn download_url_to<W>(&self, model_url: &str, writer: &mut W) -> Result<u64>
    where
        W: AsyncWrite + Unpin,
    {
        let req = self.http.get(model_url);
        let resp = self.send(req).await?;
        let mut stream = resp.bytes_stream();
        let mut written: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.transport()?;
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
#[must_use = "a request builder does nothing until awaited"]
#[derive(Clone)]
pub struct ModelList {
    client: Client,
    tone_id: ToneId,
    page: Option<u32>,
    page_size: Option<u32>,
    architecture: Option<ArchitectureVersion>,
}

impl ModelList {
    fn new(client: &Client, tone_id: ToneId) -> Self {
        Self {
            client: client.clone(),
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

impl IntoFuture for ModelList {
    type Output = Result<Page<Model>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}

impl std::fmt::Debug for ModelList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelList")
            .field("tone_id", &self.tone_id)
            .field("page", &self.page)
            .field("page_size", &self.page_size)
            .field("architecture", &self.architecture)
            .finish_non_exhaustive()
    }
}

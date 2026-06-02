use bytes::Bytes;
use futures_util::StreamExt;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::client::Client;
use crate::error::Result;
use crate::http::{check_status, json};
use crate::models::Model;

impl Client {
    /// List the models belonging to a tone.
    pub async fn models(&self, tone_id: &str) -> Result<Vec<Model>> {
        self.maybe_proactive_refresh().await;
        let req = self
            .http
            .get(format!("{}/models", self.base_url))
            .query(&[("tone_id", tone_id)])
            .headers(self.headers().await);
        let resp = check_status(req.send().await?).await?;
        json(resp).await
    }

    /// Fetch a single model by id.
    pub async fn model(&self, id: &str) -> Result<Model> {
        self.maybe_proactive_refresh().await;
        let req = self
            .http
            .get(format!("{}/models/{id}", self.base_url))
            .headers(self.headers().await);
        let resp = check_status(req.send().await?).await?;
        json(resp).await
    }

    /// Download a model's file into memory.
    pub async fn download_model(&self, model: &Model) -> Result<Bytes> {
        self.maybe_proactive_refresh().await;
        let req = self
            .http
            .get(&model.model_url)
            .headers(self.headers().await);
        let resp = check_status(req.send().await?).await?;
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
        self.maybe_proactive_refresh().await;
        let req = self
            .http
            .get(&model.model_url)
            .headers(self.headers().await);
        let resp = check_status(req.send().await?).await?;
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

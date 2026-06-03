use bytes::Bytes;
use futures_util::StreamExt;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::client::Client;
use crate::error::Result;
use crate::http::json;
use crate::models::{Model, ModelId, ModelListParams, Page, ToneId};

impl Client {
    /// List the models belonging to a tone.
    pub async fn models(&self, tone_id: ToneId, params: ModelListParams) -> Result<Page<Model>> {
        let mut req = self
            .http
            .get(format!("{}/models", self.base_url))
            .query(&[("tone_id", tone_id.to_string())]);
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

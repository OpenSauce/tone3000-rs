use crate::client::Client;
use crate::error::Result;
use crate::http::{check_status, json};
use crate::models::Model;

impl Client {
    /// List the models belonging to a tone.
    pub async fn models(&self, tone_id: &str) -> Result<Vec<Model>> {
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
        let req = self
            .http
            .get(format!("{}/models/{id}", self.base_url))
            .headers(self.headers().await);
        let resp = check_status(req.send().await?).await?;
        json(resp).await
    }
}

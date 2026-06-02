use crate::client::Client;
use crate::error::Result;
use crate::http::{check_status, json};
use crate::models::{User, UserListParams};

impl Client {
    /// Public list of users, sorted by metrics.
    pub async fn users(&self, params: UserListParams) -> Result<Vec<User>> {
        let mut req = self
            .http
            .get(format!("{}/users", self.base_url))
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

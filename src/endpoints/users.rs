use crate::client::Client;
use crate::error::Result;
use crate::http::json;
use crate::models::{Page, PublicUser, User, UserListParams};

impl Client {
    /// Public list of users, sorted by metrics.
    pub async fn users(&self, params: UserListParams) -> Result<Page<PublicUser>> {
        let mut req = self.http.get(format!("{}/users", self.base_url));
        if let Some(sort) = params.sort {
            req = req.query(&[("sort", sort.as_str())]);
        }
        if let Some(page) = params.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(page_size) = params.page_size {
            req = req.query(&[("page_size", page_size)]);
        }
        if let Some(q) = &params.query {
            req = req.query(&[("query", q)]);
        }
        let resp = self.send(req).await?;
        json(resp).await
    }

    /// The authenticated user's profile.
    pub async fn user(&self) -> Result<User> {
        let req = self.http.get(format!("{}/user", self.base_url));
        let resp = self.send(req).await?;
        json(resp).await
    }
}

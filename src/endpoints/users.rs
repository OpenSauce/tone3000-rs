use crate::client::Client;
use crate::error::Result;
use crate::http::json;
use crate::models::{User, UserListParams};

impl Client {
    /// Public list of users, sorted by metrics.
    pub async fn users(&self, params: UserListParams) -> Result<Vec<User>> {
        let mut req = self.http.get(format!("{}/users", self.base_url));
        if let Some(sort) = params.sort {
            req = req.query(&[("sort", sort.as_str())]);
        }
        if let Some(page) = params.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(per_page) = params.per_page {
            req = req.query(&[("per_page", per_page)]);
        }
        let resp = self.send(req).await?;
        json(resp).await
    }

    /// The authenticated user's profile. Requires an access token.
    pub async fn user(&self) -> Result<User> {
        if !self.has_access_token().await {
            return Err(crate::Error::Unauthenticated);
        }
        let req = self.http.get(format!("{}/user", self.base_url));
        let resp = self.send(req).await?;
        json(resp).await
    }
}

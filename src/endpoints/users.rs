use std::future::{Future, IntoFuture};
use std::pin::Pin;

use crate::client::Client;
use crate::error::Result;
use crate::http::json;
use crate::models::{Page, PublicUser, User, UserSort};

impl Client {
    /// The public user directory, sorted by metrics.
    pub fn users(&self) -> UserList {
        UserList::new(self)
    }

    /// The authenticated user's profile.
    pub async fn user(&self) -> Result<User> {
        let req = self.http.get(format!("{}/user", self.base_url));
        let resp = self.send(req).await?;
        json(resp).await
    }
}

/// A pending listing of the public user directory, built by [`Client::users`].
///
/// Await it directly to send the request; every method is optional.
#[must_use = "a request builder does nothing until awaited"]
#[derive(Clone)]
pub struct UserList {
    client: Client,
    query: Option<String>,
    sort: Option<UserSort>,
    page: Option<u32>,
    page_size: Option<u32>,
}

impl UserList {
    fn new(client: &Client) -> Self {
        Self {
            client: client.clone(),
            query: None,
            sort: None,
            page: None,
            page_size: None,
        }
    }

    /// Free-text search over usernames.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Order the results.
    pub fn sort(mut self, sort: UserSort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Request a specific 1-based page.
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set how many users a page holds.
    pub fn page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    async fn send(self) -> Result<Page<PublicUser>> {
        let mut req = self
            .client
            .http
            .get(format!("{}/users", self.client.base_url));
        if let Some(sort) = self.sort {
            req = req.query(&[("sort", sort.as_str())]);
        }
        if let Some(page) = self.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(page_size) = self.page_size {
            req = req.query(&[("page_size", page_size)]);
        }
        if let Some(q) = &self.query {
            req = req.query(&[("query", q)]);
        }
        let resp = self.client.send(req).await?;
        json(resp).await
    }
}

impl IntoFuture for UserList {
    type Output = Result<Page<PublicUser>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}

impl std::fmt::Debug for UserList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserList")
            .field("query", &self.query)
            .field("sort", &self.sort)
            .field("page", &self.page)
            .field("page_size", &self.page_size)
            .finish_non_exhaustive()
    }
}

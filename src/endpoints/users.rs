use std::future::{Future, IntoFuture};
use std::pin::Pin;

use crate::client::Client;
use crate::error::Result;
use crate::http::json;
use crate::models::{Page, PublicUser, User, UserSort};

impl Client {
    /// The public user directory, sorted by metrics.
    pub fn users(&self) -> UserList<'_> {
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
pub struct UserList<'a> {
    client: &'a Client,
    query: Option<String>,
    sort: Option<UserSort>,
    page: Option<u32>,
    page_size: Option<u32>,
}

impl<'a> UserList<'a> {
    fn new(client: &'a Client) -> Self {
        Self {
            client,
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

impl<'a> IntoFuture for UserList<'a> {
    type Output = Result<Page<PublicUser>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}

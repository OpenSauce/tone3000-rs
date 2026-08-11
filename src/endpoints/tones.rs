use std::future::{Future, IntoFuture};
use std::pin::Pin;

use crate::client::Client;
use crate::error::Result;
use crate::http::json;
use crate::models::{Format, Gear, Page, Size, Tone, ToneId, ToneSort};

impl Client {
    /// Browse and search the public tone library.
    ///
    /// With no [`query`](ToneSearch::query) this is the browse endpoint — the tone
    /// library sorted by [`ToneSort`] — and with one it is search. Both are the same
    /// API call. Heavily rate-limited.
    ///
    /// ```no_run
    /// # async fn run(client: tone3000::Client) -> tone3000::Result<()> {
    /// use tone3000::{Gear, ToneSort};
    /// // The top of the library, 24 at a time.
    /// let page = client.tones().sort(ToneSort::DownloadsAllTime).page_size(24).await?;
    /// // Or a search.
    /// let page = client.tones().query("plexi").gear(Gear::Amp).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn tones(&self) -> ToneSearch<'_> {
        ToneSearch::new(self)
    }

    /// Fetch a single tone by id.
    ///
    /// The response does not embed the tone's models; list those with
    /// [`Client::models`].
    pub async fn tone(&self, id: ToneId) -> Result<Tone> {
        let req = self.http.get(format!("{}/tones/{id}", self.base_url));
        let resp = self.send(req).await?;
        json(resp).await
    }

    /// The authenticated user's created tones.
    pub fn created(&self) -> ToneList<'_> {
        ToneList::new(self, "created")
    }

    /// The authenticated user's favorited tones.
    pub fn favorited(&self) -> ToneList<'_> {
        ToneList::new(self, "favorited")
    }
}

/// A pending browse/search of the tone library, built by [`Client::tones`].
///
/// Await it directly to send the request; every method is optional.
pub struct ToneSearch<'a> {
    client: &'a Client,
    query: Option<String>,
    gears: Vec<Gear>,
    format: Option<Format>,
    sizes: Vec<Size>,
    tags: Vec<String>,
    makes: Vec<String>,
    creators: Vec<String>,
    sort: Option<ToneSort>,
    page: Option<u32>,
    page_size: Option<u32>,
    architecture: Option<u32>,
}

impl<'a> ToneSearch<'a> {
    fn new(client: &'a Client) -> Self {
        Self {
            client,
            query: None,
            gears: Vec::new(),
            format: None,
            sizes: Vec::new(),
            tags: Vec::new(),
            makes: Vec::new(),
            creators: Vec::new(),
            sort: None,
            page: None,
            page_size: None,
            architecture: None,
        }
    }

    /// Free-text search terms. Omit to browse rather than search.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Restrict to a gear category. Repeatable; multiple values are OR'd.
    pub fn gear(mut self, gear: Gear) -> Self {
        self.gears.push(gear);
        self
    }

    /// Restrict to several gear categories at once. OR'd.
    pub fn gears(mut self, gears: impl IntoIterator<Item = Gear>) -> Self {
        self.gears.extend(gears);
        self
    }

    /// Restrict to a model format. Filtering for IRs goes here, not through
    /// [`gear`](Self::gear).
    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    /// Restrict to a model size class. Repeatable; multiple values are OR'd.
    pub fn size(mut self, size: Size) -> Self {
        self.sizes.push(size);
        self
    }

    /// Restrict to several size classes at once. OR'd.
    pub fn sizes(mut self, sizes: impl IntoIterator<Item = Size>) -> Self {
        self.sizes.extend(sizes);
        self
    }

    /// Restrict to a tag name, matched exactly. Repeatable; OR'd.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Restrict to several tag names at once. OR'd.
    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    /// Restrict to a make name, matched exactly. Repeatable; OR'd.
    pub fn make(mut self, make: impl Into<String>) -> Self {
        self.makes.push(make.into());
        self
    }

    /// Restrict to several make names at once. OR'd.
    pub fn makes<I, S>(mut self, makes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.makes.extend(makes.into_iter().map(Into::into));
        self
    }

    /// Restrict to a creator's username, matched exactly. Repeatable; OR'd.
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creators.push(creator.into());
        self
    }

    /// Restrict to several creator usernames at once. OR'd.
    pub fn creators<I, S>(mut self, creators: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.creators.extend(creators.into_iter().map(Into::into));
        self
    }

    /// Order the results.
    pub fn sort(mut self, sort: ToneSort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Request a specific 1-based page.
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set how many tones a page holds.
    pub fn page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Restrict to tones with models of a given neural architecture version.
    pub fn architecture(mut self, architecture: u32) -> Self {
        self.architecture = Some(architecture);
        self
    }

    async fn send(self) -> Result<Page<Tone>> {
        let mut req = self
            .client
            .http
            .get(format!("{}/tones/search", self.client.base_url));
        if let Some(q) = &self.query {
            req = req.query(&[("query", q)]);
        }
        if !self.gears.is_empty() {
            req = req.query(&[("gears", join(self.gears.iter().map(Gear::as_str), "_"))]);
        }
        if !self.sizes.is_empty() {
            req = req.query(&[("sizes", join(self.sizes.iter().map(Size::as_str), "_"))]);
        }
        if let Some(format) = &self.format {
            req = req.query(&[("format", format.as_str())]);
        }
        // NOTE: tags/makes join with `_`, but creators joins with `,`. The asymmetry is in
        // the API — see tone-3000/api src/tone3000-client.ts, buildSearchTonesQuery.
        if !self.tags.is_empty() {
            req = req.query(&[("tags", self.tags.join("_"))]);
        }
        if !self.makes.is_empty() {
            req = req.query(&[("makes", self.makes.join("_"))]);
        }
        if !self.creators.is_empty() {
            req = req.query(&[("creators", self.creators.join(","))]);
        }
        if let Some(sort) = self.sort {
            req = req.query(&[("sort", sort.as_str())]);
        }
        if let Some(page) = self.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(page_size) = self.page_size {
            req = req.query(&[("page_size", page_size)]);
        }
        if let Some(arch) = self.architecture {
            req = req.query(&[("architecture", arch)]);
        }
        let resp = self.client.send(req).await?;
        json(resp).await
    }
}

impl<'a> IntoFuture for ToneSearch<'a> {
    type Output = Result<Page<Tone>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}

/// A pending listing of the authenticated user's tones, built by [`Client::created`] or
/// [`Client::favorited`].
///
/// Await it directly to send the request; every method is optional.
pub struct ToneList<'a> {
    client: &'a Client,
    kind: &'static str,
    page: Option<u32>,
    page_size: Option<u32>,
}

impl<'a> ToneList<'a> {
    fn new(client: &'a Client, kind: &'static str) -> Self {
        Self {
            client,
            kind,
            page: None,
            page_size: None,
        }
    }

    /// Request a specific 1-based page.
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set how many tones a page holds.
    pub fn page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    async fn send(self) -> Result<Page<Tone>> {
        let mut req = self
            .client
            .http
            .get(format!("{}/tones/{}", self.client.base_url, self.kind));
        if let Some(page) = self.page {
            req = req.query(&[("page", page)]);
        }
        if let Some(page_size) = self.page_size {
            req = req.query(&[("page_size", page_size)]);
        }
        let resp = self.client.send(req).await?;
        json(resp).await
    }
}

impl<'a> IntoFuture for ToneList<'a> {
    type Output = Result<Page<Tone>>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.send())
    }
}

/// Join wire values with a separator.
fn join<'s>(values: impl Iterator<Item = &'s str>, sep: &str) -> String {
    values.collect::<Vec<_>>().join(sep)
}

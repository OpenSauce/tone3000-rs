//! Async Rust client for the [TONE3000](https://www.tone3000.com) API (v1).
//!
//! ```no_run
//! # async fn run() -> tone3000::Result<()> {
//! use tone3000::{Client, SearchParams};
//! // Every call needs an OAuth access token; see `oauth`/`exchange_code`/`refresh`.
//! let client = Client::builder("t3k_pub_your_key")
//!     .access_token("user_access_token")
//!     .build();
//! let results = client.search(SearchParams { query: Some("plexi".into()), ..Default::default() }).await?;
//! for tone in results.data {
//!     println!("{}: {}", tone.id, tone.title);
//! }
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]

mod client;
mod endpoints;
mod error;
mod http;
pub mod models;
pub mod oauth;
pub mod pkce;

pub use client::{Client, ClientBuilder, DEFAULT_BASE_URL};
pub use error::{Error, Result};
pub use models::{
    EmbeddedUser, Gear, License, ListParams, Make, MakeId, Model, ModelId, ModelListParams, Page,
    Platform, PublicUser, SearchParams, Size, Tag, TagId, Tokens, Tone, ToneId, ToneSort, User,
    UserId, UserListParams, UserSort,
};
pub use oauth::{Prompt, authorize_url};
pub use pkce::{Pkce, generate as generate_pkce};

//! Async Rust client for the [TONE3000](https://www.tone3000.com) API (v1).
//!
//! ```no_run
//! # async fn run() -> tone3000::Result<()> {
//! use tone3000::{Client, SearchParams};
//! let client = Client::new("t3k_pub_your_key");
//! let results = client.search(SearchParams { query: Some("plexi".into()), ..Default::default() }).await?;
//! for tone in results.items {
//!     println!("{}: {}", tone.id, tone.name);
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
    Metrics, Model, SearchParams, SearchResults, Sort, Tokens, Tone, User, UserListParams,
};
pub use oauth::{Prompt, authorize_url};
pub use pkce::{Pkce, generate as generate_pkce};

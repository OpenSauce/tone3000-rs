//! Async Rust client for the [TONE3000](https://www.tone3000.com) API (v1).
//!
//! See [`Client`] for the entry point.

#![forbid(unsafe_code)]

mod client;
mod error;
mod http;

pub use client::{Client, ClientBuilder, DEFAULT_BASE_URL};
pub use error::{Error, Result};

pub mod models;

pub use models::{
    Metrics, Model, SearchParams, SearchResults, Sort, Tokens, Tone, User, UserListParams,
};

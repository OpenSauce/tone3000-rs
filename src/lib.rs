//! Async Rust client for the [TONE3000](https://www.tone3000.com) API (v1).
//!
//! See [`Client`] for the entry point.

#![forbid(unsafe_code)]

mod error;

pub use error::{Error, Result};

pub mod models;

pub use models::{Metrics, Model, SearchParams, SearchResults, Sort, Tone, User, UserListParams};

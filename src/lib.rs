#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod client;
mod endpoints;
mod error;
mod http;
pub mod models;
pub mod oauth;
pub mod pkce;

pub use client::{Client, ClientBuilder, DEFAULT_BASE_URL};
pub use endpoints::models::ModelList;
pub use endpoints::tones::{ToneList, ToneSearch};
pub use endpoints::users::UserList;
pub use error::{Error, HttpError, Result};
pub use models::{
    ArchitectureVersion, EmbeddedUser, Format, Gear, License, Make, MakeId, Model, ModelId, Page,
    PublicUser, Size, Tag, TagId, Tokens, Tone, ToneId, ToneSort, User, UserId, UserSort,
};
pub use oauth::{AuthorizeOptions, Prompt, authorize_url};
pub use pkce::{Pkce, generate as generate_pkce};

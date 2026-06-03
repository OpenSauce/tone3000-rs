//! Typed API models. All structs are lenient: unexpected fields are ignored and
//! non-essential fields default, so a single API change never sinks a whole response.

mod enums;
mod ids;
mod model;
mod page;
mod token;
mod tone;
mod user;

pub use model::Model;
pub use tone::{Metrics, SearchParams, SearchResults, Sort, Tokens, Tone};
pub use user::{User, UserListParams};

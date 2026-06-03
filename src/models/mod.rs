//! Typed API models. All structs are lenient: unexpected fields are ignored and
//! non-identity fields default, so a single API change never sinks a whole response.

mod enums;
mod ids;
mod model;
mod page;
mod token;
mod tone;
mod user;

pub use enums::{Gear, License, Platform, Size, ToneSort, UserSort};
pub use ids::{MakeId, ModelId, TagId, ToneId, UserId};
pub use model::{Model, ModelListParams};
pub use page::Page;
pub use token::Tokens;
pub use tone::{EmbeddedUser, ListParams, Make, SearchParams, Tag, Tone};
pub use user::{PublicUser, User, UserListParams};

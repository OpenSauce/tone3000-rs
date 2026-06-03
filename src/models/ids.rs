//! Strongly-typed identifiers. Tone/model/make/tag ids are integers; user ids are
//! UUID strings. Newtypes prevent accidentally passing one id where another is expected.

use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! int_id {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<u64> for $name {
            fn from(v: u64) -> Self {
                Self(v)
            }
        }
    };
}

int_id!(
    /// Identifier for a [`crate::Tone`].
    ToneId
);
int_id!(
    /// Identifier for a [`crate::Model`].
    ModelId
);
int_id!(
    /// Identifier for a gear make.
    MakeId
);
int_id!(
    /// Identifier for a tag.
    TagId
);

/// Identifier for a user (a UUID string).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub String);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(v: String) -> Self {
        Self(v)
    }
}

impl From<&str> for UserId {
    fn from(v: &str) -> Self {
        Self(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_id_is_transparent() {
        assert_eq!(serde_json::to_string(&ToneId(51949)).unwrap(), "51949");
        let id: ToneId = serde_json::from_str("51949").unwrap();
        assert_eq!(id, ToneId(51949));
        assert_eq!(ToneId(7).to_string(), "7");
        assert_eq!(ToneId::from(7u64), ToneId(7));
    }

    #[test]
    fn user_id_is_transparent_string() {
        let raw = "\"57af4bb9-80fe-43be-93b3-e3c9e9d92717\"";
        let id: UserId = serde_json::from_str(raw).unwrap();
        assert_eq!(id, UserId("57af4bb9-80fe-43be-93b3-e3c9e9d92717".into()));
        assert_eq!(serde_json::to_string(&id).unwrap(), raw);
    }
}

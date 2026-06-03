use serde::{Deserialize, Serialize};

use super::enums::UserSort;
use super::ids::UserId;

/// The authenticated user's profile (`GET /user`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: UserId,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub links: Option<Vec<String>>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub url: String,
}

/// A public user as returned by the user list (`GET /users`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicUser {
    pub id: UserId,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub links: Option<Vec<String>>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub downloads_count: u64,
    #[serde(default)]
    pub favorites_count: u64,
    #[serde(default)]
    pub models_count: u64,
    #[serde(default)]
    pub tones_count: u64,
    #[serde(default)]
    pub url: String,
}

/// Parameters for [`crate::Client::users`].
#[derive(Debug, Clone, Default)]
pub struct UserListParams {
    pub sort: Option<UserSort>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub query: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_parses_and_tolerates_nulls() {
        let json = r#"{ "id": "ec47", "username": "testuser", "bio": null, "links": null }"#;
        let u: User = serde_json::from_str(json).unwrap();
        assert_eq!(u.id, UserId("ec47".into()));
        assert_eq!(u.username, "testuser");
        assert!(u.bio.is_none());
    }

    #[test]
    fn public_user_parses_counts() {
        let json =
            r#"{ "id": "6d6f", "username": "akka5", "tones_count": 153, "models_count": 661 }"#;
        let p: PublicUser = serde_json::from_str(json).unwrap();
        assert_eq!(p.username, "akka5");
        assert_eq!(p.tones_count, 153);
    }
}

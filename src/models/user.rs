use serde::{Deserialize, Serialize};

use super::ids::UserId;

/// The account behind the current access token, as returned by
/// [`Client::user`](crate::Client::user).
///
/// The caller's own profile. For anyone else, see [`PublicUser`].
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct User {
    /// The account id. Matches [`Tone::user_id`](crate::Tone::user_id) on tones you created.
    pub id: UserId,
    /// Display name, as shown on TONE3000.
    #[serde(default)]
    pub username: String,
    /// Profile picture, if set.
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// The self-written profile blurb.
    #[serde(default)]
    pub bio: Option<String>,
    /// Links the user added to their profile.
    #[serde(default, deserialize_with = "crate::models::de_null_as_default")]
    pub links: Vec<String>,
    /// When the account was created, as the API's raw timestamp string.
    #[serde(default)]
    pub created_at: Option<String>,
    /// When the profile was last edited, as the API's raw timestamp string.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// The profile page on tone3000.com.
    #[serde(default)]
    pub url: String,
}

/// Another account as it appears in the public directory, from
/// [`Client::users`](crate::Client::users).
///
/// Carries contribution metrics that [`User`] does not, and omits private detail.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PublicUser {
    /// The account id.
    pub id: UserId,
    /// Display name. This is the value [`ToneSearch::creator`] matches against, exactly.
    ///
    /// [`ToneSearch::creator`]: crate::ToneSearch::creator
    #[serde(default)]
    pub username: String,
    /// The self-written profile blurb.
    #[serde(default)]
    pub bio: Option<String>,
    /// Links the user added to their profile.
    #[serde(default, deserialize_with = "crate::models::de_null_as_default")]
    pub links: Vec<String>,
    /// Profile picture, if set.
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// Total downloads across everything they have published.
    #[serde(default)]
    pub downloads_count: u64,
    /// Total favourites their tones have received.
    #[serde(default)]
    pub favorites_count: u64,
    /// Individual model files they have published.
    #[serde(default)]
    pub models_count: u64,
    /// Capture projects they have published. Usually far fewer than
    /// [`models_count`](Self::models_count), since one tone holds many models.
    #[serde(default)]
    pub tones_count: u64,
    /// The profile page on tone3000.com.
    #[serde(default)]
    pub url: String,
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

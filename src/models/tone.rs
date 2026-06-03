use serde::{Deserialize, Serialize};

use super::enums::{Gear, License, Platform, Size, ToneSort};
use super::ids::{MakeId, TagId, ToneId, UserId};

/// A user embedded in a tone payload.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddedUser {
    pub id: UserId,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub url: String,
}

/// A gear make. `id` is absent in search results (RPC) and present in tone detail.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Make {
    #[serde(default)]
    pub id: Option<MakeId>,
    #[serde(default)]
    pub name: String,
}

/// A tag. `id` is absent in search results (RPC) and present in tone detail.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tag {
    #[serde(default)]
    pub id: Option<TagId>,
    #[serde(default)]
    pub name: String,
}

/// A community tone (a capture/profile grouping one or more models).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tone {
    pub id: ToneId,
    pub user_id: UserId,
    #[serde(default)]
    pub user: Option<EmbeddedUser>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub gear: Option<Gear>,
    #[serde(default)]
    pub platform: Option<Platform>,
    #[serde(default)]
    pub license: Option<License>,
    #[serde(default)]
    pub sizes: Vec<Size>,
    #[serde(default, deserialize_with = "crate::models::de_null_as_default")]
    pub images: Vec<String>,
    #[serde(default)]
    pub links: Option<Vec<String>>,
    #[serde(default)]
    pub makes: Vec<Make>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub models_count: u64,
    #[serde(default)]
    pub favorites_count: u64,
    #[serde(default)]
    pub downloads_count: u64,
    #[serde(default)]
    pub a1_models_count: u64,
    #[serde(default)]
    pub a2_models_count: u64,
    #[serde(default)]
    pub irs_count: u64,
    #[serde(default)]
    pub custom_models_count: u64,
}

/// Parameters for [`crate::Client::search`].
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub query: Option<String>,
    pub gears: Vec<Gear>,
    pub sizes: Vec<Size>,
    pub sort: Option<ToneSort>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub architecture: Option<u32>,
}

/// Parameters for [`crate::Client::created`] / [`crate::Client::favorited`].
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_parses_core_fields() {
        let json = r#"{
            "id": 51949, "user_id": "57af", "title": "Plexi 51", "gear": "amp",
            "license": "t3k", "platform": "nam", "makes": [{"name": "Marshall Plexi"}],
            "user": {"id": "57af", "username": "brucew", "url": "u"},
            "models_count": 6, "a1_models_count": 3
        }"#;
        let tone: Tone = serde_json::from_str(json).unwrap();
        assert_eq!(tone.id, ToneId(51949));
        assert_eq!(tone.title, "Plexi 51");
        assert_eq!(tone.gear, Some(Gear::Amp));
        assert_eq!(tone.license, Some(License::T3k));
        assert_eq!(tone.makes[0].name, "Marshall Plexi");
        assert_eq!(tone.user.as_ref().unwrap().username, "brucew");
        assert_eq!(tone.models_count, 6);
        assert_eq!(tone.a1_models_count, 3);
    }

    #[test]
    fn tone_tolerates_extra_and_missing_fields() {
        let json = r#"{ "id": 1, "user_id": "u", "unexpected": 42 }"#;
        let tone: Tone = serde_json::from_str(json).unwrap();
        assert_eq!(tone.id, ToneId(1));
        assert_eq!(tone.title, "");
        assert!(tone.gear.is_none());
        assert_eq!(tone.downloads_count, 0);
    }
}

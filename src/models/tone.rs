use serde::{Deserialize, Serialize};

use super::enums::{Format, Gear, License, Size};
use super::ids::{MakeId, TagId, ToneId, UserId};

/// The creator stub attached to a [`Tone`] payload.
///
/// Enough to credit and link a tone's author; fetch [`Client::users`] for full profiles.
///
/// [`Client::users`]: crate::Client::users
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct EmbeddedUser {
    /// The creator's account id.
    pub id: UserId,
    /// Display name, as shown on TONE3000.
    #[serde(default)]
    pub username: String,
    /// Profile picture, if the user set one.
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// The creator's public profile page on tone3000.com.
    #[serde(default)]
    pub url: String,
}

/// A piece of real-world gear a tone captures, e.g. "Mesa Boogie Badlander".
///
/// A tone may list several — the amp and the cabinet of a full rig, say.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Make {
    /// Absent in search results, present in tone detail — the two endpoints return
    /// different shapes. Use [`name`](Self::name) for filtering, which works in both.
    #[serde(default)]
    pub id: Option<MakeId>,
    /// The make's name. This is the value [`ToneSearch::make`] matches against.
    ///
    /// [`ToneSearch::make`]: crate::ToneSearch::make
    #[serde(default)]
    pub name: String,
}

/// A free-form label a creator attached to a tone, e.g. "1970s" or "high-gain".
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Tag {
    /// Absent in search results, present in tone detail. Use [`name`](Self::name) for
    /// filtering, which works in both.
    #[serde(default)]
    pub id: Option<TagId>,
    /// The tag text. This is the value [`ToneSearch::tag`] matches against, exactly.
    ///
    /// [`ToneSearch::tag`]: crate::ToneSearch::tag
    #[serde(default)]
    pub name: String,
}

/// A community capture project: one piece of gear, captured once, published under a
/// title and licence.
///
/// A tone is not itself downloadable — it owns one or more [`Model`]s, which are. Fetch
/// them with [`Client::models`].
///
/// # Two shapes, one type
///
/// Search results and tone detail return *different* payloads, and this struct covers
/// both. [`sizes`](Self::sizes), [`license`](Self::license), [`links`](Self::links) and
/// [`is_public`](Self::is_public) appear only in detail — in a tone that came from
/// [`Client::tones`] they are empty or `None`, not absent from the server's records. Call
/// [`Client::tone`] when you need them.
///
/// [`Model`]: crate::Model
/// [`Client::models`]: crate::Client::models
/// [`Client::tone`]: crate::Client::tone
/// [`Client::tones`]: crate::Client::tones
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Tone {
    /// Stable identifier, used by [`Client::tone`](crate::Client::tone) and
    /// [`Client::models`](crate::Client::models).
    pub id: ToneId,
    /// The creator's account id.
    pub user_id: UserId,
    /// The creator, embedded. Present in both search results and detail.
    #[serde(default)]
    pub user: Option<EmbeddedUser>,
    /// The creator's title for the tone, e.g. "Mesa Boogie Badlander 100W EL34".
    #[serde(default)]
    pub title: String,
    /// The creator's write-up: rig details, capture chain, how it's meant to be used.
    #[serde(default)]
    pub description: Option<String>,
    /// What kind of thing was captured — amp, pedal, cabinet.
    ///
    /// Distinct from [`format`](Self::format), which is how the capture is encoded.
    #[serde(default)]
    pub gear: Option<Gear>,
    /// The file format its models are published in, e.g. NAM or an impulse response.
    #[serde(default)]
    pub format: Option<Format>,
    /// Reuse terms the creator published under. **Detail only** — `None` in search results.
    ///
    /// Check this before redistributing a downloaded model or shipping it with an app.
    #[serde(default)]
    pub license: Option<License>,
    /// The distinct size classes across this tone's models. **Detail only** — empty in
    /// search results.
    #[serde(default, deserialize_with = "crate::models::de_null_as_default")]
    pub sizes: Vec<Size>,
    /// Creator-uploaded photographs of the rig.
    #[serde(default, deserialize_with = "crate::models::de_null_as_default")]
    pub images: Vec<String>,
    /// Related URLs the creator added, typically demo videos. **Detail only** — empty in
    /// search results.
    #[serde(default, deserialize_with = "crate::models::de_null_as_default")]
    pub links: Vec<String>,
    /// The real-world gear captured. May list several for a full rig.
    #[serde(default, deserialize_with = "crate::models::de_null_as_default")]
    pub makes: Vec<Make>,
    /// Creator-applied labels.
    #[serde(default, deserialize_with = "crate::models::de_null_as_default")]
    pub tags: Vec<Tag>,
    /// Whether the tone is publicly listed. **Detail only** — `None` in search results.
    #[serde(default)]
    pub is_public: Option<bool>,
    /// When the tone was published, as the API's raw timestamp string.
    ///
    /// Left unparsed deliberately: a transport crate should not pick a date library for
    /// you. Parse with whichever you already depend on.
    #[serde(default)]
    pub created_at: Option<String>,
    /// When the tone was last edited, as the API's raw timestamp string.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// The tone's page on tone3000.com — the link to show a user.
    #[serde(default)]
    pub url: String,
    /// How many models the tone has — **but the API means two different things by this
    /// depending on where the tone came from.**
    ///
    /// From [`Client::tones`](crate::Client::tones) (search) it is the total across every
    /// architecture. From [`Client::tone`](crate::Client::tone) (detail) it is the
    /// architecture-1 count alone. Measured on three tones:
    ///
    /// | Tone | via search | via detail | `a1` / `a2` |
    /// |------|-----------|-----------|-------------|
    /// | 19 | 6 | 3 | 3 / 3 |
    /// | 51949 | 6 | 3 | 3 / 3 |
    /// | 6298 | 223 | 112 | 112 / 111 |
    ///
    /// So a list screen built from search results can promise 223 models while the detail
    /// screen shows 112. To count reliably, add up the per-architecture fields yourself;
    /// to *list* them, see [`Client::models`](crate::Client::models).
    #[serde(default)]
    pub models_count: u64,
    /// How many users have favourited this tone.
    #[serde(default)]
    pub favorites_count: u64,
    /// How many times its models have been downloaded.
    #[serde(default)]
    pub downloads_count: u64,
    /// Models trained on architecture version 1.
    #[serde(default)]
    pub a1_models_count: u64,
    /// Models trained on architecture version 2.
    #[serde(default)]
    pub a2_models_count: u64,
    /// Impulse responses, as opposed to neural captures.
    #[serde(default)]
    pub irs_count: u64,
    /// Models using a user-supplied architecture rather than a standard one.
    #[serde(default)]
    pub custom_models_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_parses_core_fields() {
        let json = r#"{
            "id": 51949, "user_id": "57af", "title": "Plexi 51", "gear": "amp",
            "license": "t3k", "format": "nam", "makes": [{"name": "Marshall Plexi"}],
            "user": {"id": "57af", "username": "brucew", "url": "u"},
            "models_count": 6, "a1_models_count": 3
        }"#;
        let tone: Tone = serde_json::from_str(json).unwrap();
        assert_eq!(tone.id, ToneId(51949));
        assert_eq!(tone.title, "Plexi 51");
        assert_eq!(tone.gear, Some(Gear::Amp));
        assert_eq!(tone.license, Some(License::T3k));
        assert_eq!(tone.format, Some(Format::Nam));
        assert_eq!(tone.makes[0].name, "Marshall Plexi");
        assert_eq!(tone.user.as_ref().unwrap().username, "brucew");
        assert_eq!(tone.models_count, 6);
        assert_eq!(tone.a1_models_count, 3);
    }

    #[test]
    fn tone_tolerates_explicit_null_arrays() {
        // The real API returns `"images": null` (and may null other arrays); explicit
        // null must deserialize to an empty Vec, not fail the whole response.
        let json = r#"{
            "id": 1, "user_id": "u",
            "images": null, "sizes": null, "makes": null, "tags": null, "links": null
        }"#;
        let tone: Tone = serde_json::from_str(json).unwrap();
        assert!(tone.images.is_empty());
        assert!(tone.sizes.is_empty());
        assert!(tone.makes.is_empty());
        assert!(tone.tags.is_empty());
        assert!(tone.links.is_empty());
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

use serde::{Deserialize, Serialize};

/// Engagement metrics attached to tones and users.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Metrics {
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub favorites: u64,
}

/// A community tone (a capture/profile that groups one or more models).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tone {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub gears: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub metrics: Metrics,
}

/// Sort order accepted by list/search endpoints.
#[derive(Debug, Clone, Copy, Serialize)]
#[non_exhaustive]
pub enum Sort {
    Trending,
    Newest,
    Popular,
}

impl Sort {
    /// The query-string value the API expects.
    pub fn as_str(self) -> &'static str {
        match self {
            Sort::Trending => "Trending",
            Sort::Newest => "Newest",
            Sort::Popular => "Popular",
        }
    }
}

/// Parameters for [`crate::Client::search`].
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub query: Option<String>,
    pub gears: Vec<String>,
    pub sort: Option<Sort>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// A page of tones plus pagination metadata.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchResults {
    #[serde(default, alias = "data")]
    pub items: Vec<Tone>,
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub has_more: bool,
}

/// OAuth token set returned by the token endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tokens {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_tolerates_extra_and_missing_fields() {
        // Extra field `unexpected` must not fail; missing fields fall back to defaults.
        let json = r#"{ "id": "t1", "unexpected": 42 }"#;
        let tone: Tone = serde_json::from_str(json).unwrap();
        assert_eq!(tone.id, "t1");
        assert_eq!(tone.name, "");
        assert_eq!(tone.metrics.downloads, 0);
    }

    #[test]
    fn sort_serializes_to_api_value() {
        assert_eq!(Sort::Trending.as_str(), "Trending");
    }
}

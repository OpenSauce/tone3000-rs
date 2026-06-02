use serde::{Deserialize, Serialize};

/// A downloadable model belonging to a tone. `model_url` is the `.nam`/IR file location.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub model_url: String,
    #[serde(default)]
    pub tone_id: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_parses_url() {
        let json = r#"{ "id": "m1", "model_url": "https://example.com/a.nam" }"#;
        let m: Model = serde_json::from_str(json).unwrap();
        assert_eq!(m.model_url, "https://example.com/a.nam");
    }
}

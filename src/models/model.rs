use serde::{Deserialize, Serialize};

use super::enums::Size;
use super::ids::{ModelId, ToneId, UserId};

/// A downloadable model belonging to a tone. `model_url` is the `.nam`/IR file location.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Model {
    pub id: ModelId,
    pub tone_id: ToneId,
    pub user_id: UserId,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub model_url: String,
    #[serde(default)]
    pub size: Option<Size>,
    #[serde(default)]
    pub architecture_version: Option<String>,
}

/// Parameters for [`crate::Client::models`].
#[derive(Debug, Clone, Default)]
pub struct ModelListParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub architecture: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_parses_core_fields() {
        let json = r#"{
            "id": 293886, "tone_id": 51949, "user_id": "57af",
            "name": "Plexi 51 DI#03",
            "model_url": "https://x/api/v1/models/293886/download/a.nam",
            "size": "standard", "architecture_version": "1"
        }"#;
        let m: Model = serde_json::from_str(json).unwrap();
        assert_eq!(m.id, ModelId(293886));
        assert_eq!(m.tone_id, ToneId(51949));
        assert_eq!(m.size, Some(Size::Standard));
        assert_eq!(m.architecture_version.as_deref(), Some("1"));
    }
}

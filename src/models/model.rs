use serde::{Deserialize, Serialize};

use super::enums::{ArchitectureVersion, Size};
use super::ids::{ModelId, ToneId, UserId};

/// One downloadable capture belonging to a [`Tone`] — a single file at one size and
/// architecture.
///
/// A tone with six models is six variants of the same capture, not six different amps.
/// This struct is metadata: the file itself lives at [`model_url`](Self::model_url), and
/// [`Client::download_model`] fetches it.
///
/// [`Tone`]: crate::Tone
/// [`Client::download_model`]: crate::Client::download_model
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Model {
    /// Stable identifier, used by [`Client::model`](crate::Client::model).
    pub id: ModelId,
    /// The tone this model belongs to.
    pub tone_id: ToneId,
    /// The creator's account id.
    pub user_id: UserId,
    /// When the model was uploaded, as the API's raw timestamp string.
    #[serde(default)]
    pub created_at: Option<String>,
    /// When the model was last changed, as the API's raw timestamp string.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// The creator's name for this variant, e.g. "Plexi 51 DI#03".
    ///
    /// Distinguishes models within one tone — typically the mic, speaker or gain setting
    /// captured.
    #[serde(default)]
    pub name: String,
    /// Where the file lives. Fetching it needs the same Bearer token as any other call,
    /// which [`Client::download_model`](crate::Client::download_model) handles.
    #[serde(default)]
    pub model_url: String,
    /// The CPU/quality trade-off this variant was trained at.
    #[serde(default)]
    pub size: Option<Size>,
    /// Which generation of the NAM architecture trained this model.
    ///
    /// Determines which models [`Client::models`](crate::Client::models) returns: it
    /// serves one architecture at a time and defaults to v1.
    #[serde(default)]
    pub architecture_version: Option<ArchitectureVersion>,
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
        assert_eq!(m.architecture_version, Some(ArchitectureVersion::V1));
    }

    #[test]
    fn architecture_version_covers_documented_vocabulary() {
        for (wire, want) in [
            ("1", ArchitectureVersion::V1),
            ("2", ArchitectureVersion::V2),
            ("custom", ArchitectureVersion::Custom),
        ] {
            let got: ArchitectureVersion = serde_json::from_str(&format!("\"{wire}\"")).unwrap();
            assert_eq!(got, want);
            assert_eq!(got.as_str(), wire);
        }
    }
}

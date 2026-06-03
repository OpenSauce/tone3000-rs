//! OAuth token set returned by the token endpoint.

use serde::{Deserialize, Serialize};

/// Tokens returned by `POST /oauth/token` (exchange or refresh).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tokens {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

//! OAuth token set returned by the token endpoint.

use serde::{Deserialize, Serialize};

/// Tokens returned by `POST /oauth/token` (exchange or refresh).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Tokens {
    /// The Bearer credential for API calls. Short-lived.
    pub access_token: String,
    /// Used to mint a new access token once this one expires.
    ///
    /// TONE3000 **rotates** these: each refresh returns a new one and invalidates the
    /// old, so persist every value you receive or the session dies. Register
    /// [`ClientBuilder::on_tokens_changed`] to catch them.
    ///
    /// [`ClientBuilder::on_tokens_changed`]: crate::ClientBuilder::on_tokens_changed
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// Lifetime of the access token in seconds from issue.
    ///
    /// [`ClientBuilder::expires_at`] wants an absolute unix timestamp, not this duration:
    /// add it to the current unix time and pass the sum. Without it, proactive refresh
    /// cannot know when to fire and the client only refreshes reactively on a 401.
    ///
    /// [`ClientBuilder::expires_at`]: crate::ClientBuilder::expires_at
    #[serde(default)]
    pub expires_in: Option<u64>,
}

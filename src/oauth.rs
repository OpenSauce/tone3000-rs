//! OAuth helpers. The app owns the redirect; this module builds the authorize URL,
//! and performs the form-encoded token exchange/refresh against `/oauth/token`.

use url::Url;

use crate::client::DEFAULT_BASE_URL;
use crate::error::{Error, Result};
use crate::models::Tokens;

/// The `prompt` parameter selecting which OAuth flow to start.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Prompt {
    /// User browses & picks a tone in TONE3000's UI.
    SelectTone,
    /// App names a tone; user verifies access or picks a replacement.
    LoadTone { tone_id: String },
    /// Full custom API access.
    FullAccess,
}

/// Build the `/oauth/authorize` URL the app should open in a browser.
pub fn authorize_url(client_id: &str, redirect_uri: &str, challenge: &str, prompt: Prompt) -> Url {
    let mut url = Url::parse(DEFAULT_BASE_URL).expect("valid base url");
    url.set_path("/api/v1/oauth/authorize");
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("response_type", "code");
        q.append_pair("client_id", client_id);
        q.append_pair("redirect_uri", redirect_uri);
        q.append_pair("code_challenge", challenge);
        q.append_pair("code_challenge_method", "S256");
        match prompt {
            Prompt::SelectTone => {
                q.append_pair("prompt", "select_tone");
            }
            Prompt::LoadTone { tone_id } => {
                q.append_pair("prompt", "load_tone");
                q.append_pair("tone_id", &tone_id);
            }
            Prompt::FullAccess => {
                q.append_pair("prompt", "full_access");
            }
        }
    }
    url
}

/// Error body shape returned by the token endpoint on failure.
#[derive(serde::Deserialize)]
struct OauthError {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
}

/// Parse a `/oauth/token` response body into `Tokens`, mapping error bodies.
pub(crate) fn parse_token_response(status_ok: bool, body: &[u8]) -> Result<Tokens> {
    if status_ok {
        Ok(serde_json::from_slice(body)?)
    } else if let Ok(e) = serde_json::from_slice::<OauthError>(body) {
        Err(Error::Oauth {
            error: e.error,
            description: e.error_description,
        })
    } else {
        Err(Error::Status {
            code: 400,
            body: String::from_utf8_lossy(body).into_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorize_url_includes_pkce_and_prompt() {
        let u = authorize_url(
            "t3k_pub_x",
            "http://localhost:8080/cb",
            "CHAL",
            Prompt::LoadTone {
                tone_id: "t1".into(),
            },
        );
        let s = u.as_str();
        assert!(s.contains("code_challenge=CHAL"));
        assert!(s.contains("code_challenge_method=S256"));
        assert!(s.contains("prompt=load_tone"));
        assert!(s.contains("tone_id=t1"));
    }

    #[test]
    fn parse_token_response_maps_error_body() {
        let err = parse_token_response(false, br#"{"error":"invalid_grant"}"#).unwrap_err();
        assert!(matches!(err, Error::Oauth { error, .. } if error == "invalid_grant"));
    }
}

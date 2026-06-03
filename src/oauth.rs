//! OAuth helpers. The app owns the redirect; this module builds the authorize URL,
//! and performs the form-encoded token exchange/refresh against `/oauth/token`.

use url::Url;

use crate::client::DEFAULT_BASE_URL;
use crate::error::{Error, Result};
use crate::models::{ModelId, ToneId, Tokens};

/// The `prompt` parameter selecting which OAuth flow to start.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Prompt {
    /// User browses & picks a tone in TONE3000's UI.
    SelectTone,
    /// App names a tone; user verifies access or picks a replacement.
    LoadTone { tone_id: ToneId },
    /// App names a model; user verifies access (no replacement offered).
    LoadModel { model_id: ModelId },
    /// Long-lived full API access; no `prompt` parameter is sent.
    Standard,
}

/// Build the `/oauth/authorize` URL the app should open in a browser.
///
/// `state` is an opaque, unguessable value the app generates and **must** persist: the
/// authorize redirect returns it unchanged, and the app verifies it matches before
/// trusting the `code`. TONE3000 requires it. The app owns this (as it owns the redirect
/// transport and the PKCE verifier), so it is passed in rather than generated here.
pub fn authorize_url(
    client_id: &str,
    redirect_uri: &str,
    challenge: &str,
    state: &str,
    prompt: Prompt,
) -> Url {
    let mut url = Url::parse(DEFAULT_BASE_URL).expect("valid base url");
    url.set_path("/api/v1/oauth/authorize");
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("response_type", "code");
        q.append_pair("client_id", client_id);
        q.append_pair("redirect_uri", redirect_uri);
        q.append_pair("code_challenge", challenge);
        q.append_pair("code_challenge_method", "S256");
        q.append_pair("state", state);
        match prompt {
            Prompt::SelectTone => {
                q.append_pair("prompt", "select_tone");
            }
            Prompt::LoadTone { tone_id } => {
                q.append_pair("prompt", "load_tone");
                q.append_pair("tone_id", &tone_id.to_string());
            }
            Prompt::LoadModel { model_id } => {
                q.append_pair("prompt", "load_model");
                q.append_pair("model_id", &model_id.to_string());
            }
            // Standard flow sends no `prompt` parameter.
            Prompt::Standard => {}
        }
    }
    url
}

/// OAuth-standard error body.
#[derive(serde::Deserialize)]
struct OauthErrorStd {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
}

/// GoTrue-style error body (the other schema the token endpoint emits).
#[derive(serde::Deserialize)]
struct GoTrueError {
    error_code: String,
    #[serde(default)]
    msg: Option<String>,
}

/// Parse a `/oauth/token` response body into `Tokens`, mapping both observed error schemas.
///
/// `status` is the real HTTP status code; on a non-2xx response with an unrecognized body
/// it is preserved in [`Error::Status`] rather than guessed.
pub(crate) fn parse_token_response(status: u16, body: &[u8]) -> Result<Tokens> {
    if (200..300).contains(&status) {
        Ok(serde_json::from_slice(body)?)
    } else if let Ok(e) = serde_json::from_slice::<OauthErrorStd>(body) {
        Err(Error::Oauth {
            error: e.error,
            description: e.error_description,
        })
    } else if let Ok(g) = serde_json::from_slice::<GoTrueError>(body) {
        Err(Error::Oauth {
            error: g.error_code,
            description: g.msg,
        })
    } else {
        Err(Error::Status {
            code: status,
            body: String::from_utf8_lossy(body).into_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ModelId, ToneId};

    #[test]
    fn authorize_url_includes_pkce_state_and_prompt() {
        let u = authorize_url(
            "t3k_pub_x",
            "http://localhost:8080/cb",
            "CHAL",
            "xyz-state",
            Prompt::LoadTone { tone_id: ToneId(1) },
        );
        let s = u.as_str();
        assert!(s.contains("code_challenge=CHAL"));
        assert!(s.contains("code_challenge_method=S256"));
        assert!(s.contains("state=xyz-state"));
        assert!(s.contains("prompt=load_tone"));
        assert!(s.contains("tone_id=1"));
    }

    #[test]
    fn authorize_url_load_model_sends_model_id() {
        let u = authorize_url(
            "t3k_pub_x",
            "http://localhost/cb",
            "C",
            "S",
            Prompt::LoadModel { model_id: ModelId(42) },
        );
        let s = u.as_str();
        assert!(s.contains("prompt=load_model"));
        assert!(s.contains("model_id=42"));
    }

    #[test]
    fn authorize_url_standard_sends_no_prompt() {
        let u = authorize_url("t3k_pub_x", "http://localhost/cb", "C", "S", Prompt::Standard);
        assert!(!u.as_str().contains("prompt="));
    }

    #[test]
    fn parse_token_response_maps_oauth_error_body() {
        let err = parse_token_response(400, br#"{"error":"invalid_grant"}"#).unwrap_err();
        assert!(matches!(err, Error::Oauth { error, .. } if error == "invalid_grant"));
    }

    #[test]
    fn parse_token_response_maps_gotrue_error_body() {
        let body = br#"{"code":400,"error_code":"refresh_token_not_found","msg":"Invalid Refresh Token"}"#;
        let err = parse_token_response(400, body).unwrap_err();
        assert!(matches!(err, Error::Oauth { error, description }
            if error == "refresh_token_not_found" && description.as_deref() == Some("Invalid Refresh Token")));
    }

    #[test]
    fn parse_token_response_preserves_real_status() {
        let err = parse_token_response(503, b"upstream down").unwrap_err();
        assert!(matches!(err, Error::Status { code: 503, .. }));
    }
}

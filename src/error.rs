use std::time::Duration;

/// Errors returned by this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Transport-level failure (DNS, TLS, connection, timeout).
    #[error("http transport error: {0}")]
    Http(#[from] reqwest::Error),

    /// Non-2xx response that does not map to a more specific variant.
    #[error("unexpected status {code}: {body}")]
    Status { code: u16, body: String },

    /// 401 Unauthorized from the API.
    #[error("unauthorized (401)")]
    Unauthorized,

    /// 403 Forbidden from the API.
    #[error("forbidden (403)")]
    Forbidden,

    /// 429 Too Many Requests. `retry_after` is set if the server sent `Retry-After`.
    #[error("rate limited (429)")]
    RateLimited { retry_after: Option<Duration> },

    /// A user-scoped call was made on a client that has no access token.
    #[error("operation requires an access token, but client is in app-key mode")]
    Unauthenticated,

    /// Response body could not be deserialized.
    #[error("deserialize error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// OAuth token endpoint returned an error body.
    #[error("oauth error: {error}")]
    Oauth {
        error: String,
        description: Option<String>,
    },
}

/// Crate result alias.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limited_carries_retry_after() {
        let e = Error::RateLimited {
            retry_after: Some(Duration::from_secs(30)),
        };
        assert!(matches!(e, Error::RateLimited { retry_after: Some(d) } if d.as_secs() == 30));
    }

    #[test]
    fn display_is_human_readable() {
        assert_eq!(Error::Unauthorized.to_string(), "unauthorized (401)");
    }
}

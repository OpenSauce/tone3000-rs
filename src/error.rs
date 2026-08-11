use std::time::Duration;

/// Errors returned by this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Transport-level failure (DNS, TLS, connection, timeout).
    #[error("http transport error: {0}")]
    Http(#[from] HttpError),

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

    /// An API call was made on a client with no access token (and no refresh token to
    /// mint one). Every TONE3000 endpoint requires a Bearer access token.
    #[error("operation requires an access token, but none is set")]
    Unauthenticated,

    /// Response body could not be deserialized.
    #[error("deserialize error: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// Failure writing a streamed download to the provided writer.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A downloaded body was expected to be UTF-8 text (e.g. `.nam` JSON) but was not.
    #[error("invalid utf-8 in response body: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// OAuth token endpoint returned an error body.
    #[error("oauth error: {error}")]
    Oauth {
        error: String,
        description: Option<String>,
    },
}

impl Error {
    /// The request timed out.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Error::Http(e) if e.is_timeout())
    }

    /// The failure happened while establishing the connection.
    pub fn is_connect(&self) -> bool {
        matches!(self, Error::Http(e) if e.is_connect())
    }
}

/// A transport-level failure.
///
/// The HTTP client behind this is an implementation detail and may be swapped or
/// version-bumped without a breaking release, so the underlying error type is not part of
/// this crate's public API. The predicates below cover what callers actually branch on.
#[derive(Debug)]
pub struct HttpError(reqwest::Error);

impl HttpError {
    /// The request timed out.
    pub fn is_timeout(&self) -> bool {
        self.0.is_timeout()
    }

    /// The failure happened while establishing the connection.
    pub fn is_connect(&self) -> bool {
        self.0.is_connect()
    }

    /// The status code, when the failure carried one.
    pub fn status(&self) -> Option<u16> {
        self.0.status().map(|s| s.as_u16())
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<reqwest::Error> for HttpError {
    fn from(e: reqwest::Error) -> Self {
        HttpError(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(HttpError(e))
    }
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

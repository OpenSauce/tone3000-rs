use std::time::Duration;

use reqwest::{Response, StatusCode};

use crate::error::{Error, Result};

/// Convert a completed response into either its body (on 2xx) or a mapped [`Error`].
pub(crate) async fn check_status(resp: Response) -> Result<Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    match status {
        StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
        StatusCode::FORBIDDEN => Err(Error::Forbidden),
        StatusCode::TOO_MANY_REQUESTS => {
            let retry_after = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(Duration::from_secs);
            Err(Error::RateLimited { retry_after })
        }
        other => {
            let body = resp.text().await.unwrap_or_default();
            Err(Error::Status {
                code: other.as_u16(),
                body,
            })
        }
    }
}

/// Deserialize a checked 2xx response body as JSON into `T`.
pub(crate) async fn json<T: serde::de::DeserializeOwned>(resp: Response) -> Result<T> {
    let bytes = resp.bytes().await?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn get(server: &MockServer) -> Response {
        reqwest::Client::new()
            .get(server.uri())
            .send()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn maps_401() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let err = check_status(get(&server).await).await.unwrap_err();
        assert!(matches!(err, Error::Unauthorized));
    }

    #[tokio::test]
    async fn maps_429_with_retry_after() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "12"))
            .mount(&server)
            .await;
        let err = check_status(get(&server).await).await.unwrap_err();
        assert!(matches!(err, Error::RateLimited { retry_after: Some(d) } if d.as_secs() == 12));
    }
}

use tone3000::Client;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn exchange_code_stores_tokens_and_fires_callback() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"access_token":"AT","refresh_token":"RT","expires_in":3600}"#,
        ))
        .mount(&server)
        .await;

    let seen = std::sync::Arc::new(std::sync::Mutex::new(None));
    let seen2 = seen.clone();
    let client = Client::builder("t3k_pub_x")
        .base_url(server.uri())
        .on_tokens_changed(move |t| {
            *seen2.lock().unwrap() = Some(t.access_token.clone());
        })
        .build();

    let tokens = client
        .exchange_code("code123", "verifier123", "http://localhost/cb")
        .await
        .unwrap();

    assert_eq!(tokens.access_token, "AT");
    assert_eq!(seen.lock().unwrap().as_deref(), Some("AT"));
}

#[tokio::test]
async fn token_error_body_maps_to_oauth_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(400).set_body_string(r#"{"error":"invalid_grant"}"#))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x").base_url(server.uri()).build();
    let err = client
        .exchange_code("bad", "v", "http://localhost/cb")
        .await
        .unwrap_err();
    assert!(matches!(err, tone3000::Error::Oauth { error, .. } if error == "invalid_grant"));
}

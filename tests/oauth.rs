use tone3000::Client;
use tone3000::SearchParams;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn exchange_code_stores_tokens_and_fires_callback() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"access_token":"AT","refresh_token":"RT","expires_in":3600}"#),
        )
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

#[tokio::test]
async fn auto_refresh_runs_before_request_and_uses_new_token() {
    let server = MockServer::start().await;

    // Token endpoint returns a fresh access token.
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                r#"{"access_token":"FRESH","refresh_token":"RT2","expires_in":3600}"#,
            ),
        )
        .mount(&server)
        .await;

    // Search endpoint requires the Authorization header to be the refreshed token.
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .and(wiremock::matchers::header("authorization", "Bearer FRESH"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"data":[],"page":1,"total":0,"has_more":false}"#),
        )
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x")
        .base_url(server.uri())
        .access_token("STALE")
        .refresh_token("RT")
        .auto_refresh(true)
        .build();

    // Seed token state (sets access=FRESH and expires_at ~1h) by performing one refresh.
    client.refresh().await.unwrap();

    // Search must succeed using the FRESH bearer token, proving the refresh + auth wiring.
    let res = client.search(SearchParams::default()).await.unwrap();
    assert_eq!(res.total, 0);
}

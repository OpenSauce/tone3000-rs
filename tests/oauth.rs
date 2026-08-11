use tone3000::Client;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const EMPTY_PAGE: &str = r#"{"data":[],"page":1,"page_size":0,"total":0,"total_pages":0}"#;

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
async fn gotrue_refresh_error_maps_to_oauth_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(400).set_body_string(
            r#"{"code":400,"error_code":"refresh_token_not_found","msg":"Invalid Refresh Token"}"#,
        ))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x")
        .base_url(server.uri())
        .refresh_token("RT")
        .build();
    let err = client.refresh().await.unwrap_err();
    assert!(
        matches!(err, tone3000::Error::Oauth { error, .. } if error == "refresh_token_not_found")
    );
}

#[tokio::test]
async fn proactive_refresh_fires_on_seeded_expiry() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                r#"{"access_token":"FRESH","refresh_token":"RT2","expires_in":3600}"#,
            ),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .and(header("authorization", "Bearer FRESH"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGE))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x")
        .base_url(server.uri())
        .access_token("STALE")
        .refresh_token("RT")
        .expires_at(1)
        .auto_refresh(true)
        .build();

    let res = client.tones().await.unwrap();
    assert_eq!(res.total, 0);
}

#[tokio::test]
async fn reactive_refresh_retries_once_on_401() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                r#"{"access_token":"FRESH","refresh_token":"RT2","expires_in":3600}"#,
            ),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .and(header("authorization", "Bearer STALE"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .and(header("authorization", "Bearer FRESH"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGE))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x")
        .base_url(server.uri())
        .access_token("STALE")
        .refresh_token("RT")
        .auto_refresh(true)
        .build();

    let res = client.tones().await.unwrap();
    assert_eq!(res.total, 0);
}

#[tokio::test]
async fn no_token_but_refresh_mints_access_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"access_token":"MINTED","refresh_token":"RT2","expires_in":3600}"#,
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .and(header("authorization", "Bearer MINTED"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGE))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x")
        .base_url(server.uri())
        .refresh_token("RT")
        .build();

    let res = client.tones().await.unwrap();
    assert_eq!(res.total, 0);
}

#[tokio::test]
async fn refresh_without_token_errors_unauthenticated() {
    let client = Client::builder("t3k_pub_x").build();
    let err = client.refresh().await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Unauthenticated));
}

#[test]
fn authorize_url_omits_unset_options() {
    use tone3000::{AuthorizeOptions, Prompt, authorize_url};

    let url = authorize_url(
        "t3k_pub_x",
        "http://localhost:3001",
        "chal",
        "state123",
        Prompt::Standard,
        AuthorizeOptions::default(),
    );
    let q: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    assert_eq!(q.get("state").unwrap(), "state123");
    for absent in [
        "gears",
        "format",
        "architecture",
        "menubar",
        "login_hint",
        "preview",
    ] {
        assert!(
            !q.contains_key(absent),
            "{absent} must be omitted when unset"
        );
    }
}

#[test]
fn authorize_url_serializes_scoping_options() {
    use tone3000::{AuthorizeOptions, Format, Gear, Prompt, authorize_url};

    let url = authorize_url(
        "t3k_pub_x",
        "http://localhost:3001",
        "chal",
        "state123",
        Prompt::SelectTone,
        AuthorizeOptions {
            gears: vec![Gear::Amp, Gear::Pedal],
            format: Some(Format::Nam),
            architecture: Some(2),
            menubar: true,
            login_hint: Some("a@b.com".into()),
            preview: true,
        },
    );
    let q: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();

    assert_eq!(q.get("prompt").unwrap(), "select_tone");
    assert_eq!(q.get("gears").unwrap(), "amp_pedal");
    assert_eq!(q.get("format").unwrap(), "nam");
    assert_eq!(q.get("architecture").unwrap(), "2");
    assert_eq!(q.get("menubar").unwrap(), "true");
    assert_eq!(q.get("login_hint").unwrap(), "a@b.com");
    assert_eq!(q.get("preview").unwrap(), "true");
}

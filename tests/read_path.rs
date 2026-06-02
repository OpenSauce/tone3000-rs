use tone3000::{Client, SearchParams};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn search_parses_fixture_and_sends_auth() {
    let server = MockServer::start().await;
    let body = include_str!("fixtures/search.json");
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        // App-key mode must send the publishable key as the bearer.
        .and(header("authorization", "Bearer t3k_pub_x"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x").base_url(server.uri()).build();
    let results = client
        .search(SearchParams {
            query: Some("plexi".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.total, 1);
    assert_eq!(results.items.len(), 1);
    assert_eq!(results.items[0].name, "Plexi Crunch");
}

#[tokio::test]
async fn tone_404_maps_to_status_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x").base_url(server.uri()).build();
    let err = client.tone("missing").await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Status { code: 404, .. }));
}

#[tokio::test]
async fn download_model_fetches_bytes_with_bearer() {
    use tone3000::Model;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/a.nam"))
        // Bearer mode must attach the user access token to the file download.
        .and(header("authorization", "Bearer user_tok"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1u8, 2, 3, 4]))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x")
        .access_token("user_tok")
        .base_url(server.uri())
        .build();
    let model = Model {
        id: "m1".into(),
        name: String::new(),
        model_url: format!("{}/files/a.nam", server.uri()),
        tone_id: None,
        format: None,
    };

    let bytes = client.download_model(&model).await.unwrap();
    assert_eq!(&bytes[..], &[1, 2, 3, 4]);
}

#[tokio::test]
async fn download_model_json_returns_string() {
    use tone3000::Model;
    let server = MockServer::start().await;
    let nam_json = r#"{"version":"0.5.4","architecture":"WaveNet"}"#;
    Mock::given(method("GET"))
        .and(path("/files/c.nam"))
        .respond_with(ResponseTemplate::new(200).set_body_string(nam_json))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x").base_url(server.uri()).build();
    let model = Model {
        id: "m3".into(),
        name: String::new(),
        model_url: format!("{}/files/c.nam", server.uri()),
        tone_id: None,
        format: None,
    };

    let json = client.download_model_json(&model).await.unwrap();
    assert_eq!(json, nam_json);
}

#[tokio::test]
async fn download_model_to_streams_to_writer() {
    use tone3000::Model;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/b.nam"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![9u8; 100]))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x").base_url(server.uri()).build();
    let model = Model {
        id: "m2".into(),
        name: String::new(),
        model_url: format!("{}/files/b.nam", server.uri()),
        tone_id: None,
        format: None,
    };

    let mut buf: Vec<u8> = Vec::new();
    let n = client.download_model_to(&model, &mut buf).await.unwrap();
    assert_eq!(n, 100);
    assert_eq!(buf.len(), 100);
}

#[tokio::test]
async fn user_scoped_call_without_token_errors() {
    let client = Client::new("t3k_pub_x");
    let err = client.user().await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Unauthenticated));
}

#[tokio::test]
async fn created_returns_tones_with_token() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/created"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"data":[{"id":"t9","name":"Mine"}],"page":1,"total":1,"has_more":false}"#,
        ))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x")
        .base_url(server.uri())
        .access_token("AT")
        .build();
    let res = client.created(SearchParams::default()).await.unwrap();
    assert_eq!(res.items[0].name, "Mine");
}

#[tokio::test]
async fn forbidden_maps_to_forbidden_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x").base_url(server.uri()).build();
    let err = client.search(SearchParams::default()).await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Forbidden));
}

#[tokio::test]
async fn download_model_json_rejects_non_utf8() {
    use tone3000::Model;
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/bad.nam"))
        // 0xFF is never valid UTF-8.
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0xFFu8, 0xFE]))
        .mount(&server)
        .await;

    let client = Client::builder("t3k_pub_x").base_url(server.uri()).build();
    let model = Model {
        id: "m4".into(),
        name: String::new(),
        model_url: format!("{}/files/bad.nam", server.uri()),
        tone_id: None,
        format: None,
    };

    let err = client.download_model_json(&model).await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Utf8(_)));
}

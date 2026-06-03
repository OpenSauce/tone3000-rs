use tone3000::{Client, ListParams, Model, ModelId, SearchParams, ToneId, UserListParams};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A client in Bearer mode with a fixed access token, for read-path tests.
fn client(server: &MockServer) -> Client {
    Client::builder("t3k_pub_x")
        .access_token("AT")
        .base_url(server.uri())
        .build()
}

fn model_fixture(server: &MockServer, file_path: &str) -> Model {
    Model {
        id: ModelId(1),
        tone_id: ToneId(2),
        user_id: "u".into(),
        created_at: None,
        updated_at: None,
        name: String::new(),
        model_url: format!("{}{}", server.uri(), file_path),
        size: None,
        architecture_version: None,
    }
}

#[tokio::test]
async fn search_parses_fixture_and_sends_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .and(header("authorization", "Bearer AT"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/search.json")),
        )
        .mount(&server)
        .await;

    let results = client(&server)
        .search(SearchParams {
            query: Some("plexi".into()),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results.total, 254);
    assert_eq!(results.data[0].id, ToneId(51949));
    assert_eq!(results.data[0].title, "Plexi 51");
}

#[tokio::test]
async fn tone_parses_fixture() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/51949"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/tone.json")),
        )
        .mount(&server)
        .await;

    let tone = client(&server).tone(ToneId(51949)).await.unwrap();
    assert_eq!(tone.id, ToneId(51949));
    assert_eq!(tone.title, "Plexi 51");
}

#[tokio::test]
async fn tone_404_maps_to_status_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/999"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    let err = client(&server).tone(ToneId(999)).await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Status { code: 404, .. }));
}

#[tokio::test]
async fn models_parses_paginated_fixture() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/models.json")),
        )
        .mount(&server)
        .await;

    let page = client(&server)
        .models(ToneId(51949), Default::default())
        .await
        .unwrap();
    assert_eq!(page.total, 3);
    assert_eq!(page.data[0].id, ModelId(293886));
    assert_eq!(page.data[0].tone_id, ToneId(51949));
}

#[tokio::test]
async fn users_parses_paginated_fixture() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/users.json")),
        )
        .mount(&server)
        .await;

    let page = client(&server)
        .users(UserListParams::default())
        .await
        .unwrap();
    assert_eq!(page.data[0].username, "akka5");
    assert_eq!(page.data[0].tones_count, 153);
}

#[tokio::test]
async fn created_parses_empty_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/created"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/created.json")),
        )
        .mount(&server)
        .await;

    let page = client(&server)
        .created(ListParams::default())
        .await
        .unwrap();
    assert_eq!(page.total, 0);
    assert!(page.data.is_empty());
}

#[tokio::test]
async fn download_model_fetches_bytes_with_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/a.nam"))
        .and(header("authorization", "Bearer AT"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![1u8, 2, 3, 4]))
        .mount(&server)
        .await;

    let model = model_fixture(&server, "/files/a.nam");
    let bytes = client(&server).download_model(&model).await.unwrap();
    assert_eq!(&bytes[..], &[1, 2, 3, 4]);
}

#[tokio::test]
async fn download_model_to_streams_to_writer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/b.nam"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![9u8; 100]))
        .mount(&server)
        .await;

    let model = model_fixture(&server, "/files/b.nam");
    let mut buf: Vec<u8> = Vec::new();
    let n = client(&server)
        .download_model_to(&model, &mut buf)
        .await
        .unwrap();
    assert_eq!(n, 100);
    assert_eq!(buf.len(), 100);
}

#[tokio::test]
async fn download_model_json_rejects_non_utf8() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/files/bad.nam"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0xFFu8, 0xFE]))
        .mount(&server)
        .await;

    let model = model_fixture(&server, "/files/bad.nam");
    let err = client(&server)
        .download_model_json(&model)
        .await
        .unwrap_err();
    assert!(matches!(err, tone3000::Error::Utf8(_)));
}

#[tokio::test]
async fn forbidden_maps_to_forbidden_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let err = client(&server)
        .search(SearchParams::default())
        .await
        .unwrap_err();
    assert!(matches!(err, tone3000::Error::Forbidden));
}

#[tokio::test]
async fn call_without_token_errors_unauthenticated() {
    let client = Client::new("t3k_pub_x");
    let err = client.user().await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Unauthenticated));
}

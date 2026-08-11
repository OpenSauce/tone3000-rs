use tone3000::{Client, Model, ModelId, ToneId};
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

    let results = client(&server).tones().query("plexi").await.unwrap();

    assert_eq!(results.total, 254);
    assert_eq!(results.data[0].id, ToneId(51949));
    assert_eq!(results.data[0].title, "Plexi 51");
}

#[tokio::test]
async fn search_serializes_all_filters() {
    use tone3000::{Format, Gear, Size};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/tones/search"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(include_str!("fixtures/search.json")),
        )
        .mount(&server)
        .await;

    client(&server)
        .tones()
        .query("plexi")
        .gears([Gear::Amp, Gear::Pedal])
        .format(Format::Nam)
        .size(Size::Standard)
        .tags(["clean", "crunch"])
        .make("Marshall")
        .creators(["brucew", "akka5"])
        .await
        .unwrap();

    let req = &server.received_requests().await.unwrap()[0];
    let q: std::collections::HashMap<_, _> = req.url.query_pairs().into_owned().collect();

    assert_eq!(q.get("gears").unwrap(), "amp_pedal");
    assert_eq!(q.get("format").unwrap(), "nam");
    assert_eq!(q.get("sizes").unwrap(), "standard");
    assert_eq!(q.get("tags").unwrap(), "clean_crunch");
    assert_eq!(q.get("makes").unwrap(), "Marshall");
    // creators is comma-joined, unlike every other multi-value filter. This asymmetry is
    // in the API, not a typo — see tone-3000/api src/tone3000-client.ts.
    assert_eq!(q.get("creators").unwrap(), "brucew,akka5");
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

    let page = client(&server).models(ToneId(51949)).await.unwrap();
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

    let page = client(&server).users().await.unwrap();
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

    let page = client(&server).created().await.unwrap();
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

    let err = client(&server).tones().await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Forbidden));
}

#[tokio::test]
async fn call_without_token_errors_unauthenticated() {
    let client = Client::builder("t3k_pub_x").build();
    let err = client.user().await.unwrap_err();
    assert!(matches!(err, tone3000::Error::Unauthenticated));
}

#[tokio::test]
async fn list_builders_serialize_pagination() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"data":[]}"#))
        .mount(&server)
        .await;

    let c = client(&server);
    c.tones().page(2).page_size(24).await.unwrap();
    c.created().page(3).await.unwrap();
    c.favorited().page_size(5).await.unwrap();
    c.models(ToneId(51949)).page(4).await.unwrap();
    c.users().query("akka").page_size(10).await.unwrap();

    let reqs = server.received_requests().await.unwrap();
    let q = |i: usize| -> std::collections::HashMap<String, String> {
        reqs[i].url.query_pairs().into_owned().collect()
    };

    assert_eq!(reqs[0].url.path(), "/tones/search");
    assert_eq!(q(0).get("page").unwrap(), "2");
    assert_eq!(q(0).get("page_size").unwrap(), "24");

    assert_eq!(reqs[1].url.path(), "/tones/created");
    assert_eq!(q(1).get("page").unwrap(), "3");

    assert_eq!(reqs[2].url.path(), "/tones/favorited");
    assert_eq!(q(2).get("page_size").unwrap(), "5");

    assert_eq!(reqs[3].url.path(), "/models");
    assert_eq!(q(3).get("tone_id").unwrap(), "51949");
    assert_eq!(q(3).get("page").unwrap(), "4");

    assert_eq!(reqs[4].url.path(), "/users");
    assert_eq!(q(4).get("query").unwrap(), "akka");
    assert_eq!(q(4).get("page_size").unwrap(), "10");
}

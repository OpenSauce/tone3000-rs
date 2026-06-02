use tone3000::{Client, SearchParams};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn search_parses_fixture_and_sends_auth() {
    let server = MockServer::start().await;
    let body = include_str!("fixtures/search.json");
    Mock::given(method("GET"))
        .and(path("/tones/search"))
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

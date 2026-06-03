mod common;

use tone3000::{SearchParams, UserListParams};

#[tokio::test]
#[ignore = "live: hits the real TONE3000 API; run via `make test-live`"]
async fn public_endpoints_contract() {
    let (client, access) = common::authed().await;
    let base = common::api_base();

    let results = client
        .search(SearchParams {
            query: Some("plexi".into()),
            ..Default::default()
        })
        .await
        .expect("search succeeds");
    assert!(
        !results.data.is_empty(),
        "search returned no tones — API drift or empty library?"
    );
    let tone_id = results.data[0].id;
    assert!(
        !results.data[0].title.is_empty(),
        "tone title should be non-empty"
    );

    let tone = client.tone(tone_id).await.expect("tone fetch succeeds");
    assert_eq!(tone.id, tone_id, "tone(id) must return the requested id");
    let raw = common::raw_json(&format!("{base}/tones/{tone_id}"), &access).await;
    common::drift_warn(
        &format!("GET /tones/{tone_id}"),
        &raw,
        &serde_json::to_value(&tone).unwrap(),
    );

    let models = client
        .models(tone_id, Default::default())
        .await
        .expect("models list succeeds");
    assert!(!models.data.is_empty(), "tone {tone_id} has no models?");
    for m in &models.data {
        assert_eq!(m.tone_id, tone_id, "model.tone_id must match the queried tone");
    }
    let model_id = models.data[0].id;

    let model = client.model(model_id).await.expect("model fetch succeeds");
    assert_eq!(model.id, model_id, "model(id) must return the requested id");
    assert!(
        model.model_url.starts_with("http://") || model.model_url.starts_with("https://"),
        "model_url must be an absolute URL: {:?}",
        model.model_url
    );
    let raw = common::raw_json(&format!("{base}/models/{model_id}"), &access).await;
    common::drift_warn(
        &format!("GET /models/{model_id}"),
        &raw,
        &serde_json::to_value(&model).unwrap(),
    );

    let bytes = client.download_model(&model).await.expect("download succeeds");
    assert!(!bytes.is_empty(), "downloaded model should be non-empty");

    match client.download_model_json(&model).await {
        Ok(s) => assert!(!s.is_empty(), "json model should be non-empty"),
        Err(tone3000::Error::Utf8(_)) => {
            eprintln!("note: model {model_id} is not UTF-8 text; skipping json assertion");
        }
        Err(e) => panic!("unexpected download_model_json error: {e}"),
    }

    let mut buf: Vec<u8> = Vec::new();
    let n = client
        .download_model_to(&model, &mut buf)
        .await
        .expect("streamed download succeeds");
    assert_eq!(n, buf.len() as u64, "returned count must equal bytes written");
    assert_eq!(n, bytes.len() as u64, "streamed and in-memory sizes must match");
}

#[tokio::test]
#[ignore = "live: hits the real TONE3000 API; run via `make test-live`"]
async fn users_list_contract() {
    let (client, access) = common::authed().await;
    let base = common::api_base();

    let users = client
        .users(UserListParams::default())
        .await
        .expect("users list succeeds");
    assert!(!users.data.is_empty(), "users list should be non-empty");
    for u in &users.data {
        assert!(!u.id.0.is_empty(), "user id should be non-empty");
    }

    let raw = common::raw_json(&format!("{base}/users"), &access).await;
    if let Some(first) = raw
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|a| a.first())
    {
        common::drift_warn(
            "GET /users data[0]",
            first,
            &serde_json::to_value(&users.data[0]).unwrap(),
        );
    }
}

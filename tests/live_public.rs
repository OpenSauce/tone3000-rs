mod common;

use tone3000::{SearchParams, UserListParams};

#[tokio::test]
#[ignore = "live: hits the real TONE3000 API; run via `make test-live`"]
async fn public_endpoints_contract() {
    let client = common::public_client();
    let key = common::require_env("TONE3000_API_KEY");
    let base = common::api_base();

    // 1. search — heavily rate-limited, so we only call it once.
    let results = client
        .search(SearchParams {
            query: Some("plexi".into()),
            ..Default::default()
        })
        .await
        .expect("search succeeds");
    assert!(
        !results.items.is_empty(),
        "search returned no tones — API drift or empty library?"
    );
    let tone_id = results.items[0].id.clone();
    assert!(!tone_id.is_empty(), "tone id should be non-empty");

    // 2. tone(id): correlation + drift.
    let tone = client.tone(&tone_id).await.expect("tone fetch succeeds");
    assert_eq!(tone.id, tone_id, "tone(id) must return the requested id");
    let raw = common::raw_json(&format!("{base}/tones/{tone_id}"), &key).await;
    common::drift_warn(
        &format!("GET /tones/{tone_id}"),
        &raw,
        &serde_json::to_value(&tone).unwrap(),
    );

    // 3. models(tone_id): every model that reports a tone_id must match.
    let models = client.models(&tone_id).await.expect("models list succeeds");
    assert!(
        !models.is_empty(),
        "tone {tone_id} has no models — pick a different search term?"
    );
    for m in &models {
        if let Some(tid) = &m.tone_id {
            assert_eq!(tid, &tone_id, "model.tone_id must match the queried tone");
        }
    }
    let model_id = models[0].id.clone();

    // 4. model(id): correlation + absolute-URL check + drift.
    let model = client.model(&model_id).await.expect("model fetch succeeds");
    assert_eq!(model.id, model_id, "model(id) must return the requested id");
    assert!(
        model.model_url.starts_with("http://") || model.model_url.starts_with("https://"),
        "model_url must be an absolute URL: {:?}",
        model.model_url
    );
    let raw = common::raw_json(&format!("{base}/models/{model_id}"), &key).await;
    common::drift_warn(
        &format!("GET /models/{model_id}"),
        &raw,
        &serde_json::to_value(&model).unwrap(),
    );

    // 5. download_model -> non-empty bytes.
    let bytes = client
        .download_model(&model)
        .await
        .expect("download succeeds");
    assert!(!bytes.is_empty(), "downloaded model should be non-empty");

    // 6. download_model_json -> best-effort (some IRs are binary, not UTF-8 text).
    match client.download_model_json(&model).await {
        Ok(s) => assert!(!s.is_empty(), "json model should be non-empty"),
        Err(tone3000::Error::Utf8(_)) => {
            eprintln!("note: model {model_id} is not UTF-8 text; skipping json assertion");
        }
        Err(e) => panic!("unexpected download_model_json error: {e}"),
    }

    // 7. download_model_to -> byte count matches both the writer and the in-memory body.
    let mut buf: Vec<u8> = Vec::new();
    let n = client
        .download_model_to(&model, &mut buf)
        .await
        .expect("streamed download succeeds");
    assert_eq!(
        n,
        buf.len() as u64,
        "returned count must equal bytes written"
    );
    assert_eq!(
        n,
        bytes.len() as u64,
        "streamed and in-memory sizes must match"
    );
}

#[tokio::test]
#[ignore = "live: hits the real TONE3000 API; run via `make test-live`"]
async fn users_list_contract() {
    let client = common::public_client();
    let key = common::require_env("TONE3000_API_KEY");
    let base = common::api_base();

    let users = client
        .users(UserListParams::default())
        .await
        .expect("users list succeeds");
    assert!(!users.is_empty(), "users list should be non-empty");
    for u in &users {
        assert!(!u.id.is_empty(), "user id should be non-empty");
    }

    let raw = common::raw_json(&format!("{base}/users"), &key).await;
    if let Some(first) = raw.as_array().and_then(|a| a.first()) {
        common::drift_warn(
            "GET /users[0]",
            first,
            &serde_json::to_value(&users[0]).unwrap(),
        );
    }
}

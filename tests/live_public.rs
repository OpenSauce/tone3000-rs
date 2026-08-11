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
        assert_eq!(
            m.tone_id, tone_id,
            "model.tone_id must match the queried tone"
        );
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

    let bytes = client
        .download_model(&model)
        .await
        .expect("download succeeds");
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

/// Fail when the live API returns an enum value this SDK does not model.
///
/// Open enums fall back to `Other(String)` so an unknown value never sinks a response —
/// correct at runtime, and precisely why drift here is otherwise invisible.
#[tokio::test]
#[ignore = "live: hits the real TONE3000 API; run via `make test-live`"]
async fn enum_vocabulary_is_current() {
    use tone3000::{ArchitectureVersion, Format, Gear, License, Size, ToneSort};

    let (client, _access) = common::authed().await;
    let mut unknown: Vec<String> = Vec::new();

    // A broad sweep rather than a targeted query — the point is to see as much of the
    // live vocabulary as one page allows. Sorted newest-first: an unsorted search skews
    // toward popular tones, and rare vocabulary (e.g. `space`, `experimental`) may never
    // land on page 1 of a popularity-ranked result. New tones surface new vocabulary first.
    let tones = client
        .search(SearchParams {
            page_size: Some(100),
            sort: Some(ToneSort::Newest),
            ..Default::default()
        })
        .await
        .expect("search succeeds");
    assert!(!tones.data.is_empty(), "search returned no tones");

    for t in &tones.data {
        if let Some(Gear::Other(v)) = &t.gear {
            unknown.push(format!("tone {}: unknown Gear {v:?}", t.id));
        }
        if let Some(Format::Other(v)) = &t.format {
            unknown.push(format!("tone {}: unknown Format {v:?}", t.id));
        }
        if let Some(License::Other(v)) = &t.license {
            unknown.push(format!("tone {}: unknown License {v:?}", t.id));
        }
        for s in &t.sizes {
            if let Size::Other(v) = s {
                unknown.push(format!("tone {}: unknown Size {v:?}", t.id));
            }
        }
    }

    // Every `if let Some(X::Other(v))` above stays silently green if the API renames or
    // drops a field (exactly what happened to `platform` -> `format`): the field
    // deserializes to `None`, `unknown` stays empty, and the test passes having compared
    // nothing. Assert each field was actually populated by at least one sampled tone, so a
    // rename/drop trips loudly instead of vanishing into a green run.
    assert!(
        tones.data.iter().any(|t| t.format.is_some()),
        "no tone in the sample carried `format` — field renamed or dropped upstream?"
    );
    assert!(
        tones.data.iter().any(|t| t.gear.is_some()),
        "no tone in the sample carried `gear` — field renamed or dropped upstream?"
    );
    assert!(
        tones.data.iter().any(|t| t.license.is_some()),
        "no tone in the sample carried `license` — field renamed or dropped upstream?"
    );
    assert!(
        tones.data.iter().any(|t| !t.sizes.is_empty()),
        "no tone in the sample carried `sizes` — field renamed or dropped upstream?"
    );

    // Models carry their own size and architecture vocabulary. Use the first tone with a
    // model in the sample rather than hard-failing on `tones.data[0]`: a tone with zero
    // models is not itself a sign of drift, just a confusing place to fail.
    let sample_tone = tones
        .data
        .iter()
        .find(|t| t.models_count > 0)
        .unwrap_or_else(|| {
            panic!(
                "no tone in the sample of {} has any models",
                tones.data.len()
            )
        });

    let models = client
        .models(sample_tone.id, Default::default())
        .await
        .expect("models list succeeds");
    assert!(
        !models.data.is_empty(),
        "tone {} reported models_count > 0 but the models list is empty",
        sample_tone.id
    );
    for m in &models.data {
        if let Some(Size::Other(v)) = &m.size {
            unknown.push(format!("model {}: unknown Size {v:?}", m.id));
        }
        if let Some(ArchitectureVersion::Other(v)) = &m.architecture_version {
            unknown.push(format!("model {}: unknown ArchitectureVersion {v:?}", m.id));
        }
    }
    assert!(
        models.data.iter().any(|m| m.size.is_some()),
        "no model in the sample carried `size` — field renamed or dropped upstream?"
    );
    assert!(
        models.data.iter().any(|m| m.architecture_version.is_some()),
        "no model in the sample carried `architecture_version` — field renamed or dropped upstream?"
    );

    assert!(
        unknown.is_empty(),
        "\n\nThe API returned {} enum value(s) this SDK does not model:\n\n{}\n\n\
         Add the missing variant(s) to src/models/enums.rs.\n",
        unknown.len(),
        unknown.join("\n")
    );
}

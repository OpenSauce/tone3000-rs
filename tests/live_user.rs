mod common;

use tone3000::SearchParams;

#[tokio::test]
#[ignore = "live: hits the real TONE3000 API; run via `make test-live`"]
async fn user_scoped_contract() {
    let client = common::user_client();
    let base = common::api_base();

    // 1. refresh(): token rotation end-to-end. Seeded with only a refresh token, so a
    //    non-empty access token proves the rotation succeeded.
    let tokens = client.refresh().await.expect("refresh succeeds");
    assert!(
        !tokens.access_token.is_empty(),
        "refresh must yield an access token"
    );
    let access = tokens.access_token.clone();

    // 2. user(): correlation + drift (reuses the freshly minted access token).
    let user = client.user().await.expect("user profile succeeds");
    assert!(!user.id.is_empty(), "user id should be non-empty");
    let raw = common::raw_json(&format!("{base}/user"), &access).await;
    common::drift_warn("GET /user", &raw, &serde_json::to_value(&user).unwrap());

    // 3. created(): user-scoped tone list parses; ids populated.
    let created = client
        .created(SearchParams::default())
        .await
        .expect("created tones succeeds");
    for t in &created.items {
        assert!(!t.id.is_empty(), "created tone id should be non-empty");
    }

    // 4. favorited(): same.
    let favorited = client
        .favorited(SearchParams::default())
        .await
        .expect("favorited tones succeeds");
    for t in &favorited.items {
        assert!(!t.id.is_empty(), "favorited tone id should be non-empty");
    }
}

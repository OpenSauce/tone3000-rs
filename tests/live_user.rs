mod common;

use tone3000::ListParams;

#[tokio::test]
#[ignore = "live: hits the real TONE3000 API; run via `make test-live`"]
async fn user_scoped_contract() {
    let (client, access) = common::authed().await;
    let base = common::api_base();

    // refresh already happened inside authed(); a non-empty access token proves rotation.
    assert!(!access.is_empty(), "refresh must yield an access token");

    let user = client.user().await.expect("user profile succeeds");
    assert!(!user.id.0.is_empty(), "user id should be non-empty");
    let raw = common::raw_json(&format!("{base}/user"), &access).await;
    common::drift_warn("GET /user", &raw, &serde_json::to_value(&user).unwrap());

    let created = client
        .created(ListParams::default())
        .await
        .expect("created tones succeeds");
    for t in &created.data {
        assert!(t.id.0 > 0, "created tone id should be set");
    }
    let favorited = client
        .favorited(ListParams::default())
        .await
        .expect("favorited tones succeeds");
    for t in &favorited.data {
        assert!(t.id.0 > 0, "favorited tone id should be set");
    }
}

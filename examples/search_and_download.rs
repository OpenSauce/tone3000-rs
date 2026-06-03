//! Run with: cargo run --example search_and_download
//!
//! Every TONE3000 API call requires an OAuth access token, so set both:
//!   T3K_PUB_KEY          your publishable key (the OAuth client_id)
//!   T3K_ACCESS_TOKEN     a user access token (obtain via the OAuth flow; see `oauth`)
//! A token-less client would fail every call with `Error::Unauthenticated`.

use tone3000::{Client, ModelListParams, SearchParams};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let key = std::env::var("T3K_PUB_KEY").expect("set T3K_PUB_KEY");
    let access = std::env::var("T3K_ACCESS_TOKEN").expect("set T3K_ACCESS_TOKEN");
    let client = Client::builder(key).access_token(access).build();

    let results = client
        .search(SearchParams {
            query: Some("plexi".into()),
            ..Default::default()
        })
        .await?;

    let Some(tone) = results.data.first() else {
        println!("no tones found");
        return Ok(());
    };
    println!("tone: {} ({})", tone.title, tone.id);

    let models = client.models(tone.id, ModelListParams::default()).await?;
    if let Some(model) = models.data.first() {
        let bytes = client.download_model(model).await?;
        println!("downloaded {} bytes for model {}", bytes.len(), model.id);
    }
    Ok(())
}

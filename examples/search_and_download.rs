//! Run with: cargo run --example search_and_download
//! Set T3K_PUB_KEY in the environment.

use tone3000::{Client, ModelListParams, SearchParams};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let key = std::env::var("T3K_PUB_KEY").expect("set T3K_PUB_KEY");
    let client = Client::new(key);

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

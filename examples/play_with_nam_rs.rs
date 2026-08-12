//! Download a tone's model and run audio through it with `nam-rs`.
//!
//! This is the whole pitch in one file: browse the TONE3000 library, pull a capture, and
//! process samples through it — what an amp sim does when a user picks a tone.
//!
//!   export T3K_PUB_KEY=t3k_pub_...
//!   export T3K_ACCESS_TOKEN=...           # from `cargo run --example oauth_desktop`
//!   cargo run --example play_with_nam_rs
//!
//! `nam-rs` is a dev-dependency here, so running this example does not make it a
//! dependency of your project. This crate is transport only.

use tone3000::{ArchitectureVersion, Client, Format, ToneSort};

#[tokio::main]
async fn main() {
    // Print the crate's Display, not the derived Debug a `Result`-returning
    // main would dump.
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let key = std::env::var("T3K_PUB_KEY").expect("set T3K_PUB_KEY");
    let access = std::env::var("T3K_ACCESS_TOKEN")
        .expect("set T3K_ACCESS_TOKEN — run `cargo run --example oauth_desktop` to get one");
    let client = Client::builder(key).access_token(access).build();

    // NAM only: an impulse response is a convolution file, not something nam-rs loads.
    // Architecture v1, because that is what the models endpoint serves by default.
    let page = client
        .tones()
        .format(Format::Nam)
        .sort(ToneSort::DownloadsAllTime)
        .page_size(10)
        .await?;

    let mut chosen = None;
    for tone in &page.data {
        if tone.a1_models_count == 0 {
            continue;
        }
        let models = client
            .models(tone.id)
            .architecture(ArchitectureVersion::V1)
            .page_size(1)
            .await?;
        if let Some(model) = models.data.into_iter().next() {
            chosen = Some((tone.clone(), model));
            break;
        }
    }
    let Some((tone, model)) = chosen else {
        println!("no NAM model found in the top tones — try again or widen the search");
        return Ok(());
    };

    // Search results omit `license`, so re-fetch the tone before reporting it — printing
    // `None` here would read as "no licence" rather than "wrong endpoint".
    let tone = client.tone(tone.id).await?;

    println!("tone:    {} (#{})", tone.title, tone.id);
    println!(
        "model:   {} ({:?})",
        model.name,
        model.size.as_ref().map(|s| s.as_str())
    );
    println!("licence: {:?}", tone.license);

    // Download straight to a String — no disk round-trip. `nam-rs` also has `from_file`
    // if you would rather stream to disk with `download_model_to` first.
    let json = client.download_model_json(&model).await?;
    println!("downloaded {} bytes of model JSON", json.len());

    // The loaders are synchronous. Both are called after the download resolves; for a
    // large model on an async runtime, wrap them in `tokio::task::spawn_blocking`.
    let nam = nam_rs::NamModel::from_json_str(&json)?;
    let mut runtime = nam_rs::Model::from_nam(&nam)?;
    println!("loaded into nam-rs");

    // A quarter-second of quiet-ish signal, processed in place. Real code feeds this from
    // the audio callback.
    let mut buffer: Vec<f32> = (0..12_000).map(|i| (i as f32 * 0.01).sin() * 0.1).collect();
    let peak_in = peak(&buffer);
    runtime.process_buffer(&mut buffer);
    let peak_out = peak(&buffer);

    println!(
        "processed {} samples: peak {peak_in:.4} in -> {peak_out:.4} out",
        buffer.len()
    );
    println!("\nthat is the whole path: search -> download -> play");

    Ok(())
}

fn peak(buffer: &[f32]) -> f32 {
    buffer.iter().fold(0.0f32, |acc, s| acc.max(s.abs()))
}

//! Browse, search, inspect a tone, and download one of its models.
//!
//!   export T3K_PUB_KEY=t3k_pub_...        # your publishable key (the OAuth client_id)
//!   export T3K_ACCESS_TOKEN=...           # from `cargo run --example oauth_desktop`
//!   cargo run --example search_and_download
//!
//! Every TONE3000 endpoint requires an access token — there is no anonymous access, so a
//! token-less client fails every call with `Error::Unauthenticated`.

use tone3000::{ArchitectureVersion, Client, Gear, ToneSort};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let key = std::env::var("T3K_PUB_KEY").expect("set T3K_PUB_KEY (TONE3000 Settings → API Keys)");
    let access = std::env::var("T3K_ACCESS_TOKEN")
        .expect("set T3K_ACCESS_TOKEN — run `cargo run --example oauth_desktop` to get one");
    let client = Client::builder(key).access_token(access).build();

    // 1. A landing page. No query, so this is browse rather than search.
    let landing = client
        .tones()
        .sort(ToneSort::DownloadsAllTime)
        .page_size(5)
        .await?;
    println!(
        "Top tones ({} total, {} pages):",
        landing.total, landing.total_pages
    );
    for tone in &landing.data {
        println!(
            "  {:>7}  {:<50} {:>7} downloads",
            tone.id,
            truncate(&tone.title, 50),
            tone.downloads_count
        );
    }

    // 2. A search. Same endpoint, same builder, now with filters.
    let results = client
        .tones()
        .query("plexi")
        .gear(Gear::Amp)
        .page_size(5)
        .await?;
    println!("\n'plexi' amps: {} matches", results.total);

    let Some(summary) = results.data.first() else {
        println!("no matches — nothing further to show");
        return Ok(());
    };

    // 3. Detail. Search results omit `license`, `sizes`, `links` and `is_public`, so fetch
    //    the tone itself when you need them.
    let tone = client.tone(summary.id).await?;
    println!("\n{} (#{})", tone.title, tone.id);
    if let Some(user) = &tone.user {
        println!("  by         {}", user.username);
    }
    println!("  gear       {:?}", tone.gear);
    println!("  format     {:?}", tone.format);
    println!(
        "  license    {:?}   <- check before redistributing",
        tone.license
    );
    println!(
        "  sizes      {:?}",
        tone.sizes.iter().map(|s| s.as_str()).collect::<Vec<_>>()
    );
    println!(
        "  makes      {:?}",
        tone.makes
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>()
    );
    println!(
        "  models     {} total: {} v1, {} v2, {} IR, {} custom",
        tone.models_count,
        tone.a1_models_count,
        tone.a2_models_count,
        tone.irs_count,
        tone.custom_models_count
    );

    // 4. Models. The API serves one architecture at a time and defaults to v1, so asking
    //    once can return fewer models than `models_count` promised.
    let mut models = Vec::new();
    for (count, arch) in [
        (tone.a1_models_count, ArchitectureVersion::V1),
        (tone.a2_models_count, ArchitectureVersion::V2),
        (tone.custom_models_count, ArchitectureVersion::Custom),
    ] {
        if count > 0 {
            let page = client.models(tone.id).architecture(arch.clone()).await?;
            println!(
                "  fetched {} model(s) for architecture {}",
                page.data.len(),
                arch.as_str()
            );
            models.extend(page.data);
        }
    }

    // 5. Download one.
    let Some(model) = models.first() else {
        println!("\ntone has no downloadable models");
        return Ok(());
    };
    let bytes = client.download_model(model).await?;
    println!(
        "\ndownloaded '{}' ({:?}, arch {:?}): {} bytes",
        model.name,
        model.size.as_ref().map(|s| s.as_str()),
        model.architecture_version.as_ref().map(|a| a.as_str()),
        bytes.len()
    );
    println!("hand these bytes to nam-rs — see the play_with_nam_rs example");

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max - 1).collect::<String>() + "…"
}

# tone3000

Async Rust client for the [TONE3000](https://www.tone3000.com) API (v1): search the
community tone library, read tone/model metadata, download `.nam`/IR files, and
authenticate users via OAuth 2.0 + PKCE.

## Quick start

```rust
use tone3000::{Client, SearchParams};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let client = Client::new("t3k_pub_your_key");
    let results = client
        .search(SearchParams { query: Some("plexi".into()), ..Default::default() })
        .await?;
    for tone in results.items {
        println!("{}: {}", tone.id, tone.name);
    }
    Ok(())
}
```

## Auth

- **App key:** `Client::new("t3k_pub_…")` for public reads.
- **OAuth PKCE:** use `tone3000::pkce::generate()` + `tone3000::oauth::authorize_url(...)`,
  then `client.exchange_code(...)`. The app owns the redirect transport.

## Downloading models

A model's file is fetched from its `model_url`. Three download methods cover the common
shapes:

- `download_model(&model) -> Bytes` — buffered, raw bytes.
- `download_model_json(&model) -> String` — the `.nam` file as a JSON string.
- `download_model_to(&model, &mut writer) -> u64` — streams to any `AsyncWrite`
  (e.g. a file), returning bytes written.

This crate is transport-only; it does not parse `.nam` internals or place files. It pairs
with [`nam-rs`](https://github.com/OpenSauce/nam-rs), which loads a model from either a JSON
string or a file path — so both download paths plug straight in:

```rust
// In-memory: download JSON, hand it to nam-rs (no disk round-trip)
let json = client.download_model_json(&model).await?;
let nam = nam_rs::NamModel::from_json_str(&json)?;
let runtime = nam_rs::Model::from_nam(&nam)?;

// On disk: stream to a file, then load by path
let mut file = tokio::fs::File::create(&path).await?;
client.download_model_to(&model, &mut file).await?;
let nam = nam_rs::NamModel::from_file(&path)?; // sync, blocking IO
let runtime = nam_rs::Model::from_nam(&nam)?;
```

(`nam-rs`'s loaders are synchronous; call them after the async download completes, or wrap
`from_file` in `tokio::task::spawn_blocking` for large models on an async runtime.)

## License

MIT

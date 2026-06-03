# tone3000

Async Rust client for the [TONE3000](https://www.tone3000.com) API (v1): search the
community tone library, read tone/model metadata, download `.nam`/IR files, and
authenticate users via OAuth 2.0 + PKCE.

## Quick start

```rust
use tone3000::{Client, SearchParams};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    // Every API call requires an OAuth access token (see "Auth" below).
    let client = Client::builder("t3k_pub_your_key")
        .access_token("user_access_token")
        .build();
    let results = client
        .search(SearchParams { query: Some("plexi".into()), ..Default::default() })
        .await?;
    for tone in results.data {
        println!("{}: {}", tone.id, tone.title);
    }
    Ok(())
}
```

Results come back as a `Page<T>` (`data`, `page`, `page_size`, `total`, `total_pages`).

## Auth

Every endpoint requires `Authorization: Bearer <access token>` — there is no anonymous or
app-key access. The publishable key (`t3k_pub_…`) is only the OAuth `client_id`; it is not a
valid Bearer token. A call made without an access token returns `Error::Unauthenticated`.

Obtain a token via OAuth 2.0 + PKCE (the app owns the redirect transport):

```rust
let pkce = tone3000::pkce::generate();
let state = /* an unguessable value you store and verify on the callback */;
let url = tone3000::oauth::authorize_url(
    "t3k_pub_…", "http://localhost:8765/callback", &pkce.challenge, state, tone3000::Prompt::Standard,
);
// open `url`, capture the redirected `code`, then:
let tokens = client.exchange_code(&code, &pkce.verifier, "http://localhost:8765/callback").await?;
```

Seed a client from stored tokens with `Client::builder(key).access_token(at).refresh_token(rt)`;
with `.auto_refresh(true)` the client transparently refreshes near expiry and retries once on a 401.

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

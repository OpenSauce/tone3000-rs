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

## License

MIT

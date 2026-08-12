# tone3000

[![crates.io](https://img.shields.io/crates/v/tone3000.svg)](https://crates.io/crates/tone3000)
[![docs.rs](https://img.shields.io/docsrs/tone3000)](https://docs.rs/tone3000)
[![CI](https://github.com/OpenSauce/tone3000-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/OpenSauce/tone3000-rs/actions/workflows/ci.yml)

Async Rust client for the [TONE3000](https://www.tone3000.com) API (v1): browse and search
the community tone library, download `.nam` and IR files, and authenticate users with
OAuth 2.0 + PKCE.

Pairs with [`nam-rs`](https://github.com/OpenSauce/nam-rs), which runs the models this
crate downloads.

```console
$ cargo add tone3000
$ cargo add tokio -F macros,rt-multi-thread   # the snippets below are async
```

## Read this first

**Every endpoint requires an end-user OAuth login.** There is no anonymous access.
Searching the public library needs a user access token exactly as much as reading that
user's favourites does. The publishable key (`t3k_pub_…`) is the OAuth `client_id`, not a
credential — a call without an access token fails with `Error::Unauthenticated` before it
reaches the network.

If your product can't put a user through a browser login, this API isn't usable.

## Concepts

| Type | What it is |
|---|---|
| `Tone` | A capture project: one piece of gear, captured once, published with a title and licence. Not downloadable itself. |
| `Model` | One downloadable file belonging to a tone, at a given size and architecture. Six models is six variants of one capture, not six amps. |
| `Make` | The real-world gear captured, e.g. "Mesa Boogie Badlander". |
| `Size` | The CPU/quality trade-off (`standard`, `lite`, `feather`, `nano`, `custom`), not a byte count. |

A tone has many models; a model has a `model_url` you download. `nam-rs` parses those bytes
into a `NamModel` and loads it into a `nam_rs::Model` you can run audio through — so "model"
means something different in each crate. Fully qualify it in code using both.

## Quick start

```rust,no_run
use tone3000::{Client, ToneSort};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let client = Client::builder("t3k_pub_your_key")
        .access_token("user_access_token") // see Authentication
        .build();

    let page = client
        .tones()
        .sort(ToneSort::DownloadsAllTime)
        .page_size(24)
        .await?;

    for tone in &page.data {
        println!("{} — {} downloads", tone.title, tone.downloads_count);
    }
    println!("page {} of {}", page.page, page.total_pages);
    Ok(())
}
```

`client.tones()` is browse and search both — add `.query("plexi")` and it becomes a search.
Filters are optional and chain:

```rust,no_run
use tone3000::{Client, Format, Gear, Size};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let client = Client::builder("t3k_pub_your_key").access_token("token").build();

    let page = client
        .tones()
        .query("plexi")
        .gear(Gear::Amp)      // repeatable; multiple values are OR'd
        .format(Format::Nam)  // filter for IRs here, not through gear
        .size(Size::Standard)
        .make("Marshall")     // exact match, not substring
        .await?;

    println!("{} matches", page.total);

    if page.has_next() {
        let next = client.tones().query("plexi").page(page.page + 1).await?;
        println!("next page has {}", next.data.len());
    }
    Ok(())
}
```

## Authentication

Get a publishable key from TONE3000 **Settings → API Keys** and register your redirect URI
there; localhost is allowed in development. This crate handles the token exchange, and your
app owns the redirect, because a desktop loopback, a plugin host and a web callback are
different problems.

```rust,no_run
use tone3000::{AuthorizeOptions, Client, Prompt, oauth, pkce};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let pkce = pkce::generate();
    let state = "an-unguessable-value-you-store";

    let url = oauth::authorize_url(
        "t3k_pub_your_key",
        "http://localhost:8765/callback",
        &pkce.challenge,
        state,
        Prompt::Standard,
        AuthorizeOptions::default(),
    );
    println!("open this: {url}");

    // Capture `code` from the redirect, and check `state` matches before trusting it.
    let code = "from_the_redirect";

    let client = Client::builder("t3k_pub_your_key").build();
    let tokens = client
        .exchange_code(code, &pkce.verifier, "http://localhost:8765/callback")
        .await?;

    println!("{}", tokens.access_token);
    Ok(())
}
```

`Prompt::SelectTone`, `Prompt::LoadTone { tone_id }` and `Prompt::LoadModel { model_id }`
hand the browsing to TONE3000's own UI instead, which saves building one.

[`examples/oauth_desktop.rs`](https://github.com/OpenSauce/tone3000-rs/blob/main/examples/oauth_desktop.rs) does this end to end with a real
loopback listener, in std only.

**Refresh tokens rotate.** Each refresh invalidates the previous one, so store every token
you are handed or the session dies:

```rust,no_run
use tone3000::Client;

fn main() {
    let client = Client::builder("t3k_pub_your_key")
        .access_token("stored_access_token")
        .refresh_token("stored_refresh_token")
        .expires_at(1_800_000_000) // unix seconds; without it, refresh is reactive only
        .auto_refresh(true)        // refresh near expiry, retry once on a 401
        .on_tokens_changed(|tokens| {
            // persist tokens.access_token and tokens.refresh_token
        })
        .build();
}
```

## Downloading

```rust,no_run
use tone3000::{Client, Model};

async fn download(client: &Client, model: &Model) -> tone3000::Result<()> {
    let bytes = client.download_model(model).await?;
    let json = client.download_model_json(model).await?;

    let mut file = tokio::fs::File::create("model.nam").await?;
    client.download_model_to(model, &mut file).await?;
    Ok(())
}
```

`download_url` and `download_url_to` take a `model_url` directly, for when you stored one
rather than a whole `Model`.

`client.models(id)` returns one architecture at a time — the API defaults to v1 and has no
"all" option. `Client::models` documents the loop that collects them all.

## With nam-rs

`cargo add nam-rs`, then:

```rust,no_run
use tone3000::{Client, Model};

async fn play(client: &Client, model: &Model) -> Result<(), Box<dyn std::error::Error>> {
    let json = client.download_model_json(model).await?;

    // nam-rs's loaders are synchronous: call them once the download resolves.
    let nam = nam_rs::NamModel::from_json_str(&json)?;
    let mut runtime = nam_rs::Model::from_nam(&nam)?;

    let mut buffer = vec![0.0f32; 512];
    runtime.process_buffer(&mut buffer);
    Ok(())
}
```

## Examples

| Example | Shows | Needs |
|---|---|---|
| [`oauth_desktop`](https://github.com/OpenSauce/tone3000-rs/blob/main/examples/oauth_desktop.rs) | The PKCE flow with a loopback listener. **Start here** — it produces the token the others need | `T3K_PUB_KEY` |
| [`search_and_download`](https://github.com/OpenSauce/tone3000-rs/blob/main/examples/search_and_download.rs) | Browse, search, tone detail, models, download | both |
| [`play_with_nam_rs`](https://github.com/OpenSauce/tone3000-rs/blob/main/examples/play_with_nam_rs.rs) | Download a capture and run audio through it | both |

```console
$ export T3K_PUB_KEY=t3k_pub_...
$ cargo run --example oauth_desktop        # prints an access token
$ export T3K_ACCESS_TOKEN=...
$ cargo run --example search_and_download
```

## Errors and limits

`Error` separates what you would branch on: `Unauthenticated` (no token set), `Unauthorized`
(401), `Forbidden` (403), `RateLimited { retry_after }`, `Status { code, body }`, and `Http`
for transport failures. `Error::is_timeout()` and `Error::is_connect()` answer the usual
retry questions without matching the variant out.

The API allows 100 requests per minute by default, and search is limited more tightly.
Honour `retry_after`.

## Coverage

Implemented: tone search and browse, tone detail, created, favorited, model list and detail,
model download, the user profile, the public user directory, and the OAuth token flows.

Not yet implemented, tracked as issues: `/tones/trending` and `/tones/latest`
([#9](https://github.com/OpenSauce/tone3000-rs/issues/9)), `/tones/downloaded`
([#10](https://github.com/OpenSauce/tone3000-rs/issues/10)), `/makes` and `/tags`
([#11](https://github.com/OpenSauce/tone3000-rs/issues/11)), and the API's deprecation
headers ([#12](https://github.com/OpenSauce/tone3000-rs/issues/12)). `GET /tones/download`
is restricted to approved partners, so it is deliberately absent — download individual
models via `model_url`.

## Compatibility

- **MSRV 1.86**, set by `url` → `idna_adapter` → `icu_*`. An MSRV bump is a minor bump.
- Async only, on `tokio`. TLS via `rustls`, no OpenSSL.

Your integration must also follow TONE3000's
[Design Requirements and Commercial Terms](https://www.tone3000.com/api), and you should
check `tone.license` before redistributing a model.

## Development

| Command | What it does | Credentials |
|---|---|---|
| `cargo test` | Unit + wiremock suites; live tests are `#[ignore]`d | No |
| `make test-live` | Live contract and enum-vocabulary checks | Yes |
| `make test-oauth` | Interactive OAuth bootstrap; prints a refresh token | Yes, a browser |
| `make check-upstream` | Diffs upstream `types.ts` against a pinned SHA | Needs `gh` |

`make check-upstream` also runs weekly in CI.

Licensed under MIT.

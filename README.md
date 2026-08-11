# tone3000

[![crates.io](https://img.shields.io/crates/v/tone3000.svg)](https://crates.io/crates/tone3000)
[![docs.rs](https://img.shields.io/docsrs/tone3000)](https://docs.rs/tone3000)
[![CI](https://github.com/OpenSauce/tone3000-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/OpenSauce/tone3000-rs/actions/workflows/ci.yml)

Async Rust client for the [TONE3000](https://www.tone3000.com) API (v1): browse and search
the community tone library, read metadata, download `.nam` and IR files, and authenticate
users with OAuth 2.0 + PKCE.

Pairs with [`nam-rs`](https://github.com/OpenSauce/nam-rs), which runs the models this
crate downloads — together they put the whole TONE3000 library inside an amp sim.

```console
$ cargo add tone3000
```

The crate is `tone3000`; the repository is `tone3000-rs`, following the Rust habit of
suffixing repo names. Nothing you type says `-rs`.

## Read this first

**Every endpoint requires an end-user OAuth login.** There is no anonymous access and no
app-key mode. Searching the public library needs a user access token exactly as much as
reading that user's favourites does. Your publishable key (`t3k_pub_…`) is the OAuth
`client_id`, not a credential — a call without an access token fails with
`Error::Unauthenticated` before it reaches the network.

If your product can't put a user through a browser login, this API isn't usable. That's
worth knowing now rather than twenty lines in.

## Concepts

Three things are called "model" across these two crates. The pipeline tells them apart:

```text
Tone ──has many──> Model ──model_url──> bytes ──> nam_rs::NamModel ──> nam_rs::Model
(the capture        (one                (the      (the parsed          (the loaded
 project)            downloadable        file)     file)                inference engine)
                     variant)
```

- **Tone** — a community capture project: one piece of gear, captured once, published with
  a title and a licence. Not downloadable itself.
- **Model** — one downloadable file belonging to a tone, at a particular size and
  architecture. A tone with six models is six variants of the same capture, not six
  different amps.
- **Make** — the real-world gear captured, e.g. "Mesa Boogie Badlander".
- **Size** — the CPU/quality trade-off (`standard`, `lite`, `feather`, `nano`), not a byte
  count.

Names here mirror the API's own vocabulary deliberately, so what you read in TONE3000's
docs is what you type here.

## Quick start

```rust,no_run
use tone3000::{Client, ToneSort};

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let client = Client::builder("t3k_pub_your_key")
        .access_token("user_access_token") // see Authentication below
        .build();

    // A landing page: the most-downloaded tones, 24 at a time.
    let page = client
        .tones()
        .sort(ToneSort::DownloadsAllTime)
        .page_size(24)
        .await?;

    for tone in &page.data {
        println!("{} — {} models, {} downloads", tone.title, tone.models_count, tone.downloads_count);
    }
    println!("page {} of {}, more: {}", page.page, page.total_pages, page.has_next());
    Ok(())
}
```

`client.tones()` is both browse and search — add `.query("plexi")` and it becomes a search.
Every filter is optional and chains:

```rust,no_run
# async fn run(client: tone3000::Client) -> tone3000::Result<()> {
use tone3000::{Format, Gear, Size};

let page = client
    .tones()
    .query("plexi")
    .gear(Gear::Amp)          // repeatable; multiple values are OR'd
    .format(Format::Nam)      // filter for IRs here, not through gear
    .size(Size::Standard)
    .make("Marshall")         // exact match — see the caveat below
    .page_size(50)
    .await?;
# Ok(())
# }
```

Requests are builders that implement `IntoFuture`: nothing is sent until you `.await`. They
are `Clone`, so a configured search can walk its own pages.

> **Caveat:** `tag`, `make` and `creator` match **exactly**, not by substring. The `/makes`
> and `/tags` endpoints that would let you discover valid values aren't implemented yet —
> see [#11](https://github.com/OpenSauce/tone3000-rs/issues/11).

## Authentication

Get a publishable key from TONE3000 **Settings → API Keys**, and register your redirect
URI there (localhost origins are allowed in development). Then run OAuth 2.0 + PKCE. This
crate owns the token exchange; your app owns the redirect transport, because a desktop
loopback, a plugin host and a web callback are genuinely different problems.

```rust,no_run
# fn run() -> tone3000::Result<()> {
use tone3000::{oauth, pkce, AuthorizeOptions, Prompt};

let pkce = pkce::generate();
let state = "an-unguessable-value-you-store"; // verify it on the callback

let url = oauth::authorize_url(
    "t3k_pub_your_key",
    "http://localhost:8765/callback",
    &pkce.challenge,
    state,
    Prompt::Standard,
    AuthorizeOptions::default(),
);
println!("open this: {url}");
# Ok(())
# }
```

Open that URL, capture the `code` from the redirect, verify `state` matches, then exchange:

```rust,no_run
# async fn run(client: tone3000::Client, code: &str, pkce: tone3000::Pkce) -> tone3000::Result<()> {
let tokens = client
    .exchange_code(code, &pkce.verifier, "http://localhost:8765/callback")
    .await?;
# Ok(())
# }
```

The `examples/oauth_desktop.rs` example does all of this end to end with a real loopback
listener, in about eighty lines and no extra dependencies.

### Keeping a session alive

**Refresh tokens rotate.** Every refresh returns a new one and invalidates the old, so
persisting only the first token you saw will kill the session. Register a callback and
store whatever it hands you:

```rust,no_run
# fn run() {
use tone3000::Client;

let client = Client::builder("t3k_pub_your_key")
    .access_token("stored_access_token")
    .refresh_token("stored_refresh_token")
    .expires_at(1_800_000_000) // unix seconds; without it, refresh is reactive only
    .auto_refresh(true)        // refresh before expiry, and retry once on a 401
    .on_tokens_changed(|tokens| {
        // persist tokens.access_token and tokens.refresh_token here
    })
    .build();
# }
```

`Prompt` also drives TONE3000's hosted flows — `Prompt::SelectTone` lets the user pick a
tone in TONE3000's own UI, and `Prompt::LoadTone { tone_id }` asks them to confirm access to
a specific one. Both save you building a browser.

## Downloading models

A model's file lives at its `model_url` and needs the same Bearer token as any other call.

```rust,no_run
# async fn run(client: tone3000::Client, model: tone3000::Model) -> tone3000::Result<()> {
// In memory:
let bytes = client.download_model(&model).await?;
// As a JSON string, ready for a NAM loader:
let json = client.download_model_json(&model).await?;
// Or streamed to disk, without buffering the whole file:
let mut file = tokio::fs::File::create("model.nam").await?;
let written = client.download_model_to(&model, &mut file).await?;
# Ok(())
# }
```

If you persisted a URL rather than a whole `Model`, `download_url` and `download_url_to`
take one directly.

### One tone's models arrive one architecture at a time

`GET /models` defaults to architecture 1 and has no "all" option, so `client.models(id)`
returns only the v1 models. Tone 6298 has 112 on v1 and 111 on v2; a bare call returns 112.

Worse, `models_count` means different things on different endpoints — the cross-architecture
total in search results, the v1 count in tone detail. The same tone reports 223 via
`tones()` and 112 via `tone()`. Build a list screen from search and a detail screen from
detail and they will disagree.

To show everything, ask for each architecture the tone reports:

```rust,no_run
# async fn run(client: tone3000::Client, tone: tone3000::Tone) -> tone3000::Result<()> {
use tone3000::ArchitectureVersion::{Custom, V1, V2};

let mut models = Vec::new();
for (count, arch) in [
    (tone.a1_models_count, V1),
    (tone.a2_models_count, V2),
    (tone.custom_models_count, Custom),
] {
    if count > 0 {
        models.extend(client.models(tone.id).architecture(arch).await?.data);
    }
}
# Ok(())
# }
```

## Playing a tone with nam-rs

This crate is transport only — it doesn't parse `.nam` internals or decide where files go.
`nam-rs` takes it from there, loading from a JSON string or a path, so both download paths
plug straight in:

```rust,no_run
# async fn run(client: tone3000::Client, model: tone3000::Model) -> Result<(), Box<dyn std::error::Error>> {
// In memory: no disk round-trip.
let json = client.download_model_json(&model).await?;
let nam = nam_rs::NamModel::from_json_str(&json)?;
let mut runtime = nam_rs::Model::from_nam(&nam)?;

let mut buffer = vec![0.0f32; 512]; // your audio, in place
runtime.process_buffer(&mut buffer);
# Ok(())
# }
```

`nam-rs`'s loaders are synchronous, so call them after the download resolves, or wrap
`NamModel::from_file` in `tokio::task::spawn_blocking` for large models.

## Examples

| Example | Shows | Needs |
|---|---|---|
| [`oauth_desktop`](examples/oauth_desktop.rs) | The full PKCE flow with a loopback listener — **start here**, it produces the token the others need | `T3K_PUB_KEY` |
| [`search_and_download`](examples/search_and_download.rs) | Browse, search, list a tone's models, download one | `T3K_PUB_KEY`, `T3K_ACCESS_TOKEN` |
| [`play_with_nam_rs`](examples/play_with_nam_rs.rs) | Download a model and run audio through it | both, plus `--features` note in the file |

```console
$ export T3K_PUB_KEY=t3k_pub_...
$ cargo run --example oauth_desktop        # prints an access token
$ export T3K_ACCESS_TOKEN=...
$ cargo run --example search_and_download
```

## Errors

`Error` distinguishes what you'd branch on: `Unauthenticated` (no token set),
`Unauthorized` (401), `Forbidden` (403), `RateLimited { retry_after }`, `Status { code, body }`,
and `Http` for transport failures. Transport errors expose `is_timeout()` and `is_connect()`
directly on `Error`, so retry logic doesn't need to match on the variant.

The API allows **100 requests per minute** by default, and search is limited more tightly
than that. Honour `retry_after`.

## Coverage

Implemented: tone search/browse, tone detail, created, favorited, model list, model detail,
model download, the user profile, the public user directory, and the OAuth token flows.

Not yet implemented, tracked as issues: the homepage feeds `/tones/trending` and
`/tones/latest` ([#9](https://github.com/OpenSauce/tone3000-rs/issues/9)),
`/tones/downloaded` ([#10](https://github.com/OpenSauce/tone3000-rs/issues/10)), the
`/makes` and `/tags` taxonomies ([#11](https://github.com/OpenSauce/tone3000-rs/issues/11)),
and surfacing the API's deprecation headers
([#12](https://github.com/OpenSauce/tone3000-rs/issues/12)). `GET /tones/download` is
restricted to approved partners and 403s for everyone else, so it is deliberately absent —
download individual models via `model_url` instead.

## Before you ship an integration

TONE3000 publishes [Design Requirements and Commercial
Terms](https://www.tone3000.com/api) covering entry-point placement, a partnership splash
before sign-in, and creator attribution. They apply to your product, not to this crate, and
they are easier to build in than to retrofit.

Check `tone.license` before redistributing a downloaded model or shipping one with your app.

## Compatibility

- **MSRV 1.86.** Set by `url` → `idna_adapter` → `icu_*`, not by our own code. An MSRV bump
  is a minor version bump.
- Async only, on `tokio`. A `blocking` feature is a possible followup.
- TLS via `rustls`, no OpenSSL.

## Development

| Command | What it does | Needs credentials |
|---|---|---|
| `cargo test` | Unit + wiremock suites. Live tests are `#[ignore]`d | No |
| `make test-live` | Live contract and enum-vocabulary checks | Yes |
| `make test-oauth` | Interactive OAuth bootstrap; prints a refresh token | Yes, a browser |
| `make check-upstream` | Diffs upstream `types.ts` against a pinned SHA | No, but needs `gh` |

`make check-upstream` also runs weekly in CI. Note that GitHub disables scheduled workflows
after 60 days of repository inactivity.

## License

MIT. See [LICENSE](LICENSE).

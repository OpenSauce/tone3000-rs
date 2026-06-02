# CLAUDE.md — tone3000-rs

Guidance for working in this repo. Keep it short; update it as the crate evolves.

## What this is

`tone3000-rs` is a general-purpose **async** Rust client for the [TONE3000](https://www.tone3000.com)
API (v1). It is the **transport + data layer**: typed models, endpoint methods, OAuth
token exchange/refresh, and PKCE helpers. App-specific concerns (OAuth *redirect* listener,
UI, file placement, token storage) stay in the consuming app.

## Working agreements

- **Design docs are not committed.** `DESIGN.md` and `docs/superpowers/` are gitignored
  working notes. The current design spec lives at
  `docs/superpowers/specs/2026-06-02-tone3000-sdk-design.md` — read it before making
  architectural changes.
- **Async-only for v1.** A `blocking` feature is a documented followup, not built yet.
  Keep `src/http.rs` as the single seam touching `reqwest` so the facade stays cheap to add.
- **Lenient deserialization.** Models use `#[serde(default)]` + `Option<T>`; an unexpected
  field must never fail a whole response. Public enums are `#[non_exhaustive]`.
- **Errors via `thiserror`.** Surface 401 / 403 / 429 distinctly; `RateLimited` carries
  `retry_after`. User-scoped calls in app-key mode return `Error::Unauthenticated`.
- **No live network in tests.** Use `wiremock` + scrubbed response fixtures.

## Architecture (Approach A)

Single configurable `Client` holding an `Auth` state enum (`AppKey` | `Bearer`). Every
method works in any auth mode; user-scoped calls error clearly when unauthenticated.
`oauth` and `pkce` are standalone modules — the app owns the redirect transport.

```
src/
  lib.rs        client.rs     error.rs     http.rs      oauth.rs     pkce.rs
  models/  tone.rs  model.rs  user.rs
  endpoints/  tones.rs  models.rs  users.rs
```

## Conventions

- Edition 2024. License MIT.
- CI: `cargo fmt --check`, `cargo clippy` (strict), `cargo test`.
- HTTP via `reqwest` (rustls, no OpenSSL). PKCE hand-rolled with `sha2` + `base64` + `rand`
  (avoid the heavy `oauth2` crate).
- Don't over-build: cover the documented v1 endpoints; the API will churn.

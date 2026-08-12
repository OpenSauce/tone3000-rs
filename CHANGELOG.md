# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) — while on `0.x`, breaking
changes land in minor releases.

## [Unreleased]

## [0.1.0] - 2026-08-12

First release.

### Added

- Tone browse and search via a fluent `client.tones()` builder — the same endpoint serves
  both, so omitting the query browses. Filters for gear, format, size, tag, make, creator,
  architecture, sort and pagination.
- `client.tone(id)` for tone detail, `client.created()` and `client.favorited()` for the
  authenticated user's tones, `client.models(tone_id)` and `client.model(id)` for models,
  `client.user()` and `client.users()` for profiles and the public directory.
- Model downloads: `download_model`, `download_model_json`, and `download_model_to` for
  streaming to any `AsyncWrite`, plus `download_url`/`download_url_to` for a stored URL.
- OAuth 2.0 + PKCE: `pkce::generate`, `oauth::authorize_url`, `Client::exchange_code` and
  `Client::refresh`, with optional proactive refresh, one retry on a 401, and an
  `on_tokens_changed` callback. The app owns the redirect transport.
- Typed models with lenient deserialization, open enums carrying an `Other(String)` for
  values this version does not know, and `Page<T>` with `has_next()`/`has_prev()`.
- Three runnable examples: `oauth_desktop`, `search_and_download`, `play_with_nam_rs`.

### Notes

- Every endpoint requires an end-user OAuth access token; there is no anonymous access.
- `client.models(id)` returns one architecture at a time — the API defaults to v1 and has
  no "all" option.
- `Tone::models_count` means the cross-architecture total in search results and the v1
  count in tone detail.
- Not yet covered: `/tones/trending`, `/tones/latest`, `/tones/downloaded`, `/makes`,
  `/tags`, and the API's deprecation headers. See issues #9–#12.

[Unreleased]: https://github.com/OpenSauce/tone3000-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/OpenSauce/tone3000-rs/releases/tag/v0.1.0

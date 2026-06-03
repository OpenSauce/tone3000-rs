# Sources credentials from a gitignored .env if present; otherwise relies on the
# ambient environment. See docs/superpowers/specs/2026-06-03-live-test-suite-design.md.
-include .env
export

.PHONY: test test-live test-oauth fmt clippy

# Offline suite: unit + wiremock. Live tests are #[ignore]d and excluded here.
test:
	cargo test

# Non-interactive live contract tests against the real API.
test-live:
	cargo test --test live_public --test live_user -- --ignored --nocapture

# Interactive OAuth bootstrap: prints the authorize URL, reads a pasted code.
test-oauth:
	cargo test --test live_oauth -- --ignored --nocapture

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets -- -D warnings

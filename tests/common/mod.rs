//! Shared helpers for the opt-in live test suite. Each `tests/live_*.rs` binary uses a
//! subset of these, so unused-warnings are silenced here.
#![allow(dead_code)]

use serde_json::Value;
use tone3000::{Client, DEFAULT_BASE_URL};

/// Read a required env var, panicking with a clear message if unset/empty.
pub fn require_env(name: &str) -> String {
    match std::env::var(name) {
        Ok(v) if !v.trim().is_empty() => v,
        _ => panic!("set {name} to run the live test suite (e.g. add it to .env)"),
    }
}

fn base_url_override() -> Option<String> {
    std::env::var("TONE3000_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// The API base URL in effect (override or the crate default).
pub fn api_base() -> String {
    base_url_override().unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
}

/// Build an authenticated client (publishable key + refresh token) and mint a fresh access
/// token, returning both the client and the access token (for raw drift GETs). Every
/// endpoint requires an access token, so all live tests go through this.
pub async fn authed() -> (Client, String) {
    let key = require_env("TONE3000_API_KEY");
    let refresh = require_env("TONE3000_REFRESH_TOKEN");
    let mut b = Client::builder(key).refresh_token(refresh);
    if let Some(u) = base_url_override() {
        b = b.base_url(u);
    }
    let client = b.build();
    let tokens = client
        .refresh()
        .await
        .expect("refresh to mint an access token");
    (client, tokens.access_token)
}

/// GET a URL with a bearer token and parse the body as raw JSON, for drift comparison.
pub async fn raw_json(url: &str, bearer: &str) -> Value {
    reqwest::Client::new()
        .get(url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {bearer}"))
        .send()
        .await
        .expect("raw GET sends")
        .json::<Value>()
        .await
        .expect("raw body is JSON")
}

/// Warn loudly (never fail) on any key present in `raw` but not in `modeled`.
pub fn drift_warn(label: &str, raw: &Value, modeled: &Value) {
    let mut unmodeled = Vec::new();
    collect_unmodeled("", raw, modeled, &mut unmodeled);
    if !unmodeled.is_empty() {
        eprintln!(
            "\n⚠ drift: {label} response has unmodeled field(s): {}\n",
            unmodeled.join(", ")
        );
    }
}

fn collect_unmodeled(path: &str, raw: &Value, modeled: &Value, out: &mut Vec<String>) {
    match (raw, modeled) {
        (Value::Object(raw_map), Value::Object(mod_map)) => {
            for (k, raw_v) in raw_map {
                let child = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                match mod_map.get(k) {
                    None => out.push(child),
                    Some(mod_v) => collect_unmodeled(&child, raw_v, mod_v, out),
                }
            }
        }
        (Value::Array(raw_arr), Value::Array(mod_arr)) => {
            if let (Some(r), Some(m)) = (raw_arr.first(), mod_arr.first()) {
                collect_unmodeled(path, r, m, out);
            }
        }
        _ => {}
    }
}

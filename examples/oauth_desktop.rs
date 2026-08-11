//! The full OAuth 2.0 + PKCE flow for a desktop app, end to end.
//!
//! Start here: the other examples need an access token, and this is what produces one.
//! It needs only your publishable key.
//!
//!   export T3K_PUB_KEY=t3k_pub_...
//!   cargo run --example oauth_desktop
//!
//! Register `http://localhost:8765/callback` as a redirect URI in TONE3000
//! Settings → API Keys first. Localhost origins are allowed in development.
//!
//! This crate deliberately does not own the redirect listener — a desktop loopback, a
//! plugin host and a web callback are different problems. The listener below is the
//! desktop answer, in std only.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use tone3000::{AuthorizeOptions, Client, Prompt, oauth, pkce};

const REDIRECT_URI: &str = "http://localhost:8765/callback";

#[tokio::main]
async fn main() -> tone3000::Result<()> {
    let key = std::env::var("T3K_PUB_KEY")
        .expect("set T3K_PUB_KEY to your publishable key (TONE3000 Settings → API Keys)");

    // The verifier stays here; only its hash goes in the URL.
    let pkce = pkce::generate();

    // `state` is yours to generate and verify. It is what stops an attacker feeding your
    // listener a code from a different session, so it must be unguessable and checked.
    let state = pkce::generate().verifier;

    let url = oauth::authorize_url(
        &key,
        REDIRECT_URI,
        &pkce.challenge,
        &state,
        Prompt::Standard,
        AuthorizeOptions::default(),
    );

    println!("Open this URL and authorize:\n\n{url}\n");
    println!("Waiting for the redirect on {REDIRECT_URI} ...");

    let (code, returned_state) = wait_for_callback();

    // Verify before trusting the code.
    assert_eq!(
        returned_state, state,
        "state mismatch — discarding this code"
    );

    let client = Client::builder(&key).build();
    let tokens = client
        .exchange_code(&code, &pkce.verifier, REDIRECT_URI)
        .await?;

    println!("\nAccess token:\n{}\n", tokens.access_token);
    if let Some(refresh) = &tokens.refresh_token {
        // Refresh tokens rotate: each refresh invalidates the previous one. Persist the
        // newest value every time, or the session dies at the next refresh.
        println!("Refresh token (store this, it rotates on every use):\n{refresh}\n");
    }
    if let Some(secs) = tokens.expires_in {
        println!("Expires in {secs}s.");
    }
    println!(
        "\nTry it:\n  export T3K_ACCESS_TOKEN={}\n  cargo run --example search_and_download",
        tokens.access_token
    );

    Ok(())
}

/// Block until the browser redirects, then return `(code, state)`.
///
/// A single-shot HTTP listener: read the request line, pull the query parameters out of
/// it, and reply with something human-readable so the browser tab is not left blank.
fn wait_for_callback() -> (String, String) {
    let listener = TcpListener::bind("127.0.0.1:8765").expect("port 8765 is free");
    let (mut stream, _) = listener.accept().expect("accepts the redirect");

    let mut request_line = String::new();
    BufReader::new(&stream)
        .read_line(&mut request_line)
        .expect("reads the request line");

    // "GET /callback?code=...&state=... HTTP/1.1"
    let target = request_line
        .split_whitespace()
        .nth(1)
        .expect("request line has a target");
    let query = target.split_once('?').map(|(_, q)| q).unwrap_or_default();

    let mut code = None;
    let mut state = None;
    for pair in query.split('&') {
        match pair.split_once('=') {
            Some(("code", v)) => code = Some(percent_decode(v)),
            Some(("state", v)) => state = Some(percent_decode(v)),
            _ => {}
        }
    }

    let body = "<h1>Authorized</h1><p>You can close this tab and return to your terminal.</p>";
    let _ = write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );

    (
        code.expect("redirect carried a `code` — if not, the authorize call was rejected"),
        state.unwrap_or_default(),
    )
}

/// Minimal percent-decoding, enough for an authorization code and state value.
fn percent_decode(s: &str) -> String {
    let bytes = s.replace('+', " ").into_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(byte) = u8::from_str_radix(&String::from_utf8_lossy(&bytes[i + 1..i + 3]), 16)
        {
            out.push(byte);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

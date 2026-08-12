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
async fn main() {
    // Print the crate's Display, not the derived Debug a `Result`-returning
    // main would dump.
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> tone3000::Result<()> {
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
    // Blocking IO inside an async main is fine for a one-shot CLI; a real app would run
    // its listener off the runtime.

    let (code, returned_state) = wait_for_callback();

    // Verify before trusting the code. A mismatch means this redirect did not come from
    // the flow we started, so the code is discarded rather than exchanged.
    if returned_state != state {
        eprintln!("state mismatch — discarding this authorization code");
        std::process::exit(1);
    }

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

    // Browsers open speculative connections that carry no request, so keep accepting
    // until one actually delivers the redirect.
    let (mut stream, query) = loop {
        let Ok((stream, _)) = listener.accept() else {
            continue;
        };
        let mut request_line = String::new();
        if BufReader::new(&stream)
            .read_line(&mut request_line)
            .is_err()
        {
            continue;
        }
        // "GET /callback?code=...&state=... HTTP/1.1"
        let Some(target) = request_line.split_whitespace().nth(1) else {
            continue;
        };
        let Some((_, q)) = target.split_once('?') else {
            continue;
        };
        if q.contains("code=") {
            break (stream, q.to_string());
        }
    };
    let query = query.as_str();

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
        // Deliberately nested rather than a let-chain: let-chains need Rust 1.88 and
        // this crate advertises MSRV 1.86.
        #[allow(clippy::collapsible_if)]
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&String::from_utf8_lossy(&bytes[i + 1..i + 3]), 16)
            {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

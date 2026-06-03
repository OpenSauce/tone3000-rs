mod common;

use std::io::{self, Write};

use tone3000::{Client, Prompt, authorize_url, generate_pkce};

#[tokio::test]
#[ignore = "interactive: opens an OAuth login; run via `make test-oauth`"]
async fn oauth_exchange_code_interactive() {
    let key = common::require_env("TONE3000_API_KEY");
    let redirect_uri = std::env::var("TONE3000_REDIRECT_URI")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "http://localhost:8765/callback".to_string());

    let pkce = generate_pkce();
    // The app owns `state`: generate an unguessable value and (in a real app) verify the
    // callback returns it unchanged. A fresh PKCE verifier is a convenient random source.
    let state = generate_pkce().verifier;
    let url = authorize_url(
        &key,
        &redirect_uri,
        &pkce.challenge,
        &state,
        Prompt::FullAccess,
    );

    println!(
        "\n1. Open this URL in a browser and authorize:\n\n{}\n",
        url.as_str()
    );
    println!("2. After it redirects to {redirect_uri}, copy the `code` query parameter.");
    print!("3. Paste the code here and press Enter: ");
    io::stdout().flush().unwrap();

    let mut code = String::new();
    io::stdin()
        .read_line(&mut code)
        .expect("read code from stdin");
    let code = code.trim();
    assert!(!code.is_empty(), "no code entered");

    let mut builder = Client::builder(key);
    if let Some(u) = std::env::var("TONE3000_BASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
    {
        builder = builder.base_url(u);
    }
    let client = builder.build();

    let tokens = client
        .exchange_code(code, &pkce.verifier, &redirect_uri)
        .await
        .expect("exchange_code succeeds");

    assert!(
        !tokens.access_token.is_empty(),
        "exchange must yield an access token"
    );
    println!("\n✓ access_token: {}", tokens.access_token);
    match &tokens.refresh_token {
        Some(rt) => {
            println!("✓ refresh_token (copy into .env as TONE3000_REFRESH_TOKEN):\n\n{rt}\n")
        }
        None => println!("(no refresh_token returned)"),
    }
}

use std::sync::Arc;

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use tokio::sync::Mutex;

use crate::error::Result;
use crate::models::Tokens;
use crate::oauth::parse_token_response;

/// Default base URL for the TONE3000 v1 API.
pub const DEFAULT_BASE_URL: &str = "https://www.tone3000.com/api/v1";

/// Callback invoked whenever tokens change (after exchange or refresh), so the
/// consuming app can persist them.
pub type TokensChanged = Arc<dyn Fn(&Tokens) + Send + Sync>;

/// Mutable token/auth state, guarded for interior mutability.
#[derive(Debug, Default)]
pub(crate) struct TokenState {
    pub access: Option<String>,
    pub refresh: Option<String>,
    /// Unix-epoch seconds at which `access` expires, if known.
    pub expires_at: Option<u64>,
}

/// Async client for the TONE3000 API.
#[derive(Clone)]
pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) pubkey: String,
    pub(crate) tokens: Arc<Mutex<TokenState>>,
    pub(crate) auto_refresh: bool,
    pub(crate) on_tokens_changed: Option<TokensChanged>,
}

impl Client {
    /// Create an app-key-only client (no user token).
    pub fn new(publishable_key: impl Into<String>) -> Self {
        ClientBuilder::new(publishable_key).build()
    }

    /// Start building a configured client.
    pub fn builder(publishable_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(publishable_key)
    }

    /// Build the request `Authorization` header value, preferring a bearer token.
    pub(crate) async fn auth_header(&self) -> HeaderValue {
        let guard = self.tokens.lock().await;
        let value = match &guard.access {
            Some(access) => format!("Bearer {access}"),
            None => format!("Bearer {}", self.pubkey),
        };
        HeaderValue::from_str(&value).expect("header value is valid ascii")
    }

    /// True if the client currently holds a user access token.
    pub(crate) async fn has_access_token(&self) -> bool {
        self.tokens.lock().await.access.is_some()
    }

    /// Default headers for a request (currently just auth).
    pub(crate) async fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, self.auth_header().await);
        h
    }

    /// Returns true if auto-refresh is enabled and the access token is at/near expiry.
    pub(crate) async fn needs_proactive_refresh(&self) -> bool {
        if !self.auto_refresh {
            return false;
        }
        let guard = self.tokens.lock().await;
        match (guard.access.as_ref(), guard.expires_at) {
            // refresh when within 30s of expiry
            (Some(_), Some(exp)) => now_unix() + 30 >= exp,
            _ => false,
        }
    }

    /// Refresh proactively if needed; ignore errors so the caller still attempts the request.
    pub(crate) async fn maybe_proactive_refresh(&self) {
        if self.needs_proactive_refresh().await {
            let _ = self.refresh().await;
        }
    }
}

impl Client {
    /// Exchange an authorization code for tokens, storing them on the client.
    pub async fn exchange_code(
        &self,
        code: &str,
        verifier: &str,
        redirect_uri: &str,
    ) -> Result<Tokens> {
        let form = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", redirect_uri),
            ("client_id", self.pubkey.as_str()),
        ];
        self.post_token(&form).await
    }

    /// Refresh using the stored refresh token, updating stored tokens.
    pub async fn refresh(&self) -> Result<Tokens> {
        let refresh = {
            let guard = self.tokens.lock().await;
            guard.refresh.clone()
        };
        let refresh = refresh.ok_or(crate::Error::Unauthenticated)?;
        let form = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh.as_str()),
            ("client_id", self.pubkey.as_str()),
        ];
        self.post_token(&form).await
    }

    /// Shared token-endpoint POST + state update + change callback.
    async fn post_token(&self, form: &[(&str, &str)]) -> Result<Tokens> {
        let resp = self
            .http
            .post(format!("{}/oauth/token", self.base_url))
            .form(form)
            .send()
            .await?;
        let ok = resp.status().is_success();
        let body = resp.bytes().await?;
        let tokens = parse_token_response(ok, &body)?;
        self.store_tokens(&tokens).await;
        Ok(tokens)
    }

    /// Persist tokens into client state and fire the change callback.
    pub(crate) async fn store_tokens(&self, tokens: &Tokens) {
        {
            let mut guard = self.tokens.lock().await;
            guard.access = Some(tokens.access_token.clone());
            if tokens.refresh_token.is_some() {
                guard.refresh = tokens.refresh_token.clone();
            }
            guard.expires_at = tokens.expires_in.map(|secs| now_unix() + secs);
        }
        if let Some(cb) = &self.on_tokens_changed {
            cb(tokens);
        }
    }
}

/// Current unix-epoch seconds.
pub(crate) fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Builder for [`Client`].
pub struct ClientBuilder {
    pubkey: String,
    base_url: String,
    access: Option<String>,
    refresh: Option<String>,
    auto_refresh: bool,
    on_tokens_changed: Option<TokensChanged>,
}

impl ClientBuilder {
    fn new(publishable_key: impl Into<String>) -> Self {
        Self {
            pubkey: publishable_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            access: None,
            refresh: None,
            auto_refresh: false,
            on_tokens_changed: None,
        }
    }

    /// Override the base URL (useful for tests / self-hosting).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the user access token (switches the client into bearer mode).
    pub fn access_token(mut self, token: impl Into<String>) -> Self {
        self.access = Some(token.into());
        self
    }

    /// Set the refresh token used by `refresh()` and auto-refresh.
    pub fn refresh_token(mut self, token: impl Into<String>) -> Self {
        self.refresh = Some(token.into());
        self
    }

    /// Enable transparent refresh-on-expiry/401.
    pub fn auto_refresh(mut self, enabled: bool) -> Self {
        self.auto_refresh = enabled;
        self
    }

    /// Register a callback fired whenever tokens change.
    pub fn on_tokens_changed<F>(mut self, f: F) -> Self
    where
        F: Fn(&Tokens) + Send + Sync + 'static,
    {
        self.on_tokens_changed = Some(Arc::new(f));
        self
    }

    /// Finish building the client.
    pub fn build(self) -> Client {
        Client {
            http: reqwest::Client::new(),
            base_url: self.base_url,
            pubkey: self.pubkey,
            tokens: Arc::new(Mutex::new(TokenState {
                access: self.access,
                refresh: self.refresh,
                expires_at: None,
            })),
            auto_refresh: self.auto_refresh,
            on_tokens_changed: self.on_tokens_changed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn app_key_mode_uses_pubkey_bearer() {
        let c = Client::new("t3k_pub_abc");
        assert_eq!(c.auth_header().await.to_str().unwrap(), "Bearer t3k_pub_abc");
        assert!(!c.has_access_token().await);
    }

    #[tokio::test]
    async fn bearer_mode_prefers_access_token() {
        let c = Client::builder("t3k_pub_abc")
            .access_token("user_tok")
            .build();
        assert_eq!(c.auth_header().await.to_str().unwrap(), "Bearer user_tok");
        assert!(c.has_access_token().await);
    }
}

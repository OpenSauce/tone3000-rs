use std::sync::Arc;

use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use tokio::sync::Mutex;

use crate::error::{Error, Result};
use crate::http::check_status;
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
    /// Serializes token refreshes so concurrent requests don't all POST `/oauth/token`
    /// and invalidate each other's rotated refresh tokens.
    pub(crate) refresh_lock: Arc<Mutex<()>>,
    pub(crate) auto_refresh: bool,
    pub(crate) on_tokens_changed: Option<TokensChanged>,
}

impl Client {
    /// Create a client with no token yet (useful for driving the OAuth bootstrap).
    ///
    /// API calls return [`Error::Unauthenticated`] until an access token is set via the
    /// builder or obtained through `exchange_code`/`refresh`.
    pub fn new(publishable_key: impl Into<String>) -> Self {
        ClientBuilder::new(publishable_key).build()
    }

    /// Start building a configured client.
    pub fn builder(publishable_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(publishable_key)
    }

    /// Build the request `Authorization` header from the stored access token.
    ///
    /// Errors with [`Error::Unauthenticated`] if no access token is set; callers go through
    /// [`Client::send`], which first ensures a token via [`Client::ensure_authenticated`].
    pub(crate) async fn auth_header(&self) -> Result<HeaderValue> {
        let guard = self.tokens.lock().await;
        match &guard.access {
            Some(access) => Ok(HeaderValue::from_str(&format!("Bearer {access}"))
                .expect("header value is valid ascii")),
            None => Err(Error::Unauthenticated),
        }
    }

    /// Ensure an access token is available, minting one from the refresh token if needed.
    /// Returns [`Error::Unauthenticated`] when there is neither an access nor a refresh token.
    pub(crate) async fn ensure_authenticated(&self) -> Result<()> {
        if self.has_access_token().await {
            return Ok(());
        }
        if !self.has_refresh_token().await {
            return Err(Error::Unauthenticated);
        }
        let _guard = self.refresh_lock.lock().await;
        if !self.has_access_token().await {
            self.refresh_locked().await?;
        }
        Ok(())
    }

    /// True if the client currently holds a user access token.
    pub(crate) async fn has_access_token(&self) -> bool {
        self.tokens.lock().await.access.is_some()
    }

    /// Default headers for a request (currently just auth).
    pub(crate) async fn headers(&self) -> Result<HeaderMap> {
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, self.auth_header().await?);
        Ok(h)
    }

    /// Snapshot of the current access token, used to detect refreshes by other tasks.
    async fn current_access(&self) -> Option<String> {
        self.tokens.lock().await.access.clone()
    }

    /// True if a refresh token is stored.
    async fn has_refresh_token(&self) -> bool {
        self.tokens.lock().await.refresh.is_some()
    }

    /// True if the access token's known expiry has passed.
    async fn is_expired(&self) -> bool {
        let guard = self.tokens.lock().await;
        matches!(
            (guard.access.as_ref(), guard.expires_at),
            (Some(_), Some(exp)) if now_unix() >= exp
        )
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

    /// Refresh proactively if the token is near expiry. Serialized so racing requests
    /// don't all refresh at once. Only surfaces an error if the token is already expired;
    /// otherwise the (still-valid) request is allowed to proceed.
    pub(crate) async fn maybe_proactive_refresh(&self) -> Result<()> {
        if !self.needs_proactive_refresh().await {
            return Ok(());
        }
        let _guard = self.refresh_lock.lock().await;
        // Another task may have refreshed while we waited for the lock.
        if !self.needs_proactive_refresh().await {
            return Ok(());
        }
        match self.refresh_locked().await {
            Ok(_) => Ok(()),
            Err(e) if self.is_expired().await => Err(e),
            Err(_) => Ok(()),
        }
    }

    pub(crate) async fn send(&self, req: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        self.ensure_authenticated().await?;
        self.maybe_proactive_refresh().await?;

        let retry = req.try_clone();
        let used = self.current_access().await;
        let resp = req.headers(self.headers().await?).send().await?;

        match check_status(resp).await {
            Err(Error::Unauthorized) if self.auto_refresh && self.has_refresh_token().await => {
                self.reactive_refresh(used).await?;
                match retry {
                    Some(rb) => {
                        let resp = rb.headers(self.headers().await?).send().await?;
                        check_status(resp).await
                    }
                    None => Err(Error::Unauthorized),
                }
            }
            other => other,
        }
    }

    /// Refresh once on a 401, serialized and only if no other task already rotated the
    /// token we just used.
    async fn reactive_refresh(&self, used: Option<String>) -> Result<()> {
        let _guard = self.refresh_lock.lock().await;
        // If the stored access token already changed, another task refreshed; reuse it.
        if self.current_access().await == used {
            self.refresh_locked().await?;
        }
        Ok(())
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
    ///
    /// Serialized via the refresh lock so concurrent callers don't race the token
    /// rotation endpoint.
    pub async fn refresh(&self) -> Result<Tokens> {
        let _guard = self.refresh_lock.lock().await;
        self.refresh_locked().await
    }

    /// Refresh without taking the refresh lock. Callers that already hold it
    /// (proactive/reactive refresh) use this to avoid re-entrant locking.
    async fn refresh_locked(&self) -> Result<Tokens> {
        let refresh = {
            let guard = self.tokens.lock().await;
            guard.refresh.clone()
        };
        let refresh = refresh.ok_or(Error::Unauthenticated)?;
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
        let status = resp.status().as_u16();
        let body = resp.bytes().await?;
        let tokens = parse_token_response(status, &body)?;
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
    expires_at: Option<u64>,
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
            expires_at: None,
            auto_refresh: false,
            on_tokens_changed: None,
        }
    }

    /// Override the base URL (useful for tests / self-hosting).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the user access token used as the Bearer credential for API calls.
    pub fn access_token(mut self, token: impl Into<String>) -> Self {
        self.access = Some(token.into());
        self
    }

    /// Set the refresh token used by `refresh()` and auto-refresh.
    pub fn refresh_token(mut self, token: impl Into<String>) -> Self {
        self.refresh = Some(token.into());
        self
    }

    /// Seed the access token's expiry as Unix-epoch seconds.
    ///
    /// Required for proactive [`auto_refresh`](Self::auto_refresh) to fire when
    /// restoring a session from persisted tokens — otherwise the client has no
    /// idea when the access token expires and will only refresh reactively on a 401.
    pub fn expires_at(mut self, unix_secs: u64) -> Self {
        self.expires_at = Some(unix_secs);
        self
    }

    /// Enable transparent proactive token refresh shortly before the access token expires.
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
                expires_at: self.expires_at,
            })),
            refresh_lock: Arc::new(Mutex::new(())),
            auto_refresh: self.auto_refresh,
            on_tokens_changed: self.on_tokens_changed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_token_auth_header_errors() {
        let c = Client::new("t3k_pub_abc");
        assert!(matches!(c.auth_header().await, Err(Error::Unauthenticated)));
        assert!(!c.has_access_token().await);
    }

    #[tokio::test]
    async fn bearer_mode_uses_access_token() {
        let c = Client::builder("t3k_pub_abc")
            .access_token("user_tok")
            .build();
        assert_eq!(
            c.auth_header().await.unwrap().to_str().unwrap(),
            "Bearer user_tok"
        );
        assert!(c.has_access_token().await);
    }

    #[tokio::test]
    async fn ensure_authenticated_errors_without_any_token() {
        let c = Client::new("t3k_pub_abc");
        assert!(matches!(
            c.ensure_authenticated().await,
            Err(Error::Unauthenticated)
        ));
    }
}

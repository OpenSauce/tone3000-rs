//! PKCE (RFC 7636) helpers. The consuming app owns the OAuth redirect transport;
//! this module only generates the verifier/challenge pair.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// A PKCE verifier/challenge pair. Keep `verifier` for the token exchange; send
/// `challenge` in the authorize URL.
#[derive(Debug, Clone)]
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

/// Generate a fresh PKCE pair using the S256 method.
pub fn generate() -> Pkce {
    let mut raw = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut raw);
    let verifier = URL_SAFE_NO_PAD.encode(raw);
    let challenge = challenge_for(&verifier);
    Pkce {
        verifier,
        challenge,
    }
}

/// Compute the S256 challenge for a given verifier: base64url(sha256(verifier)).
pub fn challenge_for(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_matches_rfc_test_vector() {
        // RFC 7636 Appendix B verifier/challenge pair.
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert_eq!(challenge_for(verifier), expected);
    }

    #[test]
    fn generate_roundtrips() {
        let p = generate();
        assert_eq!(challenge_for(&p.verifier), p.challenge);
        assert!(!p.verifier.is_empty());
    }
}

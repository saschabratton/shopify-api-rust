//! JWT payload handling for Shopify session tokens.
//!
//! This module provides the [`JwtPayload`] struct for decoding and validating
//! Shopify session tokens (JWTs) issued by App Bridge for embedded apps.
//!
//! # Overview
//!
//! When a Shopify embedded app loads in the Shopify admin, App Bridge provides
//! a session token (JWT) that can be exchanged for an access token. This module
//! handles the decoding and validation of those JWTs.
//!
//! # JWT Structure
//!
//! Shopify session tokens contain the following claims:
//!
//! - `iss`: Issuer (e.g., `https://shop.myshopify.com/admin`)
//! - `dest`: Destination shop (e.g., `https://shop.myshopify.com`)
//! - `aud`: Audience (should match the app's API key)
//! - `sub`: Subject (user ID for online tokens, optional)
//! - `exp`: Expiration timestamp
//! - `nbf`: Not before timestamp
//! - `iat`: Issued at timestamp
//! - `jti`: JWT ID (unique identifier)
//! - `sid`: Session ID (optional)
//!
//! # Dual-Key Validation
//!
//! To support seamless API key rotation, JWT validation first attempts with the
//! primary API secret key, then falls back to the old secret key if configured.
//!
//! # Reference
//!
//! This implementation matches the Ruby SDK's `ShopifyAPI::Auth::JwtPayload`:
//! - File: `lib/shopify_api/auth/jwt_payload.rb`
//! - JWT leeway: 10 seconds for time-based claims

use crate::auth::oauth::OAuthError;
use crate::config::ShopifyConfig;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

/// Leeway for JWT time-based claims validation (10 seconds).
///
/// This matches the Ruby SDK's `JWT_LEEWAY = 10` constant.
const JWT_LEEWAY_SECS: u64 = 10;

/// JWT payload for Shopify session tokens.
///
/// This struct represents the decoded claims from a Shopify App Bridge session token.
/// It is used internally during token exchange to validate the session token before
/// exchanging it for an access token.
///
/// # Thread Safety
///
/// `JwtPayload` is `Send + Sync`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct JwtPayload {
    /// Issuer - the Shopify admin URL that issued the token.
    ///
    /// Example: `https://shop.myshopify.com/admin`
    pub iss: String,

    /// Destination - the target shop domain.
    ///
    /// Example: `https://shop.myshopify.com`
    pub dest: String,

    /// Audience - should match the app's API key.
    pub aud: String,

    /// Subject - the user ID for online tokens.
    ///
    /// For offline tokens or non-admin contexts, this may be `None` or a non-numeric value.
    pub sub: Option<String>,

    /// Expiration timestamp (Unix timestamp).
    pub exp: i64,

    /// Not before timestamp (Unix timestamp).
    pub nbf: i64,

    /// Issued at timestamp (Unix timestamp).
    pub iat: i64,

    /// JWT ID - unique identifier for this token.
    pub jti: String,

    /// Shopify session ID.
    pub sid: Option<String>,
}

impl JwtPayload {
    /// Decodes and validates a Shopify session token (JWT).
    ///
    /// This function:
    /// 1. Attempts to decode the JWT using the primary API secret key
    /// 2. Falls back to the old API secret key if decoding fails and it's configured
    /// 3. Validates that the `aud` claim matches the app's API key
    ///
    /// # Arguments
    ///
    /// * `token` - The session token JWT string from App Bridge
    /// * `config` - The Shopify SDK configuration
    ///
    /// # Returns
    ///
    /// The decoded JWT payload on success.
    ///
    /// # Errors
    ///
    /// - [`OAuthError::InvalidJwt`] if the token cannot be decoded or validated
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::auth::oauth::jwt_payload::JwtPayload;
    ///
    /// let payload = JwtPayload::decode(session_token, &config)?;
    /// println!("Shop: {}", payload.shop());
    /// ```
    pub fn decode(token: &str, config: &ShopifyConfig) -> Result<Self, OAuthError> {
        // Try primary API secret key first
        let payload = match Self::decode_with_key(token, config.api_secret_key().as_ref()) {
            Ok(payload) => payload,
            Err(primary_err) => {
                // Try old API secret key if configured
                if let Some(old_key) = config.old_api_secret_key() {
                    Self::decode_with_key(token, old_key.as_ref()).map_err(|_| {
                        // Return the original error if both keys fail
                        OAuthError::InvalidJwt {
                            reason: format!("Error decoding session token: {primary_err}"),
                        }
                    })?
                } else {
                    return Err(OAuthError::InvalidJwt {
                        reason: format!("Error decoding session token: {primary_err}"),
                    });
                }
            }
        };

        // Validate that aud claim matches the API key
        if payload.aud != config.api_key().as_ref() {
            return Err(OAuthError::InvalidJwt {
                reason: "Session token had invalid API key".to_string(),
            });
        }

        Ok(payload)
    }

    /// Decodes a JWT using a specific secret key.
    fn decode_with_key(token: &str, secret: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = JWT_LEEWAY_SECS;
        // Disable audience validation - we do it manually after decoding
        validation.validate_aud = false;

        let key = DecodingKey::from_secret(secret.as_bytes());
        let token_data = decode::<Self>(token, &key, &validation)?;

        Ok(token_data.claims)
    }

    /// Returns the shop domain extracted from the `dest` claim.
    ///
    /// This strips the `https://` prefix from the destination URL.
    ///
    /// # Example
    ///
    /// If `dest` is `https://my-store.myshopify.com`, this returns `my-store.myshopify.com`.
    #[must_use]
    #[allow(dead_code)] // Part of internal API, used in tests
    pub fn shop(&self) -> &str {
        self.dest
            .strip_prefix("https://")
            .unwrap_or(self.dest.as_str())
    }

    /// Returns the Shopify user ID if this is an admin online session token.
    ///
    /// This returns `Some(user_id)` only when:
    /// 1. The `sub` claim is present and contains only digits
    /// 2. The `iss` claim ends with `/admin` (indicating an admin session)
    ///
    /// For offline tokens or non-admin contexts, this returns `None`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let payload = JwtPayload::decode(token, &config)?;
    /// if let Some(user_id) = payload.shopify_user_id() {
    ///     println!("User ID: {}", user_id);
    /// }
    /// ```
    #[must_use]
    #[allow(dead_code)] // Part of internal API, used in tests
    pub fn shopify_user_id(&self) -> Option<u64> {
        // Only return user ID if this is an admin session token
        if !self.is_admin_session_token() {
            return None;
        }

        // Only return user ID if sub is numeric
        self.sub.as_ref().and_then(|sub| {
            if Self::is_numeric(sub) {
                sub.parse().ok()
            } else {
                None
            }
        })
    }

    /// Checks if the `iss` claim ends with `/admin`.
    fn is_admin_session_token(&self) -> bool {
        self.iss.ends_with("/admin")
    }

    /// Checks if a string contains only ASCII digits.
    fn is_numeric(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
    }
}

// Verify JwtPayload is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<JwtPayload>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiKey, ApiSecretKey};
    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::Serialize;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Helper struct for creating test JWTs (same structure as JwtPayload but with Serialize)
    #[derive(Debug, Serialize)]
    struct TestJwtClaims {
        iss: String,
        dest: String,
        aud: String,
        sub: Option<String>,
        exp: i64,
        nbf: i64,
        iat: i64,
        jti: String,
        sid: Option<String>,
    }

    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn create_test_config(secret: &str) -> ShopifyConfig {
        ShopifyConfig::builder()
            .api_key(ApiKey::new("test-api-key").unwrap())
            .api_secret_key(ApiSecretKey::new(secret).unwrap())
            .build()
            .unwrap()
    }

    fn create_config_with_old_key(primary_secret: &str, old_secret: &str) -> ShopifyConfig {
        ShopifyConfig::builder()
            .api_key(ApiKey::new("test-api-key").unwrap())
            .api_secret_key(ApiSecretKey::new(primary_secret).unwrap())
            .old_api_secret_key(ApiSecretKey::new(old_secret).unwrap())
            .build()
            .unwrap()
    }

    fn create_valid_claims() -> TestJwtClaims {
        let now = current_timestamp();
        TestJwtClaims {
            iss: "https://test-shop.myshopify.com/admin".to_string(),
            dest: "https://test-shop.myshopify.com".to_string(),
            aud: "test-api-key".to_string(),
            sub: Some("12345".to_string()),
            exp: now + 300, // 5 minutes from now
            nbf: now - 10,
            iat: now,
            jti: "unique-jwt-id".to_string(),
            sid: Some("session-id".to_string()),
        }
    }

    fn encode_jwt(claims: &TestJwtClaims, secret: &str) -> String {
        let header = Header::new(Algorithm::HS256);
        let key = EncodingKey::from_secret(secret.as_bytes());
        encode(&header, claims, &key).unwrap()
    }

    #[test]
    fn test_successful_jwt_decode_with_valid_token_and_secret() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);
        let claims = create_valid_claims();
        let token = encode_jwt(&claims, secret);

        let result = JwtPayload::decode(&token, &config);

        assert!(result.is_ok());
        let payload = result.unwrap();
        assert_eq!(payload.iss, claims.iss);
        assert_eq!(payload.dest, claims.dest);
        assert_eq!(payload.aud, claims.aud);
        assert_eq!(payload.sub, claims.sub);
        assert_eq!(payload.jti, claims.jti);
    }

    #[test]
    fn test_dual_key_fallback_fails_primary_succeeds_with_old_key() {
        let primary_secret = "new-secret-key";
        let old_secret = "old-secret-key";
        let config = create_config_with_old_key(primary_secret, old_secret);

        // Encode with the OLD secret
        let claims = create_valid_claims();
        let token = encode_jwt(&claims, old_secret);

        // Should succeed by falling back to old key
        let result = JwtPayload::decode(&token, &config);

        assert!(result.is_ok());
        let payload = result.unwrap();
        assert_eq!(payload.aud, "test-api-key");
    }

    #[test]
    fn test_invalid_jwt_error_when_both_keys_fail() {
        let primary_secret = "new-secret-key";
        let old_secret = "old-secret-key";
        let config = create_config_with_old_key(primary_secret, old_secret);

        // Encode with a DIFFERENT secret (neither primary nor old)
        let claims = create_valid_claims();
        let token = encode_jwt(&claims, "wrong-secret-key");

        let result = JwtPayload::decode(&token, &config);

        assert!(matches!(result, Err(OAuthError::InvalidJwt { .. })));
        if let Err(OAuthError::InvalidJwt { reason }) = result {
            assert!(reason.contains("Error decoding session token"));
        }
    }

    #[test]
    fn test_invalid_jwt_error_when_aud_claim_doesnt_match_api_key() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);

        // Create claims with wrong API key
        let mut claims = create_valid_claims();
        claims.aud = "wrong-api-key".to_string();
        let token = encode_jwt(&claims, secret);

        let result = JwtPayload::decode(&token, &config);

        assert!(matches!(result, Err(OAuthError::InvalidJwt { .. })));
        if let Err(OAuthError::InvalidJwt { reason }) = result {
            assert_eq!(reason, "Session token had invalid API key");
        }
    }

    #[test]
    fn test_shop_method_extracts_domain_from_dest_claim() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);
        let claims = create_valid_claims();
        let token = encode_jwt(&claims, secret);

        let payload = JwtPayload::decode(&token, &config).unwrap();

        assert_eq!(payload.shop(), "test-shop.myshopify.com");
    }

    #[test]
    fn test_shopify_user_id_returns_some_for_numeric_sub_when_iss_ends_with_admin() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);
        let mut claims = create_valid_claims();
        claims.iss = "https://test-shop.myshopify.com/admin".to_string();
        claims.sub = Some("12345".to_string());
        let token = encode_jwt(&claims, secret);

        let payload = JwtPayload::decode(&token, &config).unwrap();

        assert_eq!(payload.shopify_user_id(), Some(12345));
    }

    #[test]
    fn test_shopify_user_id_returns_none_for_non_numeric_sub() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);
        let mut claims = create_valid_claims();
        claims.iss = "https://test-shop.myshopify.com/admin".to_string();
        claims.sub = Some("not-a-number".to_string());
        let token = encode_jwt(&claims, secret);

        let payload = JwtPayload::decode(&token, &config).unwrap();

        assert_eq!(payload.shopify_user_id(), None);
    }

    #[test]
    fn test_shopify_user_id_returns_none_when_iss_doesnt_end_with_admin() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);
        let mut claims = create_valid_claims();
        claims.iss = "https://test-shop.myshopify.com".to_string(); // No /admin suffix
        claims.sub = Some("12345".to_string());
        let token = encode_jwt(&claims, secret);

        let payload = JwtPayload::decode(&token, &config).unwrap();

        assert_eq!(payload.shopify_user_id(), None);
    }

    #[test]
    fn test_jwt_payload_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<JwtPayload>();
    }

    #[test]
    fn test_expired_token_fails_validation() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);
        let mut claims = create_valid_claims();
        // Set expiration to 1 hour ago (beyond 10-second leeway)
        claims.exp = current_timestamp() - 3600;
        let token = encode_jwt(&claims, secret);

        let result = JwtPayload::decode(&token, &config);

        assert!(matches!(result, Err(OAuthError::InvalidJwt { .. })));
    }

    #[test]
    fn test_token_within_leeway_is_accepted() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);
        let mut claims = create_valid_claims();
        // Set expiration to 5 seconds ago (within 10-second leeway)
        claims.exp = current_timestamp() - 5;
        let token = encode_jwt(&claims, secret);

        let result = JwtPayload::decode(&token, &config);

        // Should succeed because we're within the 10-second leeway
        assert!(result.is_ok());
    }

    #[test]
    fn test_shopify_user_id_returns_none_when_sub_is_none() {
        let secret = "test-secret-key";
        let config = create_test_config(secret);
        let mut claims = create_valid_claims();
        claims.iss = "https://test-shop.myshopify.com/admin".to_string();
        claims.sub = None;
        let token = encode_jwt(&claims, secret);

        let payload = JwtPayload::decode(&token, &config).unwrap();

        assert_eq!(payload.shopify_user_id(), None);
    }
}

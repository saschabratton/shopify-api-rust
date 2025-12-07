//! HMAC validation for Shopify OAuth callbacks.
//!
//! This module provides functions for computing and validating HMAC-SHA256
//! signatures used in Shopify's OAuth callback verification.
//!
//! # Security
//!
//! All HMAC comparisons use constant-time comparison to prevent timing attacks.
//! The module also supports key rotation by falling back to an old secret key
//! if validation with the primary key fails.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::auth::oauth::hmac::compute_signature;
//!
//! let message = "code=abc123&shop=example.myshopify.com&state=xyz";
//! let secret = "my-api-secret";
//! let signature = compute_signature(message, secret);
//!
//! // Signature is lowercase hex-encoded
//! assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
//! ```

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::auth::oauth::AuthQuery;
use crate::config::ShopifyConfig;

type HmacSha256 = Hmac<Sha256>;

/// Computes an HMAC-SHA256 signature for the given message.
///
/// The signature is returned as a lowercase hexadecimal string, matching
/// the format used by Shopify and the Ruby SDK's `OpenSSL::HMAC.hexdigest`.
///
/// # Arguments
///
/// * `message` - The message to sign (typically a query string without the HMAC)
/// * `secret` - The secret key (API secret key)
///
/// # Returns
///
/// A lowercase hex-encoded HMAC-SHA256 signature.
///
/// # Note
///
/// This function uses `expect()` internally but this will never panic because
/// HMAC-SHA256 accepts keys of any length.
///
/// # Example
///
/// ```rust
/// use shopify_api::auth::oauth::hmac::compute_signature;
///
/// let sig = compute_signature("test-message", "secret-key");
/// assert_eq!(sig.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
/// ```
#[must_use]
#[allow(clippy::missing_panics_doc)] // HMAC accepts any key size, so this never panics
pub fn compute_signature(message: &str, secret: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Performs constant-time comparison of two strings.
///
/// This function is used for security-sensitive comparisons like HMAC
/// verification and state parameter validation to prevent timing attacks.
///
/// # Arguments
///
/// * `a` - First string to compare
/// * `b` - Second string to compare
///
/// # Returns
///
/// `true` if the strings are equal, `false` otherwise.
#[must_use]
pub fn constant_time_compare(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    // ConstantTimeEq handles different lengths securely
    a_bytes.ct_eq(b_bytes).into()
}

/// Validates the HMAC signature of an OAuth callback.
///
/// This function validates that the `hmac` parameter in the callback matches
/// the expected signature computed from the other parameters.
///
/// # Key Rotation Support
///
/// If the primary `api_secret_key` fails validation, the function will
/// automatically try `old_api_secret_key` if configured. This allows
/// seamless key rotation without breaking in-flight OAuth flows.
///
/// # Arguments
///
/// * `query` - The OAuth callback query parameters
/// * `config` - The Shopify configuration containing the secret key(s)
///
/// # Returns
///
/// `true` if the HMAC is valid, `false` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::auth::oauth::{AuthQuery, validate_hmac};
/// use shopify_api::ShopifyConfig;
///
/// let query = AuthQuery { /* ... */ };
/// let config = ShopifyConfig::builder()
///     .api_key(/* ... */)
///     .api_secret_key(/* ... */)
///     .build()
///     .unwrap();
///
/// if validate_hmac(&query, &config) {
///     // Proceed with token exchange
/// } else {
///     // Reject the callback
/// }
/// ```
#[must_use]
pub fn validate_hmac(query: &AuthQuery, config: &ShopifyConfig) -> bool {
    let signable = query.to_signable_string();
    let received_hmac = &query.hmac;

    // Try primary secret key first
    let computed = compute_signature(&signable, config.api_secret_key().as_ref());
    if constant_time_compare(&computed, received_hmac) {
        return true;
    }

    // Fall back to old secret key if configured
    if let Some(old_secret) = config.old_api_secret_key() {
        let computed_old = compute_signature(&signable, old_secret.as_ref());
        if constant_time_compare(&computed_old, received_hmac) {
            return true;
        }
    }

    false
}

// Internal hex encoding since we don't want to add another dependency
mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        let mut result = String::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            result.push(HEX_CHARS[(byte >> 4) as usize] as char);
            result.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiKey, ApiSecretKey};

    #[test]
    fn test_compute_signature_produces_correct_hex() {
        // Test with known values
        let sig = compute_signature("test", "secret");

        // Should be 64 characters (32 bytes * 2 hex chars)
        assert_eq!(sig.len(), 64);
        // Should be lowercase hex
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(sig.chars().all(|c| !c.is_ascii_uppercase()));
    }

    #[test]
    fn test_compute_signature_matches_known_value() {
        // Known HMAC-SHA256 test vector
        // HMAC-SHA256("message", "key") = 6e9ef29b75fffc5b7abae527d58fdadb2fe42e7219011976917343065f58ed4a
        let sig = compute_signature("message", "key");
        assert_eq!(
            sig,
            "6e9ef29b75fffc5b7abae527d58fdadb2fe42e7219011976917343065f58ed4a"
        );
    }

    #[test]
    fn test_compute_signature_with_empty_message() {
        let sig = compute_signature("", "secret");
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_constant_time_compare_equal_strings() {
        assert!(constant_time_compare("abc123", "abc123"));
        assert!(constant_time_compare("", ""));
    }

    #[test]
    fn test_constant_time_compare_different_strings() {
        assert!(!constant_time_compare("abc123", "abc124"));
        assert!(!constant_time_compare("abc", "abcd"));
        assert!(!constant_time_compare("ABC", "abc"));
    }

    #[test]
    fn test_constant_time_compare_different_lengths() {
        assert!(!constant_time_compare("short", "longer string"));
        assert!(!constant_time_compare("a", ""));
    }

    #[test]
    fn test_validate_hmac_succeeds_with_correct_hmac() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .build()
            .unwrap();

        // Create a query and compute the expected HMAC
        let mut query = AuthQuery::new(
            "auth-code".to_string(),
            "test-shop.myshopify.com".to_string(),
            "1234567890".to_string(),
            "state-value".to_string(),
            "host-value".to_string(),
            String::new(), // Will compute HMAC
        );

        let signable = query.to_signable_string();
        let computed_hmac = compute_signature(&signable, "test-secret");
        query.hmac = computed_hmac;

        assert!(validate_hmac(&query, &config));
    }

    #[test]
    fn test_validate_hmac_fails_with_incorrect_hmac() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .build()
            .unwrap();

        let query = AuthQuery::new(
            "auth-code".to_string(),
            "test-shop.myshopify.com".to_string(),
            "1234567890".to_string(),
            "state-value".to_string(),
            "host-value".to_string(),
            "invalid-hmac".to_string(),
        );

        assert!(!validate_hmac(&query, &config));
    }

    #[test]
    fn test_validate_hmac_falls_back_to_old_secret() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("new-secret").unwrap())
            .old_api_secret_key(ApiSecretKey::new("old-secret").unwrap())
            .build()
            .unwrap();

        // Create query with HMAC computed using OLD secret
        let mut query = AuthQuery::new(
            "auth-code".to_string(),
            "test-shop.myshopify.com".to_string(),
            "1234567890".to_string(),
            "state-value".to_string(),
            "host-value".to_string(),
            String::new(),
        );

        let signable = query.to_signable_string();
        let computed_hmac = compute_signature(&signable, "old-secret");
        query.hmac = computed_hmac;

        // Should succeed by falling back to old secret
        assert!(validate_hmac(&query, &config));
    }

    #[test]
    fn test_validate_hmac_fails_when_both_keys_fail() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret-1").unwrap())
            .old_api_secret_key(ApiSecretKey::new("secret-2").unwrap())
            .build()
            .unwrap();

        // Create query with HMAC computed using a DIFFERENT secret
        let mut query = AuthQuery::new(
            "auth-code".to_string(),
            "test-shop.myshopify.com".to_string(),
            "1234567890".to_string(),
            "state-value".to_string(),
            "host-value".to_string(),
            String::new(),
        );

        let signable = query.to_signable_string();
        let computed_hmac = compute_signature(&signable, "secret-3");
        query.hmac = computed_hmac;

        // Should fail - neither secret matches
        assert!(!validate_hmac(&query, &config));
    }

    #[test]
    fn test_validate_hmac_prefers_primary_key() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("primary-secret").unwrap())
            .old_api_secret_key(ApiSecretKey::new("old-secret").unwrap())
            .build()
            .unwrap();

        // Create query with HMAC computed using PRIMARY secret
        let mut query = AuthQuery::new(
            "auth-code".to_string(),
            "test-shop.myshopify.com".to_string(),
            "1234567890".to_string(),
            "state-value".to_string(),
            "host-value".to_string(),
            String::new(),
        );

        let signable = query.to_signable_string();
        let computed_hmac = compute_signature(&signable, "primary-secret");
        query.hmac = computed_hmac;

        // Should succeed with primary secret
        assert!(validate_hmac(&query, &config));
    }

    #[test]
    fn test_hex_encoding() {
        assert_eq!(hex::encode([0x00, 0xff, 0xab, 0xcd]), "00ffabcd");
        assert_eq!(hex::encode([]), "");
        assert_eq!(hex::encode([0x12, 0x34]), "1234");
    }
}

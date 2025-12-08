//! HMAC validation for Shopify OAuth callbacks and webhook verification.
//!
//! This module provides functions for computing and validating HMAC-SHA256
//! signatures used in Shopify's OAuth callback verification and webhook
//! signature validation.
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
//! use shopify_api::auth::oauth::hmac::{compute_signature, compute_signature_base64};
//!
//! // Hex-encoded signature for OAuth
//! let message = "code=abc123&shop=example.myshopify.com&state=xyz";
//! let secret = "my-api-secret";
//! let signature = compute_signature(message, secret);
//! assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
//!
//! // Base64-encoded signature for webhooks
//! let body = b"webhook payload";
//! let webhook_sig = compute_signature_base64(body, secret);
//! assert_eq!(webhook_sig.len(), 44); // Base64 of 32 bytes
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

/// Computes an HMAC-SHA256 signature for raw bytes, returning base64-encoded output.
///
/// This function is used for webhook signature verification, where Shopify sends
/// base64-encoded HMAC signatures in the `X-Shopify-Hmac-SHA256` header.
///
/// # Arguments
///
/// * `message` - The raw message bytes to sign (webhook request body)
/// * `secret` - The secret key (API secret key)
///
/// # Returns
///
/// A base64-encoded HMAC-SHA256 signature (RFC 4648 standard base64).
///
/// # Note
///
/// This function accepts raw bytes (not strings) to preserve the exact webhook
/// payload without UTF-8 interpretation. HMAC-SHA256 accepts keys of any length,
/// so this function will never panic.
///
/// # Example
///
/// ```rust
/// use shopify_api::auth::oauth::hmac::compute_signature_base64;
///
/// let body = b"webhook payload";
/// let sig = compute_signature_base64(body, "secret-key");
/// assert_eq!(sig.len(), 44); // SHA256 produces 32 bytes = 44 base64 chars
/// ```
#[must_use]
#[allow(clippy::missing_panics_doc)] // HMAC accepts any key size, so this never panics
pub fn compute_signature_base64(message: &[u8], secret: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message);
    let result = mac.finalize();
    base64::encode(result.into_bytes())
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

// Internal base64 encoding (RFC 4648 standard base64)
mod base64 {
    const BASE64_CHARS: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        let len = bytes.len();
        // Calculate output size: ceil(len / 3) * 4
        let capacity = ((len + 2) / 3) * 4;
        let mut result = String::with_capacity(capacity);

        let mut i = 0;
        while i + 3 <= len {
            // Process 3 bytes at a time
            let b0 = bytes[i] as usize;
            let b1 = bytes[i + 1] as usize;
            let b2 = bytes[i + 2] as usize;

            result.push(BASE64_CHARS[b0 >> 2] as char);
            result.push(BASE64_CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
            result.push(BASE64_CHARS[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
            result.push(BASE64_CHARS[b2 & 0x3f] as char);

            i += 3;
        }

        // Handle remaining bytes with padding
        let remaining = len - i;
        if remaining == 1 {
            let b0 = bytes[i] as usize;
            result.push(BASE64_CHARS[b0 >> 2] as char);
            result.push(BASE64_CHARS[(b0 & 0x03) << 4] as char);
            result.push('=');
            result.push('=');
        } else if remaining == 2 {
            let b0 = bytes[i] as usize;
            let b1 = bytes[i + 1] as usize;
            result.push(BASE64_CHARS[b0 >> 2] as char);
            result.push(BASE64_CHARS[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
            result.push(BASE64_CHARS[(b1 & 0x0f) << 2] as char);
            result.push('=');
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

    // Base64 HMAC computation tests (Task Group 1)

    #[test]
    fn test_compute_signature_base64_produces_correct_length() {
        // SHA256 produces 32 bytes, base64 of 32 bytes = ceil(32/3)*4 = 44 characters
        let sig = compute_signature_base64(b"test", "secret");
        assert_eq!(sig.len(), 44);
    }

    #[test]
    fn test_compute_signature_base64_matches_known_value() {
        // Known HMAC-SHA256 test vector, base64-encoded
        // HMAC-SHA256("message", "key") in hex: 6e9ef29b75fffc5b7abae527d58fdadb2fe42e7219011976917343065f58ed4a
        // Same in base64: bp7ym3X//Ft6uuUn1Y/a2y/kLnIZARl2kXNDBl9Y7Uo=
        let sig = compute_signature_base64(b"message", "key");
        assert_eq!(sig, "bp7ym3X//Ft6uuUn1Y/a2y/kLnIZARl2kXNDBl9Y7Uo=");
    }

    #[test]
    fn test_compute_signature_base64_with_empty_message() {
        let sig = compute_signature_base64(b"", "secret");
        // Should still produce 44-character base64 output
        assert_eq!(sig.len(), 44);
    }

    #[test]
    fn test_compute_signature_base64_valid_characters() {
        let sig = compute_signature_base64(b"test payload", "secret");
        // Valid base64 characters: A-Z, a-z, 0-9, +, /, =
        assert!(sig
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_compute_signature_base64_with_non_utf8_bytes() {
        // Test with raw bytes that are not valid UTF-8
        let non_utf8_bytes: &[u8] = &[0x80, 0x81, 0x82, 0xff, 0xfe];
        let sig = compute_signature_base64(non_utf8_bytes, "secret");
        assert_eq!(sig.len(), 44);
        // Should still produce valid base64
        assert!(sig
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_base64_encoding() {
        // Standard base64 test vectors
        assert_eq!(base64::encode([]), "");
        assert_eq!(base64::encode([0x66]), "Zg=="); // "f"
        assert_eq!(base64::encode([0x66, 0x6f]), "Zm8="); // "fo"
        assert_eq!(base64::encode([0x66, 0x6f, 0x6f]), "Zm9v"); // "foo"
        assert_eq!(base64::encode([0x66, 0x6f, 0x6f, 0x62]), "Zm9vYg=="); // "foob"
        assert_eq!(base64::encode([0x66, 0x6f, 0x6f, 0x62, 0x61]), "Zm9vYmE="); // "fooba"
        assert_eq!(
            base64::encode([0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72]),
            "Zm9vYmFy"
        ); // "foobar"
    }
}

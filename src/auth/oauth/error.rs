//! OAuth-specific error types for the Shopify API SDK.
//!
//! This module contains error types for OAuth operations including HMAC validation,
//! state verification, and token exchange failures.
//!
//! # Error Types
//!
//! - [`OAuthError::InvalidHmac`]: HMAC signature validation failed
//! - [`OAuthError::StateMismatch`]: OAuth state parameter doesn't match expected
//! - [`OAuthError::TokenExchangeFailed`]: Token exchange request failed
//! - [`OAuthError::InvalidCallback`]: Callback parameters are malformed
//! - [`OAuthError::MissingHostConfig`]: Host URL not configured for redirect URI
//! - [`OAuthError::HttpError`]: Wrapped HTTP client error
//!
//! # Example
//!
//! ```rust
//! use shopify_api::auth::oauth::OAuthError;
//!
//! let error = OAuthError::InvalidHmac;
//! assert_eq!(error.to_string(), "HMAC signature validation failed");
//!
//! let error = OAuthError::StateMismatch {
//!     expected: "abc123".to_string(),
//!     received: "xyz789".to_string(),
//! };
//! assert!(error.to_string().contains("abc123"));
//! ```

use crate::clients::HttpError;
use thiserror::Error;

/// Errors that can occur during OAuth operations.
///
/// This enum covers all failure modes in the OAuth authorization code flow,
/// from HMAC validation to token exchange.
///
/// # Thread Safety
///
/// `OAuthError` is `Send + Sync`, making it safe to use across async boundaries.
///
/// # Example
///
/// ```rust
/// use shopify_api::auth::oauth::OAuthError;
///
/// fn handle_oauth_error(err: OAuthError) {
///     match err {
///         OAuthError::InvalidHmac => {
///             eprintln!("Security: HMAC validation failed");
///         }
///         OAuthError::StateMismatch { expected, received } => {
///             eprintln!("CSRF: State mismatch - expected {}, got {}", expected, received);
///         }
///         OAuthError::TokenExchangeFailed { status, message } => {
///             eprintln!("Token exchange failed ({}): {}", status, message);
///         }
///         OAuthError::InvalidCallback { reason } => {
///             eprintln!("Invalid callback: {}", reason);
///         }
///         OAuthError::MissingHostConfig => {
///             eprintln!("Configuration error: Host URL not configured");
///         }
///         OAuthError::HttpError(e) => {
///             eprintln!("HTTP error: {}", e);
///         }
///     }
/// }
/// ```
#[derive(Debug, Error)]
pub enum OAuthError {
    /// HMAC signature validation failed.
    ///
    /// This indicates the callback request's HMAC signature does not match
    /// the expected value computed with the API secret key. This could indicate
    /// a tampered request or misconfigured secret key.
    #[error("HMAC signature validation failed")]
    InvalidHmac,

    /// OAuth state parameter mismatch.
    ///
    /// The state parameter in the callback does not match the expected state
    /// that was generated during `begin_auth()`. This is a security measure
    /// against CSRF attacks.
    #[error("State parameter mismatch: expected '{expected}', received '{received}'")]
    StateMismatch {
        /// The expected state value that was generated.
        expected: String,
        /// The state value received in the callback.
        received: String,
    },

    /// Token exchange request failed.
    ///
    /// The POST request to exchange the authorization code for an access token
    /// returned a non-success HTTP status.
    #[error("Token exchange failed with status {status}: {message}")]
    TokenExchangeFailed {
        /// The HTTP status code returned.
        status: u16,
        /// The error message from the response.
        message: String,
    },

    /// Callback parameters are invalid or malformed.
    ///
    /// One or more parameters in the OAuth callback are missing, empty,
    /// or have invalid formats.
    #[error("Invalid callback: {reason}")]
    InvalidCallback {
        /// Description of what's invalid about the callback.
        reason: String,
    },

    /// Host URL is not configured in `ShopifyConfig`.
    ///
    /// The `begin_auth()` function requires a host URL to construct the
    /// redirect URI. Configure this via `ShopifyConfigBuilder::host()`.
    #[error("Host URL must be configured in ShopifyConfig for OAuth")]
    MissingHostConfig,

    /// Wrapped HTTP client error.
    ///
    /// An error occurred during HTTP communication, such as a network failure
    /// or request validation error.
    #[error(transparent)]
    HttpError(#[from] HttpError),
}

// Verify OAuthError is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<OAuthError>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::{HttpResponseError, InvalidHttpRequestError};

    #[test]
    fn test_invalid_hmac_formats_correctly() {
        let error = OAuthError::InvalidHmac;
        assert_eq!(error.to_string(), "HMAC signature validation failed");
    }

    #[test]
    fn test_state_mismatch_includes_expected_and_received() {
        let error = OAuthError::StateMismatch {
            expected: "abc123".to_string(),
            received: "xyz789".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("abc123"));
        assert!(message.contains("xyz789"));
        assert!(message.contains("expected"));
        assert!(message.contains("received"));
    }

    #[test]
    fn test_token_exchange_failed_includes_status_and_message() {
        let error = OAuthError::TokenExchangeFailed {
            status: 401,
            message: "Invalid client credentials".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("401"));
        assert!(message.contains("Invalid client credentials"));
    }

    #[test]
    fn test_from_http_error_conversion() {
        let http_error = HttpError::Response(HttpResponseError {
            code: 500,
            message: "Internal server error".to_string(),
            error_reference: None,
        });
        let oauth_error: OAuthError = http_error.into();
        match oauth_error {
            OAuthError::HttpError(_) => {}
            _ => panic!("Expected HttpError variant"),
        }
    }

    #[test]
    fn test_oauth_error_implements_std_error() {
        let error: &dyn std::error::Error = &OAuthError::InvalidHmac;
        let _ = error;

        let error: &dyn std::error::Error = &OAuthError::StateMismatch {
            expected: "a".to_string(),
            received: "b".to_string(),
        };
        let _ = error;

        let error: &dyn std::error::Error = &OAuthError::TokenExchangeFailed {
            status: 400,
            message: "test".to_string(),
        };
        let _ = error;

        let error: &dyn std::error::Error = &OAuthError::InvalidCallback {
            reason: "test".to_string(),
        };
        let _ = error;

        let error: &dyn std::error::Error = &OAuthError::MissingHostConfig;
        let _ = error;
    }

    #[test]
    fn test_invalid_callback_includes_reason() {
        let error = OAuthError::InvalidCallback {
            reason: "Shop domain is invalid".to_string(),
        };
        assert!(error.to_string().contains("Shop domain is invalid"));
    }

    #[test]
    fn test_missing_host_config_message() {
        let error = OAuthError::MissingHostConfig;
        assert!(error.to_string().contains("Host URL"));
        assert!(error.to_string().contains("configured"));
    }

    #[test]
    fn test_http_error_from_invalid_request() {
        let invalid = InvalidHttpRequestError::MissingBodyType;
        let http_error = HttpError::InvalidRequest(invalid);
        let oauth_error: OAuthError = http_error.into();

        match oauth_error {
            OAuthError::HttpError(HttpError::InvalidRequest(_)) => {}
            _ => panic!("Expected HttpError::InvalidRequest variant"),
        }
    }

    #[test]
    fn test_oauth_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OAuthError>();
    }
}

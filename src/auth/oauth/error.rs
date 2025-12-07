//! OAuth-specific error types for the Shopify API SDK.
//!
//! This module contains error types for OAuth operations including HMAC validation,
//! state verification, token exchange failures, client credentials failures,
//! token refresh failures, and JWT validation for embedded apps.
//!
//! # Error Types
//!
//! - [`OAuthError::InvalidHmac`]: HMAC signature validation failed
//! - [`OAuthError::StateMismatch`]: OAuth state parameter doesn't match expected
//! - [`OAuthError::TokenExchangeFailed`]: Token exchange request failed
//! - [`OAuthError::ClientCredentialsFailed`]: Client credentials exchange request failed
//! - [`OAuthError::TokenRefreshFailed`]: Token refresh or migration request failed
//! - [`OAuthError::InvalidCallback`]: Callback parameters are malformed
//! - [`OAuthError::MissingHostConfig`]: Host URL not configured for redirect URI
//! - [`OAuthError::InvalidJwt`]: JWT validation failed (for token exchange)
//! - [`OAuthError::NotEmbeddedApp`]: Token exchange requires embedded app configuration
//! - [`OAuthError::NotPrivateApp`]: Client credentials requires non-embedded app configuration
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
//!
//! let error = OAuthError::InvalidJwt {
//!     reason: "Token expired".to_string(),
//! };
//! assert!(error.to_string().contains("Token expired"));
//!
//! let error = OAuthError::ClientCredentialsFailed {
//!     status: 401,
//!     message: "Invalid credentials".to_string(),
//! };
//! assert!(error.to_string().contains("401"));
//!
//! let error = OAuthError::TokenRefreshFailed {
//!     status: 400,
//!     message: "Invalid refresh token".to_string(),
//! };
//! assert!(error.to_string().contains("400"));
//! ```

use crate::clients::HttpError;
use thiserror::Error;

/// Errors that can occur during OAuth operations.
///
/// This enum covers all failure modes in OAuth flows, including the authorization
/// code flow, token exchange, client credentials, token refresh, and JWT validation
/// for embedded apps.
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
///         OAuthError::ClientCredentialsFailed { status, message } => {
///             eprintln!("Client credentials failed ({}): {}", status, message);
///         }
///         OAuthError::TokenRefreshFailed { status, message } => {
///             eprintln!("Token refresh failed ({}): {}", status, message);
///         }
///         OAuthError::InvalidCallback { reason } => {
///             eprintln!("Invalid callback: {}", reason);
///         }
///         OAuthError::MissingHostConfig => {
///             eprintln!("Configuration error: Host URL not configured");
///         }
///         OAuthError::InvalidJwt { reason } => {
///             eprintln!("JWT validation failed: {}", reason);
///         }
///         OAuthError::NotEmbeddedApp => {
///             eprintln!("Token exchange only works for embedded apps");
///         }
///         OAuthError::NotPrivateApp => {
///             eprintln!("Client credentials only works for private apps");
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

    /// Client credentials exchange request failed.
    ///
    /// The POST request to obtain an access token using client credentials
    /// returned a non-success HTTP status. This error is specific to the
    /// Client Credentials Grant flow used by private/organization apps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::auth::oauth::OAuthError;
    ///
    /// let error = OAuthError::ClientCredentialsFailed {
    ///     status: 401,
    ///     message: "Invalid client credentials".to_string(),
    /// };
    /// assert!(error.to_string().contains("Client credentials"));
    /// assert!(error.to_string().contains("401"));
    /// ```
    #[error("Client credentials exchange failed with status {status}: {message}")]
    ClientCredentialsFailed {
        /// The HTTP status code returned (0 for network errors).
        status: u16,
        /// The error message from the response or network error description.
        message: String,
    },

    /// Token refresh or migration request failed.
    ///
    /// The POST request to refresh an access token or migrate to expiring tokens
    /// returned a non-success HTTP status. This error is used for both the
    /// `refresh_access_token` and `migrate_to_expiring_token` functions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::auth::oauth::OAuthError;
    ///
    /// let error = OAuthError::TokenRefreshFailed {
    ///     status: 400,
    ///     message: "Invalid refresh token".to_string(),
    /// };
    /// assert!(error.to_string().contains("Token refresh"));
    /// assert!(error.to_string().contains("400"));
    /// ```
    #[error("Token refresh failed with status {status}: {message}")]
    TokenRefreshFailed {
        /// The HTTP status code returned (0 for network errors).
        status: u16,
        /// The error message from the response or network error description.
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

    /// JWT validation failed.
    ///
    /// This error occurs during token exchange when the session token (JWT)
    /// provided by App Bridge cannot be validated. Common causes include:
    ///
    /// - Token is expired or not yet valid
    /// - Token was signed with a different secret key
    /// - Token's audience (`aud`) claim doesn't match the app's API key
    /// - Token structure is malformed
    /// - Shopify rejected the token during token exchange
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::auth::oauth::OAuthError;
    ///
    /// let error = OAuthError::InvalidJwt {
    ///     reason: "Session token had invalid API key".to_string(),
    /// };
    /// assert!(error.to_string().contains("Invalid JWT"));
    /// ```
    #[error("Invalid JWT: {reason}")]
    InvalidJwt {
        /// Description of why the JWT validation failed.
        reason: String,
    },

    /// Token exchange requires an embedded app configuration.
    ///
    /// Token exchange OAuth flow is only available for embedded apps that
    /// receive session tokens from Shopify App Bridge. Ensure that
    /// `ShopifyConfigBuilder::is_embedded(true)` is set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::auth::oauth::OAuthError;
    ///
    /// let error = OAuthError::NotEmbeddedApp;
    /// assert!(error.to_string().contains("embedded app"));
    /// ```
    #[error("Token exchange requires an embedded app configuration")]
    NotEmbeddedApp,

    /// Client credentials requires a non-embedded app configuration.
    ///
    /// Client Credentials Grant OAuth flow is only available for private or
    /// organization apps that are NOT embedded in the Shopify admin. Ensure
    /// that `ShopifyConfigBuilder::is_embedded(false)` is set (or not set,
    /// as `false` is the default).
    ///
    /// This error is the inverse of [`NotEmbeddedApp`](OAuthError::NotEmbeddedApp),
    /// which is used for token exchange flows that require embedded apps.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::auth::oauth::OAuthError;
    ///
    /// let error = OAuthError::NotPrivateApp;
    /// assert!(error.to_string().contains("non-embedded"));
    /// ```
    #[error("Client credentials requires a non-embedded app configuration")]
    NotPrivateApp,

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

        let error: &dyn std::error::Error = &OAuthError::InvalidJwt {
            reason: "test".to_string(),
        };
        let _ = error;

        let error: &dyn std::error::Error = &OAuthError::NotEmbeddedApp;
        let _ = error;

        let error: &dyn std::error::Error = &OAuthError::ClientCredentialsFailed {
            status: 401,
            message: "test".to_string(),
        };
        let _ = error;

        let error: &dyn std::error::Error = &OAuthError::NotPrivateApp;
        let _ = error;

        let error: &dyn std::error::Error = &OAuthError::TokenRefreshFailed {
            status: 400,
            message: "test".to_string(),
        };
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

    // === New tests for InvalidJwt and NotEmbeddedApp variants ===

    #[test]
    fn test_invalid_jwt_formats_error_message_with_reason() {
        let error = OAuthError::InvalidJwt {
            reason: "Token expired".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("Invalid JWT"));
        assert!(message.contains("Token expired"));
    }

    #[test]
    fn test_not_embedded_app_has_correct_error_message() {
        let error = OAuthError::NotEmbeddedApp;
        let message = error.to_string();
        assert!(message.contains("embedded app"));
        assert!(message.contains("Token exchange"));
    }

    #[test]
    fn test_new_variants_implement_std_error() {
        // InvalidJwt implements std::error::Error
        let invalid_jwt_error: &dyn std::error::Error = &OAuthError::InvalidJwt {
            reason: "test reason".to_string(),
        };
        assert!(invalid_jwt_error.to_string().contains("Invalid JWT"));

        // NotEmbeddedApp implements std::error::Error
        let not_embedded_error: &dyn std::error::Error = &OAuthError::NotEmbeddedApp;
        assert!(not_embedded_error.to_string().contains("embedded app"));
    }

    #[test]
    fn test_new_variants_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        // These compile-time assertions verify Send + Sync
        assert_send_sync::<OAuthError>();

        // Also verify the specific variants at runtime
        let invalid_jwt = OAuthError::InvalidJwt {
            reason: "test".to_string(),
        };
        let not_embedded = OAuthError::NotEmbeddedApp;

        // Can be sent across threads
        std::thread::spawn(move || {
            let _ = invalid_jwt;
        })
        .join()
        .unwrap();

        std::thread::spawn(move || {
            let _ = not_embedded;
        })
        .join()
        .unwrap();
    }

    // === Tests for ClientCredentialsFailed and NotPrivateApp variants ===

    #[test]
    fn test_client_credentials_failed_formats_error_message_with_status_and_message() {
        let error = OAuthError::ClientCredentialsFailed {
            status: 401,
            message: "Invalid client credentials".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("Client credentials"));
        assert!(message.contains("401"));
        assert!(message.contains("Invalid client credentials"));
    }

    #[test]
    fn test_not_private_app_has_correct_error_message() {
        let error = OAuthError::NotPrivateApp;
        let message = error.to_string();
        assert!(message.contains("non-embedded"));
        assert!(message.contains("Client credentials"));
    }

    #[test]
    fn test_client_credentials_variants_implement_std_error() {
        // ClientCredentialsFailed implements std::error::Error
        let client_creds_error: &dyn std::error::Error = &OAuthError::ClientCredentialsFailed {
            status: 500,
            message: "Server error".to_string(),
        };
        assert!(client_creds_error
            .to_string()
            .contains("Client credentials"));

        // NotPrivateApp implements std::error::Error
        let not_private_error: &dyn std::error::Error = &OAuthError::NotPrivateApp;
        assert!(not_private_error.to_string().contains("non-embedded"));
    }

    #[test]
    fn test_client_credentials_variants_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}

        // These compile-time assertions verify Send + Sync
        assert_send_sync::<OAuthError>();

        // Also verify the specific variants at runtime
        let client_creds_failed = OAuthError::ClientCredentialsFailed {
            status: 401,
            message: "test".to_string(),
        };
        let not_private = OAuthError::NotPrivateApp;

        // Can be sent across threads
        std::thread::spawn(move || {
            let _ = client_creds_failed;
        })
        .join()
        .unwrap();

        std::thread::spawn(move || {
            let _ = not_private;
        })
        .join()
        .unwrap();
    }

    // === Tests for TokenRefreshFailed variant ===

    #[test]
    fn test_token_refresh_failed_formats_error_message_with_status_and_message() {
        let error = OAuthError::TokenRefreshFailed {
            status: 400,
            message: "Invalid refresh token".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("Token refresh"));
        assert!(message.contains("400"));
        assert!(message.contains("Invalid refresh token"));
    }

    #[test]
    fn test_token_refresh_failed_with_network_error_status_zero() {
        let error = OAuthError::TokenRefreshFailed {
            status: 0,
            message: "Network error: connection refused".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("Token refresh"));
        assert!(message.contains("0"));
        assert!(message.contains("Network error"));
    }

    #[test]
    fn test_token_refresh_failed_implements_std_error() {
        let error: &dyn std::error::Error = &OAuthError::TokenRefreshFailed {
            status: 401,
            message: "Unauthorized".to_string(),
        };
        assert!(error.to_string().contains("Token refresh"));
    }

    #[test]
    fn test_token_refresh_failed_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OAuthError>();

        let token_refresh_failed = OAuthError::TokenRefreshFailed {
            status: 400,
            message: "test".to_string(),
        };

        std::thread::spawn(move || {
            let _ = token_refresh_failed;
        })
        .join()
        .unwrap();
    }
}

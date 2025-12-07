//! REST-specific error types for the Shopify API SDK.
//!
//! This module contains error types for REST API operations, including
//! the REST API being disabled, invalid paths, and wrapped HTTP errors.
//!
//! # Error Handling
//!
//! The SDK uses specific error types for different failure scenarios:
//!
//! - [`RestError::RestApiDisabled`]: When the REST API is deprecated/disabled in config
//! - [`RestError::InvalidPath`]: When a REST API path fails validation
//! - [`RestError::Http`]: Wraps underlying HTTP errors
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::clients::rest::{RestClient, RestError};
//!
//! match client.get("products", None).await {
//!     Ok(response) => println!("Products: {}", response.body),
//!     Err(RestError::RestApiDisabled) => {
//!         println!("REST API is disabled. Please use GraphQL.");
//!     }
//!     Err(RestError::InvalidPath { path }) => {
//!         println!("Invalid path: {}", path);
//!     }
//!     Err(RestError::Http(e)) => {
//!         println!("HTTP error: {}", e);
//!     }
//! }
//! ```

use crate::clients::HttpError;
use thiserror::Error;

/// Error type for REST API operations.
///
/// This enum provides specific error types for REST API operations,
/// wrapping HTTP errors and adding REST-specific error cases.
///
/// # Example
///
/// ```rust
/// use shopify_api::clients::rest::RestError;
///
/// // REST API disabled error
/// let error = RestError::RestApiDisabled;
/// assert!(error.to_string().contains("deprecated"));
///
/// // Invalid path error
/// let error = RestError::InvalidPath { path: "".to_string() };
/// assert!(error.to_string().contains("Invalid"));
/// ```
#[derive(Debug, Error)]
pub enum RestError {
    /// The REST Admin API has been deprecated and is disabled.
    ///
    /// This error is returned when the REST API is disabled in the SDK
    /// configuration, indicating that users should migrate to GraphQL.
    #[error("The Admin REST API has been deprecated. Please use the GraphQL Admin API. For more information see https://www.shopify.com/ca/partners/blog/all-in-on-graphql")]
    RestApiDisabled,

    /// The REST API path is invalid.
    ///
    /// This error is returned when a path fails validation, such as
    /// when it is empty after normalization.
    #[error("Invalid REST API path: {path}")]
    InvalidPath {
        /// The invalid path that was provided.
        path: String,
    },

    /// An HTTP-level error occurred.
    ///
    /// This variant wraps [`HttpError`] for unified error handling.
    #[error(transparent)]
    Http(#[from] HttpError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::{HttpResponseError, MaxHttpRetriesExceededError};

    #[test]
    fn test_rest_api_disabled_error_message_matches_ruby_sdk() {
        let error = RestError::RestApiDisabled;
        let message = error.to_string();

        // Match Ruby SDK error message format
        assert!(message.contains("The Admin REST API has been deprecated"));
        assert!(message.contains("Please use the GraphQL Admin API"));
        assert!(message.contains("https://www.shopify.com/ca/partners/blog/all-in-on-graphql"));
    }

    #[test]
    fn test_invalid_path_error_includes_path_in_message() {
        let error = RestError::InvalidPath {
            path: "/invalid/path".to_string(),
        };
        let message = error.to_string();

        assert!(message.contains("Invalid REST API path"));
        assert!(message.contains("/invalid/path"));
    }

    #[test]
    fn test_invalid_path_error_with_empty_path() {
        let error = RestError::InvalidPath {
            path: String::new(),
        };
        let message = error.to_string();

        assert!(message.contains("Invalid REST API path"));
        // Empty path should still be mentioned (as empty string)
        assert_eq!(message, "Invalid REST API path: ");
    }

    #[test]
    fn test_http_error_wraps_http_response_error() {
        let http_error = HttpError::Response(HttpResponseError {
            code: 404,
            message: r#"{"error":"Not Found"}"#.to_string(),
            error_reference: Some("abc-123".to_string()),
        });

        let rest_error = RestError::Http(http_error);
        let message = rest_error.to_string();

        assert!(message.contains("Not Found"));
    }

    #[test]
    fn test_from_http_error_conversion() {
        let http_error = HttpError::Response(HttpResponseError {
            code: 500,
            message: r#"{"error":"Internal Server Error"}"#.to_string(),
            error_reference: None,
        });

        // Test From<HttpError> conversion
        let rest_error: RestError = http_error.into();

        assert!(matches!(rest_error, RestError::Http(_)));
    }

    #[test]
    fn test_all_error_variants_implement_std_error() {
        // RestApiDisabled
        let disabled_error: &dyn std::error::Error = &RestError::RestApiDisabled;
        let _ = disabled_error;

        // InvalidPath
        let path_error: &dyn std::error::Error = &RestError::InvalidPath {
            path: "test".to_string(),
        };
        let _ = path_error;

        // Http
        let http_error: &dyn std::error::Error =
            &RestError::Http(HttpError::Response(HttpResponseError {
                code: 400,
                message: "test".to_string(),
                error_reference: None,
            }));
        let _ = http_error;
    }

    #[test]
    fn test_http_error_wraps_max_retries_exceeded() {
        let http_error = HttpError::MaxRetries(MaxHttpRetriesExceededError {
            code: 429,
            tries: 3,
            message: r#"{"error":"Rate limited"}"#.to_string(),
            error_reference: None,
        });

        let rest_error = RestError::Http(http_error);
        let message = rest_error.to_string();

        assert!(message.contains("Exceeded maximum retry count"));
        assert!(message.contains("3"));
    }
}

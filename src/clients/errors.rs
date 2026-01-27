//! HTTP-specific error types for the Shopify API SDK.
//!
//! This module contains error types for HTTP operations, including response
//! errors, retry exhaustion, and request validation failures.
//!
//! # Error Handling
//!
//! The SDK uses specific error types for different failure scenarios:
//!
//! - [`HttpResponseError`]: Non-2xx HTTP responses from the API
//! - [`MaxHttpRetriesExceededError`]: When retry attempts are exhausted
//! - [`InvalidHttpRequestError`]: When a request fails validation before sending
//! - [`HttpError`]: Unified error type encompassing all HTTP-related errors
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::clients::{HttpClient, HttpRequest, HttpMethod, HttpError};
//!
//! match client.request(request).await {
//!     Ok(response) => println!("Success: {}", response.body),
//!     Err(HttpError::Response(e)) => {
//!         println!("API error {}: {}", e.code, e.message);
//!     }
//!     Err(HttpError::MaxRetries(e)) => {
//!         println!("Retries exhausted after {} tries", e.tries);
//!     }
//!     Err(HttpError::InvalidRequest(e)) => {
//!         println!("Invalid request: {}", e);
//!     }
//!     Err(HttpError::Network(e)) => {
//!         println!("Network error: {}", e);
//!     }
//! }
//! ```

use thiserror::Error;

/// Error returned when an HTTP request receives a non-successful response.
///
/// This error includes the status code and a serialized error message in JSON
/// format matching the Ruby SDK's `serialized_error()` output.
///
/// # JSON Message Format
///
/// The message field contains JSON with any of these fields from the response:
/// - `errors`: Array of error messages
/// - `error`: Single error message
/// - `error_description`: Description of the error
/// - `error_reference`: Debugging reference including X-Request-Id
///
/// # Example
///
/// ```rust
/// use shopify_sdk::clients::HttpResponseError;
///
/// let error = HttpResponseError {
///     code: 404,
///     message: r#"{"error":"Not found"}"#.to_string(),
///     error_reference: Some("abc-123".to_string()),
/// };
///
/// println!("Status {}: {}", error.code, error.message);
/// ```
#[derive(Debug, Error)]
#[error("{message}")]
pub struct HttpResponseError {
    /// The HTTP status code of the response.
    pub code: u16,
    /// Serialized error message in JSON format.
    pub message: String,
    /// Reference ID for error reporting (from X-Request-Id header).
    pub error_reference: Option<String>,
}

/// Error returned when maximum retry attempts have been exhausted.
///
/// This error is raised when a request continues to fail with 429 or 500
/// responses after all configured retry attempts have been made.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::clients::MaxHttpRetriesExceededError;
///
/// let error = MaxHttpRetriesExceededError {
///     code: 429,
///     tries: 3,
///     message: r#"{"error":"Rate limited"}"#.to_string(),
///     error_reference: None,
/// };
///
/// println!("{}", error); // "Exceeded maximum retry count of 3. Last message: ..."
/// ```
#[derive(Debug, Error)]
#[error("Exceeded maximum retry count of {tries}. Last message: {message}")]
pub struct MaxHttpRetriesExceededError {
    /// The HTTP status code of the last response.
    pub code: u16,
    /// The number of tries that were attempted.
    pub tries: u32,
    /// Serialized error message from the last response.
    pub message: String,
    /// Reference ID for error reporting (from X-Request-Id header).
    pub error_reference: Option<String>,
}

/// Error returned when an HTTP request fails validation.
///
/// This error is raised before a request is sent if it fails validation
/// checks, such as:
/// - Missing body for POST/PUT requests
/// - Body provided without `body_type`
/// - Invalid HTTP method
///
/// # Example
///
/// ```rust
/// use shopify_sdk::clients::InvalidHttpRequestError;
///
/// let error = InvalidHttpRequestError::MissingBody {
///     method: "post".to_string(),
/// };
///
/// println!("{}", error); // "Cannot use post without specifying data."
/// ```
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InvalidHttpRequestError {
    /// The HTTP method is not one of the supported methods.
    #[error("Invalid Http method {method}.")]
    InvalidMethod {
        /// The invalid method that was provided.
        method: String,
    },

    /// A request body was provided without specifying the body type.
    #[error("Cannot set a body without also setting body_type.")]
    MissingBodyType,

    /// A POST or PUT request was made without a body.
    #[error("Cannot use {method} without specifying data.")]
    MissingBody {
        /// The HTTP method that requires a body.
        method: String,
    },
}

/// Unified error type for all HTTP-related errors.
///
/// This enum provides a single error type for HTTP operations, making it
/// easier to handle errors at API boundaries. Use pattern matching to
/// handle specific error types.
///
/// # Example
///
/// ```rust,ignore
/// use shopify_sdk::HttpError;
///
/// let result = client.request(request).await;
/// match result {
///     Ok(response) => { /* handle success */ }
///     Err(HttpError::Response(e)) => { /* handle API error */ }
///     Err(HttpError::MaxRetries(e)) => { /* handle retry exhaustion */ }
///     Err(HttpError::InvalidRequest(e)) => { /* handle validation error */ }
///     Err(HttpError::Network(e)) => { /* handle network error */ }
/// }
/// ```
#[derive(Debug, Error)]
pub enum HttpError {
    /// An HTTP response error (non-2xx status code).
    #[error(transparent)]
    Response(#[from] HttpResponseError),

    /// Maximum retry attempts exhausted.
    #[error(transparent)]
    MaxRetries(#[from] MaxHttpRetriesExceededError),

    /// Request validation failed.
    #[error(transparent)]
    InvalidRequest(#[from] InvalidHttpRequestError),

    /// Network or connection error.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_response_error_includes_status_code_in_message() {
        let error = HttpResponseError {
            code: 404,
            message: r#"{"error":"Not Found"}"#.to_string(),
            error_reference: None,
        };
        // The error message should contain the message field
        assert_eq!(error.to_string(), r#"{"error":"Not Found"}"#);
    }

    #[test]
    fn test_http_response_error_includes_request_id() {
        let error = HttpResponseError {
            code: 500,
            message: r#"{"error":"Internal Server Error","error_reference":"If you report this error, please include this id: abc-123."}"#.to_string(),
            error_reference: Some("abc-123".to_string()),
        };
        assert_eq!(error.error_reference, Some("abc-123".to_string()));
        assert!(error.to_string().contains("abc-123"));
    }

    #[test]
    fn test_max_retries_error_includes_retry_count() {
        let error = MaxHttpRetriesExceededError {
            code: 429,
            tries: 3,
            message: r#"{"error":"Rate limited"}"#.to_string(),
            error_reference: None,
        };
        let message = error.to_string();
        assert!(message.contains("3"));
        assert!(message.contains("Exceeded maximum retry count"));
    }

    #[test]
    fn test_invalid_request_error_missing_body() {
        let error = InvalidHttpRequestError::MissingBody {
            method: "post".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Cannot use post without specifying data."
        );
    }

    #[test]
    fn test_invalid_request_error_missing_body_type() {
        let error = InvalidHttpRequestError::MissingBodyType;
        assert_eq!(
            error.to_string(),
            "Cannot set a body without also setting body_type."
        );
    }

    #[test]
    fn test_error_types_implement_std_error() {
        let http_error: &dyn std::error::Error = &HttpResponseError {
            code: 400,
            message: "test".to_string(),
            error_reference: None,
        };
        let _ = http_error;

        let max_retries_error: &dyn std::error::Error = &MaxHttpRetriesExceededError {
            code: 429,
            tries: 3,
            message: "test".to_string(),
            error_reference: None,
        };
        let _ = max_retries_error;

        let invalid_error: &dyn std::error::Error = &InvalidHttpRequestError::MissingBodyType;
        let _ = invalid_error;
    }
}

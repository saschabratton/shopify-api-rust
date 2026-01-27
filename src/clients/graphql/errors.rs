//! GraphQL-specific error types for the Shopify API SDK.
//!
//! This module contains error types for GraphQL API operations, including
//! wrapped HTTP errors.
//!
//! # Error Handling
//!
//! The SDK uses specific error types for different failure scenarios.
//! For GraphQL, only HTTP-level errors are exposed. GraphQL-level errors
//! (such as user errors or validation errors) are returned in the response
//! body with HTTP status 200, and are the user's responsibility to parse.
//!
//! - [`GraphqlError::Http`]: Wraps underlying HTTP errors
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::clients::graphql::{GraphqlClient, GraphqlError};
//!
//! match client.query("query { shop { name } }", None, None, None).await {
//!     Ok(response) => {
//!         // Check for GraphQL errors in response body
//!         if let Some(errors) = response.body.get("errors") {
//!             println!("GraphQL errors: {}", errors);
//!         } else {
//!             println!("Data: {}", response.body["data"]);
//!         }
//!     }
//!     Err(GraphqlError::Http(e)) => {
//!         println!("HTTP error: {}", e);
//!     }
//! }
//! ```

use crate::clients::HttpError;
use thiserror::Error;

/// Error type for GraphQL API operations.
///
/// This enum provides error types for GraphQL API operations,
/// wrapping HTTP errors. Unlike REST errors, GraphQL does not need
/// path validation or API-disabled variants.
///
/// Note that GraphQL-level errors (like user errors, validation errors)
/// are returned with HTTP 200 status and are contained in the response
/// body's `errors` field. These are not treated as SDK errors.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::clients::graphql::GraphqlError;
/// use shopify_sdk::clients::{HttpError, HttpResponseError};
///
/// // HTTP error wrapping
/// let http_error = HttpError::Response(HttpResponseError {
///     code: 401,
///     message: r#"{"error":"Unauthorized"}"#.to_string(),
///     error_reference: None,
/// });
/// let graphql_error: GraphqlError = http_error.into();
/// assert!(graphql_error.to_string().contains("Unauthorized"));
/// ```
#[derive(Debug, Error)]
pub enum GraphqlError {
    /// An HTTP-level error occurred.
    ///
    /// This variant wraps [`HttpError`] for unified error handling.
    /// It includes network errors, non-2xx responses, and retry exhaustion.
    #[error(transparent)]
    Http(#[from] HttpError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::{HttpResponseError, MaxHttpRetriesExceededError};

    #[test]
    fn test_graphql_error_http_variant_wraps_http_error() {
        let http_error = HttpError::Response(HttpResponseError {
            code: 404,
            message: r#"{"error":"Not Found"}"#.to_string(),
            error_reference: Some("abc-123".to_string()),
        });

        let graphql_error = GraphqlError::Http(http_error);
        let message = graphql_error.to_string();

        assert!(message.contains("Not Found"));
    }

    #[test]
    fn test_all_error_variants_implement_std_error() {
        // Http variant
        let http_error: &dyn std::error::Error =
            &GraphqlError::Http(HttpError::Response(HttpResponseError {
                code: 400,
                message: "test".to_string(),
                error_reference: None,
            }));
        let _ = http_error;
    }

    #[test]
    fn test_from_http_error_conversion() {
        let http_error = HttpError::Response(HttpResponseError {
            code: 500,
            message: r#"{"error":"Internal Server Error"}"#.to_string(),
            error_reference: None,
        });

        // Test From<HttpError> conversion
        let graphql_error: GraphqlError = http_error.into();

        assert!(matches!(graphql_error, GraphqlError::Http(_)));
    }

    #[test]
    fn test_http_error_wraps_max_retries_exceeded() {
        let http_error = HttpError::MaxRetries(MaxHttpRetriesExceededError {
            code: 429,
            tries: 3,
            message: r#"{"error":"Rate limited"}"#.to_string(),
            error_reference: None,
        });

        let graphql_error = GraphqlError::Http(http_error);
        let message = graphql_error.to_string();

        assert!(message.contains("Exceeded maximum retry count"));
        assert!(message.contains("3"));
    }
}

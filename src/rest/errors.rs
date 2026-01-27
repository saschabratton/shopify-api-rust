//! Resource-specific error types for REST API operations.
//!
//! This module contains error types for REST resource operations, extending
//! the base [`RestError`](crate::clients::RestError) with resource-specific
//! semantics like `NotFound` and `ValidationFailed`.
//!
//! # Error Handling
//!
//! The SDK maps HTTP status codes to semantic error variants:
//!
//! - **404**: [`ResourceError::NotFound`] - Resource doesn't exist
//! - **422**: [`ResourceError::ValidationFailed`] - Validation errors from the API
//! - **Other 4xx/5xx**: [`ResourceError::Http`] - Wrapped HTTP error
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceError};
//!
//! match Product::find(&client, 123, None).await {
//!     Ok(product) => println!("Found: {}", product.title),
//!     Err(ResourceError::NotFound { resource, id }) => {
//!         println!("{} with id {} not found", resource, id);
//!     }
//!     Err(ResourceError::ValidationFailed { errors, .. }) => {
//!         for (field, messages) in errors {
//!             println!("{}: {:?}", field, messages);
//!         }
//!     }
//!     Err(e) => println!("Other error: {}", e),
//! }
//! ```

use std::collections::HashMap;

use crate::clients::{HttpError, RestError};
use thiserror::Error;

/// Error type for REST resource operations.
///
/// This enum provides semantic error types for resource operations,
/// mapping HTTP error codes to meaningful variants while preserving
/// the request ID for debugging.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::rest::ResourceError;
/// use std::collections::HashMap;
///
/// // Not found error
/// let error = ResourceError::NotFound {
///     resource: "Product",
///     id: "123".to_string(),
/// };
/// assert!(error.to_string().contains("Product"));
/// assert!(error.to_string().contains("123"));
///
/// // Validation failed error
/// let mut errors = HashMap::new();
/// errors.insert("title".to_string(), vec!["can't be blank".to_string()]);
/// let error = ResourceError::ValidationFailed {
///     errors,
///     request_id: Some("abc-123".to_string()),
/// };
/// assert!(error.to_string().contains("Validation failed"));
/// ```
#[derive(Debug, Error)]
pub enum ResourceError {
    /// The resource was not found (HTTP 404).
    ///
    /// This error is returned when attempting to find, update, or delete
    /// a resource that doesn't exist.
    #[error("{resource} with id {id} not found")]
    NotFound {
        /// The type name of the resource (e.g., "Product", "Order").
        resource: &'static str,
        /// The ID that was requested.
        id: String,
    },

    /// Validation failed for the resource (HTTP 422).
    ///
    /// This error is returned when the API rejects a create or update
    /// request due to validation errors.
    #[error("Validation failed: {errors:?}")]
    ValidationFailed {
        /// A map of field names to error messages.
        errors: HashMap<String, Vec<String>>,
        /// The request ID for debugging (from X-Request-Id header).
        request_id: Option<String>,
    },

    /// No valid path matches the provided IDs and operation.
    ///
    /// This error is returned when attempting an operation without
    /// providing the required parent resource IDs.
    #[error("Cannot resolve path for {resource}::{operation} with provided IDs")]
    PathResolutionFailed {
        /// The type name of the resource.
        resource: &'static str,
        /// The operation being attempted (e.g., "find", "all", "delete").
        operation: &'static str,
    },

    /// An HTTP-level error occurred.
    ///
    /// This variant wraps [`HttpError`] for errors that don't map to
    /// a specific resource error type.
    #[error(transparent)]
    Http(#[from] HttpError),

    /// A REST-level error occurred.
    ///
    /// This variant wraps [`RestError`] for REST client errors.
    #[error(transparent)]
    Rest(#[from] RestError),
}

impl ResourceError {
    /// Creates a `ResourceError` from an HTTP response status code.
    ///
    /// Maps HTTP status codes to semantic error variants:
    /// - 404 -> `NotFound`
    /// - 422 -> `ValidationFailed` (parsing errors from body)
    /// - Other -> `Http`
    ///
    /// # Arguments
    ///
    /// * `code` - The HTTP status code
    /// * `body` - The response body as JSON
    /// * `resource` - The resource type name (e.g., "Product")
    /// * `id` - The resource ID (if applicable)
    /// * `request_id` - The X-Request-Id header value
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::rest::ResourceError;
    /// use serde_json::json;
    ///
    /// let error = ResourceError::from_http_response(
    ///     404,
    ///     &json!({"error": "Not found"}),
    ///     "Product",
    ///     Some("123"),
    ///     Some("req-123"),
    /// );
    /// assert!(matches!(error, ResourceError::NotFound { .. }));
    /// ```
    #[must_use]
    pub fn from_http_response(
        code: u16,
        body: &serde_json::Value,
        resource: &'static str,
        id: Option<&str>,
        request_id: Option<&str>,
    ) -> Self {
        match code {
            404 => Self::NotFound {
                resource,
                id: id.unwrap_or("unknown").to_string(),
            },
            422 => {
                let errors = parse_validation_errors(body);
                Self::ValidationFailed {
                    errors,
                    request_id: request_id.map(ToString::to_string),
                }
            }
            _ => {
                // For other errors, create an HttpResponseError
                let message = body.to_string();
                Self::Http(HttpError::Response(crate::clients::HttpResponseError {
                    code,
                    message,
                    error_reference: request_id.map(ToString::to_string),
                }))
            }
        }
    }

    /// Returns the request ID if available.
    ///
    /// Useful for debugging and error reporting.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::ValidationFailed { request_id, .. } => request_id.as_deref(),
            Self::Http(HttpError::Response(e)) => e.error_reference.as_deref(),
            Self::Http(HttpError::MaxRetries(e)) => e.error_reference.as_deref(),
            _ => None,
        }
    }
}

/// Parses validation errors from an API response body.
///
/// Shopify returns validation errors in the format:
/// ```json
/// {
///   "errors": {
///     "title": ["can't be blank", "is too short"],
///     "price": ["must be greater than 0"]
///   }
/// }
/// ```
///
/// Or as an array:
/// ```json
/// {
///   "errors": ["Title can't be blank", "Price must be greater than 0"]
/// }
/// ```
fn parse_validation_errors(body: &serde_json::Value) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();

    if let Some(errors) = body.get("errors") {
        match errors {
            // Object format: {"field": ["error1", "error2"]}
            serde_json::Value::Object(map) => {
                for (field, messages) in map {
                    let msgs: Vec<String> = match messages {
                        serde_json::Value::Array(arr) => arr
                            .iter()
                            .filter_map(|v| v.as_str().map(ToString::to_string))
                            .collect(),
                        serde_json::Value::String(s) => vec![s.clone()],
                        _ => vec![messages.to_string()],
                    };
                    result.insert(field.clone(), msgs);
                }
            }
            // Array format: ["error1", "error2"]
            serde_json::Value::Array(arr) => {
                let msgs: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect();
                if !msgs.is_empty() {
                    result.insert("base".to_string(), msgs);
                }
            }
            // String format: "single error"
            serde_json::Value::String(s) => {
                result.insert("base".to_string(), vec![s.clone()]);
            }
            _ => {}
        }
    }

    result
}

// Verify ResourceError is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ResourceError>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_not_found_error_formats_message_with_resource_and_id() {
        let error = ResourceError::NotFound {
            resource: "Product",
            id: "123456".to_string(),
        };
        let message = error.to_string();

        assert!(message.contains("Product"));
        assert!(message.contains("123456"));
        assert!(message.contains("not found"));
    }

    #[test]
    fn test_validation_failed_stores_and_retrieves_field_errors() {
        let mut errors = HashMap::new();
        errors.insert("title".to_string(), vec!["can't be blank".to_string()]);
        errors.insert(
            "price".to_string(),
            vec![
                "must be greater than 0".to_string(),
                "is invalid".to_string(),
            ],
        );

        let error = ResourceError::ValidationFailed {
            errors: errors.clone(),
            request_id: Some("abc-123".to_string()),
        };

        if let ResourceError::ValidationFailed {
            errors: returned_errors,
            request_id,
        } = error
        {
            assert_eq!(returned_errors.len(), 2);
            assert_eq!(
                returned_errors.get("title"),
                Some(&vec!["can't be blank".to_string()])
            );
            assert_eq!(returned_errors.get("price").map(|v| v.len()), Some(2));
            assert_eq!(request_id, Some("abc-123".to_string()));
        } else {
            panic!("Expected ValidationFailed variant");
        }
    }

    #[test]
    fn test_path_resolution_failed_includes_operation_context() {
        let error = ResourceError::PathResolutionFailed {
            resource: "Variant",
            operation: "find",
        };
        let message = error.to_string();

        assert!(message.contains("Variant"));
        assert!(message.contains("find"));
        assert!(message.contains("path"));
    }

    #[test]
    fn test_http_error_wraps_correctly() {
        let http_error = HttpError::Response(crate::clients::HttpResponseError {
            code: 500,
            message: r#"{"error":"Internal Server Error"}"#.to_string(),
            error_reference: Some("req-xyz".to_string()),
        });

        let resource_error = ResourceError::Http(http_error);
        let message = resource_error.to_string();

        assert!(message.contains("Internal Server Error"));
    }

    #[test]
    fn test_from_http_error_conversion() {
        let http_error = HttpError::Response(crate::clients::HttpResponseError {
            code: 503,
            message: "Service unavailable".to_string(),
            error_reference: None,
        });

        let resource_error: ResourceError = http_error.into();
        assert!(matches!(resource_error, ResourceError::Http(_)));
    }

    #[test]
    fn test_from_rest_error_conversion() {
        let rest_error = RestError::InvalidPath {
            path: "/bad/path".to_string(),
        };

        let resource_error: ResourceError = rest_error.into();
        assert!(matches!(resource_error, ResourceError::Rest(_)));
    }

    #[test]
    fn test_all_error_variants_implement_std_error() {
        // NotFound
        let not_found_error: &dyn std::error::Error = &ResourceError::NotFound {
            resource: "Product",
            id: "123".to_string(),
        };
        let _ = not_found_error;

        // ValidationFailed
        let validation_error: &dyn std::error::Error = &ResourceError::ValidationFailed {
            errors: HashMap::new(),
            request_id: None,
        };
        let _ = validation_error;

        // PathResolutionFailed
        let path_error: &dyn std::error::Error = &ResourceError::PathResolutionFailed {
            resource: "Variant",
            operation: "all",
        };
        let _ = path_error;

        // Http
        let http_error: &dyn std::error::Error =
            &ResourceError::Http(HttpError::Response(crate::clients::HttpResponseError {
                code: 400,
                message: "test".to_string(),
                error_reference: None,
            }));
        let _ = http_error;

        // Rest
        let rest_error: &dyn std::error::Error = &ResourceError::Rest(RestError::InvalidPath {
            path: "test".to_string(),
        });
        let _ = rest_error;
    }

    #[test]
    fn test_from_http_response_maps_404_to_not_found() {
        let error = ResourceError::from_http_response(
            404,
            &json!({"error": "Not found"}),
            "Product",
            Some("123"),
            Some("req-123"),
        );

        assert!(matches!(
            error,
            ResourceError::NotFound { resource: "Product", id } if id == "123"
        ));
    }

    #[test]
    fn test_from_http_response_maps_422_to_validation_failed() {
        let body = json!({
            "errors": {
                "title": ["can't be blank"],
                "price": ["must be a number", "must be positive"]
            }
        });

        let error =
            ResourceError::from_http_response(422, &body, "Product", Some("123"), Some("req-456"));

        if let ResourceError::ValidationFailed { errors, request_id } = error {
            assert_eq!(
                errors.get("title"),
                Some(&vec!["can't be blank".to_string()])
            );
            assert_eq!(errors.get("price").map(|v| v.len()), Some(2));
            assert_eq!(request_id, Some("req-456".to_string()));
        } else {
            panic!("Expected ValidationFailed variant");
        }
    }

    #[test]
    fn test_from_http_response_maps_other_codes_to_http() {
        let error = ResourceError::from_http_response(
            500,
            &json!({"error": "Internal error"}),
            "Product",
            None,
            Some("req-789"),
        );

        assert!(matches!(error, ResourceError::Http(_)));
    }

    #[test]
    fn test_parse_validation_errors_object_format() {
        let body = json!({
            "errors": {
                "title": ["can't be blank"],
                "tags": ["is invalid", "has too many items"]
            }
        });

        let errors = parse_validation_errors(&body);
        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors.get("title"),
            Some(&vec!["can't be blank".to_string()])
        );
        assert_eq!(errors.get("tags").map(|v| v.len()), Some(2));
    }

    #[test]
    fn test_parse_validation_errors_array_format() {
        let body = json!({
            "errors": ["Error 1", "Error 2"]
        });

        let errors = parse_validation_errors(&body);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors.get("base").map(|v| v.len()), Some(2));
    }

    #[test]
    fn test_request_id_extraction() {
        let error = ResourceError::ValidationFailed {
            errors: HashMap::new(),
            request_id: Some("req-abc".to_string()),
        };
        assert_eq!(error.request_id(), Some("req-abc"));

        let error = ResourceError::NotFound {
            resource: "Product",
            id: "123".to_string(),
        };
        assert_eq!(error.request_id(), None);
    }
}

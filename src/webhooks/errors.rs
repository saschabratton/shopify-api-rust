//! Webhook-specific error types for the Shopify API SDK.
//!
//! This module contains error types for webhook registration and verification operations.
//!
//! # Error Handling
//!
//! The SDK uses specific error types for different webhook failure scenarios:
//!
//! - [`WebhookError::HostNotConfigured`]: When `config.host()` is `None`
//! - [`WebhookError::RegistrationNotFound`]: When a topic is not in the local registry
//! - [`WebhookError::GraphqlError`]: Wrapped GraphQL errors
//! - [`WebhookError::ShopifyError`]: For userErrors in GraphQL responses
//! - [`WebhookError::InvalidHmac`]: When webhook signature verification fails
//!
//! # Example
//!
//! ```rust
//! use shopify_api::webhooks::WebhookError;
//! use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
//!
//! let error = WebhookError::RegistrationNotFound {
//!     topic: WebhookTopic::OrdersCreate,
//! };
//! println!("Error: {}", error);
//! ```

use crate::clients::GraphqlError;
use crate::rest::resources::v2025_10::common::WebhookTopic;
use thiserror::Error;

/// Error type for webhook registration and verification operations.
///
/// This enum provides error types for webhook operations, including
/// host configuration errors, registration lookup failures, signature
/// verification failures, and wrapped GraphQL errors.
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::WebhookError;
/// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
///
/// // Create a registration not found error
/// let error = WebhookError::RegistrationNotFound {
///     topic: WebhookTopic::OrdersCreate,
/// };
/// assert!(error.to_string().contains("not found"));
/// ```
#[derive(Debug, Error)]
pub enum WebhookError {
    /// Host URL is not configured in ShopifyConfig.
    ///
    /// This error occurs when attempting to register webhooks but
    /// `config.host()` returns `None`. The host URL is required to
    /// construct callback URLs for webhook subscriptions.
    #[error("Host URL is not configured. Please set host in ShopifyConfig to register webhooks.")]
    HostNotConfigured,

    /// Webhook registration not found in the local registry.
    ///
    /// This error occurs when attempting to register a webhook topic
    /// that hasn't been added to the registry via `add_registration()`.
    #[error("Webhook registration not found for topic: {topic:?}")]
    RegistrationNotFound {
        /// The webhook topic that was not found.
        topic: WebhookTopic,
    },

    /// An underlying GraphQL error occurred.
    ///
    /// This variant wraps [`GraphqlError`] for unified error handling.
    #[error(transparent)]
    GraphqlError(#[from] GraphqlError),

    /// A Shopify API error occurred (from userErrors in GraphQL response).
    ///
    /// This error is returned when the GraphQL mutation succeeds (HTTP 200)
    /// but Shopify returns userErrors in the response body.
    #[error("Shopify API error: {message}")]
    ShopifyError {
        /// The error message from Shopify.
        message: String,
    },

    /// Webhook subscription not found in Shopify.
    ///
    /// This error occurs when attempting to unregister a webhook that
    /// doesn't exist in Shopify for the given topic.
    #[error("Webhook subscription not found in Shopify for topic: {topic:?}")]
    SubscriptionNotFound {
        /// The webhook topic that was not found.
        topic: WebhookTopic,
    },

    /// Webhook signature verification failed.
    ///
    /// This error occurs when the HMAC signature in the webhook request
    /// does not match the expected signature computed from the request body.
    /// The error message is intentionally generic to avoid leaking security details.
    #[error("Webhook signature verification failed")]
    InvalidHmac,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::{HttpError, HttpResponseError};

    #[test]
    fn test_host_not_configured_error_message() {
        let error = WebhookError::HostNotConfigured;
        let message = error.to_string();
        assert!(message.contains("Host URL is not configured"));
        assert!(message.contains("ShopifyConfig"));
    }

    #[test]
    fn test_registration_not_found_error_message() {
        let error = WebhookError::RegistrationNotFound {
            topic: WebhookTopic::OrdersCreate,
        };
        let message = error.to_string();
        assert!(message.contains("not found"));
        assert!(message.contains("OrdersCreate"));
    }

    #[test]
    fn test_shopify_error_message() {
        let error = WebhookError::ShopifyError {
            message: "Invalid callback URL".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("Shopify API error"));
        assert!(message.contains("Invalid callback URL"));
    }

    #[test]
    fn test_from_graphql_error_conversion() {
        let http_error = HttpError::Response(HttpResponseError {
            code: 401,
            message: r#"{"error":"Unauthorized"}"#.to_string(),
            error_reference: None,
        });
        let graphql_error = GraphqlError::Http(http_error);

        // Test From<GraphqlError> conversion
        let webhook_error: WebhookError = graphql_error.into();

        assert!(matches!(webhook_error, WebhookError::GraphqlError(_)));
        assert!(webhook_error.to_string().contains("Unauthorized"));
    }

    #[test]
    fn test_all_error_variants_implement_std_error() {
        // HostNotConfigured
        let error: &dyn std::error::Error = &WebhookError::HostNotConfigured;
        let _ = error;

        // RegistrationNotFound
        let error: &dyn std::error::Error = &WebhookError::RegistrationNotFound {
            topic: WebhookTopic::OrdersCreate,
        };
        let _ = error;

        // ShopifyError
        let error: &dyn std::error::Error = &WebhookError::ShopifyError {
            message: "test".to_string(),
        };
        let _ = error;

        // GraphqlError
        let http_error = HttpError::Response(HttpResponseError {
            code: 400,
            message: "test".to_string(),
            error_reference: None,
        });
        let error: &dyn std::error::Error =
            &WebhookError::GraphqlError(GraphqlError::Http(http_error));
        let _ = error;

        // InvalidHmac
        let error: &dyn std::error::Error = &WebhookError::InvalidHmac;
        let _ = error;
    }

    #[test]
    fn test_subscription_not_found_error_message() {
        let error = WebhookError::SubscriptionNotFound {
            topic: WebhookTopic::ProductsUpdate,
        };
        let message = error.to_string();
        assert!(message.contains("not found in Shopify"));
        assert!(message.contains("ProductsUpdate"));
    }

    #[test]
    fn test_invalid_hmac_error_message() {
        let error = WebhookError::InvalidHmac;
        let message = error.to_string();
        assert_eq!(message, "Webhook signature verification failed");
        // Ensure the message is generic and doesn't leak security details
        assert!(!message.contains("key"));
        assert!(!message.contains("secret"));
    }
}

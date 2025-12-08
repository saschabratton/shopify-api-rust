//! Webhook registration types for the Shopify API SDK.
//!
//! This module contains types for configuring webhook registrations,
//! including the registration struct, builder, and result types.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::webhooks::{WebhookRegistration, WebhookRegistrationBuilder};
//! use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
//!
//! // Create a registration using the builder
//! let registration = WebhookRegistrationBuilder::new(
//!     WebhookTopic::OrdersCreate,
//!     "/webhooks/orders/create".to_string(),
//! )
//! .include_fields(vec!["id".to_string(), "email".to_string()])
//! .filter("status:active".to_string())
//! .build();
//!
//! assert_eq!(registration.topic, WebhookTopic::OrdersCreate);
//! ```

use super::WebhookError;
pub use crate::rest::resources::v2025_10::common::WebhookTopic;

/// Represents a webhook registration configuration.
///
/// This struct holds the configuration for a webhook subscription,
/// including the topic, callback path, and optional filtering options.
///
/// # Fields
///
/// - `topic`: The webhook topic to subscribe to
/// - `path`: The path portion of the callback URL (combined with `config.host()`)
/// - `include_fields`: Optional list of fields to include in the webhook payload
/// - `metafield_namespaces`: Optional list of metafield namespaces to include
/// - `filter`: Optional filter string (e.g., "status:active")
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::{WebhookRegistration, WebhookRegistrationBuilder};
/// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
///
/// let registration = WebhookRegistrationBuilder::new(
///     WebhookTopic::OrdersCreate,
///     "/webhooks/orders".to_string(),
/// )
/// .build();
///
/// assert_eq!(registration.path, "/webhooks/orders");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebhookRegistration {
    /// The webhook topic to subscribe to.
    pub topic: WebhookTopic,

    /// The path portion of the callback URL.
    ///
    /// This is combined with `config.host()` to form the full callback URL.
    /// Example: "/webhooks/orders/create"
    pub path: String,

    /// Optional list of fields to include in the webhook payload.
    ///
    /// When specified, only these fields will be included in the webhook payload.
    pub include_fields: Option<Vec<String>>,

    /// Optional list of metafield namespaces to include in the webhook payload.
    pub metafield_namespaces: Option<Vec<String>>,

    /// Optional filter string for the webhook subscription.
    ///
    /// Example: "status:active"
    pub filter: Option<String>,
}

/// Builder for constructing [`WebhookRegistration`] instances.
///
/// This builder provides a fluent API for configuring webhook registrations.
/// Required fields (`topic` and `path`) are set via the constructor, while
/// optional fields can be set using method chaining.
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::WebhookRegistrationBuilder;
/// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
///
/// let registration = WebhookRegistrationBuilder::new(
///     WebhookTopic::ProductsUpdate,
///     "/api/webhooks/products".to_string(),
/// )
/// .include_fields(vec!["id".to_string(), "title".to_string()])
/// .metafield_namespaces(vec!["custom".to_string()])
/// .filter("vendor:MyVendor".to_string())
/// .build();
///
/// assert!(registration.include_fields.is_some());
/// assert!(registration.filter.is_some());
/// ```
#[derive(Debug)]
pub struct WebhookRegistrationBuilder {
    topic: WebhookTopic,
    path: String,
    include_fields: Option<Vec<String>>,
    metafield_namespaces: Option<Vec<String>>,
    filter: Option<String>,
}

impl WebhookRegistrationBuilder {
    /// Creates a new builder with the required fields.
    ///
    /// # Arguments
    ///
    /// * `topic` - The webhook topic to subscribe to
    /// * `path` - The path portion of the callback URL
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::WebhookRegistrationBuilder;
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let builder = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::OrdersCreate,
    ///     "/webhooks/orders".to_string(),
    /// );
    /// ```
    #[must_use]
    pub fn new(topic: WebhookTopic, path: String) -> Self {
        Self {
            topic,
            path,
            include_fields: None,
            metafield_namespaces: None,
            filter: None,
        }
    }

    /// Sets the fields to include in the webhook payload.
    ///
    /// When specified, only these fields will be included in the webhook payload.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::WebhookRegistrationBuilder;
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::OrdersCreate,
    ///     "/webhooks".to_string(),
    /// )
    /// .include_fields(vec!["id".to_string(), "email".to_string()])
    /// .build();
    ///
    /// assert_eq!(
    ///     registration.include_fields,
    ///     Some(vec!["id".to_string(), "email".to_string()])
    /// );
    /// ```
    #[must_use]
    pub fn include_fields(mut self, fields: Vec<String>) -> Self {
        self.include_fields = Some(fields);
        self
    }

    /// Sets the metafield namespaces to include in the webhook payload.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::WebhookRegistrationBuilder;
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::ProductsUpdate,
    ///     "/webhooks".to_string(),
    /// )
    /// .metafield_namespaces(vec!["custom".to_string(), "app".to_string()])
    /// .build();
    ///
    /// assert!(registration.metafield_namespaces.is_some());
    /// ```
    #[must_use]
    pub fn metafield_namespaces(mut self, namespaces: Vec<String>) -> Self {
        self.metafield_namespaces = Some(namespaces);
        self
    }

    /// Sets the filter string for the webhook subscription.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::WebhookRegistrationBuilder;
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::OrdersCreate,
    ///     "/webhooks".to_string(),
    /// )
    /// .filter("status:active".to_string())
    /// .build();
    ///
    /// assert_eq!(registration.filter, Some("status:active".to_string()));
    /// ```
    #[must_use]
    pub fn filter(mut self, filter: String) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Builds the [`WebhookRegistration`].
    ///
    /// This method is infallible since required fields are set in the constructor.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::WebhookRegistrationBuilder;
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::CustomersCreate,
    ///     "/webhooks/customers".to_string(),
    /// )
    /// .build();
    ///
    /// assert_eq!(registration.topic, WebhookTopic::CustomersCreate);
    /// ```
    #[must_use]
    pub fn build(self) -> WebhookRegistration {
        WebhookRegistration {
            topic: self.topic,
            path: self.path,
            include_fields: self.include_fields,
            metafield_namespaces: self.metafield_namespaces,
            filter: self.filter,
        }
    }
}

/// Result of a webhook registration operation.
///
/// This enum represents the outcome of attempting to register a webhook
/// with Shopify. It indicates whether the webhook was created, updated,
/// already existed, or failed.
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::WebhookRegistrationResult;
///
/// let result = WebhookRegistrationResult::Created {
///     id: "gid://shopify/WebhookSubscription/12345".to_string(),
/// };
///
/// match result {
///     WebhookRegistrationResult::Created { id } => {
///         println!("Created webhook with ID: {}", id);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug)]
pub enum WebhookRegistrationResult {
    /// Webhook subscription was created in Shopify.
    Created {
        /// The Shopify webhook subscription ID.
        id: String,
    },

    /// Webhook subscription was updated in Shopify.
    Updated {
        /// The Shopify webhook subscription ID.
        id: String,
    },

    /// Webhook subscription already exists and matches the desired configuration.
    AlreadyRegistered {
        /// The Shopify webhook subscription ID.
        id: String,
    },

    /// Webhook registration failed.
    Failed(WebhookError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_registration_builder_required_fields() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            "/webhooks/orders".to_string(),
        )
        .build();

        assert_eq!(registration.topic, WebhookTopic::OrdersCreate);
        assert_eq!(registration.path, "/webhooks/orders");
        assert!(registration.include_fields.is_none());
        assert!(registration.metafield_namespaces.is_none());
        assert!(registration.filter.is_none());
    }

    #[test]
    fn test_webhook_registration_builder_optional_fields() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::ProductsUpdate,
            "/webhooks/products".to_string(),
        )
        .include_fields(vec!["id".to_string(), "title".to_string()])
        .metafield_namespaces(vec!["custom".to_string()])
        .filter("vendor:Test".to_string())
        .build();

        assert_eq!(
            registration.include_fields,
            Some(vec!["id".to_string(), "title".to_string()])
        );
        assert_eq!(
            registration.metafield_namespaces,
            Some(vec!["custom".to_string()])
        );
        assert_eq!(registration.filter, Some("vendor:Test".to_string()));
    }

    #[test]
    fn test_webhook_registration_result_created_variant() {
        let result = WebhookRegistrationResult::Created {
            id: "gid://shopify/WebhookSubscription/123".to_string(),
        };

        match result {
            WebhookRegistrationResult::Created { id } => {
                assert_eq!(id, "gid://shopify/WebhookSubscription/123");
            }
            _ => panic!("Expected Created variant"),
        }
    }

    #[test]
    fn test_webhook_registration_result_updated_variant() {
        let result = WebhookRegistrationResult::Updated {
            id: "gid://shopify/WebhookSubscription/456".to_string(),
        };

        match result {
            WebhookRegistrationResult::Updated { id } => {
                assert_eq!(id, "gid://shopify/WebhookSubscription/456");
            }
            _ => panic!("Expected Updated variant"),
        }
    }

    #[test]
    fn test_webhook_registration_result_already_registered_variant() {
        let result = WebhookRegistrationResult::AlreadyRegistered {
            id: "gid://shopify/WebhookSubscription/789".to_string(),
        };

        match result {
            WebhookRegistrationResult::AlreadyRegistered { id } => {
                assert_eq!(id, "gid://shopify/WebhookSubscription/789");
            }
            _ => panic!("Expected AlreadyRegistered variant"),
        }
    }

    #[test]
    fn test_webhook_registration_result_failed_variant() {
        let result = WebhookRegistrationResult::Failed(WebhookError::HostNotConfigured);

        match result {
            WebhookRegistrationResult::Failed(error) => {
                assert!(matches!(error, WebhookError::HostNotConfigured));
            }
            _ => panic!("Expected Failed variant"),
        }
    }

    #[test]
    fn test_webhook_registration_derives_clone() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            "/webhooks".to_string(),
        )
        .build();

        let cloned = registration.clone();
        assert_eq!(registration, cloned);
    }

    #[test]
    fn test_webhook_registration_derives_debug() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            "/webhooks".to_string(),
        )
        .build();

        let debug_str = format!("{:?}", registration);
        assert!(debug_str.contains("WebhookRegistration"));
        assert!(debug_str.contains("OrdersCreate"));
    }

    #[test]
    fn test_webhook_registration_derives_partial_eq() {
        let reg1 = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            "/webhooks".to_string(),
        )
        .build();

        let reg2 = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            "/webhooks".to_string(),
        )
        .build();

        let reg3 = WebhookRegistrationBuilder::new(
            WebhookTopic::ProductsCreate,
            "/webhooks".to_string(),
        )
        .build();

        assert_eq!(reg1, reg2);
        assert_ne!(reg1, reg3);
    }
}

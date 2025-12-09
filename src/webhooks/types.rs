//! Webhook registration types for the Shopify API SDK.
//!
//! This module contains types for configuring webhook registrations,
//! including the registration struct, builder, handler trait, and result types.
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

use std::fmt;
use std::future::Future;
use std::pin::Pin;

use super::verification::WebhookContext;
use super::WebhookError;
pub use crate::rest::resources::v2025_10::common::WebhookTopic;

/// A boxed future that is Send.
///
/// This type alias is used for the return type of [`WebhookHandler::handle`]
/// to allow trait objects (dyn compatibility).
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait for handling incoming webhook requests.
///
/// Implement this trait to define custom webhook handling logic.
/// Handlers are invoked by [`WebhookRegistry::process()`](crate::webhooks::WebhookRegistry::process)
/// after webhook signature verification succeeds.
///
/// # Thread Safety
///
/// Handlers must be `Send + Sync` to allow sharing across async tasks.
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::{WebhookHandler, WebhookContext, WebhookError, BoxFuture};
/// use serde_json::Value;
///
/// struct OrderCreatedHandler;
///
/// impl WebhookHandler for OrderCreatedHandler {
///     fn handle<'a>(
///         &'a self,
///         context: WebhookContext,
///         payload: Value,
///     ) -> BoxFuture<'a, Result<(), WebhookError>> {
///         Box::pin(async move {
///             // Access webhook metadata
///             if let Some(shop) = context.shop_domain() {
///                 println!("Received webhook from: {}", shop);
///             }
///
///             // Process the payload
///             if let Some(order_id) = payload.get("id") {
///                 println!("Order created: {}", order_id);
///             }
///
///             Ok(())
///         })
///     }
/// }
/// ```
pub trait WebhookHandler: Send + Sync {
    /// Handles an incoming webhook request.
    ///
    /// # Arguments
    ///
    /// * `context` - Verified webhook metadata (topic, shop domain, etc.)
    /// * `payload` - The parsed JSON payload from the webhook body
    ///
    /// # Returns
    ///
    /// A boxed future that resolves to `Ok(())` on success, or a `WebhookError` if handling fails.
    fn handle<'a>(
        &'a self,
        context: WebhookContext,
        payload: serde_json::Value,
    ) -> BoxFuture<'a, Result<(), WebhookError>>;
}

/// Represents a webhook registration configuration.
///
/// This struct holds the configuration for a webhook subscription,
/// including the topic, callback path, optional filtering options,
/// and an optional handler for processing incoming webhooks.
///
/// # Fields
///
/// - `topic`: The webhook topic to subscribe to
/// - `path`: The path portion of the callback URL (combined with `config.host()`)
/// - `include_fields`: Optional list of fields to include in the webhook payload
/// - `metafield_namespaces`: Optional list of metafield namespaces to include
/// - `filter`: Optional filter string (e.g., "status:active")
/// - `handler`: Optional handler for processing incoming webhooks
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

    /// Optional handler for processing incoming webhooks.
    ///
    /// When set, the handler will be invoked by [`WebhookRegistry::process()`](crate::webhooks::WebhookRegistry::process)
    /// after webhook signature verification succeeds.
    pub(crate) handler: Option<Box<dyn WebhookHandler>>,
}

// Manual Debug implementation since trait objects don't implement Debug
impl fmt::Debug for WebhookRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebhookRegistration")
            .field("topic", &self.topic)
            .field("path", &self.path)
            .field("include_fields", &self.include_fields)
            .field("metafield_namespaces", &self.metafield_namespaces)
            .field("filter", &self.filter)
            .field(
                "handler",
                &if self.handler.is_some() {
                    "Some(<handler>)"
                } else {
                    "None"
                },
            )
            .finish()
    }
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
pub struct WebhookRegistrationBuilder {
    topic: WebhookTopic,
    path: String,
    include_fields: Option<Vec<String>>,
    metafield_namespaces: Option<Vec<String>>,
    filter: Option<String>,
    handler: Option<Box<dyn WebhookHandler>>,
}

// Manual Debug implementation since trait objects don't implement Debug
impl fmt::Debug for WebhookRegistrationBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebhookRegistrationBuilder")
            .field("topic", &self.topic)
            .field("path", &self.path)
            .field("include_fields", &self.include_fields)
            .field("metafield_namespaces", &self.metafield_namespaces)
            .field("filter", &self.filter)
            .field(
                "handler",
                &if self.handler.is_some() {
                    "Some(<handler>)"
                } else {
                    "None"
                },
            )
            .finish()
    }
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
            handler: None,
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

    /// Sets the handler for processing incoming webhooks.
    ///
    /// The handler will be invoked by [`WebhookRegistry::process()`](crate::webhooks::WebhookRegistry::process)
    /// after webhook signature verification succeeds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::{
    ///     WebhookRegistrationBuilder, WebhookHandler, WebhookContext, WebhookError, BoxFuture
    /// };
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    /// use serde_json::Value;
    ///
    /// struct MyHandler;
    ///
    /// impl WebhookHandler for MyHandler {
    ///     fn handle<'a>(
    ///         &'a self,
    ///         _context: WebhookContext,
    ///         _payload: Value,
    ///     ) -> BoxFuture<'a, Result<(), WebhookError>> {
    ///         Box::pin(async move {
    ///             println!("Webhook received!");
    ///             Ok(())
    ///         })
    ///     }
    /// }
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::OrdersCreate,
    ///     "/webhooks/orders".to_string(),
    /// )
    /// .handler(MyHandler)
    /// .build();
    /// ```
    #[must_use]
    pub fn handler(mut self, handler: impl WebhookHandler + 'static) -> Self {
        self.handler = Some(Box::new(handler));
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
            handler: self.handler,
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
        assert!(registration.handler.is_none());
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

    // ========================================================================
    // Task Group 1 Tests: WebhookHandler Trait
    // ========================================================================

    // Test handler implementation for testing purposes
    struct TestHandler {
        name: String,
    }

    impl WebhookHandler for TestHandler {
        fn handle<'a>(
            &'a self,
            _context: WebhookContext,
            _payload: serde_json::Value,
        ) -> BoxFuture<'a, Result<(), WebhookError>> {
            let name = self.name.clone();
            Box::pin(async move {
                println!("TestHandler {} invoked", name);
                Ok(())
            })
        }
    }

    #[test]
    fn test_webhook_handler_trait_can_be_implemented_on_struct() {
        // This test verifies that WebhookHandler trait can be implemented
        let handler = TestHandler {
            name: "test".to_string(),
        };

        // Verify the handler exists and is the right type
        let _: &dyn WebhookHandler = &handler;
    }

    #[test]
    fn test_handler_with_send_sync_bounds_compiles() {
        // This test verifies that handlers with Send + Sync bounds compile correctly
        fn assert_send_sync<T: Send + Sync>() {}

        // TestHandler should satisfy Send + Sync bounds
        assert_send_sync::<TestHandler>();

        // Box<dyn WebhookHandler> should also be Send + Sync
        let handler = TestHandler {
            name: "test".to_string(),
        };
        let boxed: Box<dyn WebhookHandler> = Box::new(handler);
        let _ = boxed;
    }

    // ========================================================================
    // Task Group 2 Tests: Builder Handler Functionality
    // ========================================================================

    #[test]
    fn test_builder_handler_method_accepts_webhook_handler() {
        let handler = TestHandler {
            name: "order_handler".to_string(),
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            "/webhooks/orders".to_string(),
        )
        .handler(handler)
        .build();

        assert!(registration.handler.is_some());
    }

    #[test]
    fn test_builder_without_handler_produces_none() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            "/webhooks/orders".to_string(),
        )
        .build();

        assert!(registration.handler.is_none());
    }

    #[test]
    fn test_builder_with_handler_produces_some() {
        let handler = TestHandler {
            name: "test".to_string(),
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::ProductsUpdate,
            "/webhooks/products".to_string(),
        )
        .handler(handler)
        .build();

        assert!(registration.handler.is_some());
    }

    #[test]
    fn test_handler_is_properly_boxed_as_trait_object() {
        let handler = TestHandler {
            name: "boxed_handler".to_string(),
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::CustomersCreate,
            "/webhooks/customers".to_string(),
        )
        .handler(handler)
        .build();

        // Verify handler is boxed as trait object
        let boxed_handler: &Box<dyn WebhookHandler> = registration.handler.as_ref().unwrap();
        let _ = boxed_handler;
    }
}

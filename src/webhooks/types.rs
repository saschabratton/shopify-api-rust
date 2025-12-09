//! Webhook registration types for the Shopify API SDK.
//!
//! This module contains types for configuring webhook registrations,
//! including the registration struct, builder, handler trait, and result types.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::webhooks::{WebhookRegistration, WebhookRegistrationBuilder, WebhookDeliveryMethod};
//! use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
//!
//! // Create a registration using the builder with HTTP delivery
//! let registration = WebhookRegistrationBuilder::new(
//!     WebhookTopic::OrdersCreate,
//!     WebhookDeliveryMethod::Http {
//!         uri: "https://example.com/webhooks/orders/create".to_string(),
//!     },
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

/// Represents the delivery method for a webhook subscription.
///
/// Webhooks can be delivered via HTTP callback, Amazon EventBridge, or Google Cloud Pub/Sub.
///
/// # Variants
///
/// - [`Http`](Self::Http): Delivers webhooks to an HTTP/HTTPS endpoint
/// - [`EventBridge`](Self::EventBridge): Delivers webhooks to Amazon EventBridge
/// - [`PubSub`](Self::PubSub): Delivers webhooks to Google Cloud Pub/Sub
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::WebhookDeliveryMethod;
///
/// // HTTP delivery
/// let http = WebhookDeliveryMethod::Http {
///     uri: "https://example.com/webhooks".to_string(),
/// };
///
/// // Amazon EventBridge delivery
/// let eventbridge = WebhookDeliveryMethod::EventBridge {
///     arn: "arn:aws:events:us-east-1::event-source/aws.partner/shopify.com/12345/my-source".to_string(),
/// };
///
/// // Google Cloud Pub/Sub delivery
/// let pubsub = WebhookDeliveryMethod::PubSub {
///     project_id: "my-project".to_string(),
///     topic_id: "shopify-webhooks".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum WebhookDeliveryMethod {
    /// HTTP/HTTPS webhook delivery.
    ///
    /// Webhooks are delivered via HTTP POST to the specified URI.
    Http {
        /// The full HTTPS URI for webhook delivery.
        uri: String,
    },

    /// Amazon EventBridge webhook delivery.
    ///
    /// Webhooks are delivered to an Amazon EventBridge event source.
    /// The ARN format is typically:
    /// `arn:aws:events:{region}::event-source/aws.partner/shopify.com/{org}/{event_source}`
    EventBridge {
        /// The Amazon Resource Name (ARN) for the EventBridge event source.
        arn: String,
    },

    /// Google Cloud Pub/Sub webhook delivery.
    ///
    /// Webhooks are delivered to a Google Cloud Pub/Sub topic.
    PubSub {
        /// The Google Cloud project ID.
        project_id: String,
        /// The Pub/Sub topic ID.
        topic_id: String,
    },
}

/// Represents a webhook registration configuration.
///
/// This struct holds the configuration for a webhook subscription,
/// including the topic, delivery method, optional filtering options,
/// and an optional handler for processing incoming webhooks.
///
/// # Fields
///
/// - `topic`: The webhook topic to subscribe to
/// - `delivery_method`: How the webhook should be delivered (HTTP, EventBridge, or Pub/Sub)
/// - `include_fields`: Optional list of fields to include in the webhook payload
/// - `metafield_namespaces`: Optional list of metafield namespaces to include
/// - `filter`: Optional filter string (e.g., "status:active")
/// - `handler`: Optional handler for processing incoming webhooks
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::{WebhookRegistration, WebhookRegistrationBuilder, WebhookDeliveryMethod};
/// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
///
/// let registration = WebhookRegistrationBuilder::new(
///     WebhookTopic::OrdersCreate,
///     WebhookDeliveryMethod::Http {
///         uri: "https://example.com/webhooks/orders".to_string(),
///     },
/// )
/// .build();
///
/// assert!(matches!(
///     registration.delivery_method,
///     WebhookDeliveryMethod::Http { .. }
/// ));
/// ```
pub struct WebhookRegistration {
    /// The webhook topic to subscribe to.
    pub topic: WebhookTopic,

    /// The delivery method for the webhook.
    ///
    /// Determines how webhooks are delivered: via HTTP callback,
    /// Amazon EventBridge, or Google Cloud Pub/Sub.
    pub delivery_method: WebhookDeliveryMethod,

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
            .field("delivery_method", &self.delivery_method)
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
/// Required fields (`topic` and `delivery_method`) are set via the constructor, while
/// optional fields can be set using method chaining.
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::{WebhookRegistrationBuilder, WebhookDeliveryMethod};
/// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
///
/// let registration = WebhookRegistrationBuilder::new(
///     WebhookTopic::ProductsUpdate,
///     WebhookDeliveryMethod::Http {
///         uri: "https://example.com/api/webhooks/products".to_string(),
///     },
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
    delivery_method: WebhookDeliveryMethod,
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
            .field("delivery_method", &self.delivery_method)
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
    /// * `delivery_method` - The delivery method for the webhook
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::{WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let builder = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::OrdersCreate,
    ///     WebhookDeliveryMethod::Http {
    ///         uri: "https://example.com/webhooks/orders".to_string(),
    ///     },
    /// );
    /// ```
    #[must_use]
    pub fn new(topic: WebhookTopic, delivery_method: WebhookDeliveryMethod) -> Self {
        Self {
            topic,
            delivery_method,
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
    /// use shopify_api::webhooks::{WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::OrdersCreate,
    ///     WebhookDeliveryMethod::Http {
    ///         uri: "https://example.com/webhooks".to_string(),
    ///     },
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
    /// use shopify_api::webhooks::{WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::ProductsUpdate,
    ///     WebhookDeliveryMethod::Http {
    ///         uri: "https://example.com/webhooks".to_string(),
    ///     },
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
    /// use shopify_api::webhooks::{WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::OrdersCreate,
    ///     WebhookDeliveryMethod::Http {
    ///         uri: "https://example.com/webhooks".to_string(),
    ///     },
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
    ///     WebhookRegistrationBuilder, WebhookHandler, WebhookContext, WebhookError, BoxFuture,
    ///     WebhookDeliveryMethod
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
    ///     WebhookDeliveryMethod::Http {
    ///         uri: "https://example.com/webhooks/orders".to_string(),
    ///     },
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
    /// use shopify_api::webhooks::{WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registration = WebhookRegistrationBuilder::new(
    ///     WebhookTopic::CustomersCreate,
    ///     WebhookDeliveryMethod::Http {
    ///         uri: "https://example.com/webhooks/customers".to_string(),
    ///     },
    /// )
    /// .build();
    ///
    /// assert_eq!(registration.topic, WebhookTopic::CustomersCreate);
    /// ```
    #[must_use]
    pub fn build(self) -> WebhookRegistration {
        WebhookRegistration {
            topic: self.topic,
            delivery_method: self.delivery_method,
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

    // ========================================================================
    // Task Group 1 Tests: WebhookDeliveryMethod Enum
    // ========================================================================

    #[test]
    fn test_delivery_method_http_variant_construction() {
        let method = WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks".to_string(),
        };

        match method {
            WebhookDeliveryMethod::Http { uri } => {
                assert_eq!(uri, "https://example.com/webhooks");
            }
            _ => panic!("Expected Http variant"),
        }
    }

    #[test]
    fn test_delivery_method_eventbridge_variant_construction() {
        let method = WebhookDeliveryMethod::EventBridge {
            arn: "arn:aws:events:us-east-1::event-source/aws.partner/shopify.com/12345/source"
                .to_string(),
        };

        match method {
            WebhookDeliveryMethod::EventBridge { arn } => {
                assert!(arn.starts_with("arn:aws:events"));
            }
            _ => panic!("Expected EventBridge variant"),
        }
    }

    #[test]
    fn test_delivery_method_pubsub_variant_construction() {
        let method = WebhookDeliveryMethod::PubSub {
            project_id: "my-project".to_string(),
            topic_id: "shopify-webhooks".to_string(),
        };

        match method {
            WebhookDeliveryMethod::PubSub {
                project_id,
                topic_id,
            } => {
                assert_eq!(project_id, "my-project");
                assert_eq!(topic_id, "shopify-webhooks");
            }
            _ => panic!("Expected PubSub variant"),
        }
    }

    #[test]
    fn test_delivery_method_derives_debug() {
        let http = WebhookDeliveryMethod::Http {
            uri: "https://example.com".to_string(),
        };
        let debug_str = format!("{:?}", http);
        assert!(debug_str.contains("Http"));
        assert!(debug_str.contains("uri"));

        let eventbridge = WebhookDeliveryMethod::EventBridge {
            arn: "arn:test".to_string(),
        };
        let debug_str = format!("{:?}", eventbridge);
        assert!(debug_str.contains("EventBridge"));
        assert!(debug_str.contains("arn"));

        let pubsub = WebhookDeliveryMethod::PubSub {
            project_id: "proj".to_string(),
            topic_id: "topic".to_string(),
        };
        let debug_str = format!("{:?}", pubsub);
        assert!(debug_str.contains("PubSub"));
        assert!(debug_str.contains("project_id"));
        assert!(debug_str.contains("topic_id"));
    }

    #[test]
    fn test_delivery_method_derives_clone() {
        let original = WebhookDeliveryMethod::Http {
            uri: "https://example.com".to_string(),
        };
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_delivery_method_derives_partial_eq() {
        let http1 = WebhookDeliveryMethod::Http {
            uri: "https://example.com".to_string(),
        };
        let http2 = WebhookDeliveryMethod::Http {
            uri: "https://example.com".to_string(),
        };
        let http3 = WebhookDeliveryMethod::Http {
            uri: "https://different.com".to_string(),
        };

        assert_eq!(http1, http2);
        assert_ne!(http1, http3);

        let eventbridge = WebhookDeliveryMethod::EventBridge {
            arn: "arn:test".to_string(),
        };
        assert_ne!(http1, eventbridge);
    }

    #[test]
    fn test_delivery_method_equality_across_variants() {
        let http = WebhookDeliveryMethod::Http {
            uri: "https://example.com".to_string(),
        };
        let eventbridge = WebhookDeliveryMethod::EventBridge {
            arn: "arn:test".to_string(),
        };
        let pubsub = WebhookDeliveryMethod::PubSub {
            project_id: "proj".to_string(),
            topic_id: "topic".to_string(),
        };

        // Different variants should never be equal
        assert_ne!(http, eventbridge);
        assert_ne!(http, pubsub);
        assert_ne!(eventbridge, pubsub);
    }

    // ========================================================================
    // Task Group 2 Tests: WebhookRegistration Modification
    // ========================================================================

    #[test]
    fn test_webhook_registration_with_http_delivery_method() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks/orders".to_string(),
            },
        )
        .build();

        assert_eq!(registration.topic, WebhookTopic::OrdersCreate);
        match registration.delivery_method {
            WebhookDeliveryMethod::Http { uri } => {
                assert_eq!(uri, "https://example.com/webhooks/orders");
            }
            _ => panic!("Expected Http delivery method"),
        }
    }

    #[test]
    fn test_webhook_registration_with_eventbridge_delivery_method() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::ProductsUpdate,
            WebhookDeliveryMethod::EventBridge {
                arn: "arn:aws:events:us-east-1::event-source/aws.partner/shopify.com/123/src"
                    .to_string(),
            },
        )
        .build();

        assert_eq!(registration.topic, WebhookTopic::ProductsUpdate);
        assert!(matches!(
            registration.delivery_method,
            WebhookDeliveryMethod::EventBridge { .. }
        ));
    }

    #[test]
    fn test_webhook_registration_with_pubsub_delivery_method() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::CustomersCreate,
            WebhookDeliveryMethod::PubSub {
                project_id: "my-gcp-project".to_string(),
                topic_id: "shopify-events".to_string(),
            },
        )
        .build();

        assert_eq!(registration.topic, WebhookTopic::CustomersCreate);
        match registration.delivery_method {
            WebhookDeliveryMethod::PubSub {
                project_id,
                topic_id,
            } => {
                assert_eq!(project_id, "my-gcp-project");
                assert_eq!(topic_id, "shopify-events");
            }
            _ => panic!("Expected PubSub delivery method"),
        }
    }

    #[test]
    fn test_webhook_registration_debug_includes_delivery_method() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
        )
        .build();

        let debug_str = format!("{:?}", registration);
        assert!(debug_str.contains("WebhookRegistration"));
        assert!(debug_str.contains("delivery_method"));
        assert!(debug_str.contains("Http"));
    }

    #[test]
    fn test_webhook_registration_optional_fields_still_work() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::ProductsUpdate,
            WebhookDeliveryMethod::EventBridge {
                arn: "arn:test".to_string(),
            },
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

    // ========================================================================
    // Task Group 3 Tests: WebhookRegistrationBuilder Modification
    // ========================================================================

    #[test]
    fn test_builder_new_with_http_delivery_method() {
        let builder = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
        );
        let registration = builder.build();

        assert_eq!(registration.topic, WebhookTopic::OrdersCreate);
        assert!(matches!(
            registration.delivery_method,
            WebhookDeliveryMethod::Http { .. }
        ));
    }

    #[test]
    fn test_builder_new_with_eventbridge_delivery_method() {
        let builder = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::EventBridge {
                arn: "arn:aws:events:test".to_string(),
            },
        );
        let registration = builder.build();

        assert!(matches!(
            registration.delivery_method,
            WebhookDeliveryMethod::EventBridge { .. }
        ));
    }

    #[test]
    fn test_builder_new_with_pubsub_delivery_method() {
        let builder = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::PubSub {
                project_id: "project".to_string(),
                topic_id: "topic".to_string(),
            },
        );
        let registration = builder.build();

        assert!(matches!(
            registration.delivery_method,
            WebhookDeliveryMethod::PubSub { .. }
        ));
    }

    #[test]
    fn test_builder_method_chaining_still_works() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com".to_string(),
            },
        )
        .include_fields(vec!["id".to_string()])
        .metafield_namespaces(vec!["ns".to_string()])
        .filter("active".to_string())
        .build();

        assert!(registration.include_fields.is_some());
        assert!(registration.metafield_namespaces.is_some());
        assert!(registration.filter.is_some());
    }

    #[test]
    fn test_builder_build_produces_correct_registration() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::ProductsCreate,
            WebhookDeliveryMethod::PubSub {
                project_id: "my-project".to_string(),
                topic_id: "my-topic".to_string(),
            },
        )
        .include_fields(vec!["id".to_string(), "title".to_string()])
        .build();

        assert_eq!(registration.topic, WebhookTopic::ProductsCreate);
        match registration.delivery_method {
            WebhookDeliveryMethod::PubSub {
                project_id,
                topic_id,
            } => {
                assert_eq!(project_id, "my-project");
                assert_eq!(topic_id, "my-topic");
            }
            _ => panic!("Expected PubSub delivery method"),
        }
        assert_eq!(
            registration.include_fields,
            Some(vec!["id".to_string(), "title".to_string()])
        );
        assert!(registration.metafield_namespaces.is_none());
        assert!(registration.filter.is_none());
        assert!(registration.handler.is_none());
    }

    #[test]
    fn test_builder_debug_includes_delivery_method() {
        let builder = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::EventBridge {
                arn: "arn:test".to_string(),
            },
        );

        let debug_str = format!("{:?}", builder);
        assert!(debug_str.contains("WebhookRegistrationBuilder"));
        assert!(debug_str.contains("delivery_method"));
        assert!(debug_str.contains("EventBridge"));
    }

    // ========================================================================
    // Legacy Tests Updated: WebhookRegistrationResult and Handler Tests
    // ========================================================================

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

    // ========================================================================
    // Handler Tests
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
        let handler = TestHandler {
            name: "test".to_string(),
        };

        let _: &dyn WebhookHandler = &handler;
    }

    #[test]
    fn test_handler_with_send_sync_bounds_compiles() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<TestHandler>();

        let handler = TestHandler {
            name: "test".to_string(),
        };
        let boxed: Box<dyn WebhookHandler> = Box::new(handler);
        let _ = boxed;
    }

    #[test]
    fn test_builder_handler_method_accepts_webhook_handler() {
        let handler = TestHandler {
            name: "order_handler".to_string(),
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks/orders".to_string(),
            },
        )
        .handler(handler)
        .build();

        assert!(registration.handler.is_some());
    }

    #[test]
    fn test_builder_without_handler_produces_none() {
        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks/orders".to_string(),
            },
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
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks/products".to_string(),
            },
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
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks/customers".to_string(),
            },
        )
        .handler(handler)
        .build();

        let boxed_handler: &Box<dyn WebhookHandler> = registration.handler.as_ref().unwrap();
        let _ = boxed_handler;
    }
}

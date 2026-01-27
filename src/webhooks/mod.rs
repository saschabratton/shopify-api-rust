//! Webhook registration and verification system for the Shopify API SDK.
//!
//! This module provides an in-memory webhook registration system that allows apps
//! to configure webhook subscriptions locally, then sync them with Shopify via
//! GraphQL API mutations using a two-phase pattern. It also provides HMAC-based
//! signature verification for authenticating incoming webhook requests.
//!
//! # Overview
//!
//! The webhook system consists of:
//!
//! ## Registration
//!
//! - [`WebhookRegistry`]: Stores and manages webhook registrations
//! - [`WebhookRegistration`]: Configuration for a single webhook subscription
//! - [`WebhookRegistrationBuilder`]: Builder for creating registrations
//! - [`WebhookRegistrationResult`]: Result of registration operations
//! - [`WebhookDeliveryMethod`]: Delivery method for webhooks (HTTP, EventBridge, Pub/Sub)
//!
//! ## Handler
//!
//! - [`WebhookHandler`]: Trait for implementing webhook handlers
//! - [`BoxFuture`]: Type alias for boxed futures used in handler returns
//!
//! ## Verification
//!
//! - [`WebhookRequest`]: Incoming webhook request data
//! - [`WebhookContext`]: Verified webhook metadata
//! - [`verify_webhook`]: High-level verification with key rotation support
//! - [`verify_hmac`]: Low-level HMAC verification
//!
//! ## Common Types
//!
//! - [`WebhookError`]: Error types for webhook operations
//! - [`WebhookTopic`]: Re-exported webhook topic enum
//!
//! # Two-Phase Registration Pattern
//!
//! The registry follows a two-phase pattern similar to the Ruby SDK:
//!
//! 1. **Add Registration (Local)**: Configure webhooks at app startup using
//!    [`WebhookRegistry::add_registration`]
//! 2. **Register with Shopify (Remote)**: Sync with Shopify when a valid session
//!    is available using [`WebhookRegistry::register`] or [`WebhookRegistry::register_all`]
//!
//! # Smart Registration
//!
//! The registry performs "smart registration" to minimize API calls:
//! - Queries existing subscriptions from Shopify
//! - Compares configuration to detect changes
//! - Only creates/updates when necessary
//!
//! # Delivery Methods
//!
//! Webhooks can be delivered via three different methods:
//!
//! - **HTTP**: Delivered via HTTP POST to a callback URL
//! - **Amazon EventBridge**: Delivered to an AWS EventBridge event source
//! - **Google Cloud Pub/Sub**: Delivered to a GCP Pub/Sub topic
//!
//! # Webhook Handler Example
//!
//! ```rust
//! use shopify_sdk::webhooks::{
//!     WebhookHandler, WebhookContext, WebhookError, WebhookRegistry,
//!     WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod, BoxFuture
//! };
//! use serde_json::Value;
//!
//! // Define a handler
//! struct OrderHandler;
//!
//! impl WebhookHandler for OrderHandler {
//!     fn handle<'a>(
//!         &'a self,
//!         context: WebhookContext,
//!         payload: Value,
//!     ) -> BoxFuture<'a, Result<(), WebhookError>> {
//!         Box::pin(async move {
//!             println!("Order webhook from: {:?}", context.shop_domain());
//!             Ok(())
//!         })
//!     }
//! }
//!
//! // Register with a handler using HTTP delivery
//! let mut registry = WebhookRegistry::new();
//! registry.add_registration(
//!     WebhookRegistrationBuilder::new(
//!         WebhookTopic::OrdersCreate,
//!         WebhookDeliveryMethod::Http {
//!             uri: "https://example.com/api/webhooks/orders".to_string(),
//!         },
//!     )
//!     .handler(OrderHandler)
//!     .build()
//! );
//! ```
//!
//! # Webhook Verification Example
//!
//! ```rust
//! use shopify_sdk::webhooks::{WebhookRequest, verify_webhook, verify_hmac};
//! use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};
//! use shopify_sdk::auth::oauth::hmac::compute_signature_base64;
//!
//! // Create a config with the API secret
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("test-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("my-secret").unwrap())
//!     .build()
//!     .unwrap();
//!
//! // Simulate an incoming webhook
//! let body = b"webhook payload";
//! let hmac = compute_signature_base64(body, "my-secret");
//!
//! // Create a webhook request
//! let request = WebhookRequest::new(
//!     body.to_vec(),
//!     hmac.clone(),
//!     Some("orders/create".to_string()),
//!     Some("example.myshopify.com".to_string()),
//!     None,
//!     None,
//! );
//!
//! // Verify the webhook (high-level)
//! let context = verify_webhook(&config, &request).expect("verification failed");
//! assert_eq!(context.shop_domain(), Some("example.myshopify.com"));
//!
//! // Or use low-level verification for custom integrations
//! assert!(verify_hmac(body, &hmac, "my-secret"));
//! ```
//!
//! # Registration Examples
//!
//! ## HTTP Delivery
//!
//! ```rust
//! use shopify_sdk::webhooks::{
//!     WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
//! };
//!
//! let mut registry = WebhookRegistry::new();
//!
//! registry.add_registration(
//!     WebhookRegistrationBuilder::new(
//!         WebhookTopic::OrdersCreate,
//!         WebhookDeliveryMethod::Http {
//!             uri: "https://example.com/api/webhooks/orders/create".to_string(),
//!         },
//!     )
//!     .include_fields(vec!["id".to_string(), "email".to_string()])
//!     .build()
//! );
//! ```
//!
//! ## Amazon EventBridge Delivery
//!
//! ```rust
//! use shopify_sdk::webhooks::{
//!     WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
//! };
//!
//! let mut registry = WebhookRegistry::new();
//!
//! registry.add_registration(
//!     WebhookRegistrationBuilder::new(
//!         WebhookTopic::OrdersCreate,
//!         WebhookDeliveryMethod::EventBridge {
//!             arn: "arn:aws:events:us-east-1::event-source/aws.partner/shopify.com/12345/my-source".to_string(),
//!         },
//!     )
//!     .build()
//! );
//! ```
//!
//! ## Google Cloud Pub/Sub Delivery
//!
//! ```rust
//! use shopify_sdk::webhooks::{
//!     WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
//! };
//!
//! let mut registry = WebhookRegistry::new();
//!
//! registry.add_registration(
//!     WebhookRegistrationBuilder::new(
//!         WebhookTopic::ProductsUpdate,
//!         WebhookDeliveryMethod::PubSub {
//!             project_id: "my-gcp-project".to_string(),
//!             topic_id: "shopify-webhooks".to_string(),
//!         },
//!     )
//!     .filter("vendor:MyApp".to_string())
//!     .build()
//! );
//!
//! // Later, when you have a session:
//! // let results = registry.register_all(&session, &config).await?;
//! ```
//!
//! # Error Handling
//!
//! ```rust
//! use shopify_sdk::webhooks::{WebhookError, WebhookTopic};
//!
//! fn handle_error(error: WebhookError) {
//!     match error {
//!         WebhookError::HostNotConfigured => {
//!             println!("Please configure host URL in ShopifyConfig");
//!         }
//!         WebhookError::RegistrationNotFound { topic } => {
//!             println!("Topic {:?} not registered locally", topic);
//!         }
//!         WebhookError::GraphqlError(e) => {
//!             println!("API error: {}", e);
//!         }
//!         WebhookError::ShopifyError { message } => {
//!             println!("Shopify error: {}", message);
//!         }
//!         WebhookError::SubscriptionNotFound { topic } => {
//!             println!("Webhook for {:?} not found in Shopify", topic);
//!         }
//!         WebhookError::InvalidHmac => {
//!             println!("Webhook signature verification failed");
//!         }
//!         WebhookError::NoHandlerForTopic { topic } => {
//!             println!("No handler registered for topic: {}", topic);
//!         }
//!         WebhookError::PayloadParseError { message } => {
//!             println!("Failed to parse webhook payload: {}", message);
//!         }
//!     }
//! }
//! ```
//!
//! # Thread Safety
//!
//! All types in this module are `Send + Sync`, making them safe to share
//! across async tasks.

mod errors;
mod registry;
mod types;
mod verification;

pub use errors::WebhookError;
pub use registry::WebhookRegistry;
pub use types::{
    BoxFuture, WebhookDeliveryMethod, WebhookHandler, WebhookRegistration,
    WebhookRegistrationBuilder, WebhookRegistrationResult,
};

// Verification exports
pub use verification::{
    verify_hmac, verify_webhook, WebhookContext, WebhookRequest, HEADER_API_VERSION, HEADER_HMAC,
    HEADER_SHOP_DOMAIN, HEADER_TOPIC, HEADER_WEBHOOK_ID,
};

// Re-export WebhookTopic for convenience
pub use crate::rest::resources::v2025_10::common::WebhookTopic;

//! Webhook registration system for the Shopify API SDK.
//!
//! This module provides an in-memory webhook registration system that allows apps
//! to configure webhook subscriptions locally, then sync them with Shopify via
//! GraphQL API mutations using a two-phase pattern.
//!
//! # Overview
//!
//! The webhook system consists of:
//!
//! - [`WebhookRegistry`]: Stores and manages webhook registrations
//! - [`WebhookRegistration`]: Configuration for a single webhook subscription
//! - [`WebhookRegistrationBuilder`]: Builder for creating registrations
//! - [`WebhookRegistrationResult`]: Result of registration operations
//! - [`WebhookError`]: Error types for webhook operations
//! - [`WebhookTopic`]: Re-exported webhook topic enum
//!
//! # Two-Phase Pattern
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
//! # Example
//!
//! ```rust
//! use shopify_api::webhooks::{
//!     WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic
//! };
//!
//! // Phase 1: Configure webhooks at startup
//! let mut registry = WebhookRegistry::new();
//!
//! registry
//!     .add_registration(
//!         WebhookRegistrationBuilder::new(
//!             WebhookTopic::OrdersCreate,
//!             "/api/webhooks/orders/create".to_string(),
//!         )
//!         .include_fields(vec!["id".to_string(), "email".to_string()])
//!         .build()
//!     )
//!     .add_registration(
//!         WebhookRegistrationBuilder::new(
//!             WebhookTopic::ProductsUpdate,
//!             "/api/webhooks/products/update".to_string(),
//!         )
//!         .filter("vendor:MyApp".to_string())
//!         .build()
//!     );
//!
//! // Phase 2: Register with Shopify when session is available
//! // let results = registry.register_all(&session, &config).await?;
//! ```
//!
//! # Error Handling
//!
//! ```rust
//! use shopify_api::webhooks::{WebhookError, WebhookTopic};
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

pub use errors::WebhookError;
pub use registry::WebhookRegistry;
pub use types::{WebhookRegistration, WebhookRegistrationBuilder, WebhookRegistrationResult};

// Re-export WebhookTopic for convenience
pub use crate::rest::resources::v2025_10::common::WebhookTopic;

//! REST API client for Shopify Admin API.
//!
//! This module provides a higher-level REST API client built on top of the
//! [`HttpClient`](crate::clients::HttpClient) that offers convenient methods
//! for interacting with Shopify's REST Admin API.
//!
//! # Overview
//!
//! The main types in this module are:
//!
//! - [`RestClient`]: The REST API client with `get()`, `post()`, `put()`, `delete()` methods
//! - [`RestError`]: Error type for REST API operations
//!
//! # Deprecation Notice
//!
//! The Shopify Admin REST API is deprecated. Shopify recommends migrating to the
//! GraphQL Admin API for new development. This client logs a deprecation warning
//! when constructed and can be disabled via configuration.
//!
//! For more information, see:
//! <https://www.shopify.com/ca/partners/blog/all-in-on-graphql>
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::{RestClient, Session, ShopDomain, ShopifyConfig, ApiKey, ApiSecretKey};
//!
//! // Create a session
//! let session = Session::new(
//!     "session-id".to_string(),
//!     ShopDomain::new("my-store").unwrap(),
//!     "access-token".to_string(),
//!     "read_products".parse().unwrap(),
//!     false,
//!     None,
//! );
//!
//! // Create configuration
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
//!     .build()
//!     .unwrap();
//!
//! // Create REST client
//! let client = RestClient::new(&session, Some(&config))?;
//!
//! // Make requests
//! let response = client.get("products", None).await?;
//! println!("Products: {}", response.body);
//! ```
//!
//! # Path Normalization
//!
//! The client normalizes paths following the Ruby SDK conventions:
//!
//! - Leading slashes are stripped: `/products` -> `products`
//! - Trailing `.json` is stripped and re-added: `products.json` -> `products.json`
//! - Paths starting with `admin/` bypass the base path construction
//!
//! # Retry Behavior
//!
//! By default, requests are attempted once (`tries=1`). You can configure
//! automatic retries on 429 (rate limited) and 500 (server error) responses
//! by specifying the `tries` parameter in request methods.

mod client;
mod errors;

pub use client::RestClient;
pub use errors::RestError;

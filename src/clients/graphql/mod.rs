//! GraphQL API client for Shopify Admin API.
//!
//! This module provides a higher-level GraphQL API client built on top of the
//! [`HttpClient`](crate::clients::HttpClient) that offers convenient methods
//! for executing GraphQL queries against Shopify's Admin API.
//!
//! # Overview
//!
//! The main types in this module are:
//!
//! - [`GraphqlClient`]: The GraphQL API client with `query()` and `query_with_debug()` methods
//! - [`GraphqlError`]: Error type for GraphQL API operations
//!
//! # GraphQL is the Recommended API
//!
//! Unlike the deprecated REST Admin API, the GraphQL Admin API is Shopify's
//! recommended approach for new development. This client does not log deprecation
//! warnings.
//!
//! For more information, see:
//! <https://www.shopify.com/ca/partners/blog/all-in-on-graphql>
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::{GraphqlClient, Session, ShopDomain, ShopifyConfig, ApiKey, ApiSecretKey};
//! use serde_json::json;
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
//! // Create GraphQL client
//! let client = GraphqlClient::new(&session, Some(&config));
//!
//! // Simple query
//! let response = client.query("query { shop { name } }", None, None, None).await?;
//! println!("Shop name: {}", response.body["data"]["shop"]["name"]);
//!
//! // Query with variables
//! let response = client.query(
//!     "query GetProduct($id: ID!) { product(id: $id) { title } }",
//!     Some(json!({ "id": "gid://shopify/Product/123" })),
//!     None,
//!     None
//! ).await?;
//!
//! // Check for GraphQL errors in response (HTTP 200 with errors in body)
//! if let Some(errors) = response.body.get("errors") {
//!     println!("GraphQL errors: {}", errors);
//! }
//! ```
//!
//! # Response Structure
//!
//! GraphQL responses contain these fields in the body:
//!
//! - `data`: The query result data
//! - `errors`: Any GraphQL errors (still HTTP 200)
//! - `extensions`: Query cost and debug information
//!
//! # Debug Mode
//!
//! Use [`GraphqlClient::query_with_debug`] to enable debug mode, which appends
//! `?debug=true` to the request and returns additional query cost and execution
//! information in the response's `extensions` field.
//!
//! # Retry Behavior
//!
//! By default, requests are attempted once (`tries=1`). You can configure
//! automatic retries on 429 (rate limited) and 500 (server error) responses
//! by specifying the `tries` parameter in query methods.

mod client;
mod errors;

pub use client::GraphqlClient;
pub use errors::GraphqlError;

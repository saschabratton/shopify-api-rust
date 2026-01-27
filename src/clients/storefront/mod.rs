//! Storefront API client for Shopify.
//!
//! This module provides clients for interacting with the Shopify Storefront API,
//! which is used for building custom storefronts and headless commerce applications.
//!
//! # Overview
//!
//! The main types in this module are:
//!
//! - [`StorefrontClient`]: The GraphQL client for Storefront API operations
//! - [`StorefrontToken`]: Token type for Storefront API authentication
//!
//! # Storefront vs Admin API
//!
//! The Storefront API differs from the Admin API in several key ways:
//!
//! - **Endpoint**: Uses `/api/{version}/graphql.json` (no `/admin` prefix)
//! - **Authentication**: Uses different headers depending on token type:
//!   - Public: `X-Shopify-Storefront-Access-Token`
//!   - Private: `Shopify-Storefront-Private-Token`
//! - **Access Level**: Limited to storefront data (products, collections, cart)
//! - **Tokenless Access**: Supports unauthenticated access for basic features
//!
//! # Token Types
//!
//! The Storefront API supports two types of access tokens:
//!
//! - **Public tokens**: Safe for client-side use, limited access
//! - **Private tokens**: Server-side only, elevated access
//!
//! # Tokenless Access
//!
//! Some Storefront API operations can be performed without authentication:
//! - Product and collection queries
//! - Cart operations
//! - Search queries
//!
//! To use tokenless access, pass `None` as the token parameter.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::{StorefrontClient, StorefrontToken, ShopDomain};
//! use serde_json::json;
//!
//! // Create a shop domain
//! let shop = ShopDomain::new("my-store").unwrap();
//!
//! // With a public token
//! let token = StorefrontToken::Public("public-access-token".to_string());
//! let client = StorefrontClient::new(&shop, Some(token), None);
//!
//! // Query products
//! let response = client.query(
//!     "query { products(first: 10) { edges { node { title } } } }",
//!     None,
//!     None,
//!     None
//! ).await?;
//!
//! // Tokenless access for basic features
//! let client = StorefrontClient::new(&shop, None, None);
//! let response = client.query(
//!     "query { shop { name } }",
//!     None,
//!     None,
//!     None
//! ).await?;
//! ```

mod client;
mod storefront_http;
mod token;

pub use client::StorefrontClient;
pub use token::StorefrontToken;

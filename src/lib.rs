//! # Shopify API Rust SDK
//!
//! A Rust SDK for the Shopify API, providing type-safe configuration
//! and authentication handling for Shopify app development.
//!
//! ## Overview
//!
//! This SDK provides:
//! - Type-safe configuration via [`ShopifyConfig`] and [`ShopifyConfigBuilder`]
//! - Validated newtypes for API credentials and domain values
//! - OAuth scope handling with implied scope support
//! - Session management for authenticated API calls
//!
//! ## Quick Start
//!
//! ```rust
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion, AuthScopes};
//!
//! // Create configuration using the builder pattern
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
//!     .scopes("read_products,write_orders".parse().unwrap())
//!     .api_version(ApiVersion::latest())
//!     .build()
//!     .unwrap();
//! ```
//!
//! ## Design Principles
//!
//! - **No global state**: Configuration is instance-based and passed explicitly
//! - **Fail-fast validation**: All newtypes validate on construction
//! - **Thread-safe**: All types are `Send + Sync`
//! - **Async-first**: Designed for use with Tokio async runtime

pub mod auth;
pub mod config;
pub mod error;

// Re-export public types at crate root for convenience
pub use auth::{AuthScopes, Session};
pub use config::{
    ApiKey, ApiSecretKey, ApiVersion, HostUrl, ShopDomain, ShopifyConfig, ShopifyConfigBuilder,
};
pub use error::ConfigError;

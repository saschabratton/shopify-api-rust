//! Authentication types for the Shopify API SDK.
//!
//! This module provides types for handling OAuth scopes, sessions, and
//! associated user information used in Shopify API authentication.
//!
//! # Overview
//!
//! - [`AuthScopes`]: A set of OAuth scopes with implied scope handling
//! - [`Session`]: Represents an authenticated session for API calls
//! - [`AssociatedUser`]: User information for online (user-specific) sessions
//! - [`oauth`]: OAuth 2.0 authorization code flow implementation
//!
//! # Session Types
//!
//! Shopify supports two types of sessions:
//!
//! - **Offline sessions**: App-level tokens that don't expire and persist across
//!   user sessions. Used for background tasks and webhooks.
//! - **Online sessions**: User-specific tokens that expire and are tied to a
//!   particular user. Include associated user information.
//!
//! # OAuth Flow
//!
//! For apps that need to authenticate with Shopify stores, use the OAuth module:
//!
//! ```rust,ignore
//! use shopify_sdk::auth::oauth::{begin_auth, validate_auth_callback};
//!
//! // 1. Generate authorization URL
//! let result = begin_auth(&config, &shop, "/callback", true, None)?;
//! // Redirect user to result.auth_url
//!
//! // 2. Handle callback and get session
//! let session = validate_auth_callback(&config, &query, &state).await?;
//! ```
//!
//! # Example
//!
//! ```rust
//! use shopify_sdk::{Session, ShopDomain, AuthScopes, AssociatedUser};
//!
//! // Create an offline session
//! let offline_session = Session::new(
//!     "offline_my-store.myshopify.com".to_string(),
//!     ShopDomain::new("my-store").unwrap(),
//!     "access-token".to_string(),
//!     "read_products".parse().unwrap(),
//!     false,
//!     None,
//! );
//!
//! // Offline sessions don't expire
//! assert!(!offline_session.expired());
//! ```

mod associated_user;
pub mod oauth;
mod scopes;
pub mod session;

pub use associated_user::AssociatedUser;
pub use scopes::AuthScopes;
pub use session::Session;

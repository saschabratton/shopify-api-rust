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
//! # Example
//!
//! ```rust
//! use shopify_api::{Session, ShopDomain, AuthScopes, AssociatedUser};
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
mod scopes;
pub mod session;

pub use associated_user::AssociatedUser;
pub use scopes::AuthScopes;
pub use session::Session;

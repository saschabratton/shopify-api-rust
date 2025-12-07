//! Authentication types for the Shopify API SDK.
//!
//! This module provides types for handling OAuth scopes and sessions
//! used in Shopify API authentication.
//!
//! # Overview
//!
//! - [`AuthScopes`]: A set of OAuth scopes with implied scope handling
//! - [`Session`]: Represents an authenticated session for API calls

mod scopes;
mod session;

pub use scopes::AuthScopes;
pub use session::Session;

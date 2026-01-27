//! Storefront API token types for authentication.
//!
//! This module provides the [`StorefrontToken`] enum for type-safe authentication
//! with the Shopify Storefront API.
//!
//! # Token Types
//!
//! The Storefront API supports two types of access tokens:
//!
//! - **Public tokens**: Used for client-side applications and have limited access.
//!   These use the `X-Shopify-Storefront-Access-Token` header.
//!
//! - **Private tokens**: Used for server-side applications with elevated access.
//!   These use the `Shopify-Storefront-Private-Token` header.
//!
//! # Security
//!
//! The [`StorefrontToken`] type implements a custom [`Debug`] trait that masks
//! the token value, preventing accidental exposure in logs.
//!
//! # Example
//!
//! ```rust
//! use shopify_sdk::StorefrontToken;
//!
//! // Public token (client-side)
//! let public_token = StorefrontToken::Public("public-access-token".to_string());
//! assert_eq!(public_token.header_name(), "X-Shopify-Storefront-Access-Token");
//!
//! // Private token (server-side)
//! let private_token = StorefrontToken::Private("private-access-token".to_string());
//! assert_eq!(private_token.header_name(), "Shopify-Storefront-Private-Token");
//!
//! // Debug output masks the token value
//! let debug_output = format!("{:?}", public_token);
//! assert!(debug_output.contains("*****"));
//! assert!(!debug_output.contains("public-access-token"));
//! ```

use std::fmt;

/// HTTP header name for public storefront access tokens.
pub const PUBLIC_HEADER_NAME: &str = "X-Shopify-Storefront-Access-Token";

/// HTTP header name for private storefront access tokens.
pub const PRIVATE_HEADER_NAME: &str = "Shopify-Storefront-Private-Token";

/// A Shopify Storefront API access token.
///
/// This enum provides type-safe representation of storefront access tokens,
/// automatically selecting the correct HTTP header based on the token type.
///
/// # Token Types
///
/// - [`Public`](Self::Public): For client-side applications with limited access
/// - [`Private`](Self::Private): For server-side applications with elevated access
///
/// # Security
///
/// The [`Debug`] implementation masks token values to prevent accidental exposure:
///
/// ```rust
/// use shopify_sdk::StorefrontToken;
///
/// let token = StorefrontToken::Public("secret-token".to_string());
/// let debug = format!("{:?}", token);
/// assert_eq!(debug, "StorefrontToken::Public(*****)");
/// ```
///
/// # Example
///
/// ```rust
/// use shopify_sdk::StorefrontToken;
///
/// let token = StorefrontToken::Public("my-token".to_string());
///
/// // Get the correct header name
/// assert_eq!(token.header_name(), "X-Shopify-Storefront-Access-Token");
///
/// // Get the token value
/// assert_eq!(token.header_value(), "my-token");
/// ```
#[derive(Clone)]
pub enum StorefrontToken {
    /// Public storefront access token for client-side applications.
    ///
    /// Uses the `X-Shopify-Storefront-Access-Token` header.
    /// These tokens have limited access and are safe to expose in client-side code.
    Public(String),

    /// Private storefront access token for server-side applications.
    ///
    /// Uses the `Shopify-Storefront-Private-Token` header.
    /// These tokens have elevated access and should only be used server-side.
    Private(String),
}

impl StorefrontToken {
    /// Returns the HTTP header name for this token type.
    ///
    /// - [`Public`](Self::Public): `X-Shopify-Storefront-Access-Token`
    /// - [`Private`](Self::Private): `Shopify-Storefront-Private-Token`
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::StorefrontToken;
    ///
    /// let public = StorefrontToken::Public("token".to_string());
    /// assert_eq!(public.header_name(), "X-Shopify-Storefront-Access-Token");
    ///
    /// let private = StorefrontToken::Private("token".to_string());
    /// assert_eq!(private.header_name(), "Shopify-Storefront-Private-Token");
    /// ```
    #[must_use]
    pub const fn header_name(&self) -> &'static str {
        match self {
            Self::Public(_) => PUBLIC_HEADER_NAME,
            Self::Private(_) => PRIVATE_HEADER_NAME,
        }
    }

    /// Returns the token value as a string slice.
    ///
    /// This is used to set the HTTP header value when making requests.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::StorefrontToken;
    ///
    /// let token = StorefrontToken::Public("my-access-token".to_string());
    /// assert_eq!(token.header_value(), "my-access-token");
    /// ```
    #[must_use]
    pub fn header_value(&self) -> &str {
        match self {
            Self::Public(token) | Self::Private(token) => token,
        }
    }
}

impl fmt::Debug for StorefrontToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public(_) => f.write_str("StorefrontToken::Public(*****)"),
            Self::Private(_) => f.write_str("StorefrontToken::Private(*****)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Header Name Tests ===

    #[test]
    fn test_public_token_returns_correct_header_name() {
        let token = StorefrontToken::Public("test-token".to_string());
        assert_eq!(token.header_name(), "X-Shopify-Storefront-Access-Token");
    }

    #[test]
    fn test_private_token_returns_correct_header_name() {
        let token = StorefrontToken::Private("test-token".to_string());
        assert_eq!(token.header_name(), "Shopify-Storefront-Private-Token");
    }

    // === Header Value Tests ===

    #[test]
    fn test_public_token_header_value_returns_token_string() {
        let token = StorefrontToken::Public("my-public-token".to_string());
        assert_eq!(token.header_value(), "my-public-token");
    }

    #[test]
    fn test_private_token_header_value_returns_token_string() {
        let token = StorefrontToken::Private("my-private-token".to_string());
        assert_eq!(token.header_value(), "my-private-token");
    }

    // === Debug Masking Tests ===

    #[test]
    fn test_debug_masks_public_token_value() {
        let token = StorefrontToken::Public("super-secret-token".to_string());
        let debug_output = format!("{:?}", token);

        assert_eq!(debug_output, "StorefrontToken::Public(*****)");
        assert!(!debug_output.contains("super-secret-token"));
    }

    #[test]
    fn test_debug_masks_private_token_value() {
        let token = StorefrontToken::Private("super-secret-token".to_string());
        let debug_output = format!("{:?}", token);

        assert_eq!(debug_output, "StorefrontToken::Private(*****)");
        assert!(!debug_output.contains("super-secret-token"));
    }

    // === Clone Tests ===

    #[test]
    fn test_public_token_clone_works_correctly() {
        let original = StorefrontToken::Public("cloneable-token".to_string());
        let cloned = original.clone();

        assert_eq!(cloned.header_value(), "cloneable-token");
        assert_eq!(cloned.header_name(), original.header_name());
    }

    #[test]
    fn test_private_token_clone_works_correctly() {
        let original = StorefrontToken::Private("cloneable-token".to_string());
        let cloned = original.clone();

        assert_eq!(cloned.header_value(), "cloneable-token");
        assert_eq!(cloned.header_name(), original.header_name());
    }

    // === Constant Header Name Tests ===

    #[test]
    fn test_public_header_name_constant() {
        assert_eq!(PUBLIC_HEADER_NAME, "X-Shopify-Storefront-Access-Token");
    }

    #[test]
    fn test_private_header_name_constant() {
        assert_eq!(PRIVATE_HEADER_NAME, "Shopify-Storefront-Private-Token");
    }
}

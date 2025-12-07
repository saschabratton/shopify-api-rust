//! Error types for the Shopify API SDK.
//!
//! This module contains error types used throughout the SDK for configuration
//! and validation errors.
//!
//! # Error Handling
//!
//! All configuration constructors return `Result<T, ConfigError>` to enable
//! fail-fast validation. Error messages are designed to be clear and actionable.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::{ApiKey, ConfigError};
//!
//! let result = ApiKey::new("");
//! assert!(matches!(result, Err(ConfigError::EmptyApiKey)));
//! ```

use thiserror::Error;

/// Errors that can occur during SDK configuration.
///
/// This enum represents all possible errors that can occur when creating
/// or validating configuration types. Each variant provides a clear,
/// actionable error message.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// API key cannot be empty.
    #[error("API key cannot be empty. Please provide a valid Shopify API key.")]
    EmptyApiKey,

    /// API secret key cannot be empty.
    #[error("API secret key cannot be empty. Please provide a valid Shopify API secret key.")]
    EmptyApiSecretKey,

    /// Shop domain is invalid.
    #[error("Invalid shop domain '{domain}'. Expected format: 'shop-name' or 'shop-name.myshopify.com'.")]
    InvalidShopDomain {
        /// The invalid domain that was provided.
        domain: String,
    },

    /// API version is invalid.
    #[error("Invalid API version '{version}'. Expected format: 'YYYY-MM' (e.g., '2024-01') or 'unstable'.")]
    InvalidApiVersion {
        /// The invalid version string that was provided.
        version: String,
    },

    /// Scopes are invalid.
    #[error("Invalid scopes: {reason}")]
    InvalidScopes {
        /// The reason the scopes are invalid.
        reason: String,
    },

    /// A required field is missing.
    #[error("Missing required field: '{field}'. This field must be set before building the configuration.")]
    MissingRequiredField {
        /// The name of the missing field.
        field: &'static str,
    },

    /// Host URL is invalid.
    #[error("Invalid host URL '{url}'. Please provide a valid URL with scheme (e.g., 'https://myapp.example.com').")]
    InvalidHostUrl {
        /// The invalid URL that was provided.
        url: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_api_key_error_message() {
        let error = ConfigError::EmptyApiKey;
        let message = error.to_string();
        assert!(message.contains("API key cannot be empty"));
        assert!(message.contains("valid Shopify API key"));
    }

    #[test]
    fn test_invalid_shop_domain_error_message() {
        let error = ConfigError::InvalidShopDomain {
            domain: "bad domain!".to_string(),
        };
        let message = error.to_string();
        assert!(message.contains("bad domain!"));
        assert!(message.contains("Expected format"));
    }

    #[test]
    fn test_missing_required_field_error_message() {
        let error = ConfigError::MissingRequiredField { field: "api_key" };
        let message = error.to_string();
        assert!(message.contains("api_key"));
        assert!(message.contains("must be set"));
    }

    #[test]
    fn test_error_implements_std_error() {
        let error = ConfigError::EmptyApiKey;
        // Verify it implements std::error::Error by using it as a dyn Error
        let _: &dyn std::error::Error = &error;
    }
}

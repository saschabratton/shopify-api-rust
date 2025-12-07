//! Configuration types for the Shopify API SDK.
//!
//! This module provides the core configuration types used to initialize
//! and configure the SDK for API communication with Shopify.
//!
//! # Overview
//!
//! The main types in this module are:
//!
//! - [`ShopifyConfig`]: The main configuration struct holding all SDK settings
//! - [`ShopifyConfigBuilder`]: A builder for constructing [`ShopifyConfig`] instances
//! - [`ApiKey`]: A validated API key newtype
//! - [`ApiSecretKey`]: A validated API secret key newtype with masked debug output
//! - [`ShopDomain`]: A validated Shopify shop domain
//! - [`HostUrl`]: A validated application host URL
//! - [`ApiVersion`]: The Shopify API version to use
//!
//! # Example
//!
//! ```rust
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion};
//!
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("my-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("my-secret").unwrap())
//!     .api_version(ApiVersion::latest())
//!     .build()
//!     .unwrap();
//! ```

mod newtypes;
mod version;

pub use newtypes::{ApiKey, ApiSecretKey, HostUrl, ShopDomain};
pub use version::ApiVersion;

use crate::auth::AuthScopes;
use crate::error::ConfigError;

/// Configuration for the Shopify API SDK.
///
/// This struct holds all configuration needed for SDK operations, including
/// API credentials, OAuth scopes, and API version settings.
///
/// # Thread Safety
///
/// `ShopifyConfig` is `Clone`, `Send`, and `Sync`, making it safe to share
/// across threads and async tasks.
///
/// # Example
///
/// ```rust
/// use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey};
///
/// let config = ShopifyConfig::builder()
///     .api_key(ApiKey::new("your-api-key").unwrap())
///     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
///     .is_embedded(true)
///     .build()
///     .unwrap();
///
/// assert!(config.is_embedded());
/// ```
#[derive(Clone, Debug)]
pub struct ShopifyConfig {
    api_key: ApiKey,
    api_secret_key: ApiSecretKey,
    scopes: AuthScopes,
    host: Option<HostUrl>,
    api_version: ApiVersion,
    is_embedded: bool,
    user_agent_prefix: Option<String>,
}

impl ShopifyConfig {
    /// Creates a new builder for constructing a `ShopifyConfig`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey};
    ///
    /// let config = ShopifyConfig::builder()
    ///     .api_key(ApiKey::new("key").unwrap())
    ///     .api_secret_key(ApiSecretKey::new("secret").unwrap())
    ///     .build()
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn builder() -> ShopifyConfigBuilder {
        ShopifyConfigBuilder::new()
    }

    /// Returns the API key.
    #[must_use]
    pub const fn api_key(&self) -> &ApiKey {
        &self.api_key
    }

    /// Returns the API secret key.
    #[must_use]
    pub const fn api_secret_key(&self) -> &ApiSecretKey {
        &self.api_secret_key
    }

    /// Returns the OAuth scopes.
    #[must_use]
    pub const fn scopes(&self) -> &AuthScopes {
        &self.scopes
    }

    /// Returns the host URL, if configured.
    #[must_use]
    pub const fn host(&self) -> Option<&HostUrl> {
        self.host.as_ref()
    }

    /// Returns the API version.
    #[must_use]
    pub const fn api_version(&self) -> &ApiVersion {
        &self.api_version
    }

    /// Returns whether the app is embedded.
    #[must_use]
    pub const fn is_embedded(&self) -> bool {
        self.is_embedded
    }

    /// Returns the user agent prefix, if configured.
    #[must_use]
    pub fn user_agent_prefix(&self) -> Option<&str> {
        self.user_agent_prefix.as_deref()
    }
}

// Verify ShopifyConfig is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ShopifyConfig>();
};

/// Builder for constructing [`ShopifyConfig`] instances.
///
/// This builder provides a fluent API for configuring the SDK. Required fields
/// are `api_key` and `api_secret_key`. All other fields have sensible defaults.
///
/// # Defaults
///
/// - `api_version`: Latest stable version
/// - `is_embedded`: `true`
/// - `scopes`: Empty
/// - `host`: `None`
/// - `user_agent_prefix`: `None`
///
/// # Example
///
/// ```rust
/// use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion, HostUrl};
///
/// let config = ShopifyConfig::builder()
///     .api_key(ApiKey::new("key").unwrap())
///     .api_secret_key(ApiSecretKey::new("secret").unwrap())
///     .api_version(ApiVersion::V2024_10)
///     .host(HostUrl::new("https://myapp.example.com").unwrap())
///     .is_embedded(false)
///     .user_agent_prefix("MyApp/1.0")
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ShopifyConfigBuilder {
    api_key: Option<ApiKey>,
    api_secret_key: Option<ApiSecretKey>,
    scopes: Option<AuthScopes>,
    host: Option<HostUrl>,
    api_version: Option<ApiVersion>,
    is_embedded: Option<bool>,
    user_agent_prefix: Option<String>,
}

impl ShopifyConfigBuilder {
    /// Creates a new builder with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the API key (required).
    #[must_use]
    pub fn api_key(mut self, key: ApiKey) -> Self {
        self.api_key = Some(key);
        self
    }

    /// Sets the API secret key (required).
    #[must_use]
    pub fn api_secret_key(mut self, key: ApiSecretKey) -> Self {
        self.api_secret_key = Some(key);
        self
    }

    /// Sets the OAuth scopes.
    #[must_use]
    pub fn scopes(mut self, scopes: AuthScopes) -> Self {
        self.scopes = Some(scopes);
        self
    }

    /// Sets the host URL.
    #[must_use]
    pub fn host(mut self, host: HostUrl) -> Self {
        self.host = Some(host);
        self
    }

    /// Sets the API version.
    #[must_use]
    pub fn api_version(mut self, version: ApiVersion) -> Self {
        self.api_version = Some(version);
        self
    }

    /// Sets whether the app is embedded in the Shopify admin.
    #[must_use]
    pub const fn is_embedded(mut self, embedded: bool) -> Self {
        self.is_embedded = Some(embedded);
        self
    }

    /// Sets the user agent prefix for HTTP requests.
    #[must_use]
    pub fn user_agent_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.user_agent_prefix = Some(prefix.into());
        self
    }

    /// Builds the [`ShopifyConfig`], validating that required fields are set.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::MissingRequiredField`] if `api_key` or
    /// `api_secret_key` are not set.
    pub fn build(self) -> Result<ShopifyConfig, ConfigError> {
        let api_key = self
            .api_key
            .ok_or(ConfigError::MissingRequiredField { field: "api_key" })?;
        let api_secret_key = self
            .api_secret_key
            .ok_or(ConfigError::MissingRequiredField {
                field: "api_secret_key",
            })?;

        Ok(ShopifyConfig {
            api_key,
            api_secret_key,
            scopes: self.scopes.unwrap_or_default(),
            host: self.host,
            api_version: self.api_version.unwrap_or_else(ApiVersion::latest),
            is_embedded: self.is_embedded.unwrap_or(true),
            user_agent_prefix: self.user_agent_prefix,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_requires_api_key() {
        let result = ShopifyConfigBuilder::new()
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build();

        assert!(matches!(
            result,
            Err(ConfigError::MissingRequiredField { field: "api_key" })
        ));
    }

    #[test]
    fn test_builder_requires_api_secret_key() {
        let result = ShopifyConfigBuilder::new()
            .api_key(ApiKey::new("key").unwrap())
            .build();

        assert!(matches!(
            result,
            Err(ConfigError::MissingRequiredField {
                field: "api_secret_key"
            })
        ));
    }

    #[test]
    fn test_builder_provides_sensible_defaults() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        assert_eq!(config.api_version(), &ApiVersion::latest());
        assert!(config.is_embedded());
        assert!(config.scopes().is_empty());
        assert!(config.host().is_none());
        assert!(config.user_agent_prefix().is_none());
    }

    #[test]
    fn test_config_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ShopifyConfig>();
    }

    #[test]
    fn test_config_is_clone_and_debug() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let cloned = config.clone();
        assert_eq!(cloned.api_key(), config.api_key());

        // Verify Debug works
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ShopifyConfig"));
    }

    #[test]
    fn test_builder_with_all_optional_fields() {
        let scopes: AuthScopes = "read_products,write_orders".parse().unwrap();
        let host = HostUrl::new("https://myapp.example.com").unwrap();

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .scopes(scopes.clone())
            .host(host.clone())
            .api_version(ApiVersion::V2024_10)
            .is_embedded(false)
            .user_agent_prefix("MyApp/1.0")
            .build()
            .unwrap();

        assert_eq!(config.api_version(), &ApiVersion::V2024_10);
        assert!(!config.is_embedded());
        assert_eq!(config.host(), Some(&host));
        assert_eq!(config.user_agent_prefix(), Some("MyApp/1.0"));
    }
}

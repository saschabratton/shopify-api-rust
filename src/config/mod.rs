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
//! - [`DeprecationCallback`]: Callback type for API deprecation notices
//!
//! # Example
//!
//! ```rust
//! use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion};
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

// Re-export DeprecationCallback type (defined in this module)

use crate::auth::AuthScopes;
use crate::clients::ApiDeprecationInfo;
use crate::error::ConfigError;
use std::sync::Arc;

/// Callback type for handling API deprecation notices.
///
/// This callback is invoked whenever the SDK receives a response with the
/// `X-Shopify-API-Deprecated-Reason` header, indicating that the requested
/// endpoint or API version is deprecated.
///
/// The callback receives an [`ApiDeprecationInfo`] struct containing the
/// deprecation reason and the request path.
///
/// # Thread Safety
///
/// The callback must be `Send + Sync` to be safely shared across threads
/// and async tasks.
///
/// # Example
///
/// ```rust
/// use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, DeprecationCallback};
/// use std::sync::Arc;
///
/// let callback: DeprecationCallback = Arc::new(|info| {
///     eprintln!("Deprecation warning: {} at {:?}", info.reason, info.path);
/// });
///
/// let config = ShopifyConfig::builder()
///     .api_key(ApiKey::new("key").unwrap())
///     .api_secret_key(ApiSecretKey::new("secret").unwrap())
///     .on_deprecation(|info| {
///         println!("API deprecation: {}", info.reason);
///     })
///     .build()
///     .unwrap();
/// ```
pub type DeprecationCallback = Arc<dyn Fn(&ApiDeprecationInfo) + Send + Sync>;

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
/// # Key Rotation
///
/// The `old_api_secret_key` field supports seamless key rotation. When
/// validating OAuth HMAC signatures, the SDK will try the primary key first,
/// then fall back to the old key if configured. This allows in-flight OAuth
/// flows to complete during key rotation.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};
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
#[derive(Clone)]
pub struct ShopifyConfig {
    api_key: ApiKey,
    api_secret_key: ApiSecretKey,
    old_api_secret_key: Option<ApiSecretKey>,
    scopes: AuthScopes,
    host: Option<HostUrl>,
    api_version: ApiVersion,
    is_embedded: bool,
    user_agent_prefix: Option<String>,
    deprecation_callback: Option<DeprecationCallback>,
}

impl std::fmt::Debug for ShopifyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShopifyConfig")
            .field("api_key", &self.api_key)
            .field("api_secret_key", &self.api_secret_key)
            .field("old_api_secret_key", &self.old_api_secret_key)
            .field("scopes", &self.scopes)
            .field("host", &self.host)
            .field("api_version", &self.api_version)
            .field("is_embedded", &self.is_embedded)
            .field("user_agent_prefix", &self.user_agent_prefix)
            .field(
                "deprecation_callback",
                &self.deprecation_callback.as_ref().map(|_| "<callback>"),
            )
            .finish()
    }
}

impl ShopifyConfig {
    /// Creates a new builder for constructing a `ShopifyConfig`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};
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

    /// Returns the old API secret key, if configured.
    ///
    /// This is used during key rotation to validate HMAC signatures
    /// created with the previous secret key.
    #[must_use]
    pub const fn old_api_secret_key(&self) -> Option<&ApiSecretKey> {
        self.old_api_secret_key.as_ref()
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

    /// Returns the deprecation callback, if configured.
    ///
    /// This callback is invoked when the SDK receives a response indicating
    /// that an API endpoint is deprecated.
    #[must_use]
    pub fn deprecation_callback(&self) -> Option<&DeprecationCallback> {
        self.deprecation_callback.as_ref()
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
/// - `old_api_secret_key`: `None`
/// - `reject_deprecated_versions`: `false`
///
/// # Example
///
/// ```rust
/// use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion, HostUrl};
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
#[derive(Default)]
pub struct ShopifyConfigBuilder {
    api_key: Option<ApiKey>,
    api_secret_key: Option<ApiSecretKey>,
    old_api_secret_key: Option<ApiSecretKey>,
    scopes: Option<AuthScopes>,
    host: Option<HostUrl>,
    api_version: Option<ApiVersion>,
    is_embedded: Option<bool>,
    user_agent_prefix: Option<String>,
    reject_deprecated_versions: bool,
    deprecation_callback: Option<DeprecationCallback>,
}

impl std::fmt::Debug for ShopifyConfigBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShopifyConfigBuilder")
            .field("api_key", &self.api_key)
            .field("api_secret_key", &self.api_secret_key)
            .field("old_api_secret_key", &self.old_api_secret_key)
            .field("scopes", &self.scopes)
            .field("host", &self.host)
            .field("api_version", &self.api_version)
            .field("is_embedded", &self.is_embedded)
            .field("user_agent_prefix", &self.user_agent_prefix)
            .field("reject_deprecated_versions", &self.reject_deprecated_versions)
            .field(
                "deprecation_callback",
                &self.deprecation_callback.as_ref().map(|_| "<callback>"),
            )
            .finish()
    }
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

    /// Sets the old API secret key for key rotation support.
    ///
    /// When validating OAuth HMAC signatures, the SDK will try the primary
    /// secret key first, then fall back to this old key if validation fails.
    /// This allows in-flight OAuth flows to complete during key rotation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};
    ///
    /// // During key rotation, configure both keys
    /// let config = ShopifyConfig::builder()
    ///     .api_key(ApiKey::new("key").unwrap())
    ///     .api_secret_key(ApiSecretKey::new("new-secret").unwrap())
    ///     .old_api_secret_key(ApiSecretKey::new("old-secret").unwrap())
    ///     .build()
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn old_api_secret_key(mut self, key: ApiSecretKey) -> Self {
        self.old_api_secret_key = Some(key);
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

    /// Sets whether to reject deprecated API versions.
    ///
    /// When `true`, [`build()`](Self::build) will return a
    /// [`ConfigError::DeprecatedApiVersion`] error if the configured API version
    /// is past Shopify's support window.
    ///
    /// When `false` (the default), deprecated versions will log a warning via
    /// `tracing` but the configuration will still be created.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion, ConfigError};
    ///
    /// // This will fail because V2024_01 is deprecated
    /// let result = ShopifyConfig::builder()
    ///     .api_key(ApiKey::new("key").unwrap())
    ///     .api_secret_key(ApiSecretKey::new("secret").unwrap())
    ///     .api_version(ApiVersion::V2024_01)
    ///     .reject_deprecated_versions(true)
    ///     .build();
    ///
    /// assert!(matches!(result, Err(ConfigError::DeprecatedApiVersion { .. })));
    /// ```
    #[must_use]
    pub const fn reject_deprecated_versions(mut self, reject: bool) -> Self {
        self.reject_deprecated_versions = reject;
        self
    }

    /// Sets a callback to be invoked when API deprecation notices are received.
    ///
    /// The callback is called whenever the SDK receives a response with the
    /// `X-Shopify-API-Deprecated-Reason` header. This allows you to track
    /// deprecated API usage in your monitoring systems.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey};
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// use std::sync::Arc;
    ///
    /// let deprecation_count = Arc::new(AtomicUsize::new(0));
    /// let count_clone = Arc::clone(&deprecation_count);
    ///
    /// let config = ShopifyConfig::builder()
    ///     .api_key(ApiKey::new("key").unwrap())
    ///     .api_secret_key(ApiSecretKey::new("secret").unwrap())
    ///     .on_deprecation(move |info| {
    ///         count_clone.fetch_add(1, Ordering::SeqCst);
    ///         eprintln!("Deprecated: {} at {:?}", info.reason, info.path);
    ///     })
    ///     .build()
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn on_deprecation<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ApiDeprecationInfo) + Send + Sync + 'static,
    {
        self.deprecation_callback = Some(Arc::new(callback));
        self
    }

    /// Builds the [`ShopifyConfig`], validating that required fields are set.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::MissingRequiredField`] if `api_key` or
    /// `api_secret_key` are not set.
    ///
    /// Returns [`ConfigError::DeprecatedApiVersion`] if
    /// [`reject_deprecated_versions(true)`](Self::reject_deprecated_versions) is set
    /// and the configured API version is deprecated.
    pub fn build(self) -> Result<ShopifyConfig, ConfigError> {
        let api_key = self
            .api_key
            .ok_or(ConfigError::MissingRequiredField { field: "api_key" })?;
        let api_secret_key = self
            .api_secret_key
            .ok_or(ConfigError::MissingRequiredField {
                field: "api_secret_key",
            })?;

        let api_version = self.api_version.unwrap_or_else(ApiVersion::latest);

        // Check for deprecated API version
        if api_version.is_deprecated() {
            if self.reject_deprecated_versions {
                return Err(ConfigError::DeprecatedApiVersion {
                    version: api_version.to_string(),
                    latest: ApiVersion::latest().to_string(),
                });
            }
            tracing::warn!(
                version = %api_version,
                latest = %ApiVersion::latest(),
                "Using deprecated API version '{}'. Please upgrade to '{}' or a newer supported version.",
                api_version,
                ApiVersion::latest()
            );
        }

        Ok(ShopifyConfig {
            api_key,
            api_secret_key,
            old_api_secret_key: self.old_api_secret_key,
            scopes: self.scopes.unwrap_or_default(),
            host: self.host,
            api_version,
            is_embedded: self.is_embedded.unwrap_or(true),
            user_agent_prefix: self.user_agent_prefix,
            deprecation_callback: self.deprecation_callback,
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
        assert!(config.old_api_secret_key().is_none());
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

    #[test]
    fn test_old_api_secret_key_configuration() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("new-secret").unwrap())
            .old_api_secret_key(ApiSecretKey::new("old-secret").unwrap())
            .build()
            .unwrap();

        assert!(config.old_api_secret_key().is_some());
        assert_eq!(config.old_api_secret_key().unwrap().as_ref(), "old-secret");
    }

    #[test]
    fn test_old_api_secret_key_is_optional() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        assert!(config.old_api_secret_key().is_none());
    }

    #[test]
    fn test_build_allows_deprecated_version_by_default() {
        // By default, deprecated versions should be allowed (with a warning)
        let result = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .api_version(ApiVersion::V2024_01)
            .build();

        assert!(result.is_ok());
        assert_eq!(result.unwrap().api_version(), &ApiVersion::V2024_01);
    }

    #[test]
    fn test_build_fails_when_reject_deprecated() {
        let result = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .api_version(ApiVersion::V2024_01)
            .reject_deprecated_versions(true)
            .build();

        assert!(matches!(
            result,
            Err(ConfigError::DeprecatedApiVersion { version, latest })
            if version == "2024-01" && latest == ApiVersion::latest().to_string()
        ));
    }

    #[test]
    fn test_build_succeeds_with_supported_version_when_reject_deprecated() {
        let result = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .api_version(ApiVersion::V2025_10)
            .reject_deprecated_versions(true)
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_build_succeeds_with_unstable_when_reject_deprecated() {
        let result = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .api_version(ApiVersion::Unstable)
            .reject_deprecated_versions(true)
            .build();

        assert!(result.is_ok());
    }
}

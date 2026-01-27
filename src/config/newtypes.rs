//! Validated newtype wrappers for configuration values.
//!
//! This module provides type-safe wrappers around string values that validate
//! their contents on construction. Invalid values are rejected with clear error messages.

use crate::error::ConfigError;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A validated Shopify API key.
///
/// This newtype ensures the API key is non-empty and provides type safety
/// to prevent accidental misuse of raw strings.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::ApiKey;
///
/// let key = ApiKey::new("my-api-key").unwrap();
/// assert_eq!(key.as_ref(), "my-api-key");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Creates a new validated API key.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::EmptyApiKey`] if the key is empty.
    pub fn new(key: impl Into<String>) -> Result<Self, ConfigError> {
        let key = key.into();
        if key.is_empty() {
            return Err(ConfigError::EmptyApiKey);
        }
        Ok(Self(key))
    }
}

impl AsRef<str> for ApiKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A validated Shopify API secret key.
///
/// This newtype ensures the secret key is non-empty and masks its value
/// in debug output to prevent accidental exposure in logs.
///
/// # Security
///
/// The `Debug` implementation masks the secret value, displaying only
/// `ApiSecretKey(*****)` instead of the actual key.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::ApiSecretKey;
///
/// let secret = ApiSecretKey::new("my-secret").unwrap();
/// assert_eq!(format!("{:?}", secret), "ApiSecretKey(*****)");
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct ApiSecretKey(String);

impl ApiSecretKey {
    /// Creates a new validated API secret key.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::EmptyApiSecretKey`] if the key is empty.
    pub fn new(key: impl Into<String>) -> Result<Self, ConfigError> {
        let key = key.into();
        if key.is_empty() {
            return Err(ConfigError::EmptyApiSecretKey);
        }
        Ok(Self(key))
    }
}

impl AsRef<str> for ApiSecretKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiSecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiSecretKey(*****)")
    }
}

/// A validated Shopify shop domain.
///
/// This newtype validates and normalizes shop domains to the full
/// `shop.myshopify.com` format.
///
/// # Accepted Formats
///
/// - `shop-name` - normalized to `shop-name.myshopify.com`
/// - `shop-name.myshopify.com` - used as-is
///
/// # Serialization
///
/// `ShopDomain` serializes to and deserializes from the full domain string:
///
/// ```rust
/// use shopify_sdk::ShopDomain;
///
/// let domain = ShopDomain::new("my-store").unwrap();
/// let json = serde_json::to_string(&domain).unwrap();
/// assert_eq!(json, r#""my-store.myshopify.com""#);
/// ```
///
/// # Example
///
/// ```rust
/// use shopify_sdk::ShopDomain;
///
/// // Short format is normalized
/// let domain = ShopDomain::new("my-store").unwrap();
/// assert_eq!(domain.as_ref(), "my-store.myshopify.com");
/// assert_eq!(domain.shop_name(), "my-store");
///
/// // Full format is accepted
/// let domain = ShopDomain::new("my-store.myshopify.com").unwrap();
/// assert_eq!(domain.as_ref(), "my-store.myshopify.com");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopDomain {
    full_domain: String,
    shop_name_end: usize,
}

impl ShopDomain {
    const SUFFIX: &'static str = ".myshopify.com";

    /// Creates a new validated shop domain.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidShopDomain`] if the domain is invalid.
    pub fn new(domain: impl Into<String>) -> Result<Self, ConfigError> {
        let domain = domain.into();
        let domain = domain.trim().to_lowercase();

        if domain.is_empty() {
            return Err(ConfigError::InvalidShopDomain { domain });
        }

        // Check if it's already a full domain
        let (shop_name, full_domain) = if let Some(shop_name) = domain.strip_suffix(Self::SUFFIX) {
            (shop_name.to_string(), domain)
        } else if domain.contains('.') {
            // Contains a dot but not myshopify.com suffix - invalid
            return Err(ConfigError::InvalidShopDomain { domain });
        } else {
            // Short format - needs normalization
            (domain.clone(), format!("{}{}", domain, Self::SUFFIX))
        };

        // Validate shop name
        if !Self::is_valid_shop_name(&shop_name) {
            return Err(ConfigError::InvalidShopDomain {
                domain: full_domain,
            });
        }

        Ok(Self {
            shop_name_end: shop_name.len(),
            full_domain,
        })
    }

    /// Returns the shop name portion of the domain.
    ///
    /// For `my-store.myshopify.com`, this returns `my-store`.
    #[must_use]
    pub fn shop_name(&self) -> &str {
        &self.full_domain[..self.shop_name_end]
    }

    fn is_valid_shop_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Shop names can contain lowercase letters, numbers, and hyphens
        // They cannot start or end with a hyphen
        if name.starts_with('-') || name.ends_with('-') {
            return false;
        }

        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }
}

impl AsRef<str> for ShopDomain {
    fn as_ref(&self) -> &str {
        &self.full_domain
    }
}

impl Serialize for ShopDomain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.full_domain)
    }
}

impl<'de> Deserialize<'de> for ShopDomain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(de::Error::custom)
    }
}

/// A validated host URL for the application.
///
/// This newtype validates that the URL has a proper format with a scheme.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::HostUrl;
///
/// let url = HostUrl::new("https://myapp.example.com").unwrap();
/// assert_eq!(url.scheme(), "https");
/// assert_eq!(url.host_name(), Some("myapp.example.com"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostUrl {
    url: String,
    scheme_end: usize,
    host_start: usize,
    host_end: usize,
}

impl HostUrl {
    /// Creates a new validated host URL.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidHostUrl`] if the URL is invalid.
    pub fn new(url: impl Into<String>) -> Result<Self, ConfigError> {
        let url = url.into();
        let url = url.trim().to_string();

        // Find scheme
        let scheme_end = url
            .find("://")
            .ok_or_else(|| ConfigError::InvalidHostUrl { url: url.clone() })?;

        let scheme = &url[..scheme_end];
        if scheme.is_empty() || !scheme.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(ConfigError::InvalidHostUrl { url: url.clone() });
        }

        // Find host
        let host_start = scheme_end + 3; // Skip "://"
        if host_start >= url.len() {
            return Err(ConfigError::InvalidHostUrl { url: url.clone() });
        }

        // Host ends at port, path, query, or end of string
        let remainder = &url[host_start..];
        let host_end = remainder
            .find([':', '/', '?', '#'])
            .map_or(url.len(), |i| host_start + i);

        let host = &url[host_start..host_end];
        if host.is_empty() {
            return Err(ConfigError::InvalidHostUrl { url: url.clone() });
        }

        Ok(Self {
            url,
            scheme_end,
            host_start,
            host_end,
        })
    }

    /// Returns the URL scheme (e.g., "https").
    #[must_use]
    pub fn scheme(&self) -> &str {
        &self.url[..self.scheme_end]
    }

    /// Returns the host name portion of the URL.
    #[must_use]
    pub fn host_name(&self) -> Option<&str> {
        let host = &self.url[self.host_start..self.host_end];
        if host.is_empty() {
            None
        } else {
            Some(host)
        }
    }
}

impl AsRef<str> for HostUrl {
    fn as_ref(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_rejects_empty_string() {
        let result = ApiKey::new("");
        assert!(matches!(result, Err(ConfigError::EmptyApiKey)));
    }

    #[test]
    fn test_api_secret_key_masks_value_in_debug() {
        let secret = ApiSecretKey::new("super-secret-key").unwrap();
        let debug_output = format!("{:?}", secret);
        assert_eq!(debug_output, "ApiSecretKey(*****)");
        assert!(!debug_output.contains("super-secret-key"));
    }

    #[test]
    fn test_shop_domain_normalizes_short_format() {
        let domain = ShopDomain::new("my-store").unwrap();
        assert_eq!(domain.as_ref(), "my-store.myshopify.com");
        assert_eq!(domain.shop_name(), "my-store");
    }

    #[test]
    fn test_shop_domain_accepts_full_format() {
        let domain = ShopDomain::new("my-store.myshopify.com").unwrap();
        assert_eq!(domain.as_ref(), "my-store.myshopify.com");
        assert_eq!(domain.shop_name(), "my-store");
    }

    #[test]
    fn test_shop_domain_rejects_invalid_domains() {
        // Empty
        assert!(ShopDomain::new("").is_err());

        // Invalid characters
        assert!(ShopDomain::new("my store").is_err());
        assert!(ShopDomain::new("my_store").is_err());
        assert!(ShopDomain::new("MY-STORE").is_ok()); // normalized to lowercase

        // Starting/ending with hyphen
        assert!(ShopDomain::new("-my-store").is_err());
        assert!(ShopDomain::new("my-store-").is_err());

        // Wrong domain suffix
        assert!(ShopDomain::new("my-store.otherdomain.com").is_err());
    }

    #[test]
    fn test_host_url_validates_format() {
        let url = HostUrl::new("https://myapp.example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_name(), Some("myapp.example.com"));

        // With port
        let url = HostUrl::new("http://localhost:3000").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_name(), Some("localhost"));

        // With path
        let url = HostUrl::new("https://myapp.example.com/callback").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_name(), Some("myapp.example.com"));
    }

    #[test]
    fn test_host_url_rejects_invalid() {
        // No scheme
        assert!(HostUrl::new("myapp.example.com").is_err());

        // Empty host
        assert!(HostUrl::new("https://").is_err());

        // Invalid scheme
        assert!(HostUrl::new("://example.com").is_err());
    }

    // ShopDomain serialization tests
    #[test]
    fn test_shop_domain_serializes_to_string() {
        let domain = ShopDomain::new("my-store").unwrap();
        let json = serde_json::to_string(&domain).unwrap();
        assert_eq!(json, r#""my-store.myshopify.com""#);
    }

    #[test]
    fn test_shop_domain_deserializes_from_string() {
        let json = r#""test-shop.myshopify.com""#;
        let domain: ShopDomain = serde_json::from_str(json).unwrap();
        assert_eq!(domain.as_ref(), "test-shop.myshopify.com");
        assert_eq!(domain.shop_name(), "test-shop");
    }

    #[test]
    fn test_shop_domain_round_trip_serialization() {
        let original = ShopDomain::new("my-store").unwrap();
        let json = serde_json::to_string(&original).unwrap();
        let restored: ShopDomain = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }
}

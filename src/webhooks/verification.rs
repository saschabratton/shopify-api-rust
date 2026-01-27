//! Webhook signature verification for the Shopify API SDK.
//!
//! This module provides functions and types for verifying HMAC signatures on
//! incoming webhook requests from Shopify.
//!
//! # Overview
//!
//! Shopify signs webhook requests using HMAC-SHA256 with the app's API secret key.
//! This module provides both high-level and low-level verification functions:
//!
//! - [`verify_webhook`]: High-level function that uses `ShopifyConfig` and supports key rotation
//! - [`verify_hmac`]: Low-level function for custom integrations
//!
//! # Example
//!
//! ```rust
//! use shopify_sdk::webhooks::{WebhookRequest, verify_webhook, verify_hmac};
//! use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};
//! use shopify_sdk::auth::oauth::hmac::compute_signature_base64;
//!
//! // Create a config with the API secret
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("test-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("my-secret").unwrap())
//!     .build()
//!     .unwrap();
//!
//! // Compute a valid HMAC for testing
//! let body = b"webhook payload";
//! let hmac = compute_signature_base64(body, "my-secret");
//!
//! // Create a webhook request
//! let request = WebhookRequest::new(
//!     body.to_vec(),
//!     hmac,
//!     Some("orders/create".to_string()),
//!     Some("example.myshopify.com".to_string()),
//!     Some("2025-10".to_string()),
//!     Some("webhook-123".to_string()),
//! );
//!
//! // Verify the webhook (high-level)
//! let context = verify_webhook(&config, &request).expect("verification failed");
//! assert_eq!(context.shop_domain(), Some("example.myshopify.com"));
//!
//! // Or use low-level verification
//! let body = b"payload";
//! let hmac = compute_signature_base64(body, "secret");
//! assert!(verify_hmac(body, &hmac, "secret"));
//! ```
//!
//! # Security
//!
//! All HMAC comparisons use constant-time comparison to prevent timing attacks.
//! The high-level verification function also supports key rotation by trying
//! the primary secret key first, then falling back to the old secret key.

use crate::auth::oauth::hmac::{compute_signature_base64, constant_time_compare};
use crate::config::ShopifyConfig;
use crate::rest::resources::v2025_10::common::WebhookTopic;
use crate::webhooks::WebhookError;

// ============================================================================
// Header Constants
// ============================================================================

/// HTTP header name for the HMAC-SHA256 signature.
///
/// Shopify includes this header in all webhook requests. The value is a
/// base64-encoded HMAC-SHA256 signature of the request body.
pub const HEADER_HMAC: &str = "X-Shopify-Hmac-SHA256";

/// HTTP header name for the webhook topic.
///
/// Contains the topic string (e.g., "orders/create") that identifies
/// what event triggered the webhook.
pub const HEADER_TOPIC: &str = "X-Shopify-Topic";

/// HTTP header name for the shop domain.
///
/// Contains the myshopify.com domain of the shop that triggered the webhook
/// (e.g., "example.myshopify.com").
pub const HEADER_SHOP_DOMAIN: &str = "X-Shopify-Shop-Domain";

/// HTTP header name for the API version.
///
/// Contains the API version used for the webhook payload format
/// (e.g., "2025-10").
pub const HEADER_API_VERSION: &str = "X-Shopify-API-Version";

/// HTTP header name for the webhook ID.
///
/// Contains a unique identifier for the webhook delivery, useful for
/// idempotency and debugging.
pub const HEADER_WEBHOOK_ID: &str = "X-Shopify-Webhook-Id";

// ============================================================================
// WebhookRequest
// ============================================================================

/// Represents an incoming webhook request from Shopify.
///
/// This struct holds the raw request body and headers needed for verification.
/// The body is stored as raw bytes to preserve the exact payload for HMAC computation.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::webhooks::WebhookRequest;
///
/// let request = WebhookRequest::new(
///     b"raw body bytes".to_vec(),
///     "hmac-signature".to_string(),
///     Some("orders/create".to_string()),
///     Some("example.myshopify.com".to_string()),
///     Some("2025-10".to_string()),
///     Some("webhook-123".to_string()),
/// );
///
/// assert_eq!(request.body(), b"raw body bytes");
/// assert_eq!(request.hmac_header(), "hmac-signature");
/// ```
#[derive(Debug, Clone)]
pub struct WebhookRequest {
    /// Raw request body as bytes.
    body: Vec<u8>,
    /// HMAC signature from the X-Shopify-Hmac-SHA256 header.
    hmac_header: String,
    /// Webhook topic from the X-Shopify-Topic header.
    topic: Option<String>,
    /// Shop domain from the X-Shopify-Shop-Domain header.
    shop_domain: Option<String>,
    /// API version from the X-Shopify-API-Version header.
    api_version: Option<String>,
    /// Webhook ID from the X-Shopify-Webhook-Id header.
    webhook_id: Option<String>,
}

impl WebhookRequest {
    /// Creates a new webhook request with the given body and headers.
    ///
    /// # Arguments
    ///
    /// * `body` - Raw request body as bytes
    /// * `hmac_header` - Value of the X-Shopify-Hmac-SHA256 header
    /// * `topic` - Value of the X-Shopify-Topic header (optional)
    /// * `shop_domain` - Value of the X-Shopify-Shop-Domain header (optional)
    /// * `api_version` - Value of the X-Shopify-API-Version header (optional)
    /// * `webhook_id` - Value of the X-Shopify-Webhook-Id header (optional)
    #[must_use]
    pub fn new(
        body: Vec<u8>,
        hmac_header: String,
        topic: Option<String>,
        shop_domain: Option<String>,
        api_version: Option<String>,
        webhook_id: Option<String>,
    ) -> Self {
        Self {
            body,
            hmac_header,
            topic,
            shop_domain,
            api_version,
            webhook_id,
        }
    }

    /// Returns the raw request body as a byte slice.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Returns the HMAC signature header value.
    #[must_use]
    pub fn hmac_header(&self) -> &str {
        &self.hmac_header
    }

    /// Returns the topic header value, if present.
    #[must_use]
    pub fn topic(&self) -> Option<&str> {
        self.topic.as_deref()
    }

    /// Returns the shop domain header value, if present.
    #[must_use]
    pub fn shop_domain(&self) -> Option<&str> {
        self.shop_domain.as_deref()
    }

    /// Returns the API version header value, if present.
    #[must_use]
    pub fn api_version(&self) -> Option<&str> {
        self.api_version.as_deref()
    }

    /// Returns the webhook ID header value, if present.
    #[must_use]
    pub fn webhook_id(&self) -> Option<&str> {
        self.webhook_id.as_deref()
    }
}

// ============================================================================
// WebhookContext
// ============================================================================

/// Represents verified webhook metadata after successful signature verification.
///
/// This struct is returned by [`verify_webhook`] and contains the parsed headers
/// from a verified webhook request. It provides both the parsed topic enum (when
/// the topic is a known value) and the raw topic string (always available).
///
/// # Example
///
/// ```rust
/// use shopify_sdk::webhooks::{WebhookRequest, verify_webhook, WebhookContext};
/// use shopify_sdk::webhooks::WebhookTopic;
/// use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};
/// use shopify_sdk::auth::oauth::hmac::compute_signature_base64;
///
/// let config = ShopifyConfig::builder()
///     .api_key(ApiKey::new("key").unwrap())
///     .api_secret_key(ApiSecretKey::new("secret").unwrap())
///     .build()
///     .unwrap();
///
/// let body = b"test";
/// let hmac = compute_signature_base64(body, "secret");
/// let request = WebhookRequest::new(
///     body.to_vec(),
///     hmac,
///     Some("orders/create".to_string()),
///     Some("example.myshopify.com".to_string()),
///     None,
///     None,
/// );
///
/// let context = verify_webhook(&config, &request).unwrap();
/// assert_eq!(context.topic(), Some(WebhookTopic::OrdersCreate));
/// assert_eq!(context.topic_raw(), "orders/create");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct WebhookContext {
    /// Parsed topic enum (None for unknown topics).
    topic: Option<WebhookTopic>,
    /// Raw topic string from the header.
    topic_raw: String,
    /// Shop domain from the header.
    shop_domain: Option<String>,
    /// API version from the header.
    api_version: Option<String>,
    /// Webhook ID from the header.
    webhook_id: Option<String>,
}

impl WebhookContext {
    /// Creates a new webhook context.
    fn new(
        topic: Option<WebhookTopic>,
        topic_raw: String,
        shop_domain: Option<String>,
        api_version: Option<String>,
        webhook_id: Option<String>,
    ) -> Self {
        Self {
            topic,
            topic_raw,
            shop_domain,
            api_version,
            webhook_id,
        }
    }

    /// Returns the parsed webhook topic enum, if the topic is a known value.
    ///
    /// Returns `None` for unknown or custom topics.
    #[must_use]
    pub fn topic(&self) -> Option<WebhookTopic> {
        self.topic
    }

    /// Returns the raw topic string as received in the header.
    ///
    /// This is always available, even for unknown or custom topics.
    #[must_use]
    pub fn topic_raw(&self) -> &str {
        &self.topic_raw
    }

    /// Returns the shop domain, if present in the webhook headers.
    #[must_use]
    pub fn shop_domain(&self) -> Option<&str> {
        self.shop_domain.as_deref()
    }

    /// Returns the API version, if present in the webhook headers.
    #[must_use]
    pub fn api_version(&self) -> Option<&str> {
        self.api_version.as_deref()
    }

    /// Returns the webhook ID, if present in the webhook headers.
    #[must_use]
    pub fn webhook_id(&self) -> Option<&str> {
        self.webhook_id.as_deref()
    }
}

// ============================================================================
// Verification Functions
// ============================================================================

/// Parses a topic string into a `WebhookTopic` enum.
///
/// Returns `None` for unknown or custom topics.
fn parse_topic(topic: &str) -> Option<WebhookTopic> {
    // WebhookTopic uses serde with rename attributes like "orders/create"
    // We can deserialize a quoted JSON string to get the enum
    let quoted = format!("\"{}\"", topic);
    serde_json::from_str(&quoted).ok()
}

/// Verifies the HMAC signature of a webhook request body.
///
/// This is a low-level function that performs HMAC verification with a single
/// secret key. For most use cases, prefer [`verify_webhook`] which supports
/// key rotation.
///
/// # Arguments
///
/// * `raw_body` - The raw request body bytes
/// * `hmac_header` - The value of the X-Shopify-Hmac-SHA256 header
/// * `secret` - The API secret key to use for verification
///
/// # Returns
///
/// `true` if the signature is valid, `false` otherwise.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::webhooks::verify_hmac;
/// use shopify_sdk::auth::oauth::hmac::compute_signature_base64;
///
/// let body = b"webhook payload";
/// let secret = "my-secret-key";
/// let hmac = compute_signature_base64(body, secret);
///
/// assert!(verify_hmac(body, &hmac, secret));
/// assert!(!verify_hmac(body, "invalid", secret));
/// ```
#[must_use]
pub fn verify_hmac(raw_body: &[u8], hmac_header: &str, secret: &str) -> bool {
    let computed = compute_signature_base64(raw_body, secret);
    constant_time_compare(&computed, hmac_header)
}

/// Verifies a webhook request and returns the verified context.
///
/// This function validates the HMAC signature using the config's API secret key,
/// with automatic fallback to the old API secret key for key rotation support.
///
/// # Arguments
///
/// * `config` - The Shopify configuration containing the API secret key(s)
/// * `request` - The webhook request to verify
///
/// # Returns
///
/// A [`WebhookContext`] containing the verified webhook metadata on success,
/// or a [`WebhookError::InvalidHmac`] if verification fails.
///
/// # Key Rotation
///
/// If the primary `api_secret_key` fails verification, the function will
/// automatically try `old_api_secret_key` if configured. This allows seamless
/// key rotation without breaking in-flight webhooks.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::webhooks::{WebhookRequest, verify_webhook};
/// use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};
/// use shopify_sdk::auth::oauth::hmac::compute_signature_base64;
///
/// let config = ShopifyConfig::builder()
///     .api_key(ApiKey::new("key").unwrap())
///     .api_secret_key(ApiSecretKey::new("secret").unwrap())
///     .build()
///     .unwrap();
///
/// let body = b"test payload";
/// let hmac = compute_signature_base64(body, "secret");
/// let request = WebhookRequest::new(
///     body.to_vec(),
///     hmac,
///     Some("orders/create".to_string()),
///     None,
///     None,
///     None,
/// );
///
/// let context = verify_webhook(&config, &request).expect("verification should succeed");
/// assert_eq!(context.topic_raw(), "orders/create");
/// ```
#[must_use]
pub fn verify_webhook(
    config: &ShopifyConfig,
    request: &WebhookRequest,
) -> Result<WebhookContext, WebhookError> {
    let body = request.body();
    let hmac_header = request.hmac_header();

    // Try primary secret key first
    let mut verified = verify_hmac(body, hmac_header, config.api_secret_key().as_ref());

    // Fall back to old secret key if configured and primary fails
    if !verified {
        if let Some(old_secret) = config.old_api_secret_key() {
            verified = verify_hmac(body, hmac_header, old_secret.as_ref());
        }
    }

    if !verified {
        return Err(WebhookError::InvalidHmac);
    }

    // Parse topic string into enum (None for unknown topics)
    let topic_raw = request.topic().unwrap_or("").to_string();
    let topic = if topic_raw.is_empty() {
        None
    } else {
        parse_topic(&topic_raw)
    };

    Ok(WebhookContext::new(
        topic,
        topic_raw,
        request.shop_domain().map(String::from),
        request.api_version().map(String::from),
        request.webhook_id().map(String::from),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiKey, ApiSecretKey};

    // ========================================================================
    // Header Constants Tests
    // ========================================================================

    #[test]
    fn test_header_constants_match_shopify_documentation() {
        assert_eq!(HEADER_HMAC, "X-Shopify-Hmac-SHA256");
        assert_eq!(HEADER_TOPIC, "X-Shopify-Topic");
        assert_eq!(HEADER_SHOP_DOMAIN, "X-Shopify-Shop-Domain");
        assert_eq!(HEADER_API_VERSION, "X-Shopify-API-Version");
        assert_eq!(HEADER_WEBHOOK_ID, "X-Shopify-Webhook-Id");
    }

    // ========================================================================
    // WebhookRequest Tests
    // ========================================================================

    #[test]
    fn test_webhook_request_new_with_all_headers() {
        let request = WebhookRequest::new(
            b"test body".to_vec(),
            "hmac-value".to_string(),
            Some("orders/create".to_string()),
            Some("example.myshopify.com".to_string()),
            Some("2025-10".to_string()),
            Some("webhook-123".to_string()),
        );

        assert_eq!(request.body(), b"test body");
        assert_eq!(request.hmac_header(), "hmac-value");
        assert_eq!(request.topic(), Some("orders/create"));
        assert_eq!(request.shop_domain(), Some("example.myshopify.com"));
        assert_eq!(request.api_version(), Some("2025-10"));
        assert_eq!(request.webhook_id(), Some("webhook-123"));
    }

    #[test]
    fn test_webhook_request_with_minimal_headers() {
        let request = WebhookRequest::new(
            b"body".to_vec(),
            "hmac".to_string(),
            None,
            None,
            None,
            None,
        );

        assert_eq!(request.body(), b"body");
        assert_eq!(request.hmac_header(), "hmac");
        assert_eq!(request.topic(), None);
        assert_eq!(request.shop_domain(), None);
        assert_eq!(request.api_version(), None);
        assert_eq!(request.webhook_id(), None);
    }

    // ========================================================================
    // WebhookContext Tests
    // ========================================================================

    #[test]
    fn test_webhook_context_accessor_methods() {
        let context = WebhookContext::new(
            Some(WebhookTopic::OrdersCreate),
            "orders/create".to_string(),
            Some("shop.myshopify.com".to_string()),
            Some("2025-10".to_string()),
            Some("id-123".to_string()),
        );

        assert_eq!(context.topic(), Some(WebhookTopic::OrdersCreate));
        assert_eq!(context.topic_raw(), "orders/create");
        assert_eq!(context.shop_domain(), Some("shop.myshopify.com"));
        assert_eq!(context.api_version(), Some("2025-10"));
        assert_eq!(context.webhook_id(), Some("id-123"));
    }

    #[test]
    fn test_webhook_context_topic_returns_parsed_enum_when_valid() {
        let context = WebhookContext::new(
            Some(WebhookTopic::ProductsUpdate),
            "products/update".to_string(),
            None,
            None,
            None,
        );

        assert_eq!(context.topic(), Some(WebhookTopic::ProductsUpdate));
    }

    #[test]
    fn test_webhook_context_topic_returns_none_for_unknown_topics() {
        let context = WebhookContext::new(
            None,
            "custom/unknown_topic".to_string(),
            None,
            None,
            None,
        );

        assert_eq!(context.topic(), None);
        assert_eq!(context.topic_raw(), "custom/unknown_topic");
    }

    #[test]
    fn test_webhook_context_topic_raw_always_returns_raw_string() {
        // For known topic
        let context1 = WebhookContext::new(
            Some(WebhookTopic::OrdersCreate),
            "orders/create".to_string(),
            None,
            None,
            None,
        );
        assert_eq!(context1.topic_raw(), "orders/create");

        // For unknown topic
        let context2 = WebhookContext::new(None, "unknown/topic".to_string(), None, None, None);
        assert_eq!(context2.topic_raw(), "unknown/topic");
    }

    // ========================================================================
    // Verification Function Tests
    // ========================================================================

    #[test]
    fn test_verify_hmac_returns_true_with_valid_signature() {
        let body = b"test payload";
        let secret = "my-secret";
        let hmac = compute_signature_base64(body, secret);

        assert!(verify_hmac(body, &hmac, secret));
    }

    #[test]
    fn test_verify_hmac_returns_false_with_invalid_signature() {
        let body = b"test payload";
        let secret = "my-secret";

        assert!(!verify_hmac(body, "invalid-hmac", secret));
    }

    #[test]
    fn test_verify_hmac_handles_empty_body() {
        let body = b"";
        let secret = "secret";
        let hmac = compute_signature_base64(body, secret);

        assert!(verify_hmac(body, &hmac, secret));
    }

    #[test]
    fn test_verify_webhook_succeeds_with_primary_key() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("primary-secret").unwrap())
            .build()
            .unwrap();

        let body = b"webhook body";
        let hmac = compute_signature_base64(body, "primary-secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("orders/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            Some("2025-10".to_string()),
            Some("webhook-id".to_string()),
        );

        let result = verify_webhook(&config, &request);
        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.topic(), Some(WebhookTopic::OrdersCreate));
        assert_eq!(context.shop_domain(), Some("shop.myshopify.com"));
    }

    #[test]
    fn test_verify_webhook_falls_back_to_old_key_successfully() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("new-secret").unwrap())
            .old_api_secret_key(ApiSecretKey::new("old-secret").unwrap())
            .build()
            .unwrap();

        // Sign with OLD secret
        let body = b"webhook body";
        let hmac = compute_signature_base64(body, "old-secret");
        let request = WebhookRequest::new(body.to_vec(), hmac, None, None, None, None);

        let result = verify_webhook(&config, &request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_webhook_fails_when_both_keys_fail() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret-1").unwrap())
            .old_api_secret_key(ApiSecretKey::new("secret-2").unwrap())
            .build()
            .unwrap();

        // Sign with a DIFFERENT secret
        let body = b"webhook body";
        let hmac = compute_signature_base64(body, "wrong-secret");
        let request = WebhookRequest::new(body.to_vec(), hmac, None, None, None, None);

        let result = verify_webhook(&config, &request);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WebhookError::InvalidHmac));
    }

    #[test]
    fn test_verify_webhook_returns_correct_context() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = b"payload";
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("products/update".to_string()),
            Some("test.myshopify.com".to_string()),
            Some("2025-10".to_string()),
            Some("wh-id-123".to_string()),
        );

        let context = verify_webhook(&config, &request).unwrap();
        assert_eq!(context.topic(), Some(WebhookTopic::ProductsUpdate));
        assert_eq!(context.topic_raw(), "products/update");
        assert_eq!(context.shop_domain(), Some("test.myshopify.com"));
        assert_eq!(context.api_version(), Some("2025-10"));
        assert_eq!(context.webhook_id(), Some("wh-id-123"));
    }

    #[test]
    fn test_verify_webhook_parses_known_topic_into_enum() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = b"data";
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("customers/create".to_string()),
            None,
            None,
            None,
        );

        let context = verify_webhook(&config, &request).unwrap();
        assert_eq!(context.topic(), Some(WebhookTopic::CustomersCreate));
    }

    #[test]
    fn test_verify_webhook_handles_unknown_topic() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = b"data";
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("custom/new_event".to_string()),
            None,
            None,
            None,
        );

        let context = verify_webhook(&config, &request).unwrap();
        assert_eq!(context.topic(), None);
        assert_eq!(context.topic_raw(), "custom/new_event");
    }

    // ========================================================================
    // Topic Parsing Tests
    // ========================================================================

    #[test]
    fn test_parse_topic_known_topics() {
        assert_eq!(parse_topic("orders/create"), Some(WebhookTopic::OrdersCreate));
        assert_eq!(
            parse_topic("products/update"),
            Some(WebhookTopic::ProductsUpdate)
        );
        assert_eq!(
            parse_topic("customers/delete"),
            Some(WebhookTopic::CustomersDelete)
        );
        assert_eq!(
            parse_topic("app/uninstalled"),
            Some(WebhookTopic::AppUninstalled)
        );
    }

    #[test]
    fn test_parse_topic_unknown_topics() {
        assert_eq!(parse_topic("unknown/topic"), None);
        assert_eq!(parse_topic("custom_event"), None);
        assert_eq!(parse_topic(""), None);
    }
}

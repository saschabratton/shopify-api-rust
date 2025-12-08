//! Internal HTTP client for Storefront API communication.
//!
//! This module provides an internal HTTP client tailored for the Storefront API,
//! which uses different endpoints and headers than the Admin API.

use std::collections::HashMap;

use crate::clients::errors::{HttpError, HttpResponseError, MaxHttpRetriesExceededError};
use crate::clients::http_client::RETRY_WAIT_TIME;
use crate::clients::http_request::HttpRequest;
use crate::clients::http_response::HttpResponse;
use crate::clients::storefront::StorefrontToken;
use crate::clients::SDK_VERSION;
use crate::config::{ApiVersion, ShopDomain, ShopifyConfig};

/// Internal HTTP client for Storefront API requests.
///
/// This client is tailored for the Storefront API, which differs from the
/// Admin API in:
/// - Base path: `/api/{version}` (not `/admin/api/{version}`)
/// - Authentication headers: Storefront-specific headers based on token type
/// - No `X-Shopify-Access-Token` header (that's Admin API only)
///
/// This type is `pub(super)` and not exposed publicly.
#[derive(Debug)]
pub(super) struct StorefrontHttpClient {
    /// The internal reqwest HTTP client.
    client: reqwest::Client,
    /// Base URI (e.g., `https://my-store.myshopify.com`).
    base_uri: String,
    /// Base path (e.g., `/api/2024-10`).
    base_path: String,
    /// Default headers to include in all requests.
    default_headers: HashMap<String, String>,
}

// Verify StorefrontHttpClient is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<StorefrontHttpClient>();
};

impl StorefrontHttpClient {
    /// Creates a new Storefront HTTP client.
    ///
    /// # Arguments
    ///
    /// * `shop` - The shop domain for endpoint construction
    /// * `token` - Optional storefront access token
    /// * `config` - Optional configuration for user agent and host settings
    /// * `api_version` - The API version for endpoint construction
    #[must_use]
    pub(super) fn new(
        shop: &ShopDomain,
        token: Option<&StorefrontToken>,
        config: Option<&ShopifyConfig>,
        api_version: &ApiVersion,
    ) -> Self {
        // Construct base path: /api/{version} (NOT /admin/api/{version})
        let base_path = format!("/api/{api_version}");

        // Determine base URI - use api_host if configured, otherwise shop domain
        let api_host = config.and_then(|c| c.host());
        let default_shop_uri = || format!("https://{}", shop.as_ref());
        let base_uri = api_host.map_or_else(default_shop_uri, |host| {
            host.host_name()
                .map_or_else(default_shop_uri, |host_name| format!("https://{host_name}"))
        });

        // Build User-Agent header
        let user_agent_prefix = config
            .and_then(ShopifyConfig::user_agent_prefix)
            .map_or(String::new(), |prefix| format!("{prefix} | "));
        let rust_version = env!("CARGO_PKG_RUST_VERSION");
        let user_agent =
            format!("{user_agent_prefix}Shopify API Library v{SDK_VERSION} | Rust {rust_version}");

        // Build default headers
        let mut default_headers = HashMap::new();
        default_headers.insert("User-Agent".to_string(), user_agent);
        default_headers.insert("Accept".to_string(), "application/json".to_string());

        // Add Host header when using api_host (proxy scenario)
        if api_host.is_some() {
            default_headers.insert("Host".to_string(), shop.as_ref().to_string());
        }

        // Add storefront token header based on token type
        // NOTE: Do NOT add X-Shopify-Access-Token (that's Admin API only)
        if let Some(token) = token {
            default_headers.insert(
                token.header_name().to_string(),
                token.header_value().to_string(),
            );
        }

        // Create reqwest client
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_uri,
            base_path,
            default_headers,
        }
    }

    /// Sends an HTTP request to the Storefront API.
    ///
    /// This method handles:
    /// - Request validation
    /// - URL construction
    /// - Header merging
    /// - Response parsing
    /// - Retry logic for 429 and 500 responses
    pub(super) async fn request(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        // Validate request first
        request.verify()?;

        // Build full URL
        let url = format!("{}{}/{}", self.base_uri, self.base_path, request.path);

        // Merge headers
        let mut headers = self.default_headers.clone();
        if let Some(body_type) = &request.body_type {
            headers.insert(
                "Content-Type".to_string(),
                body_type.as_content_type().to_string(),
            );
        }
        if let Some(extra) = &request.extra_headers {
            for (key, value) in extra {
                headers.insert(key.clone(), value.clone());
            }
        }

        // Retry loop
        let mut tries: u32 = 0;
        loop {
            tries += 1;

            // Build the reqwest request
            let mut req_builder = match request.http_method {
                crate::clients::http_request::HttpMethod::Get => self.client.get(&url),
                crate::clients::http_request::HttpMethod::Post => self.client.post(&url),
                crate::clients::http_request::HttpMethod::Put => self.client.put(&url),
                crate::clients::http_request::HttpMethod::Delete => self.client.delete(&url),
            };

            // Add headers
            for (key, value) in &headers {
                req_builder = req_builder.header(key, value);
            }

            // Add query params
            if let Some(query) = &request.query {
                req_builder = req_builder.query(query);
            }

            // Add body
            if let Some(body) = &request.body {
                req_builder = req_builder.body(body.to_string());
            }

            // Send request
            let res = req_builder.send().await?;

            // Parse response
            let code = res.status().as_u16();
            let res_headers = Self::parse_response_headers(res.headers());
            let body_text = res.text().await.unwrap_or_default();

            // Parse body as JSON
            let body = if body_text.is_empty() {
                serde_json::json!({})
            } else {
                serde_json::from_str(&body_text).unwrap_or_else(|_| {
                    // For 5xx errors, return raw body as string value
                    if code >= 500 {
                        serde_json::json!({ "raw_body": body_text })
                    } else {
                        serde_json::json!({})
                    }
                })
            };

            let response = HttpResponse::new(code, res_headers, body);

            // Check if response is OK
            if response.is_ok() {
                return Ok(response);
            }

            // Build error message
            let error_message = Self::serialize_error(&response);

            // Check if we should retry
            let should_retry = code == 429 || code == 500;
            if !should_retry {
                return Err(HttpError::Response(HttpResponseError {
                    code,
                    message: error_message,
                    error_reference: response.request_id().map(String::from),
                }));
            }

            // Check if we've exhausted retries
            if tries >= request.tries {
                if request.tries == 1 {
                    return Err(HttpError::Response(HttpResponseError {
                        code,
                        message: error_message,
                        error_reference: response.request_id().map(String::from),
                    }));
                }
                return Err(HttpError::MaxRetries(MaxHttpRetriesExceededError {
                    code,
                    tries: request.tries,
                    message: error_message,
                    error_reference: response.request_id().map(String::from),
                }));
            }

            // Calculate retry delay
            let delay = Self::calculate_retry_delay(&response, code);
            tokio::time::sleep(delay).await;
        }
    }

    /// Parses response headers into a `HashMap`.
    fn parse_response_headers(
        headers: &reqwest::header::HeaderMap,
    ) -> HashMap<String, Vec<String>> {
        let mut result: HashMap<String, Vec<String>> = HashMap::new();
        for (name, value) in headers {
            let key = name.as_str().to_lowercase();
            let value = value.to_str().unwrap_or_default().to_string();
            result.entry(key).or_default().push(value);
        }
        result
    }

    /// Calculates the retry delay based on response and status code.
    fn calculate_retry_delay(response: &HttpResponse, status: u16) -> std::time::Duration {
        // For 429: use Retry-After if present, otherwise fixed delay
        // For 500: always use fixed delay (ignore Retry-After)
        if status == 429 {
            if let Some(retry_after) = response.retry_request_after {
                return std::time::Duration::from_secs_f64(retry_after);
            }
        }
        std::time::Duration::from_secs(RETRY_WAIT_TIME)
    }

    /// Serializes error response to JSON format.
    fn serialize_error(response: &HttpResponse) -> String {
        let mut error_body = serde_json::Map::new();

        if let Some(errors) = response.body.get("errors") {
            error_body.insert("errors".to_string(), errors.clone());
        }
        if let Some(error) = response.body.get("error") {
            error_body.insert("error".to_string(), error.clone());
        }
        if response.body.get("error").is_some() {
            if let Some(desc) = response.body.get("error_description") {
                error_body.insert("error_description".to_string(), desc.clone());
            }
        }

        if let Some(request_id) = response.request_id() {
            error_body.insert(
                "error_reference".to_string(),
                serde_json::json!(format!(
                    "If you report this error, please include this id: {request_id}."
                )),
            );
        }

        serde_json::to_string(&error_body).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
impl StorefrontHttpClient {
    /// Returns the base URI for this client (test helper).
    fn base_uri(&self) -> &str {
        &self.base_uri
    }

    /// Returns the base path for this client (test helper).
    fn base_path(&self) -> &str {
        &self.base_path
    }

    /// Returns the default headers for this client (test helper).
    fn default_headers(&self) -> &HashMap<String, String> {
        &self.default_headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Base URI Construction Tests ===

    #[test]
    fn test_base_uri_construction_from_shop_domain() {
        let shop = ShopDomain::new("my-store").unwrap();
        let client = StorefrontHttpClient::new(&shop, None, None, &ApiVersion::V2024_10);

        assert_eq!(client.base_uri(), "https://my-store.myshopify.com");
    }

    // === Base Path Construction Tests ===

    #[test]
    fn test_base_path_is_api_not_admin_api() {
        let shop = ShopDomain::new("my-store").unwrap();
        let client = StorefrontHttpClient::new(&shop, None, None, &ApiVersion::V2024_10);

        // Storefront API uses /api/{version}, NOT /admin/api/{version}
        assert_eq!(client.base_path(), "/api/2024-10");
        assert!(!client.base_path().contains("admin"));
    }

    #[test]
    fn test_base_path_with_different_api_versions() {
        let shop = ShopDomain::new("my-store").unwrap();

        let client_2024_10 = StorefrontHttpClient::new(&shop, None, None, &ApiVersion::V2024_10);
        assert_eq!(client_2024_10.base_path(), "/api/2024-10");

        let client_2024_07 = StorefrontHttpClient::new(&shop, None, None, &ApiVersion::V2024_07);
        assert_eq!(client_2024_07.base_path(), "/api/2024-07");

        let client_latest = StorefrontHttpClient::new(&shop, None, None, &ApiVersion::latest());
        assert!(client_latest.base_path().starts_with("/api/"));
    }

    // === Header Tests ===

    #[test]
    fn test_user_agent_header_format() {
        let shop = ShopDomain::new("my-store").unwrap();
        let client = StorefrontHttpClient::new(&shop, None, None, &ApiVersion::V2024_10);

        let user_agent = client.default_headers().get("User-Agent").unwrap();
        assert!(user_agent.contains("Shopify API Library v"));
        assert!(user_agent.contains("Rust"));
    }

    #[test]
    fn test_accept_header_is_json() {
        let shop = ShopDomain::new("my-store").unwrap();
        let client = StorefrontHttpClient::new(&shop, None, None, &ApiVersion::V2024_10);

        assert_eq!(
            client.default_headers().get("Accept"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_no_admin_api_access_token_header() {
        let shop = ShopDomain::new("my-store").unwrap();
        let public_token = StorefrontToken::Public("test-token".to_string());
        let client =
            StorefrontHttpClient::new(&shop, Some(&public_token), None, &ApiVersion::V2024_10);

        // Should NOT have X-Shopify-Access-Token (that's Admin API)
        assert!(client
            .default_headers()
            .get("X-Shopify-Access-Token")
            .is_none());
    }

    #[test]
    fn test_public_token_header_is_set() {
        let shop = ShopDomain::new("my-store").unwrap();
        let public_token = StorefrontToken::Public("my-public-token".to_string());
        let client =
            StorefrontHttpClient::new(&shop, Some(&public_token), None, &ApiVersion::V2024_10);

        assert_eq!(
            client
                .default_headers()
                .get("X-Shopify-Storefront-Access-Token"),
            Some(&"my-public-token".to_string())
        );
    }

    #[test]
    fn test_private_token_header_is_set() {
        let shop = ShopDomain::new("my-store").unwrap();
        let private_token = StorefrontToken::Private("my-private-token".to_string());
        let client =
            StorefrontHttpClient::new(&shop, Some(&private_token), None, &ApiVersion::V2024_10);

        assert_eq!(
            client
                .default_headers()
                .get("Shopify-Storefront-Private-Token"),
            Some(&"my-private-token".to_string())
        );
    }

    #[test]
    fn test_tokenless_access_no_auth_header() {
        let shop = ShopDomain::new("my-store").unwrap();
        let client = StorefrontHttpClient::new(&shop, None, None, &ApiVersion::V2024_10);

        // No token headers should be present
        assert!(client
            .default_headers()
            .get("X-Shopify-Storefront-Access-Token")
            .is_none());
        assert!(client
            .default_headers()
            .get("Shopify-Storefront-Private-Token")
            .is_none());
        assert!(client
            .default_headers()
            .get("X-Shopify-Access-Token")
            .is_none());
    }

    // === Thread Safety Tests ===

    #[test]
    fn test_storefront_http_client_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StorefrontHttpClient>();
    }

    // === Config Tests ===

    #[test]
    fn test_user_agent_with_prefix() {
        use crate::config::{ApiKey, ApiSecretKey};

        let shop = ShopDomain::new("my-store").unwrap();
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .user_agent_prefix("MyApp/1.0")
            .build()
            .unwrap();

        let client = StorefrontHttpClient::new(&shop, None, Some(&config), &ApiVersion::V2024_10);

        let user_agent = client.default_headers().get("User-Agent").unwrap();
        assert!(user_agent.starts_with("MyApp/1.0 | "));
        assert!(user_agent.contains("Shopify API Library"));
    }
}

//! HTTP client for Shopify API communication.
//!
//! This module provides the [`HttpClient`] type for making authenticated
//! requests to the Shopify API with automatic retry handling.

use std::collections::HashMap;

use crate::auth::Session;
use crate::clients::errors::{HttpError, HttpResponseError, MaxHttpRetriesExceededError};
use crate::clients::http_request::HttpRequest;
use crate::clients::http_response::HttpResponse;
use crate::config::ShopifyConfig;

/// Fixed retry wait time in seconds (matching Ruby SDK).
pub const RETRY_WAIT_TIME: u64 = 1;

/// SDK version from Cargo.toml.
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// HTTP client for making requests to the Shopify API.
///
/// The client handles:
/// - Base URI construction from session shop domain or `api_host`
/// - Default headers including User-Agent and access token
/// - Automatic retry logic for 429 and 500 responses
/// - Shopify-specific header parsing
///
/// # Thread Safety
///
/// `HttpClient` is `Send + Sync`, making it safe to share across async tasks.
///
/// # Example
///
/// ```rust,ignore
/// use shopify_sdk::{HttpClient, HttpRequest, HttpMethod, Session, ShopDomain};
///
/// let session = Session::new(
///     "session-id".to_string(),
///     ShopDomain::new("my-store").unwrap(),
///     "access-token".to_string(),
///     "read_products".parse().unwrap(),
///     false,
///     None,
/// );
///
/// let client = HttpClient::new("/admin/api/2024-10", &session, None);
///
/// let request = HttpRequest::builder(HttpMethod::Get, "products.json")
///     .build()
///     .unwrap();
///
/// let response = client.request(request).await?;
/// ```
#[derive(Debug)]
pub struct HttpClient {
    /// The internal reqwest HTTP client.
    client: reqwest::Client,
    /// Base URI (e.g., `https://my-store.myshopify.com`).
    base_uri: String,
    /// Base path (e.g., "/admin/api/2024-10").
    base_path: String,
    /// Default headers to include in all requests.
    default_headers: HashMap<String, String>,
}

// Verify HttpClient is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<HttpClient>();
};

impl HttpClient {
    /// Creates a new HTTP client for the given session.
    ///
    /// # Arguments
    ///
    /// * `base_path` - The base path for API requests (e.g., "/admin/api/2024-10")
    /// * `session` - The session providing shop domain and access token
    /// * `config` - Optional configuration for `api_host` and `user_agent_prefix`
    ///
    /// # Panics
    ///
    /// Panics if the underlying reqwest client cannot be created. This should
    /// only happen in extremely unusual circumstances (e.g., TLS initialization failure).
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::{Session, ShopDomain, AuthScopes};
    /// use shopify_sdk::clients::HttpClient;
    ///
    /// let session = Session::new(
    ///     "session-id".to_string(),
    ///     ShopDomain::new("my-store").unwrap(),
    ///     "access-token".to_string(),
    ///     AuthScopes::new(),
    ///     false,
    ///     None,
    /// );
    ///
    /// let client = HttpClient::new("/admin/api/2024-10", &session, None);
    /// ```
    #[must_use]
    pub fn new(
        base_path: impl Into<String>,
        session: &Session,
        config: Option<&ShopifyConfig>,
    ) -> Self {
        let base_path = base_path.into();

        // Determine base URI - use api_host if configured, otherwise session.shop
        let api_host = config.and_then(|c| c.host());
        let default_shop_uri = || format!("https://{}", session.shop.as_ref());
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
            default_headers.insert("Host".to_string(), session.shop.as_ref().to_string());
        }

        // Add access token header if present
        if !session.access_token.is_empty() {
            default_headers.insert(
                "X-Shopify-Access-Token".to_string(),
                session.access_token.clone(),
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

    /// Returns the base URI for this client.
    #[must_use]
    pub fn base_uri(&self) -> &str {
        &self.base_uri
    }

    /// Returns the base path for this client.
    #[must_use]
    pub fn base_path(&self) -> &str {
        &self.base_path
    }

    /// Returns the default headers for this client.
    #[must_use]
    pub const fn default_headers(&self) -> &HashMap<String, String> {
        &self.default_headers
    }

    /// Sends an HTTP request to the Shopify API.
    ///
    /// This method handles:
    /// - Request validation
    /// - URL construction
    /// - Header merging
    /// - Response parsing
    /// - Retry logic for 429 and 500 responses
    /// - Deprecation warning logging
    ///
    /// # Errors
    ///
    /// Returns [`HttpError`] if:
    /// - Request validation fails (`InvalidRequest`)
    /// - Network error occurs (`Network`)
    /// - Non-2xx response received (`Response`)
    /// - Max retries exceeded (`MaxRetries`)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = HttpRequest::builder(HttpMethod::Get, "products.json")
    ///     .tries(3) // Enable retries
    ///     .build()
    ///     .unwrap();
    ///
    /// let response = client.request(request).await?;
    /// if response.is_ok() {
    ///     println!("Products: {}", response.body);
    /// }
    /// ```
    pub async fn request(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
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

            // Log deprecation warning if present
            if let Some(reason) = response.deprecation_reason() {
                tracing::warn!(
                    "Deprecated request to Shopify API at {}, received reason: {}",
                    request.path,
                    reason
                );
            }

            // Check if response is OK
            if response.is_ok() {
                return Ok(response);
            }

            // Build error message (matching Ruby SDK format)
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

    /// Serializes error response to JSON format (matching Ruby SDK).
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
mod tests {
    use super::*;
    use crate::auth::AuthScopes;
    use crate::config::{ApiKey, ApiSecretKey, ShopDomain};

    fn create_test_session() -> Session {
        Session::new(
            "test-session".to_string(),
            ShopDomain::new("test-shop").unwrap(),
            "test-access-token".to_string(),
            AuthScopes::new(),
            false,
            None,
        )
    }

    #[test]
    fn test_client_construction_with_session() {
        let session = create_test_session();
        let client = HttpClient::new("/admin/api/2024-10", &session, None);

        assert_eq!(client.base_uri(), "https://test-shop.myshopify.com");
        assert_eq!(client.base_path(), "/admin/api/2024-10");
    }

    #[test]
    fn test_user_agent_header_format() {
        let session = create_test_session();
        let client = HttpClient::new("/admin/api/2024-10", &session, None);

        let user_agent = client.default_headers().get("User-Agent").unwrap();
        assert!(user_agent.contains("Shopify API Library v"));
        assert!(user_agent.contains("Rust"));
    }

    #[test]
    fn test_access_token_header_injection() {
        let session = create_test_session();
        let client = HttpClient::new("/admin/api/2024-10", &session, None);

        assert_eq!(
            client.default_headers().get("X-Shopify-Access-Token"),
            Some(&"test-access-token".to_string())
        );
    }

    #[test]
    fn test_no_access_token_header_when_empty() {
        let session = Session::new(
            "test-session".to_string(),
            ShopDomain::new("test-shop").unwrap(),
            String::new(), // Empty access token
            AuthScopes::new(),
            false,
            None,
        );
        let client = HttpClient::new("/admin/api/2024-10", &session, None);

        assert!(client
            .default_headers()
            .get("X-Shopify-Access-Token")
            .is_none());
    }

    #[test]
    fn test_accept_header_is_json() {
        let session = create_test_session();
        let client = HttpClient::new("/admin/api/2024-10", &session, None);

        assert_eq!(
            client.default_headers().get("Accept"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_client_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HttpClient>();
    }

    #[test]
    fn test_base_uri_with_shop_domain() {
        let session = create_test_session();
        let client = HttpClient::new("/admin/api/2024-10", &session, None);

        assert_eq!(client.base_uri(), "https://test-shop.myshopify.com");
    }

    #[test]
    fn test_user_agent_with_prefix() {
        let session = create_test_session();
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .user_agent_prefix("MyApp/1.0")
            .build()
            .unwrap();

        let client = HttpClient::new("/admin/api/2024-10", &session, Some(&config));

        let user_agent = client.default_headers().get("User-Agent").unwrap();
        assert!(user_agent.starts_with("MyApp/1.0 | "));
        assert!(user_agent.contains("Shopify API Library"));
    }
}

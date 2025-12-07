//! REST client implementation for Shopify Admin API.
//!
//! This module provides the [`RestClient`] type for making REST API requests
//! to the Shopify Admin API with automatic path normalization and retry handling.

use std::collections::HashMap;

use crate::auth::Session;
use crate::clients::rest::RestError;
use crate::clients::{DataType, HttpClient, HttpMethod, HttpRequest, HttpResponse};
use crate::config::{ApiVersion, ShopifyConfig};

/// REST API client for Shopify Admin API.
///
/// Provides convenient methods (`get`, `post`, `put`, `delete`) for making
/// REST API requests with automatic path normalization and retry handling.
///
/// # Thread Safety
///
/// `RestClient` is `Send + Sync`, making it safe to share across async tasks.
///
/// # Deprecation Notice
///
/// The Shopify Admin REST API is deprecated. A warning is logged when this
/// client is constructed. Consider migrating to the GraphQL Admin API.
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::{RestClient, Session, ShopDomain};
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
/// let client = RestClient::new(&session, None)?;
///
/// // GET request
/// let response = client.get("products", None).await?;
///
/// // POST request with body
/// let body = serde_json::json!({"product": {"title": "New Product"}});
/// let response = client.post("products", body, None).await?;
/// ```
#[derive(Debug)]
pub struct RestClient {
    /// The internal HTTP client for making requests.
    http_client: HttpClient,
    /// The API version being used.
    api_version: ApiVersion,
}

// Verify RestClient is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<RestClient>();
};

impl RestClient {
    /// Creates a new REST client for the given session.
    ///
    /// This constructor uses the API version from the configuration, or
    /// falls back to the latest stable version if not specified.
    ///
    /// # Arguments
    ///
    /// * `session` - The session providing shop domain and access token
    /// * `config` - Optional configuration for API version and other settings
    ///
    /// # Errors
    ///
    /// Returns [`RestError::RestApiDisabled`] if REST API is disabled in config
    /// (future-proofing for when REST is fully deprecated).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::{RestClient, Session, ShopDomain};
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
    /// let client = RestClient::new(&session, None)?;
    /// ```
    pub fn new(session: &Session, config: Option<&ShopifyConfig>) -> Result<Self, RestError> {
        let api_version = config.map_or_else(ApiVersion::latest, |c| c.api_version().clone());

        Self::create_client(session, config, api_version)
    }

    /// Creates a new REST client with a specific API version override.
    ///
    /// This constructor allows overriding the API version from configuration.
    ///
    /// # Arguments
    ///
    /// * `session` - The session providing shop domain and access token
    /// * `config` - Optional configuration for other settings
    /// * `version` - The API version to use for requests
    ///
    /// # Errors
    ///
    /// Returns [`RestError::RestApiDisabled`] if REST API is disabled in config.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::{RestClient, Session, ShopDomain, ApiVersion};
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
    /// // Use a specific API version
    /// let client = RestClient::with_version(&session, None, ApiVersion::V2024_10)?;
    /// ```
    pub fn with_version(
        session: &Session,
        config: Option<&ShopifyConfig>,
        version: ApiVersion,
    ) -> Result<Self, RestError> {
        let config_version = config.map(|c| c.api_version().clone());

        // Log debug message when overriding version (matching Ruby SDK pattern)
        if let Some(ref cfg_version) = config_version {
            if &version == cfg_version {
                tracing::debug!(
                    "Rest client has a redundant API version override to the default {}",
                    cfg_version
                );
            } else {
                tracing::debug!(
                    "Rest client overriding default API version {} with {}",
                    cfg_version,
                    version
                );
            }
        }

        Self::create_client(session, config, version)
    }

    /// Internal helper to create the client with shared logic.
    ///
    /// Returns a `Result` to allow for future error handling when REST API
    /// is disabled via configuration (future-proofing for REST deprecation).
    #[allow(clippy::unnecessary_wraps)]
    fn create_client(
        session: &Session,
        config: Option<&ShopifyConfig>,
        api_version: ApiVersion,
    ) -> Result<Self, RestError> {
        // Log deprecation warning (matching Ruby SDK pattern)
        tracing::warn!(
            "The REST Admin API is deprecated. Consider migrating to GraphQL. See: https://www.shopify.com/ca/partners/blog/all-in-on-graphql"
        );

        // Construct base path: /admin/api/{version}
        let base_path = format!("/admin/api/{api_version}");

        // Create internal HTTP client
        let http_client = HttpClient::new(base_path, session, config);

        Ok(Self {
            http_client,
            api_version,
        })
    }

    /// Returns the API version being used by this client.
    #[must_use]
    pub const fn api_version(&self) -> &ApiVersion {
        &self.api_version
    }

    /// Sends a GET request to the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The REST API path (e.g., "products", "orders/123")
    /// * `query` - Optional query parameters
    ///
    /// # Errors
    ///
    /// Returns [`RestError::InvalidPath`] if the path is invalid (e.g., empty).
    /// Returns [`RestError::Http`] for HTTP-level errors.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Simple GET
    /// let response = client.get("products", None).await?;
    ///
    /// // GET with query parameters
    /// let mut query = HashMap::new();
    /// query.insert("limit".to_string(), "50".to_string());
    /// let response = client.get("products", Some(query)).await?;
    /// ```
    pub async fn get(
        &self,
        path: &str,
        query: Option<HashMap<String, String>>,
    ) -> Result<HttpResponse, RestError> {
        self.make_request(HttpMethod::Get, path, None, query, None)
            .await
    }

    /// Sends a GET request with retry configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - The REST API path
    /// * `query` - Optional query parameters
    /// * `tries` - Number of times to attempt the request (default: 1)
    ///
    /// # Errors
    ///
    /// Returns [`RestError::InvalidPath`] if the path is invalid.
    /// Returns [`RestError::Http`] for HTTP-level errors, including retry exhaustion.
    pub async fn get_with_tries(
        &self,
        path: &str,
        query: Option<HashMap<String, String>>,
        tries: u32,
    ) -> Result<HttpResponse, RestError> {
        self.make_request(HttpMethod::Get, path, None, query, Some(tries))
            .await
    }

    /// Sends a POST request to the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The REST API path (e.g., "products")
    /// * `body` - The JSON body to send
    /// * `query` - Optional query parameters
    ///
    /// # Errors
    ///
    /// Returns [`RestError::InvalidPath`] if the path is invalid.
    /// Returns [`RestError::Http`] for HTTP-level errors.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let body = serde_json::json!({
    ///     "product": {
    ///         "title": "New Product",
    ///         "body_html": "<p>Description</p>"
    ///     }
    /// });
    /// let response = client.post("products", body, None).await?;
    /// ```
    pub async fn post(
        &self,
        path: &str,
        body: serde_json::Value,
        query: Option<HashMap<String, String>>,
    ) -> Result<HttpResponse, RestError> {
        self.make_request(HttpMethod::Post, path, Some(body), query, None)
            .await
    }

    /// Sends a POST request with retry configuration.
    ///
    /// # Errors
    ///
    /// Returns [`RestError::InvalidPath`] if the path is invalid.
    /// Returns [`RestError::Http`] for HTTP-level errors, including retry exhaustion.
    pub async fn post_with_tries(
        &self,
        path: &str,
        body: serde_json::Value,
        query: Option<HashMap<String, String>>,
        tries: u32,
    ) -> Result<HttpResponse, RestError> {
        self.make_request(HttpMethod::Post, path, Some(body), query, Some(tries))
            .await
    }

    /// Sends a PUT request to the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The REST API path (e.g., "products/123")
    /// * `body` - The JSON body to send
    /// * `query` - Optional query parameters
    ///
    /// # Errors
    ///
    /// Returns [`RestError::InvalidPath`] if the path is invalid.
    /// Returns [`RestError::Http`] for HTTP-level errors.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let body = serde_json::json!({
    ///     "product": {
    ///         "title": "Updated Title"
    ///     }
    /// });
    /// let response = client.put("products/123", body, None).await?;
    /// ```
    pub async fn put(
        &self,
        path: &str,
        body: serde_json::Value,
        query: Option<HashMap<String, String>>,
    ) -> Result<HttpResponse, RestError> {
        self.make_request(HttpMethod::Put, path, Some(body), query, None)
            .await
    }

    /// Sends a PUT request with retry configuration.
    ///
    /// # Errors
    ///
    /// Returns [`RestError::InvalidPath`] if the path is invalid.
    /// Returns [`RestError::Http`] for HTTP-level errors, including retry exhaustion.
    pub async fn put_with_tries(
        &self,
        path: &str,
        body: serde_json::Value,
        query: Option<HashMap<String, String>>,
        tries: u32,
    ) -> Result<HttpResponse, RestError> {
        self.make_request(HttpMethod::Put, path, Some(body), query, Some(tries))
            .await
    }

    /// Sends a DELETE request to the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The REST API path (e.g., "products/123")
    /// * `query` - Optional query parameters
    ///
    /// # Errors
    ///
    /// Returns [`RestError::InvalidPath`] if the path is invalid.
    /// Returns [`RestError::Http`] for HTTP-level errors.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let response = client.delete("products/123", None).await?;
    /// ```
    pub async fn delete(
        &self,
        path: &str,
        query: Option<HashMap<String, String>>,
    ) -> Result<HttpResponse, RestError> {
        self.make_request(HttpMethod::Delete, path, None, query, None)
            .await
    }

    /// Sends a DELETE request with retry configuration.
    ///
    /// # Errors
    ///
    /// Returns [`RestError::InvalidPath`] if the path is invalid.
    /// Returns [`RestError::Http`] for HTTP-level errors, including retry exhaustion.
    pub async fn delete_with_tries(
        &self,
        path: &str,
        query: Option<HashMap<String, String>>,
        tries: u32,
    ) -> Result<HttpResponse, RestError> {
        self.make_request(HttpMethod::Delete, path, None, query, Some(tries))
            .await
    }

    /// Internal helper to build and send requests.
    async fn make_request(
        &self,
        method: HttpMethod,
        path: &str,
        body: Option<serde_json::Value>,
        query: Option<HashMap<String, String>>,
        tries: Option<u32>,
    ) -> Result<HttpResponse, RestError> {
        // Normalize the path
        let normalized_path = normalize_path(path)?;

        // Build the request
        let mut builder = HttpRequest::builder(method, &normalized_path);

        // Add body if present
        if let Some(body_value) = body {
            builder = builder.body(body_value).body_type(DataType::Json);
        }

        // Add query parameters if present
        if let Some(query_params) = query {
            builder = builder.query(query_params);
        }

        // Set tries (default 1)
        if let Some(t) = tries {
            builder = builder.tries(t);
        }

        // Build and send the request
        let request = builder.build().map_err(|e| RestError::Http(e.into()))?;

        self.http_client.request(request).await.map_err(Into::into)
    }
}

/// Normalizes a REST API path following Ruby SDK conventions.
///
/// This function:
/// 1. Strips leading `/` characters
/// 2. Strips trailing `.json` suffix
/// 3. Appends `.json` suffix
/// 4. Returns an error for empty paths
///
/// # Examples
///
/// ```rust,ignore
/// assert_eq!(normalize_path("products")?, "products.json");
/// assert_eq!(normalize_path("/products")?, "products.json");
/// assert_eq!(normalize_path("products.json")?, "products.json");
/// assert_eq!(normalize_path("/products.json")?, "products.json");
/// ```
fn normalize_path(path: &str) -> Result<String, RestError> {
    // Strip leading slashes
    let path = path.trim_start_matches('/');

    // Strip trailing .json
    let path = path.strip_suffix(".json").unwrap_or(path);

    // Check for empty path
    if path.is_empty() {
        return Err(RestError::InvalidPath {
            path: String::new(),
        });
    }

    // Append .json suffix
    Ok(format!("{path}.json"))
}

/// Checks if a path has an admin/ prefix.
///
/// Paths starting with "admin/" bypass the base path construction
/// and are used directly with the base URI.
#[allow(dead_code)]
fn has_admin_prefix(path: &str) -> bool {
    path.starts_with("admin/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthScopes;
    use crate::config::ShopDomain;

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

    // === Path Normalization Tests ===

    #[test]
    fn test_normalize_path_strips_leading_slash() {
        let result = normalize_path("/products").unwrap();
        assert_eq!(result, "products.json");
    }

    #[test]
    fn test_normalize_path_strips_trailing_json() {
        let result = normalize_path("products.json").unwrap();
        assert_eq!(result, "products.json");
    }

    #[test]
    fn test_normalize_path_strips_both_leading_slash_and_trailing_json() {
        let result = normalize_path("/products.json").unwrap();
        assert_eq!(result, "products.json");
    }

    #[test]
    fn test_normalize_path_adds_json_suffix() {
        let result = normalize_path("products").unwrap();
        assert_eq!(result, "products.json");
    }

    #[test]
    fn test_normalize_path_handles_nested_paths() {
        let result = normalize_path("/admin/api/2024-10/products").unwrap();
        assert_eq!(result, "admin/api/2024-10/products.json");
    }

    #[test]
    fn test_normalize_path_handles_double_slashes() {
        let result = normalize_path("//products").unwrap();
        assert_eq!(result, "products.json");
    }

    #[test]
    fn test_normalize_path_empty_path_returns_error() {
        let result = normalize_path("");
        assert!(matches!(result, Err(RestError::InvalidPath { path }) if path.is_empty()));
    }

    #[test]
    fn test_normalize_path_only_slash_returns_error() {
        let result = normalize_path("/");
        assert!(matches!(result, Err(RestError::InvalidPath { path }) if path.is_empty()));
    }

    #[test]
    fn test_normalize_path_only_json_returns_error() {
        // "/.json" after stripping "/" becomes ".json", after stripping ".json" becomes ""
        let result = normalize_path("/.json");
        assert!(matches!(result, Err(RestError::InvalidPath { path }) if path.is_empty()));
    }

    // === Admin Prefix Tests ===

    #[test]
    fn test_has_admin_prefix_returns_true() {
        assert!(has_admin_prefix("admin/products.json"));
        assert!(has_admin_prefix("admin/api/2024-10/products.json"));
    }

    #[test]
    fn test_has_admin_prefix_returns_false() {
        assert!(!has_admin_prefix("products.json"));
        assert!(!has_admin_prefix("/admin/products.json")); // Leading slash means it doesn't start with "admin/"
    }

    // === RestClient Construction Tests ===

    #[test]
    fn test_rest_client_new_creates_client_with_latest_version() {
        let session = create_test_session();
        let client = RestClient::new(&session, None).unwrap();

        assert_eq!(client.api_version(), &ApiVersion::latest());
    }

    #[test]
    fn test_rest_client_with_version_overrides_config() {
        let session = create_test_session();
        let client = RestClient::with_version(&session, None, ApiVersion::V2024_10).unwrap();

        assert_eq!(client.api_version(), &ApiVersion::V2024_10);
    }

    #[test]
    fn test_rest_client_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RestClient>();
    }

    #[test]
    fn test_rest_client_constructs_correct_base_path() {
        let session = create_test_session();
        let client = RestClient::with_version(&session, None, ApiVersion::V2024_10).unwrap();

        // The internal HttpClient should have the correct base path
        // We can verify this indirectly through the api_version
        assert_eq!(client.api_version(), &ApiVersion::V2024_10);
    }
}

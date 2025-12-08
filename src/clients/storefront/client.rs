//! Storefront GraphQL client implementation for Shopify Storefront API.
//!
//! This module provides the [`StorefrontClient`] type for executing GraphQL queries
//! against the Shopify Storefront API.
//!
//! # Storefront API vs Admin API
//!
//! The Storefront API is designed for building custom storefronts and differs
//! from the Admin API in several key ways:
//!
//! - **Endpoint**: Uses `/api/{version}/graphql.json` (no `/admin` prefix)
//! - **Authentication**: Uses storefront access tokens with different headers
//! - **Access Level**: Limited to storefront data (products, collections, cart)
//! - **Tokenless Access**: Supports unauthenticated access for basic features
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::{StorefrontClient, StorefrontToken, ShopDomain};
//! use serde_json::json;
//!
//! // Create a shop domain
//! let shop = ShopDomain::new("my-store").unwrap();
//!
//! // With a public token
//! let token = StorefrontToken::Public("public-access-token".to_string());
//! let client = StorefrontClient::new(&shop, Some(token), None);
//!
//! // Simple query
//! let response = client.query("query { shop { name } }", None, None, None).await?;
//!
//! // Query with variables
//! let response = client.query(
//!     "query GetProduct($handle: String!) { productByHandle(handle: $handle) { title } }",
//!     Some(json!({ "handle": "my-product" })),
//!     None,
//!     None
//! ).await?;
//! ```

use std::collections::HashMap;

use crate::clients::graphql::GraphqlError;
use crate::clients::storefront::storefront_http::StorefrontHttpClient;
use crate::clients::storefront::StorefrontToken;
use crate::clients::{DataType, HttpMethod, HttpRequest, HttpResponse};
use crate::config::{ApiVersion, ShopDomain, ShopifyConfig};

/// GraphQL client for Shopify Storefront API.
///
/// Provides methods (`query`, `query_with_debug`) for executing GraphQL queries
/// against the Storefront API with support for public tokens, private tokens,
/// and tokenless access.
///
/// # Thread Safety
///
/// `StorefrontClient` is `Send + Sync`, making it safe to share across async tasks.
///
/// # Endpoint Format
///
/// The Storefront API uses a different endpoint format than the Admin API:
/// - Storefront: `https://{shop}/api/{version}/graphql.json`
/// - Admin: `https://{shop}/admin/api/{version}/graphql.json`
///
/// # Authentication
///
/// The client supports three authentication modes:
///
/// - **Public token**: Uses `X-Shopify-Storefront-Access-Token` header
/// - **Private token**: Uses `Shopify-Storefront-Private-Token` header
/// - **Tokenless**: No authentication header (limited access)
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::{StorefrontClient, StorefrontToken, ShopDomain};
/// use serde_json::json;
///
/// let shop = ShopDomain::new("my-store").unwrap();
///
/// // Public token access
/// let token = StorefrontToken::Public("public-access-token".to_string());
/// let client = StorefrontClient::new(&shop, Some(token), None);
///
/// // Tokenless access for basic features
/// let client = StorefrontClient::new(&shop, None, None);
///
/// // Query with variables
/// let response = client.query(
///     "query GetProducts($first: Int!) { products(first: $first) { edges { node { title } } } }",
///     Some(json!({ "first": 10 })),
///     None,
///     None
/// ).await?;
///
/// // Check for GraphQL errors (HTTP 200 with errors in body)
/// if let Some(errors) = response.body.get("errors") {
///     println!("GraphQL errors: {}", errors);
/// }
/// ```
#[derive(Debug)]
pub struct StorefrontClient {
    /// The internal HTTP client for making requests.
    http_client: StorefrontHttpClient,
    /// The API version being used.
    api_version: ApiVersion,
}

// Verify StorefrontClient is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<StorefrontClient>();
};

impl StorefrontClient {
    /// Creates a new Storefront client for the given shop.
    ///
    /// This constructor uses the API version from the configuration, or
    /// falls back to the latest stable version if not specified.
    ///
    /// Unlike [`GraphqlClient`](crate::clients::GraphqlClient), this constructor
    /// takes a [`ShopDomain`] directly instead of a [`Session`](crate::auth::Session),
    /// since Storefront API uses storefront tokens rather than admin tokens.
    ///
    /// # Arguments
    ///
    /// * `shop` - The shop domain for endpoint construction
    /// * `token` - Optional storefront access token (`None` for tokenless access)
    /// * `config` - Optional configuration for API version and other settings
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{StorefrontClient, StorefrontToken, ShopDomain};
    ///
    /// let shop = ShopDomain::new("my-store").unwrap();
    ///
    /// // With public token
    /// let token = StorefrontToken::Public("my-token".to_string());
    /// let client = StorefrontClient::new(&shop, Some(token), None);
    ///
    /// // Tokenless access
    /// let client = StorefrontClient::new(&shop, None, None);
    /// ```
    #[must_use]
    #[allow(clippy::needless_pass_by_value)] // Taking ownership is intentional for ergonomic API
    pub fn new(
        shop: &ShopDomain,
        token: Option<StorefrontToken>,
        config: Option<&ShopifyConfig>,
    ) -> Self {
        let api_version = config.map_or_else(ApiVersion::latest, |c| c.api_version().clone());
        Self::create_client(shop, token.as_ref(), config, api_version)
    }

    /// Creates a new Storefront client with a specific API version override.
    ///
    /// This constructor allows overriding the API version from configuration.
    ///
    /// # Arguments
    ///
    /// * `shop` - The shop domain for endpoint construction
    /// * `token` - Optional storefront access token (`None` for tokenless access)
    /// * `config` - Optional configuration for other settings
    /// * `version` - The API version to use for requests
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{StorefrontClient, StorefrontToken, ShopDomain, ApiVersion};
    ///
    /// let shop = ShopDomain::new("my-store").unwrap();
    /// let token = StorefrontToken::Public("my-token".to_string());
    ///
    /// // Use a specific API version
    /// let client = StorefrontClient::with_version(&shop, Some(token), None, ApiVersion::V2024_10);
    /// assert_eq!(client.api_version(), &ApiVersion::V2024_10);
    /// ```
    #[must_use]
    #[allow(clippy::needless_pass_by_value)] // Taking ownership is intentional for ergonomic API
    pub fn with_version(
        shop: &ShopDomain,
        token: Option<StorefrontToken>,
        config: Option<&ShopifyConfig>,
        version: ApiVersion,
    ) -> Self {
        let config_version = config.map(|c| c.api_version().clone());

        // Log debug message when overriding version (matching GraphqlClient pattern)
        if let Some(ref cfg_version) = config_version {
            if &version == cfg_version {
                tracing::debug!(
                    "Storefront client has a redundant API version override to the default {}",
                    cfg_version
                );
            } else {
                tracing::debug!(
                    "Storefront client overriding default API version {} with {}",
                    cfg_version,
                    version
                );
            }
        }

        Self::create_client(shop, token.as_ref(), config, version)
    }

    /// Internal helper to create the client with shared logic.
    fn create_client(
        shop: &ShopDomain,
        token: Option<&StorefrontToken>,
        config: Option<&ShopifyConfig>,
        api_version: ApiVersion,
    ) -> Self {
        // Create internal HTTP client
        let http_client = StorefrontHttpClient::new(shop, token, config, &api_version);

        Self {
            http_client,
            api_version,
        }
    }

    /// Returns the API version being used by this client.
    #[must_use]
    pub const fn api_version(&self) -> &ApiVersion {
        &self.api_version
    }

    /// Executes a GraphQL query against the Storefront API.
    ///
    /// This method sends a POST request to the `graphql.json` endpoint with
    /// the query and optional variables.
    ///
    /// # Arguments
    ///
    /// * `query` - The GraphQL query string
    /// * `variables` - Optional variables for the query
    /// * `headers` - Optional extra headers to include in the request
    /// * `tries` - Optional number of retry attempts (default: 1, no retries)
    ///
    /// # Returns
    ///
    /// Returns the raw [`HttpResponse`] containing:
    /// - `code`: HTTP status code (usually 200 for GraphQL)
    /// - `headers`: Response headers
    /// - `body`: JSON response with `data`, `errors`, and `extensions` fields
    ///
    /// # Errors
    ///
    /// Returns [`GraphqlError::Http`] for HTTP-level errors (network errors,
    /// non-2xx responses, retry exhaustion).
    ///
    /// Note that GraphQL-level errors (user errors, validation errors) are
    /// returned with HTTP 200 status and contained in `response.body["errors"]`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::StorefrontClient;
    /// use serde_json::json;
    ///
    /// // Simple query
    /// let response = client.query(
    ///     "query { shop { name } }",
    ///     None,
    ///     None,
    ///     None
    /// ).await?;
    ///
    /// println!("Shop: {}", response.body["data"]["shop"]["name"]);
    ///
    /// // Query with variables and retries
    /// let response = client.query(
    ///     "query GetProduct($handle: String!) { productByHandle(handle: $handle) { title } }",
    ///     Some(json!({ "handle": "my-product" })),
    ///     None,
    ///     Some(3) // Retry up to 3 times on 429/500
    /// ).await?;
    ///
    /// // Check for GraphQL errors
    /// if let Some(errors) = response.body.get("errors") {
    ///     println!("GraphQL errors: {}", errors);
    /// }
    /// ```
    pub async fn query(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
        tries: Option<u32>,
    ) -> Result<HttpResponse, GraphqlError> {
        self.execute_query(query, variables, headers, tries, false)
            .await
    }

    /// Executes a GraphQL query with debug mode enabled.
    ///
    /// This method is identical to [`query`](Self::query) but appends
    /// `?debug=true` to the request URL, which causes Shopify to include
    /// additional query cost and execution information in the response's
    /// `extensions` field.
    ///
    /// # Arguments
    ///
    /// * `query` - The GraphQL query string
    /// * `variables` - Optional variables for the query
    /// * `headers` - Optional extra headers to include in the request
    /// * `tries` - Optional number of retry attempts (default: 1, no retries)
    ///
    /// # Returns
    ///
    /// Returns the raw [`HttpResponse`] with additional debug information
    /// in `response.body["extensions"]`.
    ///
    /// # Errors
    ///
    /// Returns [`GraphqlError::Http`] for HTTP-level errors.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::StorefrontClient;
    ///
    /// let response = client.query_with_debug(
    ///     "query { products(first: 10) { edges { node { title } } } }",
    ///     None,
    ///     None,
    ///     None
    /// ).await?;
    ///
    /// // Debug info available in extensions
    /// if let Some(extensions) = response.body.get("extensions") {
    ///     println!("Query cost: {}", extensions["cost"]);
    /// }
    /// ```
    pub async fn query_with_debug(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
        tries: Option<u32>,
    ) -> Result<HttpResponse, GraphqlError> {
        self.execute_query(query, variables, headers, tries, true)
            .await
    }

    /// Internal helper to execute a GraphQL query with shared logic.
    async fn execute_query(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
        tries: Option<u32>,
        debug: bool,
    ) -> Result<HttpResponse, GraphqlError> {
        // Construct the request body
        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });

        // Build the request
        let mut builder = HttpRequest::builder(HttpMethod::Post, "graphql.json")
            .body(body)
            .body_type(DataType::Json)
            .tries(tries.unwrap_or(1));

        // Add debug query parameter if requested
        if debug {
            builder = builder.query_param("debug", "true");
        }

        // Add extra headers if provided
        if let Some(extra_headers) = headers {
            builder = builder.extra_headers(extra_headers);
        }

        // Build and execute the request
        let request = builder.build().map_err(|e| GraphqlError::Http(e.into()))?;
        self.http_client.request(request).await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Construction Tests ===

    #[test]
    fn test_storefront_client_new_creates_client_with_latest_version() {
        let shop = ShopDomain::new("test-shop").unwrap();
        let client = StorefrontClient::new(&shop, None, None);

        assert_eq!(client.api_version(), &ApiVersion::latest());
    }

    #[test]
    fn test_storefront_client_new_with_public_token() {
        let shop = ShopDomain::new("test-shop").unwrap();
        let token = StorefrontToken::Public("test-token".to_string());
        let client = StorefrontClient::new(&shop, Some(token), None);

        assert_eq!(client.api_version(), &ApiVersion::latest());
    }

    #[test]
    fn test_storefront_client_new_with_private_token() {
        let shop = ShopDomain::new("test-shop").unwrap();
        let token = StorefrontToken::Private("test-token".to_string());
        let client = StorefrontClient::new(&shop, Some(token), None);

        assert_eq!(client.api_version(), &ApiVersion::latest());
    }

    #[test]
    fn test_storefront_client_new_tokenless() {
        let shop = ShopDomain::new("test-shop").unwrap();
        let client = StorefrontClient::new(&shop, None, None);

        assert_eq!(client.api_version(), &ApiVersion::latest());
    }

    #[test]
    fn test_storefront_client_with_version_overrides_config() {
        let shop = ShopDomain::new("test-shop").unwrap();
        let client = StorefrontClient::with_version(&shop, None, None, ApiVersion::V2024_10);

        assert_eq!(client.api_version(), &ApiVersion::V2024_10);
    }

    #[test]
    fn test_storefront_client_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StorefrontClient>();
    }

    #[test]
    fn test_storefront_client_constructor_is_infallible() {
        let shop = ShopDomain::new("test-shop").unwrap();
        // This test verifies that new() returns Self directly, not Result
        let _client: StorefrontClient = StorefrontClient::new(&shop, None, None);
        // If this compiles, the constructor is infallible
    }

    #[test]
    fn test_storefront_client_with_config_uses_config_version() {
        use crate::config::{ApiKey, ApiSecretKey};

        let shop = ShopDomain::new("test-shop").unwrap();
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .api_version(ApiVersion::V2024_10)
            .build()
            .unwrap();

        let client = StorefrontClient::new(&shop, None, Some(&config));

        assert_eq!(client.api_version(), &ApiVersion::V2024_10);
    }

    #[test]
    fn test_storefront_client_with_version_logs_debug_when_overriding() {
        use crate::config::{ApiKey, ApiSecretKey};

        let shop = ShopDomain::new("test-shop").unwrap();
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .api_version(ApiVersion::V2024_10)
            .build()
            .unwrap();

        // Override with a different version - this should log a debug message
        let client =
            StorefrontClient::with_version(&shop, None, Some(&config), ApiVersion::V2024_07);

        assert_eq!(client.api_version(), &ApiVersion::V2024_07);
    }

    #[test]
    fn test_api_version_accessor() {
        let shop = ShopDomain::new("test-shop").unwrap();

        let client_2024_10 =
            StorefrontClient::with_version(&shop, None, None, ApiVersion::V2024_10);
        let client_2024_07 =
            StorefrontClient::with_version(&shop, None, None, ApiVersion::V2024_07);
        let client_latest = StorefrontClient::new(&shop, None, None);

        assert_eq!(client_2024_10.api_version(), &ApiVersion::V2024_10);
        assert_eq!(client_2024_07.api_version(), &ApiVersion::V2024_07);
        assert_eq!(client_latest.api_version(), &ApiVersion::latest());
    }
}

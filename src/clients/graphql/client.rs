//! GraphQL client implementation for Shopify Admin API.
//!
//! This module provides the [`GraphqlClient`] type for executing GraphQL queries
//! against the Shopify Admin API.

use std::collections::HashMap;

use crate::auth::Session;
use crate::clients::graphql::GraphqlError;
use crate::clients::{DataType, HttpClient, HttpMethod, HttpRequest, HttpResponse};
use crate::config::{ApiVersion, ShopifyConfig};

/// GraphQL API client for Shopify Admin API.
///
/// Provides methods (`query`, `query_with_debug`) for executing GraphQL queries
/// with variable support, custom headers, and retry handling.
///
/// # Thread Safety
///
/// `GraphqlClient` is `Send + Sync`, making it safe to share across async tasks.
///
/// # GraphQL is the Recommended API
///
/// Unlike the REST Admin API, the GraphQL Admin API is Shopify's recommended
/// approach. This client does not log deprecation warnings.
///
/// # Example
///
/// ```rust,ignore
/// use shopify_sdk::{GraphqlClient, Session, ShopDomain};
/// use serde_json::json;
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
/// let client = GraphqlClient::new(&session, None);
///
/// // Simple query
/// let response = client.query("query { shop { name } }", None, None, None).await?;
///
/// // Query with variables
/// let response = client.query(
///     "query GetProduct($id: ID!) { product(id: $id) { title } }",
///     Some(json!({ "id": "gid://shopify/Product/123" })),
///     None,
///     None
/// ).await?;
/// ```
#[derive(Debug)]
pub struct GraphqlClient {
    /// The internal HTTP client for making requests.
    http_client: HttpClient,
    /// The API version being used.
    api_version: ApiVersion,
}

// Verify GraphqlClient is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GraphqlClient>();
};

impl GraphqlClient {
    /// Creates a new GraphQL client for the given session.
    ///
    /// This constructor uses the API version from the configuration, or
    /// falls back to the latest stable version if not specified.
    ///
    /// Unlike [`RestClient`](crate::clients::RestClient), this constructor
    /// is infallible (returns `Self`, not `Result`) and does not log a
    /// deprecation warning since GraphQL is the recommended API.
    ///
    /// # Arguments
    ///
    /// * `session` - The session providing shop domain and access token
    /// * `config` - Optional configuration for API version and other settings
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::{GraphqlClient, Session, ShopDomain};
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
    /// let client = GraphqlClient::new(&session, None);
    /// ```
    #[must_use]
    pub fn new(session: &Session, config: Option<&ShopifyConfig>) -> Self {
        let api_version = config.map_or_else(ApiVersion::latest, |c| c.api_version().clone());
        Self::create_client(session, config, api_version)
    }

    /// Creates a new GraphQL client with a specific API version override.
    ///
    /// This constructor allows overriding the API version from configuration.
    ///
    /// # Arguments
    ///
    /// * `session` - The session providing shop domain and access token
    /// * `config` - Optional configuration for other settings
    /// * `version` - The API version to use for requests
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::{GraphqlClient, Session, ShopDomain, ApiVersion};
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
    /// let client = GraphqlClient::with_version(&session, None, ApiVersion::V2024_10);
    /// ```
    #[must_use]
    pub fn with_version(
        session: &Session,
        config: Option<&ShopifyConfig>,
        version: ApiVersion,
    ) -> Self {
        let config_version = config.map(|c| c.api_version().clone());

        // Log debug message when overriding version (matching RestClient pattern)
        if let Some(ref cfg_version) = config_version {
            if &version == cfg_version {
                tracing::debug!(
                    "GraphQL client has a redundant API version override to the default {}",
                    cfg_version
                );
            } else {
                tracing::debug!(
                    "GraphQL client overriding default API version {} with {}",
                    cfg_version,
                    version
                );
            }
        }

        Self::create_client(session, config, version)
    }

    /// Internal helper to create the client with shared logic.
    fn create_client(
        session: &Session,
        config: Option<&ShopifyConfig>,
        api_version: ApiVersion,
    ) -> Self {
        // Construct base path: /admin/api/{version}
        let base_path = format!("/admin/api/{api_version}");

        // Create internal HTTP client
        let http_client = HttpClient::new(base_path, session, config);

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

    /// Executes a GraphQL query against the Admin API.
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
    /// use shopify_sdk::GraphqlClient;
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
    ///     "query GetProduct($id: ID!) { product(id: $id) { title } }",
    ///     Some(json!({ "id": "gid://shopify/Product/123" })),
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
    /// use shopify_sdk::GraphqlClient;
    ///
    /// let response = client.query_with_debug(
    ///     "query { shop { name } }",
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

    // === Construction Tests ===

    #[test]
    fn test_graphql_client_new_creates_client_with_latest_version() {
        let session = create_test_session();
        let client = GraphqlClient::new(&session, None);

        assert_eq!(client.api_version(), &ApiVersion::latest());
    }

    #[test]
    fn test_graphql_client_with_version_overrides_config() {
        let session = create_test_session();
        let client = GraphqlClient::with_version(&session, None, ApiVersion::V2024_10);

        assert_eq!(client.api_version(), &ApiVersion::V2024_10);
    }

    #[test]
    fn test_graphql_client_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<GraphqlClient>();
    }

    #[test]
    fn test_graphql_client_constructor_is_infallible() {
        let session = create_test_session();
        // This test verifies that new() returns Self directly, not Result
        let _client: GraphqlClient = GraphqlClient::new(&session, None);
        // If this compiles, the constructor is infallible
    }

    #[test]
    fn test_graphql_client_with_config_uses_config_version() {
        use crate::config::{ApiKey, ApiSecretKey};

        let session = create_test_session();
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .api_version(ApiVersion::V2024_10)
            .build()
            .unwrap();

        let client = GraphqlClient::new(&session, Some(&config));

        assert_eq!(client.api_version(), &ApiVersion::V2024_10);
    }

    #[test]
    fn test_graphql_client_with_version_logs_debug_when_overriding() {
        use crate::config::{ApiKey, ApiSecretKey};

        let session = create_test_session();
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("test-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .api_version(ApiVersion::V2024_10)
            .build()
            .unwrap();

        // Override with a different version - this should log a debug message
        let client = GraphqlClient::with_version(&session, Some(&config), ApiVersion::V2024_07);

        assert_eq!(client.api_version(), &ApiVersion::V2024_07);
    }
}

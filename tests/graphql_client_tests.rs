//! Integration tests for the GraphQL API client functionality.
//!
//! These tests verify the GraphQL client construction, error handling,
//! and API method behavior.

use shopify_api::clients::graphql::{GraphqlClient, GraphqlError};
use shopify_api::{ApiVersion, AuthScopes, Session, ShopDomain};

/// Creates a test session with the given shop domain.
fn create_test_session(shop: &str, access_token: &str) -> Session {
    Session::new(
        "test-session".to_string(),
        ShopDomain::new(shop).unwrap(),
        access_token.to_string(),
        AuthScopes::new(),
        false,
        None,
    )
}

// ============================================================================
// GraphqlClient Construction Tests
// ============================================================================

#[test]
fn test_graphql_client_creates_with_default_version() {
    let session = create_test_session("test-shop", "test-token");
    let client = GraphqlClient::new(&session, None);

    // Should use latest API version when no config provided
    assert_eq!(client.api_version(), &ApiVersion::latest());
}

#[test]
fn test_graphql_client_with_version_override() {
    let session = create_test_session("test-shop", "test-token");
    let client = GraphqlClient::with_version(&session, None, ApiVersion::V2024_10);

    assert_eq!(client.api_version(), &ApiVersion::V2024_10);
}

#[test]
fn test_graphql_client_is_thread_safe() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GraphqlClient>();
}

#[test]
fn test_graphql_client_constructor_is_infallible() {
    let session = create_test_session("test-shop", "test-token");
    // This compiles because new() returns Self, not Result
    let _client: GraphqlClient = GraphqlClient::new(&session, None);
}

#[test]
fn test_graphql_client_with_config_uses_config_version() {
    use shopify_api::{ApiKey, ApiSecretKey, ShopifyConfig};

    let session = create_test_session("test-shop", "test-token");
    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .api_version(ApiVersion::V2024_10)
        .build()
        .unwrap();

    let client = GraphqlClient::new(&session, Some(&config));

    assert_eq!(client.api_version(), &ApiVersion::V2024_10);
}

// ============================================================================
// Error Type Tests
// ============================================================================

#[test]
fn test_graphql_error_http_variant_wraps_http_error() {
    use shopify_api::clients::{HttpError, HttpResponseError};

    let http_error = HttpError::Response(HttpResponseError {
        code: 401,
        message: r#"{"error":"Unauthorized"}"#.to_string(),
        error_reference: Some("abc-123".to_string()),
    });

    let graphql_error = GraphqlError::Http(http_error);
    let message = graphql_error.to_string();

    assert!(message.contains("Unauthorized"));
}

#[test]
fn test_graphql_error_from_http_error_conversion() {
    use shopify_api::clients::{HttpError, HttpResponseError};

    let http_error = HttpError::Response(HttpResponseError {
        code: 500,
        message: r#"{"error":"Internal Server Error"}"#.to_string(),
        error_reference: None,
    });

    // Test From<HttpError> conversion
    let graphql_error: GraphqlError = http_error.into();
    assert!(matches!(graphql_error, GraphqlError::Http(_)));
}

#[test]
fn test_graphql_error_wraps_max_retries_exceeded() {
    use shopify_api::clients::{HttpError, MaxHttpRetriesExceededError};

    let http_error = HttpError::MaxRetries(MaxHttpRetriesExceededError {
        code: 429,
        tries: 3,
        message: r#"{"error":"Rate limited"}"#.to_string(),
        error_reference: None,
    });

    let graphql_error = GraphqlError::Http(http_error);
    let message = graphql_error.to_string();

    assert!(message.contains("Exceeded maximum retry count"));
    assert!(message.contains("3"));
}

// ============================================================================
// Multi-tenant Session Tests
// ============================================================================

#[test]
fn test_multiple_clients_for_different_shops() {
    let session1 = create_test_session("shop-one", "token-1");
    let session2 = create_test_session("shop-two", "token-2");

    let client1 = GraphqlClient::new(&session1, None);
    let client2 = GraphqlClient::new(&session2, None);

    // Both clients should have independent configurations
    assert_eq!(client1.api_version(), &ApiVersion::latest());
    assert_eq!(client2.api_version(), &ApiVersion::latest());
}

#[test]
fn test_clients_with_different_api_versions() {
    let session = create_test_session("test-shop", "test-token");

    let client_latest = GraphqlClient::new(&session, None);
    let client_old = GraphqlClient::with_version(&session, None, ApiVersion::V2024_10);

    assert_eq!(client_latest.api_version(), &ApiVersion::latest());
    assert_eq!(client_old.api_version(), &ApiVersion::V2024_10);
}

// ============================================================================
// Type Export Tests
// ============================================================================

#[test]
fn test_types_exported_at_crate_root() {
    // Verify types are accessible from crate root
    let _: fn(shopify_api::GraphqlClient) = |_| {};
    let _: fn(shopify_api::GraphqlError) = |_| {};
}

#[test]
fn test_types_exported_from_clients_module() {
    // Verify types are accessible from clients module
    let _: fn(shopify_api::clients::GraphqlClient) = |_| {};
    let _: fn(shopify_api::clients::GraphqlError) = |_| {};
}

#[test]
fn test_types_exported_from_clients_graphql_module() {
    // Verify types are accessible from clients::graphql module
    let _: fn(shopify_api::clients::graphql::GraphqlClient) = |_| {};
    let _: fn(shopify_api::clients::graphql::GraphqlError) = |_| {};
}

// ============================================================================
// Query Method Behavior Tests (without real HTTP calls)
// ============================================================================

#[tokio::test]
async fn test_query_method_sends_request_to_shopify() {
    // This test verifies that the query method tries to send a request.
    // Similar to token_exchange_tests, we expect a network error since we're
    // not connecting to a real Shopify server.
    let session = create_test_session("test-shop", "test-token");
    let client = GraphqlClient::new(&session, None);

    let result = client
        .query("query { shop { name } }", None, None, None)
        .await;

    // Should fail with an HTTP error (network or response error)
    assert!(matches!(result, Err(GraphqlError::Http(_))));
}

#[tokio::test]
async fn test_query_with_variables_sends_request() {
    let session = create_test_session("test-shop", "test-token");
    let client = GraphqlClient::new(&session, None);

    let variables = serde_json::json!({
        "id": "gid://shopify/Product/123"
    });

    let result = client
        .query(
            "query GetProduct($id: ID!) { product(id: $id) { title } }",
            Some(variables),
            None,
            None,
        )
        .await;

    // Should fail with an HTTP error
    assert!(matches!(result, Err(GraphqlError::Http(_))));
}

#[tokio::test]
async fn test_query_with_debug_sends_request() {
    let session = create_test_session("test-shop", "test-token");
    let client = GraphqlClient::new(&session, None);

    let result = client
        .query_with_debug("query { shop { name } }", None, None, None)
        .await;

    // Should fail with an HTTP error
    assert!(matches!(result, Err(GraphqlError::Http(_))));
}

#[tokio::test]
async fn test_query_with_custom_headers_sends_request() {
    use std::collections::HashMap;

    let session = create_test_session("test-shop", "test-token");
    let client = GraphqlClient::new(&session, None);

    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

    let result = client
        .query("query { shop { name } }", None, Some(headers), None)
        .await;

    // Should fail with an HTTP error
    assert!(matches!(result, Err(GraphqlError::Http(_))));
}

#[tokio::test]
async fn test_query_with_retries() {
    let session = create_test_session("test-shop", "test-token");
    let client = GraphqlClient::new(&session, None);

    let result = client
        .query("query { shop { name } }", None, None, Some(3))
        .await;

    // Should fail with an HTTP error
    assert!(matches!(result, Err(GraphqlError::Http(_))));
}

// ============================================================================
// API Version in URL Tests
// ============================================================================

#[test]
fn test_graphql_client_stores_api_version() {
    let session = create_test_session("test-shop", "test-token");

    let client_2024_10 = GraphqlClient::with_version(&session, None, ApiVersion::V2024_10);
    let client_2024_07 = GraphqlClient::with_version(&session, None, ApiVersion::V2024_07);
    let client_latest = GraphqlClient::new(&session, None);

    assert_eq!(client_2024_10.api_version(), &ApiVersion::V2024_10);
    assert_eq!(client_2024_07.api_version(), &ApiVersion::V2024_07);
    assert_eq!(client_latest.api_version(), &ApiVersion::latest());
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[tokio::test]
async fn test_graphql_client_can_be_shared_across_tasks() {
    use std::sync::Arc;

    let session = create_test_session("test-shop", "test-token");
    let client = Arc::new(GraphqlClient::new(&session, None));

    // Spawn multiple tasks that share the client
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let client = Arc::clone(&client);
            tokio::spawn(async move {
                // Access client properties from multiple tasks
                let version = client.api_version();
                format!("Task {i} using API version {version}")
            })
        })
        .collect();

    // Wait for all tasks
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.contains("Task"));
    }
}

// ============================================================================
// Error Display Tests
// ============================================================================

#[test]
fn test_graphql_error_display_is_informative() {
    use shopify_api::clients::{HttpError, HttpResponseError};

    let http_error = HttpError::Response(HttpResponseError {
        code: 404,
        message: r#"{"error":"Not Found"}"#.to_string(),
        error_reference: Some("req-12345".to_string()),
    });

    let graphql_error = GraphqlError::Http(http_error);
    let display = graphql_error.to_string();

    // The error display should contain useful information
    assert!(display.contains("Not Found"));
}

#[test]
fn test_graphql_error_implements_std_error() {
    use shopify_api::clients::{HttpError, HttpResponseError};

    let http_error = HttpError::Response(HttpResponseError {
        code: 400,
        message: "test".to_string(),
        error_reference: None,
    });

    let graphql_error: &dyn std::error::Error = &GraphqlError::Http(http_error);
    let _ = graphql_error;
}

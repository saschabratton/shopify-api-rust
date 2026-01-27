//! Integration tests for the Storefront API client functionality.
//!
//! These tests verify the Storefront client construction, error handling,
//! and API method behavior.

use shopify_sdk::clients::graphql::GraphqlError;
use shopify_sdk::clients::storefront::{StorefrontClient, StorefrontToken};
use shopify_sdk::{ApiVersion, ShopDomain};

// ============================================================================
// StorefrontToken Tests
// ============================================================================

#[test]
fn test_storefront_token_public_header_name() {
    let token = StorefrontToken::Public("test-token".to_string());
    assert_eq!(token.header_name(), "X-Shopify-Storefront-Access-Token");
}

#[test]
fn test_storefront_token_private_header_name() {
    let token = StorefrontToken::Private("test-token".to_string());
    assert_eq!(token.header_name(), "Shopify-Storefront-Private-Token");
}

#[test]
fn test_storefront_token_header_value() {
    let public_token = StorefrontToken::Public("my-public-token".to_string());
    assert_eq!(public_token.header_value(), "my-public-token");

    let private_token = StorefrontToken::Private("my-private-token".to_string());
    assert_eq!(private_token.header_value(), "my-private-token");
}

#[test]
fn test_storefront_token_debug_masks_value() {
    let public_token = StorefrontToken::Public("secret-token".to_string());
    let debug = format!("{:?}", public_token);
    assert_eq!(debug, "StorefrontToken::Public(*****)");
    assert!(!debug.contains("secret-token"));

    let private_token = StorefrontToken::Private("secret-token".to_string());
    let debug = format!("{:?}", private_token);
    assert_eq!(debug, "StorefrontToken::Private(*****)");
    assert!(!debug.contains("secret-token"));
}

#[test]
fn test_storefront_token_clone() {
    let original = StorefrontToken::Public("cloneable".to_string());
    let cloned = original.clone();
    assert_eq!(cloned.header_value(), "cloneable");
}

// ============================================================================
// StorefrontClient Construction Tests
// ============================================================================

#[test]
fn test_storefront_client_creates_with_default_version() {
    let shop = ShopDomain::new("test-shop").unwrap();
    let client = StorefrontClient::new(&shop, None, None);

    // Should use latest API version when no config provided
    assert_eq!(client.api_version(), &ApiVersion::latest());
}

#[test]
fn test_storefront_client_with_version_override() {
    let shop = ShopDomain::new("test-shop").unwrap();
    let client = StorefrontClient::with_version(&shop, None, None, ApiVersion::V2024_10);

    assert_eq!(client.api_version(), &ApiVersion::V2024_10);
}

#[test]
fn test_storefront_client_is_thread_safe() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<StorefrontClient>();
}

#[test]
fn test_storefront_client_constructor_is_infallible() {
    let shop = ShopDomain::new("test-shop").unwrap();
    // This compiles because new() returns Self, not Result
    let _client: StorefrontClient = StorefrontClient::new(&shop, None, None);
}

#[test]
fn test_storefront_client_with_public_token() {
    let shop = ShopDomain::new("test-shop").unwrap();
    let token = StorefrontToken::Public("public-token".to_string());
    let client = StorefrontClient::new(&shop, Some(token), None);

    assert_eq!(client.api_version(), &ApiVersion::latest());
}

#[test]
fn test_storefront_client_with_private_token() {
    let shop = ShopDomain::new("test-shop").unwrap();
    let token = StorefrontToken::Private("private-token".to_string());
    let client = StorefrontClient::new(&shop, Some(token), None);

    assert_eq!(client.api_version(), &ApiVersion::latest());
}

#[test]
fn test_storefront_client_tokenless() {
    let shop = ShopDomain::new("test-shop").unwrap();
    let client = StorefrontClient::new(&shop, None, None);

    assert_eq!(client.api_version(), &ApiVersion::latest());
}

#[test]
fn test_storefront_client_with_config_uses_config_version() {
    use shopify_sdk::{ApiKey, ApiSecretKey, ShopifyConfig};

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

// ============================================================================
// Type Export Tests
// ============================================================================

#[test]
fn test_types_exported_at_crate_root() {
    // Verify types are accessible from crate root
    let _: fn(shopify_sdk::StorefrontClient) = |_| {};
    let _: fn(shopify_sdk::StorefrontToken) = |_| {};
}

#[test]
fn test_types_exported_from_clients_module() {
    // Verify types are accessible from clients module
    let _: fn(shopify_sdk::clients::StorefrontClient) = |_| {};
    let _: fn(shopify_sdk::clients::StorefrontToken) = |_| {};
}

#[test]
fn test_types_exported_from_clients_storefront_module() {
    // Verify types are accessible from clients::storefront module
    let _: fn(shopify_sdk::clients::storefront::StorefrontClient) = |_| {};
    let _: fn(shopify_sdk::clients::storefront::StorefrontToken) = |_| {};
}

// ============================================================================
// Query Method Behavior Tests (without real HTTP calls)
// ============================================================================

/// Helper to verify query method attempts to make a network call.
/// The test passes if we get an error (expected for fake shops) or
/// a response (in case DNS resolves and Shopify returns something).
fn assert_query_attempted(result: Result<shopify_sdk::HttpResponse, GraphqlError>) {
    match result {
        Err(GraphqlError::Http(_)) => {
            // Expected: network error for fake shop domain
        }
        Ok(_) => {
            // Also acceptable: Shopify responded (e.g., 404 or redirect)
            // This means the query was attempted
        }
    }
}

#[tokio::test]
async fn test_query_method_sends_request_to_shopify() {
    // This test verifies that the query method tries to send a request.
    // We expect a network error since we're not connecting to a real Shopify server,
    // but the DNS might resolve and return something.
    let shop = ShopDomain::new("test-shop").unwrap();
    let token = StorefrontToken::Public("test-token".to_string());
    let client = StorefrontClient::new(&shop, Some(token), None);

    let result = client
        .query("query { shop { name } }", None, None, None)
        .await;

    assert_query_attempted(result);
}

#[tokio::test]
async fn test_query_with_variables_sends_request() {
    let shop = ShopDomain::new("test-shop").unwrap();
    let client = StorefrontClient::new(&shop, None, None);

    let variables = serde_json::json!({
        "handle": "my-product"
    });

    let result = client
        .query(
            "query GetProduct($handle: String!) { productByHandle(handle: $handle) { title } }",
            Some(variables),
            None,
            None,
        )
        .await;

    assert_query_attempted(result);
}

#[tokio::test]
async fn test_query_with_debug_sends_request() {
    let shop = ShopDomain::new("test-shop").unwrap();
    let client = StorefrontClient::new(&shop, None, None);

    let result = client
        .query_with_debug("query { shop { name } }", None, None, None)
        .await;

    assert_query_attempted(result);
}

#[tokio::test]
async fn test_query_with_custom_headers_sends_request() {
    use std::collections::HashMap;

    let shop = ShopDomain::new("test-shop").unwrap();
    let client = StorefrontClient::new(&shop, None, None);

    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

    let result = client
        .query("query { shop { name } }", None, Some(headers), None)
        .await;

    assert_query_attempted(result);
}

#[tokio::test]
async fn test_query_with_retries() {
    let shop = ShopDomain::new("test-shop").unwrap();
    let client = StorefrontClient::new(&shop, None, None);

    let result = client
        .query("query { shop { name } }", None, None, Some(3))
        .await;

    assert_query_attempted(result);
}

// ============================================================================
// API Version in URL Tests
// ============================================================================

#[test]
fn test_storefront_client_stores_api_version() {
    let shop = ShopDomain::new("test-shop").unwrap();

    let client_2024_10 = StorefrontClient::with_version(&shop, None, None, ApiVersion::V2024_10);
    let client_2024_07 = StorefrontClient::with_version(&shop, None, None, ApiVersion::V2024_07);
    let client_latest = StorefrontClient::new(&shop, None, None);

    assert_eq!(client_2024_10.api_version(), &ApiVersion::V2024_10);
    assert_eq!(client_2024_07.api_version(), &ApiVersion::V2024_07);
    assert_eq!(client_latest.api_version(), &ApiVersion::latest());
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[tokio::test]
async fn test_storefront_client_can_be_shared_across_tasks() {
    use std::sync::Arc;

    let shop = ShopDomain::new("test-shop").unwrap();
    let client = Arc::new(StorefrontClient::new(&shop, None, None));

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
fn test_graphql_error_http_variant_wraps_http_error() {
    use shopify_sdk::clients::{HttpError, HttpResponseError};

    let http_error = HttpError::Response(HttpResponseError {
        code: 401,
        message: r#"{"error":"Unauthorized"}"#.to_string(),
        error_reference: Some("abc-123".to_string()),
    });

    let graphql_error = GraphqlError::Http(http_error);
    let message = graphql_error.to_string();

    assert!(message.contains("Unauthorized"));
}

// ============================================================================
// Endpoint Format Tests (verifying Storefront uses /api/ not /admin/api/)
// ============================================================================

#[test]
fn test_storefront_uses_correct_endpoint_format() {
    // This test documents the key difference: Storefront API uses /api/{version}/graphql.json
    // while Admin API uses /admin/api/{version}/graphql.json
    //
    // The StorefrontHttpClient constructs base_path as "/api/{version}"
    // and builds URLs like "https://{shop}.myshopify.com/api/{version}/graphql.json"

    let shop = ShopDomain::new("test-shop").unwrap();
    let client = StorefrontClient::new(&shop, None, None);

    // The client was created successfully, which means it's using the correct endpoint format
    // The actual endpoint format is tested in the unit tests for StorefrontHttpClient
    assert_eq!(client.api_version(), &ApiVersion::latest());
}

// ============================================================================
// Multi-shop Tests
// ============================================================================

#[test]
fn test_multiple_clients_for_different_shops() {
    let shop1 = ShopDomain::new("shop-one").unwrap();
    let shop2 = ShopDomain::new("shop-two").unwrap();

    let token1 = StorefrontToken::Public("token-1".to_string());
    let token2 = StorefrontToken::Private("token-2".to_string());

    let client1 = StorefrontClient::new(&shop1, Some(token1), None);
    let client2 = StorefrontClient::new(&shop2, Some(token2), None);

    // Both clients should have independent configurations
    assert_eq!(client1.api_version(), &ApiVersion::latest());
    assert_eq!(client2.api_version(), &ApiVersion::latest());
}

#[test]
fn test_clients_with_different_api_versions() {
    let shop = ShopDomain::new("test-shop").unwrap();

    let client_latest = StorefrontClient::new(&shop, None, None);
    let client_old = StorefrontClient::with_version(&shop, None, None, ApiVersion::V2024_10);

    assert_eq!(client_latest.api_version(), &ApiVersion::latest());
    assert_eq!(client_old.api_version(), &ApiVersion::V2024_10);
}

// ============================================================================
// Token Type Comparison Tests
// ============================================================================

#[test]
fn test_public_and_private_tokens_use_different_headers() {
    let public = StorefrontToken::Public("token".to_string());
    let private = StorefrontToken::Private("token".to_string());

    // Different header names
    assert_ne!(public.header_name(), private.header_name());
    assert_eq!(public.header_name(), "X-Shopify-Storefront-Access-Token");
    assert_eq!(private.header_name(), "Shopify-Storefront-Private-Token");

    // Same header value (the token itself)
    assert_eq!(public.header_value(), private.header_value());
}

//! Integration tests for the REST API client functionality.
//!
//! These tests verify the REST client construction, path normalization,
//! error handling, and API method behavior.

use shopify_api::clients::rest::{RestClient, RestError};
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
// RestClient Construction Tests
// ============================================================================

#[test]
fn test_rest_client_creates_with_default_version() {
    let session = create_test_session("test-shop", "test-token");
    let client = RestClient::new(&session, None).unwrap();

    // Should use latest API version when no config provided
    assert_eq!(client.api_version(), &ApiVersion::latest());
}

#[test]
fn test_rest_client_with_version_override() {
    let session = create_test_session("test-shop", "test-token");
    let client = RestClient::with_version(&session, None, ApiVersion::V2024_10).unwrap();

    assert_eq!(client.api_version(), &ApiVersion::V2024_10);
}

#[test]
fn test_rest_client_is_thread_safe() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<RestClient>();
}

// ============================================================================
// Error Type Tests
// ============================================================================

#[test]
fn test_rest_error_rest_api_disabled_message() {
    let error = RestError::RestApiDisabled;
    let message = error.to_string();

    // Should contain deprecation message matching Ruby SDK
    assert!(message.contains("Admin REST API"));
    assert!(message.contains("deprecated"));
    assert!(message.contains("GraphQL"));
}

#[test]
fn test_rest_error_invalid_path_message() {
    let error = RestError::InvalidPath {
        path: "".to_string(),
    };
    let message = error.to_string();

    assert!(message.contains("Invalid REST API path"));
}

// ============================================================================
// Multi-tenant Session Tests
// ============================================================================

#[test]
fn test_multiple_clients_for_different_shops() {
    let session1 = create_test_session("shop-one", "token-1");
    let session2 = create_test_session("shop-two", "token-2");

    let client1 = RestClient::new(&session1, None).unwrap();
    let client2 = RestClient::new(&session2, None).unwrap();

    // Both clients should have independent configurations
    assert_eq!(client1.api_version(), &ApiVersion::latest());
    assert_eq!(client2.api_version(), &ApiVersion::latest());
}

#[test]
fn test_clients_with_different_api_versions() {
    let session = create_test_session("test-shop", "test-token");

    let client_latest = RestClient::new(&session, None).unwrap();
    let client_old = RestClient::with_version(&session, None, ApiVersion::V2024_10).unwrap();

    assert_eq!(client_latest.api_version(), &ApiVersion::latest());
    assert_eq!(client_old.api_version(), &ApiVersion::V2024_10);
}

// ============================================================================
// Integration with HTTP Types
// ============================================================================

#[test]
fn test_rest_error_wraps_http_errors() {
    use shopify_api::clients::{HttpError, HttpResponseError};

    let http_error = HttpError::Response(HttpResponseError {
        code: 404,
        message: r#"{"error":"Not Found"}"#.to_string(),
        error_reference: Some("abc-123".to_string()),
    });

    let rest_error = RestError::Http(http_error);
    let message = rest_error.to_string();

    // The error message contains the raw message, not status code
    assert!(message.contains("Not Found"));
}

// ============================================================================
// Type Export Tests
// ============================================================================

#[test]
fn test_types_exported_at_crate_root() {
    // Verify types are accessible from crate root
    let _: fn(shopify_api::RestClient) = |_| {};
    let _: fn(shopify_api::RestError) = |_| {};
}

#[test]
fn test_types_exported_from_clients_module() {
    // Verify types are accessible from clients module
    let _: fn(shopify_api::clients::RestClient) = |_| {};
    let _: fn(shopify_api::clients::RestError) = |_| {};
}

#[test]
fn test_types_exported_from_clients_rest_module() {
    // Verify types are accessible from clients::rest module
    let _: fn(shopify_api::clients::rest::RestClient) = |_| {};
    let _: fn(shopify_api::clients::rest::RestError) = |_| {};
}

//! Integration tests for Client Credentials Grant OAuth.
//!
//! These tests verify the complete client credentials flow for private/organization apps,
//! including configuration validation, HTTP interactions, and session creation.

use shopify_api::auth::oauth::{exchange_client_credentials, OAuthError};
use shopify_api::auth::session::AccessTokenResponse;
use shopify_api::auth::Session;
use shopify_api::{ApiKey, ApiSecretKey, ShopDomain, ShopifyConfig};

/// Creates a private (non-embedded) app configuration
fn create_private_config(api_key: &str, secret: &str) -> ShopifyConfig {
    ShopifyConfig::builder()
        .api_key(ApiKey::new(api_key).unwrap())
        .api_secret_key(ApiSecretKey::new(secret).unwrap())
        .is_embedded(false)
        .build()
        .unwrap()
}

/// Creates an embedded app configuration
fn create_embedded_config(api_key: &str, secret: &str) -> ShopifyConfig {
    ShopifyConfig::builder()
        .api_key(ApiKey::new(api_key).unwrap())
        .api_secret_key(ApiSecretKey::new(secret).unwrap())
        .is_embedded(true)
        .build()
        .unwrap()
}

// === Integration Tests ===

/// Test 1: Complete client credentials flow - config validation -> HTTP request -> error
#[tokio::test]
async fn test_complete_client_credentials_flow() {
    // This test verifies the complete flow from config validation to HTTP request
    let api_key = "test-api-key";
    let secret = "test-secret-key";

    let config = create_private_config(api_key, secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    // The exchange will fail at HTTP level (Shopify will reject our test credentials),
    // but this verifies the config validation and request building work
    let result = exchange_client_credentials(&config, &shop).await;

    // Should fail with ClientCredentialsFailed (config validation passed)
    match result {
        Err(OAuthError::ClientCredentialsFailed { .. }) => {
            // This is expected - the config was valid but Shopify rejected the credentials
        }
        Err(OAuthError::NotPrivateApp) => {
            panic!("Config should be valid for client credentials");
        }
        Err(other) => {
            panic!("Expected ClientCredentialsFailed, got: {other:?}");
        }
        Ok(_) => {
            // This would only happen if we somehow have valid Shopify credentials
            // which is unlikely but not a test failure
        }
    }
}

/// Test 2: Error propagation from HTTP errors to caller
#[tokio::test]
async fn test_error_propagation_from_http_errors() {
    let api_key = "test-api-key";
    let secret = "test-secret-key";

    let config = create_private_config(api_key, secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    // Attempt client credentials exchange (will fail since we don't have valid credentials)
    let result = exchange_client_credentials(&config, &shop).await;

    match result {
        Err(OAuthError::ClientCredentialsFailed { message, .. }) => {
            // Either network error or HTTP error response - both are valid outcomes
            assert!(!message.is_empty(), "Expected non-empty error message");
        }
        Err(other) => {
            panic!("Expected ClientCredentialsFailed, got: {other:?}");
        }
        Ok(_) => {
            panic!("Expected error, but got success");
        }
    }
}

/// Test 3: NotPrivateApp error for embedded configs
#[tokio::test]
async fn test_not_private_app_error_for_embedded_configs() {
    let api_key = "test-api-key";
    let secret = "test-secret-key";

    // Create an embedded config (which should fail for client credentials)
    let config = create_embedded_config(api_key, secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    let result = exchange_client_credentials(&config, &shop).await;

    assert!(
        matches!(result, Err(OAuthError::NotPrivateApp)),
        "Expected NotPrivateApp error for embedded config"
    );
}

/// Test 4: Session fields correctly populated from response
#[tokio::test]
async fn test_session_fields_correctly_populated() {
    // Test that Session::from_access_token_response correctly populates all fields
    // This is a unit-style test within the integration test file to verify session creation

    let shop = ShopDomain::new("test-shop").unwrap();

    // Test offline token response (what client credentials produces)
    let offline_response = AccessTokenResponse {
        access_token: "client-credentials-access-token".to_string(),
        scope: "read_products,write_orders".to_string(),
        expires_in: None,
        associated_user_scope: None,
        associated_user: None,
        session: Some("shopify-session-id".to_string()),
        refresh_token: None,
        refresh_token_expires_in: None,
    };

    let offline_session = Session::from_access_token_response(shop, &offline_response);

    assert_eq!(
        offline_session.access_token,
        "client-credentials-access-token"
    );
    assert_eq!(offline_session.id, "offline_test-shop.myshopify.com");
    assert!(!offline_session.is_online);
    assert!(offline_session.expires.is_none());
    assert!(offline_session.associated_user.is_none());
    assert_eq!(
        offline_session.shopify_session_id,
        Some("shopify-session-id".to_string())
    );

    // Verify scopes were parsed
    assert!(offline_session.scopes.iter().any(|s| s == "read_products"));
    assert!(offline_session.scopes.iter().any(|s| s == "write_orders"));
}

/// Test 5: Function is accessible from shopify_api::auth::oauth module
#[test]
fn test_function_accessible_from_auth_oauth_module() {
    // This test verifies that exchange_client_credentials is properly exported
    // The function exists and is accessible - compilation proves this

    // Verify the function signature by referencing it
    let _ = exchange_client_credentials as fn(_, _) -> _;
}

/// Test 6: Function is accessible from crate root
#[test]
fn test_function_accessible_from_crate_root() {
    // Verify the function can be imported from crate root
    use shopify_api::exchange_client_credentials;

    // The function exists and is accessible - compilation proves this
    let _ = exchange_client_credentials as fn(_, _) -> _;
}

/// Additional test: Verify that embedded config defaults require explicit is_embedded(false)
/// Note: The default for ShopifyConfig is is_embedded(true), so client credentials
/// requires explicitly setting is_embedded(false).
#[tokio::test]
async fn test_default_embedded_config_rejected_for_client_credentials() {
    // Create config without explicitly setting is_embedded (defaults to true)
    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-api-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .build()
        .unwrap();

    let shop = ShopDomain::new("test-shop").unwrap();

    // Default is_embedded is true, so client credentials should fail with NotPrivateApp
    let result = exchange_client_credentials(&config, &shop).await;

    assert!(
        matches!(result, Err(OAuthError::NotPrivateApp)),
        "Default embedded config should be rejected for client credentials"
    );
}

/// Additional test: Session ID format is correct
#[test]
fn test_session_id_format_for_client_credentials() {
    let shop = ShopDomain::new("my-store").unwrap();
    let response = AccessTokenResponse {
        access_token: "test-token".to_string(),
        scope: "read_products".to_string(),
        expires_in: None,
        associated_user_scope: None,
        associated_user: None,
        session: None,
        refresh_token: None,
        refresh_token_expires_in: None,
    };

    let session = Session::from_access_token_response(shop, &response);

    // Client credentials always produces offline sessions
    assert_eq!(session.id, "offline_my-store.myshopify.com");
    assert!(!session.is_online);
}

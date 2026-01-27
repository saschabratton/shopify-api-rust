//! Integration tests for Token Exchange OAuth.
//!
//! These tests verify the complete token exchange flow for embedded apps,
//! including JWT validation, HTTP interactions, and session creation.

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use shopify_sdk::auth::oauth::{exchange_offline_token, exchange_online_token, OAuthError};
use shopify_sdk::auth::Session;
use shopify_sdk::{ApiKey, ApiSecretKey, ShopDomain, ShopifyConfig};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims structure for creating test tokens
#[derive(Debug, Serialize)]
struct TestJwtClaims {
    iss: String,
    dest: String,
    aud: String,
    sub: Option<String>,
    exp: i64,
    nbf: i64,
    iat: i64,
    jti: String,
    sid: Option<String>,
}

/// Returns the current Unix timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

/// Creates a test JWT with the given parameters
fn create_test_jwt(
    shop: &str,
    api_key: &str,
    secret: &str,
    sub: Option<&str>,
    with_admin_suffix: bool,
) -> String {
    let now = current_timestamp();
    let iss = if with_admin_suffix {
        format!("https://{shop}/admin")
    } else {
        format!("https://{shop}")
    };

    let claims = TestJwtClaims {
        iss,
        dest: format!("https://{shop}"),
        aud: api_key.to_string(),
        sub: sub.map(String::from),
        exp: now + 300, // 5 minutes from now
        nbf: now - 10,
        iat: now,
        jti: format!("test-jti-{now}"),
        sid: Some("test-session-id".to_string()),
    };

    let header = Header::new(Algorithm::HS256);
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&header, &claims, &key).expect("Failed to encode JWT")
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

/// Creates an embedded app configuration with dual keys
fn create_config_with_dual_keys(
    api_key: &str,
    primary_secret: &str,
    old_secret: &str,
) -> ShopifyConfig {
    ShopifyConfig::builder()
        .api_key(ApiKey::new(api_key).unwrap())
        .api_secret_key(ApiSecretKey::new(primary_secret).unwrap())
        .old_api_secret_key(ApiSecretKey::new(old_secret).unwrap())
        .is_embedded(true)
        .build()
        .unwrap()
}

// === Integration Tests ===

/// Test 1: Complete offline token exchange flow - JWT validation succeeds, HTTP fails
#[tokio::test]
async fn test_complete_offline_token_exchange_flow() {
    // This test verifies the complete flow from JWT creation to HTTP request
    let api_key = "test-api-key";
    let secret = "test-secret-key";
    let shop_domain = "test-shop.myshopify.com";

    let config = create_embedded_config(api_key, secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    // Create a valid JWT
    let session_token = create_test_jwt(shop_domain, api_key, secret, None, true);

    // The exchange will fail at HTTP level (Shopify will reject our test credentials),
    // but this verifies the JWT validation and request building work
    let result = exchange_offline_token(&config, &shop, &session_token).await;

    // Should fail with TokenExchangeFailed (not InvalidJwt), meaning JWT was validated
    match result {
        Err(OAuthError::TokenExchangeFailed { .. }) => {
            // This is expected - the JWT was valid but Shopify rejected the token exchange
        }
        Err(OAuthError::InvalidJwt { reason }) => {
            panic!("JWT should be valid but got InvalidJwt: {reason}");
        }
        Err(other) => {
            panic!("Expected TokenExchangeFailed, got: {other:?}");
        }
        Ok(_) => {
            // This would only happen if we somehow have valid Shopify credentials
            // which is unlikely but not a test failure
        }
    }
}

/// Test 2: Complete online token exchange flow - JWT validation succeeds, HTTP fails
#[tokio::test]
async fn test_complete_online_token_exchange_flow() {
    let api_key = "test-api-key";
    let secret = "test-secret-key";
    let shop_domain = "test-shop.myshopify.com";

    let config = create_embedded_config(api_key, secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    // Create a valid JWT with user ID (for online token)
    let session_token = create_test_jwt(shop_domain, api_key, secret, Some("12345"), true);

    let result = exchange_online_token(&config, &shop, &session_token).await;

    // Should fail with TokenExchangeFailed (not InvalidJwt)
    match result {
        Err(OAuthError::TokenExchangeFailed { .. }) => {
            // Expected - JWT valid, but Shopify rejected exchange
        }
        Err(OAuthError::InvalidJwt { reason }) => {
            panic!("JWT should be valid but got InvalidJwt: {reason}");
        }
        Err(other) => {
            panic!("Expected TokenExchangeFailed, got: {other:?}");
        }
        Ok(_) => {
            // Unlikely but acceptable
        }
    }
}

/// Test 3: Token exchange with dual-key JWT validation (old key scenario)
#[tokio::test]
async fn test_token_exchange_with_dual_key_jwt_validation() {
    let api_key = "test-api-key";
    let primary_secret = "new-secret-key";
    let old_secret = "old-secret-key";
    let shop_domain = "test-shop.myshopify.com";

    let config = create_config_with_dual_keys(api_key, primary_secret, old_secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    // Create JWT signed with the OLD secret (simulating key rotation scenario)
    let session_token = create_test_jwt(shop_domain, api_key, old_secret, Some("12345"), true);

    let result = exchange_offline_token(&config, &shop, &session_token).await;

    // JWT should be validated using the fallback old key
    // Should fail at HTTP level, not JWT validation
    match result {
        Err(OAuthError::TokenExchangeFailed { .. }) => {
            // Expected - JWT validated with old key, but HTTP failed
        }
        Err(OAuthError::InvalidJwt { reason }) => {
            panic!("JWT should be valid with old key fallback but got InvalidJwt: {reason}");
        }
        Err(other) => {
            panic!("Expected TokenExchangeFailed, got: {other:?}");
        }
        Ok(_) => {
            // Unlikely but acceptable
        }
    }
}

/// Test 4: Error propagation from JWT validation to caller
#[tokio::test]
async fn test_error_propagation_from_jwt_validation() {
    let api_key = "test-api-key";
    let secret = "test-secret-key";

    let config = create_embedded_config(api_key, secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    // Test 1: Invalid JWT structure
    let result = exchange_offline_token(&config, &shop, "not-a-valid-jwt").await;
    assert!(
        matches!(result, Err(OAuthError::InvalidJwt { .. })),
        "Expected InvalidJwt error for malformed token"
    );

    // Test 2: JWT signed with wrong key
    let wrong_key_token = create_test_jwt(
        "test-shop.myshopify.com",
        api_key,
        "wrong-secret",
        None,
        true,
    );
    let result = exchange_offline_token(&config, &shop, &wrong_key_token).await;
    assert!(
        matches!(result, Err(OAuthError::InvalidJwt { .. })),
        "Expected InvalidJwt error for wrong signature"
    );

    // Test 3: JWT with wrong audience (API key mismatch)
    let wrong_aud_token = create_test_jwt(
        "test-shop.myshopify.com",
        "wrong-api-key",
        secret,
        None,
        true,
    );
    let result = exchange_offline_token(&config, &shop, &wrong_aud_token).await;
    match result {
        Err(OAuthError::InvalidJwt { reason }) => {
            assert!(
                reason.contains("invalid API key"),
                "Expected API key mismatch error, got: {reason}"
            );
        }
        _ => panic!("Expected InvalidJwt error for wrong audience"),
    }
}

/// Test 5: Error propagation from HTTP errors to caller
#[tokio::test]
async fn test_error_propagation_from_http_errors() {
    let api_key = "test-api-key";
    let secret = "test-secret-key";
    let shop_domain = "test-shop.myshopify.com";

    let config = create_embedded_config(api_key, secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    // Create a valid JWT
    let session_token = create_test_jwt(shop_domain, api_key, secret, None, true);

    // Attempt token exchange (will fail since we don't have valid Shopify credentials)
    let result = exchange_offline_token(&config, &shop, &session_token).await;

    match result {
        Err(OAuthError::TokenExchangeFailed { message, .. }) => {
            // Either network error or HTTP error response - both are valid outcomes
            assert!(!message.is_empty(), "Expected non-empty error message");
        }
        Err(other) => {
            panic!("Expected TokenExchangeFailed, got: {other:?}");
        }
        Ok(_) => {
            panic!("Expected error, but got success (shouldn't reach real Shopify with valid credentials)");
        }
    }
}

/// Test 6: Session fields are correctly populated from token response
#[tokio::test]
async fn test_session_fields_correctly_populated() {
    // Test that Session::from_access_token_response correctly populates all fields
    // This is a unit-style test within the integration test file to verify session creation

    use shopify_sdk::auth::session::AccessTokenResponse;

    let shop = ShopDomain::new("test-shop").unwrap();

    // Test offline token response
    let offline_response = AccessTokenResponse {
        access_token: "offline-access-token".to_string(),
        scope: "read_products,write_orders".to_string(),
        expires_in: None,
        associated_user_scope: None,
        associated_user: None,
        session: Some("shopify-session-id".to_string()),
        refresh_token: None,
        refresh_token_expires_in: None,
    };

    let offline_session = Session::from_access_token_response(shop.clone(), &offline_response);

    assert_eq!(offline_session.access_token, "offline-access-token");
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

    // Test online token response
    use shopify_sdk::auth::session::AssociatedUserResponse;

    let online_response = AccessTokenResponse {
        access_token: "online-access-token".to_string(),
        scope: "read_products".to_string(),
        expires_in: Some(3600),
        associated_user_scope: Some("read_products".to_string()),
        associated_user: Some(AssociatedUserResponse {
            id: 12345,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            account_owner: true,
            locale: "en".to_string(),
            collaborator: false,
        }),
        session: Some("online-session-id".to_string()),
        refresh_token: None,
        refresh_token_expires_in: None,
    };

    let online_session = Session::from_access_token_response(shop, &online_response);

    assert_eq!(online_session.access_token, "online-access-token");
    assert_eq!(online_session.id, "test-shop.myshopify.com_12345");
    assert!(online_session.is_online);
    assert!(online_session.expires.is_some());

    let associated_user = online_session
        .associated_user
        .expect("Should have associated user");
    assert_eq!(associated_user.id, 12345);
    assert_eq!(associated_user.first_name, "Test");
    assert_eq!(associated_user.last_name, "User");
    assert_eq!(associated_user.email, "test@example.com");
    assert!(associated_user.email_verified);
    assert!(associated_user.account_owner);
    assert!(!associated_user.collaborator);
}

/// Additional test: Verify NotEmbeddedApp error for non-embedded configs
#[tokio::test]
async fn test_not_embedded_app_error() {
    let api_key = "test-api-key";
    let secret = "test-secret-key";

    // Create non-embedded config
    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new(api_key).unwrap())
        .api_secret_key(ApiSecretKey::new(secret).unwrap())
        .is_embedded(false)
        .build()
        .unwrap();

    let shop = ShopDomain::new("test-shop").unwrap();
    let session_token = create_test_jwt("test-shop.myshopify.com", api_key, secret, None, true);

    let result = exchange_offline_token(&config, &shop, &session_token).await;

    assert!(
        matches!(result, Err(OAuthError::NotEmbeddedApp)),
        "Expected NotEmbeddedApp error"
    );
}

/// Additional test: JWT without /admin suffix in iss is still valid
#[tokio::test]
async fn test_jwt_without_admin_suffix() {
    // This tests that JWT validation still passes without /admin suffix,
    // even though shopify_user_id() would return None
    let api_key = "test-api-key";
    let secret = "test-secret-key";
    let shop_domain = "test-shop.myshopify.com";

    let config = create_embedded_config(api_key, secret);
    let shop = ShopDomain::new("test-shop").unwrap();

    // Create JWT WITHOUT /admin suffix (should still be valid)
    let session_token = create_test_jwt(shop_domain, api_key, secret, Some("12345"), false);

    let result = exchange_offline_token(&config, &shop, &session_token).await;

    // Should fail at HTTP level (JWT is valid)
    assert!(
        matches!(result, Err(OAuthError::TokenExchangeFailed { .. })),
        "Expected TokenExchangeFailed, JWT should still be valid"
    );
}

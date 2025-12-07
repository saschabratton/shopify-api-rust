//! Integration tests for the Shopify API SDK.
//!
//! These tests verify end-to-end functionality of the SDK configuration system
//! and session management.

use shopify_api::{
    ApiKey, ApiSecretKey, ApiVersion, AssociatedUser, AuthScopes, ConfigError, HostUrl, Session,
    ShopDomain, ShopifyConfig,
};

// === Configuration Integration Tests ===

#[test]
fn test_full_workflow_create_newtypes_build_config_access_fields() {
    // Create validated newtypes
    let api_key = ApiKey::new("test-api-key").unwrap();
    let api_secret = ApiSecretKey::new("test-api-secret").unwrap();
    let scopes: AuthScopes = "read_products, write_orders".parse().unwrap();
    let host = HostUrl::new("https://myapp.example.com").unwrap();

    // Build configuration
    let config = ShopifyConfig::builder()
        .api_key(api_key.clone())
        .api_secret_key(api_secret)
        .scopes(scopes.clone())
        .host(host.clone())
        .api_version(ApiVersion::V2024_10)
        .is_embedded(false)
        .user_agent_prefix("TestApp/1.0")
        .build()
        .unwrap();

    // Access fields and verify
    assert_eq!(config.api_key().as_ref(), "test-api-key");
    assert_eq!(config.api_version(), &ApiVersion::V2024_10);
    assert!(!config.is_embedded());
    assert_eq!(config.host().unwrap().as_ref(), "https://myapp.example.com");
    assert_eq!(config.user_agent_prefix(), Some("TestApp/1.0"));

    // Verify scopes include implied scope (write_orders implies read_orders)
    assert!(config.scopes().iter().any(|s| s == "read_orders"));
}

#[test]
fn test_multi_tenant_scenario_multiple_independent_configs() {
    // Create configuration for Store A
    let config_a = ShopifyConfig::builder()
        .api_key(ApiKey::new("store-a-key").unwrap())
        .api_secret_key(ApiSecretKey::new("store-a-secret").unwrap())
        .scopes("read_products".parse().unwrap())
        .api_version(ApiVersion::V2024_10)
        .build()
        .unwrap();

    // Create configuration for Store B
    let config_b = ShopifyConfig::builder()
        .api_key(ApiKey::new("store-b-key").unwrap())
        .api_secret_key(ApiSecretKey::new("store-b-secret").unwrap())
        .scopes("write_orders".parse().unwrap())
        .api_version(ApiVersion::V2025_01)
        .build()
        .unwrap();

    // Verify configurations are independent
    assert_eq!(config_a.api_key().as_ref(), "store-a-key");
    assert_eq!(config_b.api_key().as_ref(), "store-b-key");
    assert_eq!(config_a.api_version(), &ApiVersion::V2024_10);
    assert_eq!(config_b.api_version(), &ApiVersion::V2025_01);

    // Store A has read_products but not write_orders
    assert!(config_a.scopes().iter().any(|s| s == "read_products"));
    assert!(!config_a.scopes().iter().any(|s| s == "write_orders"));

    // Store B has write_orders (and implied read_orders)
    assert!(config_b.scopes().iter().any(|s| s == "write_orders"));
    assert!(config_b.scopes().iter().any(|s| s == "read_orders"));
}

#[test]
fn test_error_handling_invalid_inputs_produce_correct_errors() {
    // Empty API key
    let result = ApiKey::new("");
    assert!(matches!(result, Err(ConfigError::EmptyApiKey)));

    // Empty API secret key
    let result = ApiSecretKey::new("");
    assert!(matches!(result, Err(ConfigError::EmptyApiSecretKey)));

    // Invalid shop domain
    let result = ShopDomain::new("invalid domain with spaces");
    assert!(matches!(result, Err(ConfigError::InvalidShopDomain { .. })));

    // Invalid host URL
    let result = HostUrl::new("not-a-valid-url");
    assert!(matches!(result, Err(ConfigError::InvalidHostUrl { .. })));

    // Invalid API version
    let result: Result<ApiVersion, _> = "invalid".parse();
    assert!(matches!(result, Err(ConfigError::InvalidApiVersion { .. })));

    // Missing required fields in builder
    let result = ShopifyConfig::builder()
        .api_key(ApiKey::new("key").unwrap())
        .build();
    assert!(matches!(
        result,
        Err(ConfigError::MissingRequiredField {
            field: "api_secret_key"
        })
    ));
}

#[test]
fn test_config_can_be_cloned_and_shared() {
    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new("key").unwrap())
        .api_secret_key(ApiSecretKey::new("secret").unwrap())
        .build()
        .unwrap();

    // Clone the config
    let config_clone = config.clone();

    // Both should have the same values
    assert_eq!(config.api_key().as_ref(), config_clone.api_key().as_ref());
    assert_eq!(config.api_version(), config_clone.api_version());

    // Verify Send + Sync by moving to thread (compile-time check)
    let handle = std::thread::spawn(move || {
        let _ = config_clone.api_key().as_ref();
    });
    handle.join().unwrap();
}

// === Session Management Integration Tests ===

#[test]
fn test_offline_session_workflow_with_factory_method() {
    use shopify_api::auth::session::AccessTokenResponse;

    let shop = ShopDomain::new("my-store").unwrap();

    // Simulate OAuth response for offline token
    let response = AccessTokenResponse {
        access_token: "shpat_offline_token_12345".to_string(),
        scope: "read_products,write_orders".to_string(),
        expires_in: None,
        associated_user_scope: None,
        associated_user: None,
        session: None,
        refresh_token: None,
        refresh_token_expires_in: None,
    };

    // Create session from response
    let session = Session::from_access_token_response(shop, &response);

    // Verify offline session properties
    assert!(!session.is_online);
    assert_eq!(session.id, "offline_my-store.myshopify.com");
    assert_eq!(session.access_token, "shpat_offline_token_12345");
    assert!(session.expires.is_none());
    assert!(session.associated_user.is_none());

    // Verify scopes with implied scopes
    assert!(session.scopes.iter().any(|s| s == "read_products"));
    assert!(session.scopes.iter().any(|s| s == "write_orders"));
    assert!(session.scopes.iter().any(|s| s == "read_orders")); // implied

    // Session should be active
    assert!(session.is_active());
    assert!(!session.expired());
}

#[test]
fn test_online_session_workflow_with_associated_user() {
    use shopify_api::auth::session::{AccessTokenResponse, AssociatedUserResponse};

    let shop = ShopDomain::new("test-shop").unwrap();

    // Simulate OAuth response for online token with user
    let response = AccessTokenResponse {
        access_token: "shpua_online_token_67890".to_string(),
        scope: "read_products,write_orders".to_string(),
        expires_in: Some(86400), // 24 hours
        associated_user_scope: Some("read_products".to_string()),
        associated_user: Some(AssociatedUserResponse {
            id: 12345,
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            email_verified: true,
            account_owner: true,
            locale: "en".to_string(),
            collaborator: false,
        }),
        session: Some("shopify_session_abc123".to_string()),
        refresh_token: None,
        refresh_token_expires_in: None,
    };

    // Create session from response
    let session = Session::from_access_token_response(shop, &response);

    // Verify online session properties
    assert!(session.is_online);
    assert_eq!(session.id, "test-shop.myshopify.com_12345");
    assert_eq!(session.access_token, "shpua_online_token_67890");
    assert!(session.expires.is_some());
    assert_eq!(
        session.shopify_session_id,
        Some("shopify_session_abc123".to_string())
    );

    // Verify associated user
    let user = session.associated_user.as_ref().unwrap();
    assert_eq!(user.id, 12345);
    assert_eq!(user.first_name, "Jane");
    assert_eq!(user.email, "jane@example.com");
    assert!(user.account_owner);

    // Verify user-specific scopes
    let user_scopes = session.associated_user_scopes.as_ref().unwrap();
    assert!(user_scopes.iter().any(|s| s == "read_products"));

    // Session should be active (not expired yet)
    assert!(session.is_active());
    assert!(!session.expired());
}

#[test]
fn test_session_serialization_round_trip() {
    let shop = ShopDomain::new("serialize-test").unwrap();
    let user = AssociatedUser::new(
        99999,
        "Test".to_string(),
        "User".to_string(),
        "test@example.com".to_string(),
        true,
        false,
        "fr".to_string(),
        true,
    );

    let original = Session::with_user(
        Session::generate_online_id(&shop, 99999),
        shop,
        "test-token".to_string(),
        "read_products,write_orders".parse().unwrap(),
        None,
        user,
        Some("read_products".parse().unwrap()),
    );

    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();

    // Deserialize from JSON
    let restored: Session = serde_json::from_str(&json).unwrap();

    // Verify all fields match
    assert_eq!(original.id, restored.id);
    assert_eq!(original.shop, restored.shop);
    assert_eq!(original.access_token, restored.access_token);
    assert_eq!(original.scopes, restored.scopes);
    assert_eq!(original.is_online, restored.is_online);
    assert_eq!(original.associated_user, restored.associated_user);
    assert_eq!(
        original.associated_user_scopes,
        restored.associated_user_scopes
    );
}

#[test]
fn test_session_with_all_fields_serializes_correctly() {
    use chrono::{Duration, Utc};

    let shop = ShopDomain::new("full-session").unwrap();
    let user = AssociatedUser::new(
        11111,
        "Full".to_string(),
        "Test".to_string(),
        "full@example.com".to_string(),
        true,
        true,
        "en".to_string(),
        false,
    );

    let mut session = Session::with_user(
        Session::generate_online_id(&shop, 11111),
        shop,
        "full-token".to_string(),
        "read_products,write_orders,read_customers".parse().unwrap(),
        Some(Utc::now() + Duration::hours(24)),
        user,
        Some("read_products,read_customers".parse().unwrap()),
    );
    session.state = Some("oauth-state-12345".to_string());
    session.shopify_session_id = Some("shopify-session-xyz".to_string());

    let json = serde_json::to_string_pretty(&session).unwrap();

    // Verify JSON contains all expected fields
    assert!(json.contains("full-session.myshopify.com_11111"));
    assert!(json.contains("full-token"));
    assert!(json.contains("oauth-state-12345"));
    assert!(json.contains("shopify-session-xyz"));
    assert!(json.contains("Full"));
    assert!(json.contains("full@example.com"));
    // Pretty print uses spaces around colon
    assert!(json.contains("\"is_online\": true"));
}

#[test]
fn test_expired_vs_active_session_detection() {
    use chrono::{Duration, Utc};

    let shop = ShopDomain::new("expiry-test").unwrap();

    // Create an already expired session
    let expired_session = Session::new(
        "expired-session".to_string(),
        shop.clone(),
        "token".to_string(),
        "read_products".parse().unwrap(),
        true,
        Some(Utc::now() - Duration::hours(1)), // Expired 1 hour ago
    );

    // Create a still-valid session
    let active_session = Session::new(
        "active-session".to_string(),
        shop.clone(),
        "token".to_string(),
        "read_products".parse().unwrap(),
        true,
        Some(Utc::now() + Duration::hours(1)), // Expires in 1 hour
    );

    // Create an offline session (never expires)
    let offline_session = Session::new(
        "offline-session".to_string(),
        shop,
        "token".to_string(),
        "read_products".parse().unwrap(),
        false,
        None,
    );

    // Verify expired session
    assert!(expired_session.expired());
    assert!(!expired_session.is_active());

    // Verify active session
    assert!(!active_session.expired());
    assert!(active_session.is_active());

    // Verify offline session
    assert!(!offline_session.expired());
    assert!(offline_session.is_active());
}

#[test]
fn test_session_can_be_shared_across_threads() {
    use std::sync::Arc;
    use std::thread;

    let shop = ShopDomain::new("thread-test").unwrap();
    let session = Arc::new(Session::new(
        Session::generate_offline_id(&shop),
        shop,
        "thread-safe-token".to_string(),
        "read_products".parse().unwrap(),
        false,
        None,
    ));

    // Spawn multiple threads that access the session
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let session = Arc::clone(&session);
            thread::spawn(move || {
                assert_eq!(session.access_token, "thread-safe-token");
                assert!(session.is_active());
                format!("Thread {} completed", i)
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
}

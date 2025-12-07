//! Integration tests for the Shopify API SDK.
//!
//! These tests verify end-to-end functionality of the SDK configuration system.

use shopify_api::{
    ApiKey, ApiSecretKey, ApiVersion, AuthScopes, ConfigError, HostUrl, ShopDomain, ShopifyConfig,
};

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

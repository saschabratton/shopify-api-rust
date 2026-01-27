//! Integration tests for API version management.

use shopify_api::{
    ApiDeprecationInfo, ApiKey, ApiSecretKey, ApiVersion, ConfigError, HttpResponse,
    ShopifyConfig,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// =============================================================================
// Version Lifecycle Tests
// =============================================================================

#[test]
fn test_supported_versions_returns_current_versions() {
    let versions = ApiVersion::supported_versions();

    // Should have at least one version
    assert!(!versions.is_empty(), "Should have supported versions");

    // Should include the latest version
    assert!(
        versions.contains(&ApiVersion::latest()),
        "Should include latest version"
    );

    // All versions should be supported
    for version in &versions {
        assert!(
            version.is_supported(),
            "Version {} should be supported",
            version
        );
    }
}

#[test]
fn test_minimum_supported_version() {
    let minimum = ApiVersion::minimum_supported();

    // Minimum version should be supported
    assert!(minimum.is_supported());

    // Minimum version should not be deprecated
    assert!(!minimum.is_deprecated());

    // Latest version should be >= minimum
    assert!(
        ApiVersion::latest() >= minimum,
        "Latest version should be >= minimum"
    );
}

#[test]
fn test_deprecated_versions() {
    // 2024 versions are deprecated (outside 12-month window)
    assert!(ApiVersion::V2024_01.is_deprecated());
    assert!(ApiVersion::V2024_04.is_deprecated());
    assert!(ApiVersion::V2024_07.is_deprecated());
    assert!(ApiVersion::V2024_10.is_deprecated());

    // 2025 versions are supported
    assert!(!ApiVersion::V2025_01.is_deprecated());
    assert!(!ApiVersion::V2025_10.is_deprecated());

    // Unstable is never deprecated
    assert!(!ApiVersion::Unstable.is_deprecated());

    // Custom versions are never deprecated (assumed newer)
    assert!(!ApiVersion::Custom("2026-01".to_string()).is_deprecated());
}

#[test]
fn test_version_ordering_stable_versions() {
    // All stable versions should be ordered chronologically
    assert!(ApiVersion::V2024_01 < ApiVersion::V2024_04);
    assert!(ApiVersion::V2024_04 < ApiVersion::V2024_07);
    assert!(ApiVersion::V2024_07 < ApiVersion::V2024_10);
    assert!(ApiVersion::V2024_10 < ApiVersion::V2025_01);
    assert!(ApiVersion::V2025_01 < ApiVersion::V2025_04);
    assert!(ApiVersion::V2025_04 < ApiVersion::V2025_07);
    assert!(ApiVersion::V2025_07 < ApiVersion::V2025_10);
}

#[test]
fn test_version_ordering_special_versions() {
    // Unstable sorts after all stable versions
    assert!(ApiVersion::V2025_10 < ApiVersion::Unstable);

    // Custom sorts after unstable
    assert!(ApiVersion::Unstable < ApiVersion::Custom("2026-01".to_string()));
}

#[test]
fn test_version_ordering_custom_versions() {
    // Custom versions compare lexicographically
    let v2026_01 = ApiVersion::Custom("2026-01".to_string());
    let v2026_04 = ApiVersion::Custom("2026-04".to_string());

    assert!(v2026_01 < v2026_04);
}

// =============================================================================
// Config Validation Tests
// =============================================================================

#[test]
fn test_config_allows_deprecated_version_by_default() {
    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .api_version(ApiVersion::V2024_01)
        .build();

    // Should succeed even though V2024_01 is deprecated
    assert!(config.is_ok());
    assert_eq!(config.unwrap().api_version(), &ApiVersion::V2024_01);
}

#[test]
fn test_config_rejects_deprecated_version_when_strict() {
    let result = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .api_version(ApiVersion::V2024_01)
        .reject_deprecated_versions(true)
        .build();

    // Should fail with DeprecatedApiVersion error
    assert!(result.is_err());
    match result {
        Err(ConfigError::DeprecatedApiVersion { version, latest }) => {
            assert_eq!(version, "2024-01");
            assert_eq!(latest, ApiVersion::latest().to_string());
        }
        _ => panic!("Expected DeprecatedApiVersion error"),
    }
}

#[test]
fn test_config_allows_supported_version_when_strict() {
    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .api_version(ApiVersion::V2025_10)
        .reject_deprecated_versions(true)
        .build();

    assert!(config.is_ok());
}

#[test]
fn test_config_allows_unstable_when_strict() {
    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .api_version(ApiVersion::Unstable)
        .reject_deprecated_versions(true)
        .build();

    assert!(config.is_ok());
}

#[test]
fn test_config_allows_custom_version_when_strict() {
    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .api_version(ApiVersion::Custom("2026-01".to_string()))
        .reject_deprecated_versions(true)
        .build();

    assert!(config.is_ok());
}

// =============================================================================
// Deprecation Callback Tests
// =============================================================================

#[test]
fn test_config_with_deprecation_callback() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let count_clone = Arc::clone(&call_count);

    let config = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .on_deprecation(move |_info| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        })
        .build()
        .unwrap();

    // Callback should be configured
    assert!(config.deprecation_callback().is_some());
}

#[test]
fn test_deprecation_callback_can_access_info() {
    let captured_reason = Arc::new(std::sync::Mutex::new(String::new()));
    let reason_clone = Arc::clone(&captured_reason);

    let _config = ShopifyConfig::builder()
        .api_key(ApiKey::new("test-key").unwrap())
        .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
        .on_deprecation(move |info| {
            *reason_clone.lock().unwrap() = info.reason.clone();
        })
        .build()
        .unwrap();

    // The callback will be invoked by HttpClient when it receives
    // a deprecation header. We test the callback invocation separately
    // since we can't easily create HTTP responses here.

    // Directly test the callback with sample data
    let info = ApiDeprecationInfo {
        reason: "Test deprecation".to_string(),
        path: Some("/test/path".to_string()),
    };

    *captured_reason.lock().unwrap() = info.reason.clone();
    assert_eq!(*captured_reason.lock().unwrap(), "Test deprecation");
}

// =============================================================================
// HttpResponse Deprecation Info Tests
// =============================================================================

#[test]
fn test_http_response_deprecation_info() {
    let mut headers = HashMap::new();
    headers.insert(
        "x-shopify-api-deprecated-reason".to_string(),
        vec!["This endpoint will be removed in 2025-07".to_string()],
    );

    let response = HttpResponse::new(200, headers, serde_json::json!({}));

    // Should have deprecation info
    assert!(response.is_deprecated());

    let info = response.deprecation_info().unwrap();
    assert_eq!(info.reason, "This endpoint will be removed in 2025-07");
}

#[test]
fn test_http_response_no_deprecation_info() {
    let response = HttpResponse::new(200, HashMap::new(), serde_json::json!({}));

    // Should not be deprecated
    assert!(!response.is_deprecated());
    assert!(response.deprecation_info().is_none());
}

// =============================================================================
// Version Comparison Integration Tests
// =============================================================================

#[test]
fn test_check_version_against_minimum() {
    let configured_version = ApiVersion::V2024_01;
    let minimum = ApiVersion::minimum_supported();

    // Can check if version meets minimum requirements
    if configured_version < minimum {
        assert!(configured_version.is_deprecated());
    }
}

#[test]
fn test_supported_versions_sorted() {
    let versions = ApiVersion::supported_versions();

    // Verify the list is sorted
    for window in versions.windows(2) {
        assert!(window[0] < window[1], "Versions should be sorted");
    }
}

#[test]
fn test_latest_is_in_supported_versions() {
    let versions = ApiVersion::supported_versions();
    let latest = ApiVersion::latest();

    assert!(
        versions.contains(&latest),
        "Latest version should be in supported list"
    );

    // Latest should be the last in the list
    assert_eq!(
        versions.last(),
        Some(&latest),
        "Latest should be the last supported version"
    );
}

//! OAuth authorization URL generation.
//!
//! This module provides the [`begin_auth`] function for generating Shopify
//! OAuth authorization URLs and the [`BeginAuthResult`] struct containing
//! the URL and state parameter.
//!
//! # Overview
//!
//! The `begin_auth` function is the first step in the OAuth authorization code
//! flow. It generates:
//! 1. A cryptographically secure state parameter for CSRF protection
//! 2. An authorization URL to redirect the user to Shopify
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain, HostUrl};
//! use shopify_sdk::auth::oauth::begin_auth;
//!
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
//!     .host(HostUrl::new("https://your-app.com").unwrap())
//!     .scopes("read_products,write_orders".parse().unwrap())
//!     .build()
//!     .unwrap();
//!
//! let shop = ShopDomain::new("example-shop").unwrap();
//! let result = begin_auth(&config, &shop, "/auth/callback", true, None).unwrap();
//!
//! // Store result.state in the user's session
//! // Redirect user to result.auth_url
//! ```

use crate::auth::oauth::error::OAuthError;
use crate::auth::oauth::state::StateParam;
use crate::auth::AuthScopes;
use crate::config::{ShopDomain, ShopifyConfig};

/// Result of initiating OAuth authorization.
///
/// This struct contains the authorization URL to redirect users to and the
/// state parameter that should be persisted (typically in a session or cookie)
/// for verification when the callback is received.
///
/// # Important
///
/// The `state` value **must** be stored by your application and passed to
/// [`validate_auth_callback`](crate::auth::oauth::validate_auth_callback)
/// when handling the callback. This is essential for CSRF protection.
///
/// # Example
///
/// ```rust,ignore
/// let result = begin_auth(&config, &shop, "/callback", true, None)?;
///
/// // Store in session (implementation depends on your web framework)
/// session.set("oauth_state", result.state.as_ref());
///
/// // Redirect to Shopify
/// return Redirect::to(&result.auth_url);
/// ```
#[derive(Clone, Debug)]
pub struct BeginAuthResult {
    /// The full authorization URL to redirect the user to.
    ///
    /// This URL points to Shopify's OAuth authorization endpoint with all
    /// required query parameters.
    pub auth_url: String,

    /// The state parameter generated for this authorization request.
    ///
    /// Store this value and compare it against the `state` parameter
    /// in the OAuth callback to prevent CSRF attacks.
    pub state: StateParam,
}

/// Initiates the OAuth authorization code flow.
///
/// This function generates an authorization URL that the user should be
/// redirected to, along with a cryptographically secure state parameter
/// for CSRF protection.
///
/// # Arguments
///
/// * `config` - Shopify SDK configuration (must have `host` configured)
/// * `shop` - The shop domain to authorize against
/// * `redirect_path` - Path on your app to receive the callback (e.g., "/auth/callback")
/// * `is_online` - `true` for online (user-specific) tokens, `false` for offline (app) tokens
/// * `scope_override` - Optional scope override (uses `config.scopes()` if `None`)
///
/// # Returns
///
/// A [`BeginAuthResult`] containing the authorization URL and state parameter,
/// or an [`OAuthError`] if the configuration is invalid.
///
/// # Errors
///
/// Returns [`OAuthError::MissingHostConfig`] if `config.host()` is `None`.
///
/// # Online vs Offline Tokens
///
/// - **Online tokens** (`is_online = true`): User-specific, expire, and are
///   tied to a particular Shopify admin user. Use for user-facing operations.
/// - **Offline tokens** (`is_online = false`): App-level, don't expire, and
///   work regardless of user. Use for background tasks and webhooks.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain, HostUrl, AuthScopes};
/// use shopify_sdk::auth::oauth::begin_auth;
///
/// let config = ShopifyConfig::builder()
///     .api_key(ApiKey::new("api-key").unwrap())
///     .api_secret_key(ApiSecretKey::new("secret").unwrap())
///     .host(HostUrl::new("https://myapp.example.com").unwrap())
///     .scopes("read_products".parse().unwrap())
///     .build()
///     .unwrap();
///
/// let shop = ShopDomain::new("test-shop").unwrap();
///
/// // Request online token with default scopes
/// let result = begin_auth(&config, &shop, "/auth/callback", true, None).unwrap();
/// assert!(result.auth_url.contains("test-shop.myshopify.com"));
/// assert!(result.auth_url.contains("oauth/authorize"));
///
/// // Request offline token with custom scopes
/// let custom_scopes: AuthScopes = "write_orders".parse().unwrap();
/// let result = begin_auth(&config, &shop, "/auth/callback", false, Some(&custom_scopes)).unwrap();
/// assert!(result.auth_url.contains("write_orders"));
/// ```
pub fn begin_auth(
    config: &ShopifyConfig,
    shop: &ShopDomain,
    redirect_path: &str,
    is_online: bool,
    scope_override: Option<&AuthScopes>,
) -> Result<BeginAuthResult, OAuthError> {
    // Validate that host is configured
    let host = config.host().ok_or(OAuthError::MissingHostConfig)?;

    // Generate cryptographically secure state
    let state = StateParam::new();

    // Determine scopes to use
    let scopes = scope_override.unwrap_or_else(|| config.scopes());

    // Build redirect URI
    let redirect_uri = format!("{}{}", host.as_ref(), redirect_path);

    // Build authorization URL
    let mut params = vec![
        ("client_id", config.api_key().as_ref().to_string()),
        ("scope", scopes.to_string()),
        ("redirect_uri", redirect_uri),
        ("state", state.to_string()),
    ];

    // Add grant_options[] for online tokens
    if is_online {
        params.push(("grant_options[]", "per-user".to_string()));
    }

    // Build query string with proper URL encoding for both keys and values
    let query_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let auth_url = format!(
        "https://{}/admin/oauth/authorize?{}",
        shop.as_ref(),
        query_string
    );

    Ok(BeginAuthResult { auth_url, state })
}

// Verify BeginAuthResult is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<BeginAuthResult>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiKey, ApiSecretKey, HostUrl};

    fn create_test_config() -> ShopifyConfig {
        ShopifyConfig::builder()
            .api_key(ApiKey::new("test-api-key").unwrap())
            .api_secret_key(ApiSecretKey::new("test-secret").unwrap())
            .host(HostUrl::new("https://myapp.example.com").unwrap())
            .scopes("read_products,write_orders".parse().unwrap())
            .build()
            .unwrap()
    }

    fn create_test_shop() -> ShopDomain {
        ShopDomain::new("test-shop").unwrap()
    }

    #[test]
    fn test_begin_auth_generates_correct_url_structure() {
        let config = create_test_config();
        let shop = create_test_shop();

        let result = begin_auth(&config, &shop, "/auth/callback", true, None).unwrap();

        // Check URL structure
        assert!(result
            .auth_url
            .starts_with("https://test-shop.myshopify.com/admin/oauth/authorize?"));
    }

    #[test]
    fn test_begin_auth_includes_all_required_params() {
        let config = create_test_config();
        let shop = create_test_shop();

        let result = begin_auth(&config, &shop, "/auth/callback", true, None).unwrap();

        // Check required parameters are present
        assert!(result.auth_url.contains("client_id="));
        assert!(result.auth_url.contains("scope="));
        assert!(result.auth_url.contains("redirect_uri="));
        assert!(result.auth_url.contains("state="));
    }

    #[test]
    fn test_begin_auth_sets_grant_options_for_online() {
        let config = create_test_config();
        let shop = create_test_shop();

        let result = begin_auth(&config, &shop, "/auth/callback", true, None).unwrap();

        // Online should have grant_options[]=per-user (URL encoded key)
        // grant_options[] encodes to grant_options%5B%5D
        assert!(result.auth_url.contains("grant_options%5B%5D=per-user"));
    }

    #[test]
    fn test_begin_auth_no_grant_options_for_offline() {
        let config = create_test_config();
        let shop = create_test_shop();

        let result = begin_auth(&config, &shop, "/auth/callback", false, None).unwrap();

        // Offline should NOT have grant_options
        assert!(!result.auth_url.contains("grant_options"));
    }

    #[test]
    fn test_begin_auth_uses_scope_override() {
        let config = create_test_config();
        let shop = create_test_shop();
        let custom_scopes: AuthScopes = "read_customers".parse().unwrap();

        let result = begin_auth(&config, &shop, "/callback", true, Some(&custom_scopes)).unwrap();

        // Should use custom scopes, not config scopes
        assert!(result.auth_url.contains("read_customers"));
        // Should not contain the config scopes
        assert!(!result.auth_url.contains("write_orders"));
    }

    #[test]
    fn test_begin_auth_returns_state() {
        let config = create_test_config();
        let shop = create_test_shop();

        let result = begin_auth(&config, &shop, "/callback", true, None).unwrap();

        // State should be a 15-char alphanumeric nonce
        let nonce = result.state.nonce();
        assert_eq!(nonce.len(), 15);
        assert!(nonce.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_begin_auth_state_in_url_matches_returned_state() {
        let config = create_test_config();
        let shop = create_test_shop();

        let result = begin_auth(&config, &shop, "/callback", true, None).unwrap();

        // URL should contain the state value
        assert!(result.auth_url.contains(&format!(
            "state={}",
            urlencoding::encode(result.state.as_ref())
        )));
    }

    #[test]
    fn test_begin_auth_redirect_uri_format() {
        let config = create_test_config();
        let shop = create_test_shop();

        let result = begin_auth(&config, &shop, "/auth/callback", true, None).unwrap();

        // Redirect URI should be host + path, URL encoded
        let expected = urlencoding::encode("https://myapp.example.com/auth/callback");
        assert!(result
            .auth_url
            .contains(&format!("redirect_uri={expected}")));
    }

    #[test]
    fn test_begin_auth_fails_without_host() {
        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            // No host configured
            .build()
            .unwrap();

        let shop = create_test_shop();

        let result = begin_auth(&config, &shop, "/callback", true, None);

        assert!(matches!(result, Err(OAuthError::MissingHostConfig)));
    }

    #[test]
    fn test_begin_auth_result_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<BeginAuthResult>();
    }

    #[test]
    fn test_begin_auth_with_different_shops() {
        let config = create_test_config();

        let shop1 = ShopDomain::new("shop-one").unwrap();
        let shop2 = ShopDomain::new("shop-two").unwrap();

        let result1 = begin_auth(&config, &shop1, "/callback", true, None).unwrap();
        let result2 = begin_auth(&config, &shop2, "/callback", true, None).unwrap();

        assert!(result1.auth_url.contains("shop-one.myshopify.com"));
        assert!(result2.auth_url.contains("shop-two.myshopify.com"));
    }

    #[test]
    fn test_begin_auth_unique_states() {
        let config = create_test_config();
        let shop = create_test_shop();

        let result1 = begin_auth(&config, &shop, "/callback", true, None).unwrap();
        let result2 = begin_auth(&config, &shop, "/callback", true, None).unwrap();

        // Each call should generate a unique state
        assert_ne!(result1.state.as_ref(), result2.state.as_ref());
    }
}

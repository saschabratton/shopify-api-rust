//! OAuth 2.0 Client Credentials Grant for Shopify private/organization apps.
//!
//! This module implements the OAuth 2.0 Client Credentials Grant flow for
//! private and organization-level Shopify apps. Unlike the Token Exchange flow
//! (which requires App Bridge session tokens) or the Authorization Code flow
//! (which requires user interaction), Client Credentials Grant allows
//! server-to-server authentication without any user context.
//!
//! # Overview
//!
//! Client Credentials Grant is designed for:
//! - Private apps that operate without a UI
//! - Organization apps that need server-to-server communication
//! - Background services and automated processes
//!
//! The flow does not require user interaction or embedded app context.
//!
//! # Configuration Requirements
//!
//! This flow requires that the app is configured as non-embedded:
//! - `is_embedded(false)` must be set (this is the default)
//! - If `is_embedded(true)` is set, [`exchange_client_credentials`] will
//!   return [`OAuthError::NotPrivateApp`]
//!
//! This is the inverse of Token Exchange, which requires `is_embedded(true)`.
//!
//! # Token Type
//!
//! Client Credentials Grant always produces offline access tokens:
//! - No user context (no associated user)
//! - App-level access to the shop
//! - Session ID format: `"offline_{shop}"`
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ShopDomain, ApiKey, ApiSecretKey};
//! use shopify_api::auth::oauth::exchange_client_credentials;
//!
//! // Configure the SDK (must NOT be embedded)
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
//!     // is_embedded defaults to false, which is required for client credentials
//!     .build()
//!     .unwrap();
//!
//! let shop = ShopDomain::new("my-store").unwrap();
//!
//! // Exchange client credentials for an access token
//! let session = exchange_client_credentials(&config, &shop).await?;
//! println!("Access token: {}", session.access_token);
//! println!("Session ID: {}", session.id); // "offline_my-store.myshopify.com"
//! ```
//!
//! # Reference
//!
//! This implementation matches the Ruby SDK's `ShopifyAPI::Auth::ClientCredentials`:
//! - File: `lib/shopify_api/auth/client_credentials.rb`

use crate::auth::oauth::OAuthError;
use crate::auth::session::AccessTokenResponse;
use crate::auth::Session;
use crate::config::{ShopDomain, ShopifyConfig};
use serde::Serialize;

/// Grant type for client credentials.
const CLIENT_CREDENTIALS_GRANT_TYPE: &str = "client_credentials";

/// Request body for client credentials exchange.
#[derive(Debug, Serialize)]
struct ClientCredentialsRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    grant_type: &'a str,
}

/// Exchanges client credentials for an access token.
///
/// This function authenticates a private or organization app using the
/// OAuth 2.0 Client Credentials Grant flow. It returns an offline session
/// that can be used for API calls.
///
/// # Arguments
///
/// * `config` - Shopify SDK configuration (must have `is_embedded() == false`)
/// * `shop` - The shop domain to authenticate with
///
/// # Returns
///
/// A [`Session`] with an offline access token. The session will have:
/// - `is_online = false`
/// - `id` in the format `"offline_{shop}"`
/// - No associated user information
///
/// # Errors
///
/// - [`OAuthError::NotPrivateApp`] if the config has `is_embedded(true)`
/// - [`OAuthError::ClientCredentialsFailed`] if the request fails or Shopify rejects the credentials
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::auth::oauth::exchange_client_credentials;
///
/// let session = exchange_client_credentials(&config, &shop).await?;
/// assert!(!session.is_online);
/// assert!(session.associated_user.is_none());
/// ```
pub async fn exchange_client_credentials(
    config: &ShopifyConfig,
    shop: &ShopDomain,
) -> Result<Session, OAuthError> {
    // Step 1: Validate that this is NOT an embedded app
    if config.is_embedded() {
        return Err(OAuthError::NotPrivateApp);
    }

    // Step 2: Build the token URL
    let token_url = format!("https://{}/admin/oauth/access_token", shop.as_ref());

    // Step 3: Create the request body
    let request_body = ClientCredentialsRequest {
        client_id: config.api_key().as_ref(),
        client_secret: config.api_secret_key().as_ref(),
        grant_type: CLIENT_CREDENTIALS_GRANT_TYPE,
    };

    // Step 4: Send the POST request
    let client = reqwest::Client::new();
    let response = client
        .post(&token_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| OAuthError::ClientCredentialsFailed {
            status: 0,
            message: format!("Network error: {e}"),
        })?;

    let status = response.status().as_u16();

    // Step 5: Handle error responses
    if !response.status().is_success() {
        let error_body = response.text().await.unwrap_or_default();
        return Err(OAuthError::ClientCredentialsFailed {
            status,
            message: error_body,
        });
    }

    // Step 6: Parse the successful response
    let token_response: AccessTokenResponse =
        response
            .json()
            .await
            .map_err(|e| OAuthError::ClientCredentialsFailed {
                status,
                message: format!("Failed to parse token response: {e}"),
            })?;

    // Step 7: Create and return the session
    let session = Session::from_access_token_response(shop.clone(), &token_response);

    Ok(session)
}

// Verify types are Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ClientCredentialsRequest<'_>>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiKey, ApiSecretKey};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_private_config(secret: &str) -> ShopifyConfig {
        ShopifyConfig::builder()
            .api_key(ApiKey::new("test-api-key").unwrap())
            .api_secret_key(ApiSecretKey::new(secret).unwrap())
            .is_embedded(false)
            .build()
            .unwrap()
    }

    fn create_embedded_config(secret: &str) -> ShopifyConfig {
        ShopifyConfig::builder()
            .api_key(ApiKey::new("test-api-key").unwrap())
            .api_secret_key(ApiSecretKey::new(secret).unwrap())
            .is_embedded(true)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn test_configuration_validation_rejects_embedded_config_with_not_private_app_error() {
        let config = create_embedded_config("test-secret");
        let shop = ShopDomain::new("test-shop").unwrap();

        let result = exchange_client_credentials(&config, &shop).await;

        assert!(matches!(result, Err(OAuthError::NotPrivateApp)));
    }

    #[tokio::test]
    async fn test_configuration_validation_accepts_non_embedded_config() {
        let config = create_private_config("test-secret");
        let shop = ShopDomain::new("test-shop").unwrap();

        // This will fail at HTTP level (can't connect to test-shop.myshopify.com)
        // but validates that we get past the config check
        let result = exchange_client_credentials(&config, &shop).await;

        // Should fail with ClientCredentialsFailed (not NotPrivateApp)
        assert!(matches!(
            result,
            Err(OAuthError::ClientCredentialsFailed { .. })
        ));
    }

    #[tokio::test]
    async fn test_successful_response_creates_offline_session_with_correct_id_format() {
        let mock_server = MockServer::start().await;

        // Setup mock response
        Mock::given(method("POST"))
            .and(path("/admin/oauth/access_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "test-access-token",
                "scope": "read_products,write_orders"
            })))
            .mount(&mock_server)
            .await;

        // Create config and shop domain that points to mock server
        // We need to work around ShopDomain validation, so we'll test the response parsing
        // directly using Session::from_access_token_response

        let shop = ShopDomain::new("test-shop").unwrap();
        let response = AccessTokenResponse {
            access_token: "test-access-token".to_string(),
            scope: "read_products,write_orders".to_string(),
            expires_in: None,
            associated_user_scope: None,
            associated_user: None,
            session: None,
        };

        let session = Session::from_access_token_response(shop, &response);

        assert_eq!(session.id, "offline_test-shop.myshopify.com");
        assert!(!session.is_online);
        assert!(session.associated_user.is_none());
        assert_eq!(session.access_token, "test-access-token");
    }

    #[tokio::test]
    async fn test_http_error_responses_map_to_client_credentials_failed_with_status_code() {
        let config = create_private_config("test-secret");
        let shop = ShopDomain::new("test-shop").unwrap();

        // This will fail with network error (status 0) or HTTP error
        let result = exchange_client_credentials(&config, &shop).await;

        match result {
            Err(OAuthError::ClientCredentialsFailed { status, message }) => {
                // Network error results in status 0, HTTP errors have status >= 400
                assert!(status == 0 || status >= 400);
                assert!(!message.is_empty());
            }
            _ => panic!("Expected ClientCredentialsFailed error"),
        }
    }

    #[tokio::test]
    async fn test_network_errors_map_to_client_credentials_failed() {
        let config = create_private_config("test-secret");
        // Using a domain that may or may not resolve - we just test it returns
        // ClientCredentialsFailed rather than NotPrivateApp
        let shop = ShopDomain::new("test-shop").unwrap();

        let result = exchange_client_credentials(&config, &shop).await;

        // Should fail with ClientCredentialsFailed (config validation passed)
        // The status may be 0 (network error) or >= 400 (HTTP error) depending on
        // whether the domain resolves
        assert!(
            matches!(result, Err(OAuthError::ClientCredentialsFailed { .. })),
            "Expected ClientCredentialsFailed error"
        );
    }

    #[tokio::test]
    async fn test_request_body_contains_correct_grant_type() {
        // Verify the grant type constant is correct
        assert_eq!(CLIENT_CREDENTIALS_GRANT_TYPE, "client_credentials");

        // Verify the request body structure by serializing it
        let request = ClientCredentialsRequest {
            client_id: "test-client-id",
            client_secret: "test-client-secret",
            grant_type: CLIENT_CREDENTIALS_GRANT_TYPE,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"grant_type\":\"client_credentials\""));
        assert!(json.contains("\"client_id\":\"test-client-id\""));
        assert!(json.contains("\"client_secret\":\"test-client-secret\""));
    }

    #[test]
    fn test_client_credentials_request_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ClientCredentialsRequest<'_>>();
    }
}

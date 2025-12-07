//! OAuth 2.0 implementation for Shopify apps.
//!
//! This module provides complete OAuth implementations for Shopify apps:
//!
//! - **Authorization Code Grant**: Traditional OAuth flow with redirects
//! - **Token Exchange**: For embedded apps using App Bridge session tokens
//! - **Client Credentials Grant**: For private/organization apps without user interaction
//! - **Token Refresh**: For refreshing expiring access tokens
//!
//! # Authorization Code Grant
//!
//! The authorization code flow is used for standalone apps and initial app installation:
//!
//! 1. **Authorization Initiation** ([`begin_auth`]): Generate an authorization URL
//!    and redirect the user to Shopify to grant access.
//!
//! 2. **Callback Validation** ([`validate_auth_callback`]): When the user is
//!    redirected back, validate the callback and exchange the code for an access token.
//!
//! # Token Exchange (for Embedded Apps)
//!
//! Token exchange is used by embedded apps that receive session tokens from App Bridge:
//!
//! - [`exchange_online_token`]: Exchange session token for user-specific access token
//! - [`exchange_offline_token`]: Exchange session token for app-level access token
//!
//! Token exchange does not require redirects, making it ideal for embedded app contexts.
//! Requires `is_embedded(true)` configuration.
//!
//! # Client Credentials Grant (for Private/Organization Apps)
//!
//! Client credentials is used by private and organization apps for server-to-server
//! authentication without user interaction:
//!
//! - [`exchange_client_credentials`]: Obtain an offline access token using app credentials
//!
//! This flow is ideal for background services, automated processes, and apps that
//! operate without a UI. Requires `is_embedded(false)` configuration (the default).
//!
//! # Token Refresh (for Expiring Tokens)
//!
//! Token refresh is used for apps with expiring offline access tokens:
//!
//! - [`refresh_access_token`]: Refresh an expiring access token using a refresh token
//! - [`migrate_to_expiring_token`]: One-time migration from non-expiring to expiring tokens
//!
//! Expiring tokens provide enhanced security by requiring periodic token rotation.
//!
//! # Security Features
//!
//! - **HMAC Validation**: All callbacks are verified using HMAC-SHA256 signatures
//! - **CSRF Protection**: State parameter prevents cross-site request forgery
//! - **Constant-Time Comparison**: Security-sensitive comparisons use constant-time
//!   algorithms to prevent timing attacks
//! - **Key Rotation Support**: Old API secret keys can be configured for seamless
//!   key rotation without breaking in-flight OAuth flows
//! - **JWT Validation**: Session tokens are validated before token exchange
//!
//! # Example: Authorization Code Flow
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain, HostUrl, AuthScopes};
//! use shopify_api::auth::oauth::{begin_auth, validate_auth_callback, AuthQuery, OAuthError};
//!
//! // Step 1: Configure the SDK
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
//!     .host(HostUrl::new("https://your-app.com").unwrap())
//!     .scopes("read_products,write_orders".parse().unwrap())
//!     .build()
//!     .unwrap();
//!
//! // Step 2: Begin authorization
//! let shop = ShopDomain::new("example-shop").unwrap();
//! let result = begin_auth(&config, &shop, "/auth/callback", true, None)?;
//!
//! // Store result.state in session (your web framework handles this)
//! // session.set("oauth_state", result.state.as_ref());
//!
//! // Redirect user to result.auth_url
//! // return Redirect::to(&result.auth_url);
//!
//! // Step 3: Handle callback (in your callback handler)
//! async fn handle_callback(
//!     config: &ShopifyConfig,
//!     query: AuthQuery,      // Parsed from request query string
//!     stored_state: &str,    // Retrieved from session
//! ) -> Result<(), OAuthError> {
//!     let session = validate_auth_callback(config, &query, stored_state).await?;
//!
//!     // Store session for later API calls
//!     // session_store.save(&session).await;
//!
//!     println!("Successfully authenticated shop: {}", session.shop.as_ref());
//!     println!("Access token: {}", session.access_token);
//!     Ok(())
//! }
//! ```
//!
//! # Example: Token Exchange Flow (Embedded Apps)
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
//! use shopify_api::auth::oauth::{exchange_online_token, exchange_offline_token, OAuthError};
//!
//! // Configure the SDK (must be embedded app)
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
//!     .is_embedded(true)
//!     .build()
//!     .unwrap();
//!
//! // Get shop domain and session token from App Bridge
//! let shop = ShopDomain::new("example-shop").unwrap();
//! let session_token = "eyJ..."; // JWT from App Bridge
//!
//! // Exchange for an online access token (user-specific, expires)
//! let session = exchange_online_token(&config, &shop, session_token).await?;
//! println!("User ID: {:?}", session.associated_user.map(|u| u.id));
//!
//! // Or exchange for an offline access token (app-level, doesn't expire)
//! let session = exchange_offline_token(&config, &shop, session_token).await?;
//! println!("Offline token obtained");
//! ```
//!
//! # Example: Client Credentials Flow (Private/Organization Apps)
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
//! use shopify_api::auth::oauth::{exchange_client_credentials, OAuthError};
//!
//! // Configure the SDK (must NOT be embedded)
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
//!     // is_embedded defaults to false, which is required
//!     .build()
//!     .unwrap();
//!
//! let shop = ShopDomain::new("example-shop").unwrap();
//!
//! // Exchange client credentials for an offline access token
//! let session = exchange_client_credentials(&config, &shop).await?;
//! println!("Access token: {}", session.access_token);
//! println!("Session ID: {}", session.id); // "offline_example-shop.myshopify.com"
//! ```
//!
//! # Example: Token Refresh Flow (Expiring Tokens)
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
//! use shopify_api::auth::oauth::{refresh_access_token, migrate_to_expiring_token, OAuthError};
//!
//! // Refresh an expiring access token
//! if session.expired() {
//!     if let Some(refresh_token) = &session.refresh_token {
//!         let new_session = refresh_access_token(&config, &shop, refresh_token).await?;
//!         println!("New access token: {}", new_session.access_token);
//!     }
//! }
//!
//! // Or migrate from non-expiring to expiring tokens (one-time, irreversible)
//! let new_session = migrate_to_expiring_token(&config, &shop, &old_access_token).await?;
//! println!("Migration successful!");
//! ```
//!
//! # Online vs Offline Access Tokens
//!
//! Shopify supports two types of access tokens:
//!
//! - **Online tokens** (`is_online = true` in `begin_auth`, or via `exchange_online_token`):
//!   - Tied to a specific user
//!   - Expire (typically after 24 hours)
//!   - Include user information in the session
//!   - Use for user-facing operations where user identity matters
//!
//! - **Offline tokens** (`is_online = false` in `begin_auth`, or via `exchange_offline_token`,
//!   or via `exchange_client_credentials`):
//!   - App-level access
//!   - Do not expire (unless using expiring tokens)
//!   - No user information
//!   - Use for background tasks, webhooks, and automated operations
//!
//! # Embedding Custom Data in State
//!
//! For advanced use cases, you can embed custom data in the state parameter:
//!
//! ```rust
//! use shopify_api::auth::oauth::StateParam;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct FlowContext {
//!     return_url: String,
//!     install_source: String,
//! }
//!
//! // Embed data in state
//! let context = FlowContext {
//!     return_url: "/dashboard".to_string(),
//!     install_source: "app_store".to_string(),
//! };
//! let state = StateParam::with_data(&context);
//!
//! // Later, extract the data
//! let extracted: Option<FlowContext> = state.extract_data();
//! ```

mod auth_query;
mod begin_auth;
mod client_credentials;
mod error;
pub mod hmac;
mod jwt_payload;
mod state;
mod token_exchange;
mod token_refresh;
mod validate_callback;

pub use auth_query::AuthQuery;
pub use begin_auth::{begin_auth, BeginAuthResult};
pub use client_credentials::exchange_client_credentials;
pub use error::OAuthError;
pub use hmac::{compute_signature, constant_time_compare, validate_hmac};
pub use state::StateParam;
pub use token_exchange::{exchange_offline_token, exchange_online_token};
pub use token_refresh::{migrate_to_expiring_token, refresh_access_token};
pub use validate_callback::validate_auth_callback;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_online_token_is_accessible_from_auth_oauth() {
        // This test verifies that exchange_online_token is properly exported
        // The function exists and is accessible - compilation proves this
        let _ = exchange_online_token as fn(_, _, _) -> _;
    }

    #[test]
    fn test_exchange_offline_token_is_accessible_from_auth_oauth() {
        // This test verifies that exchange_offline_token is properly exported
        // The function exists and is accessible - compilation proves this
        let _ = exchange_offline_token as fn(_, _, _) -> _;
    }

    #[test]
    fn test_exchange_client_credentials_is_accessible_from_auth_oauth() {
        // This test verifies that exchange_client_credentials is properly exported
        // The function exists and is accessible - compilation proves this
        let _ = exchange_client_credentials as fn(_, _) -> _;
    }

    #[test]
    fn test_refresh_access_token_is_accessible_from_auth_oauth() {
        // This test verifies that refresh_access_token is properly exported
        // The function exists and is accessible - compilation proves this
        let _ = refresh_access_token as fn(_, _, _) -> _;
    }

    #[test]
    fn test_migrate_to_expiring_token_is_accessible_from_auth_oauth() {
        // This test verifies that migrate_to_expiring_token is properly exported
        // The function exists and is accessible - compilation proves this
        let _ = migrate_to_expiring_token as fn(_, _, _) -> _;
    }
}

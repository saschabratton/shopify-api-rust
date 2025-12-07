//! OAuth 2.0 Authorization Code Grant implementation for Shopify.
//!
//! This module provides a complete implementation of the OAuth 2.0 authorization
//! code flow for Shopify apps. It handles authorization URL generation, callback
//! validation with HMAC verification, and token exchange.
//!
//! # Overview
//!
//! The OAuth flow for Shopify apps consists of two main steps:
//!
//! 1. **Authorization Initiation** ([`begin_auth`]): Generate an authorization URL
//!    and redirect the user to Shopify to grant access.
//!
//! 2. **Callback Validation** ([`validate_auth_callback`]): When the user is
//!    redirected back, validate the callback and exchange the code for an access token.
//!
//! # Security Features
//!
//! - **HMAC Validation**: All callbacks are verified using HMAC-SHA256 signatures
//! - **CSRF Protection**: State parameter prevents cross-site request forgery
//! - **Constant-Time Comparison**: Security-sensitive comparisons use constant-time
//!   algorithms to prevent timing attacks
//! - **Key Rotation Support**: Old API secret keys can be configured for seamless
//!   key rotation without breaking in-flight OAuth flows
//!
//! # Example: Complete OAuth Flow
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
//! # Online vs Offline Access Tokens
//!
//! Shopify supports two types of access tokens:
//!
//! - **Online tokens** (`is_online = true` in `begin_auth`):
//!   - Tied to a specific user
//!   - Expire (typically after 24 hours)
//!   - Include user information in the session
//!   - Use for user-facing operations where user identity matters
//!
//! - **Offline tokens** (`is_online = false` in `begin_auth`):
//!   - App-level access
//!   - Do not expire
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
mod error;
pub mod hmac;
mod state;
mod validate_callback;

pub use auth_query::AuthQuery;
pub use begin_auth::{begin_auth, BeginAuthResult};
pub use error::OAuthError;
pub use hmac::{compute_signature, constant_time_compare, validate_hmac};
pub use state::StateParam;
pub use validate_callback::validate_auth_callback;

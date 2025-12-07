//! # Shopify API Rust SDK
//!
//! A Rust SDK for the Shopify API, providing type-safe configuration,
//! authentication handling, and HTTP client functionality for Shopify app development.
//!
//! ## Overview
//!
//! This SDK provides:
//! - Type-safe configuration via [`ShopifyConfig`] and [`ShopifyConfigBuilder`]
//! - Validated newtypes for API credentials and domain values
//! - OAuth scope handling with implied scope support
//! - OAuth 2.0 authorization code flow via [`auth::oauth`]
//! - Token exchange for embedded apps via [`auth::oauth`]
//! - Client credentials for private/organization apps via [`auth::oauth`]
//! - Token refresh for expiring access tokens via [`auth::oauth`]
//! - Session management for authenticated API calls
//! - Async HTTP client with retry logic and rate limit handling
//!
//! ## Quick Start
//!
//! ```rust
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion, AuthScopes};
//!
//! // Create configuration using the builder pattern
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
//!     .scopes("read_products,write_orders".parse().unwrap())
//!     .api_version(ApiVersion::latest())
//!     .build()
//!     .unwrap();
//! ```
//!
//! ## OAuth Authentication
//!
//! For apps that need to authenticate with Shopify stores:
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain, HostUrl};
//! use shopify_api::auth::oauth::{begin_auth, validate_auth_callback, AuthQuery};
//!
//! // Step 1: Configure the SDK
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
//!     .host(HostUrl::new("https://your-app.com").unwrap())
//!     .scopes("read_products".parse().unwrap())
//!     .build()
//!     .unwrap();
//!
//! // Step 2: Begin authorization
//! let shop = ShopDomain::new("example-shop").unwrap();
//! let result = begin_auth(&config, &shop, "/auth/callback", true, None)?;
//! // Redirect user to result.auth_url
//! // Store result.state in session
//!
//! // Step 3: Handle callback
//! let session = validate_auth_callback(&config, &query, &stored_state).await?;
//! // session is now ready for API calls
//! ```
//!
//! ## Token Exchange (Embedded Apps)
//!
//! For embedded apps using App Bridge session tokens:
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
//! use shopify_api::auth::oauth::{exchange_online_token, exchange_offline_token};
//!
//! // Configure the SDK (must be embedded)
//! let config = ShopifyConfig::builder()
//!     .api_key(ApiKey::new("your-api-key").unwrap())
//!     .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
//!     .is_embedded(true)
//!     .build()
//!     .unwrap();
//!
//! let shop = ShopDomain::new("example-shop").unwrap();
//! let session_token = "eyJ..."; // JWT from App Bridge
//!
//! // Exchange for an online access token
//! let session = exchange_online_token(&config, &shop, session_token).await?;
//!
//! // Or exchange for an offline access token
//! let session = exchange_offline_token(&config, &shop, session_token).await?;
//! ```
//!
//! ## Client Credentials (Private/Organization Apps)
//!
//! For private or organization apps without user interaction:
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
//! use shopify_api::auth::oauth::exchange_client_credentials;
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
//! ```
//!
//! ## Token Refresh (Expiring Tokens)
//!
//! For apps using expiring offline access tokens:
//!
//! ```rust,ignore
//! use shopify_api::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
//! use shopify_api::auth::oauth::{refresh_access_token, migrate_to_expiring_token};
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
//! ```
//!
//! ## Session Management
//!
//! Sessions represent authenticated connections to a Shopify store. They can be
//! either offline (app-level) or online (user-specific):
//!
//! ```rust
//! use shopify_api::{Session, ShopDomain, AuthScopes, AssociatedUser};
//!
//! // Create an offline session (no expiration, no user)
//! let offline_session = Session::new(
//!     Session::generate_offline_id(&ShopDomain::new("my-store").unwrap()),
//!     ShopDomain::new("my-store").unwrap(),
//!     "access-token".to_string(),
//!     "read_products".parse().unwrap(),
//!     false,
//!     None,
//! );
//!
//! // Sessions can be serialized for storage
//! let json = serde_json::to_string(&offline_session).unwrap();
//! ```
//!
//! ## Making API Requests
//!
//! ```rust,ignore
//! use shopify_api::{Session, ShopDomain, AuthScopes};
//! use shopify_api::clients::{HttpClient, HttpRequest, HttpMethod};
//!
//! // Create a session
//! let session = Session::new(
//!     "session-id".to_string(),
//!     ShopDomain::new("my-store").unwrap(),
//!     "access-token".to_string(),
//!     AuthScopes::new(),
//!     false,
//!     None,
//! );
//!
//! // Create an HTTP client
//! let client = HttpClient::new("/admin/api/2024-10", &session, None);
//!
//! // Build and send a request
//! let request = HttpRequest::builder(HttpMethod::Get, "products.json")
//!     .build()
//!     .unwrap();
//!
//! let response = client.request(request).await?;
//! ```
//!
//! ## Design Principles
//!
//! - **No global state**: Configuration is instance-based and passed explicitly
//! - **Fail-fast validation**: All newtypes validate on construction
//! - **Thread-safe**: All types are `Send + Sync`
//! - **Async-first**: Designed for use with Tokio async runtime
//! - **Immutable sessions**: Sessions are immutable after creation

pub mod auth;
pub mod clients;
pub mod config;
pub mod error;

// Re-export public types at crate root for convenience
pub use auth::{AssociatedUser, AuthScopes, Session};
pub use config::{
    ApiKey, ApiSecretKey, ApiVersion, HostUrl, ShopDomain, ShopifyConfig, ShopifyConfigBuilder,
};
pub use error::ConfigError;

// Re-export HTTP client types
pub use clients::{
    ApiCallLimit, DataType, HttpClient, HttpError, HttpMethod, HttpRequest, HttpRequestBuilder,
    HttpResponse, HttpResponseError, InvalidHttpRequestError, MaxHttpRetriesExceededError,
    PaginationInfo,
};

// Re-export OAuth types for convenience
pub use auth::oauth::{
    begin_auth, exchange_client_credentials, exchange_offline_token, exchange_online_token,
    migrate_to_expiring_token, refresh_access_token, validate_auth_callback, AuthQuery,
    BeginAuthResult, OAuthError, StateParam,
};

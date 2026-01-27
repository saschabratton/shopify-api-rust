# OAuth Authentication

This guide covers all OAuth flows supported by the SDK for authenticating with Shopify stores.

## Table of Contents

- [Overview](#overview)
- [Flow Comparison](#flow-comparison)
- [Token Exchange (Embedded Apps)](#token-exchange-embedded-apps)
- [Authorization Code Grant](#authorization-code-grant)
- [Client Credentials (Private/Organization Apps)](#client-credentials-privateorganization-apps)
- [Expiring Tokens](#expiring-tokens)
- [Error Handling](#error-handling)

## Overview

OAuth is required for public apps that need to authenticate with multiple Shopify stores. The SDK supports four OAuth flows:

1. **Token Exchange** - For embedded apps using App Bridge session tokens
2. **Authorization Code Grant** - Traditional OAuth with browser redirects
3. **Client Credentials** - For private/organization apps without user interaction
4. **Token Refresh** - For refreshing expiring access tokens

> **Note:** If you're building a custom app with direct Admin API access, see [Custom Apps](custom_apps.md) instead.

## Flow Comparison

| Flow | Best For | Requires Redirect | User Interaction |
|------|----------|-------------------|------------------|
| Token Exchange | Embedded apps | No | No (App Bridge handles it) |
| Authorization Code | Standalone apps, installation | Yes | Yes |
| Client Credentials | Private/org apps | No | No |
| Token Refresh | Expiring tokens | No | No |

## Token Exchange (Embedded Apps)

Token exchange is the recommended flow for embedded apps. It exchanges an App Bridge session token (JWT) for a Shopify access token without any redirects.

### Configuration

Token exchange requires `is_embedded(true)`:

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};

let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
    .is_embedded(true)  // Required for token exchange
    .build()
    .unwrap();
```

### Online Token Exchange

Use `exchange_online_token` for user-specific operations. Online tokens expire (~24 hours) and include user information:

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
use shopify_sdk::auth::oauth::exchange_online_token;

let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
    .is_embedded(true)
    .build()
    .unwrap();

let shop = ShopDomain::new("example-shop").unwrap();
let session_token = "eyJ...";  // JWT from App Bridge

let session = exchange_online_token(&config, &shop, session_token).await?;

// Access user information
if let Some(user) = &session.associated_user {
    println!("User ID: {}", user.id);
    println!("Email: {:?}", user.email);
}
```

### Offline Token Exchange

Use `exchange_offline_token` for app-level operations that don't require a user context:

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
use shopify_sdk::auth::oauth::exchange_offline_token;

let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
    .is_embedded(true)
    .build()
    .unwrap();

let shop = ShopDomain::new("example-shop").unwrap();
let session_token = "eyJ...";  // JWT from App Bridge

let session = exchange_offline_token(&config, &shop, session_token).await?;

// Store this session for background operations
println!("Session ID: {}", session.id);
println!("Access token: {}", session.access_token);
```

> **Tip:** Store offline tokens for webhook processing and background jobs. They don't expire unless you're using the expiring tokens feature.

## Authorization Code Grant

The authorization code flow is used for standalone apps and initial app installation. It involves redirecting the user to Shopify and back to your app.

### Step 1: Begin Authorization

Generate an authorization URL and redirect the user:

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain, HostUrl};
use shopify_sdk::auth::oauth::begin_auth;

let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
    .host(HostUrl::new("https://your-app.example.com").unwrap())
    .scopes("read_products,write_orders".parse().unwrap())
    .build()
    .unwrap();

let shop = ShopDomain::new("example-shop").unwrap();

let result = begin_auth(
    &config,
    &shop,
    "/auth/callback",  // Your callback path
    true,              // Request online token (false for offline)
    None,              // Optional custom state data
)?;

// Store the state for CSRF validation
// session.set("oauth_state", result.state.as_ref());

// Redirect user to Shopify
// return Redirect::to(&result.auth_url);
println!("Redirect to: {}", result.auth_url);
```

### Step 2: Handle Callback

When Shopify redirects back, validate the callback and exchange the code for a token:

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};
use shopify_sdk::auth::oauth::{validate_auth_callback, AuthQuery};

// Parse query parameters from the callback URL
let query = AuthQuery {
    code: "authorization-code-from-shopify".to_string(),
    shop: "example-shop.myshopify.com".to_string(),
    state: "state-from-callback".to_string(),
    timestamp: "1234567890".to_string(),
    host: Some("base64-encoded-host".to_string()),
    hmac: "hmac-signature".to_string(),
};

// Retrieve stored state
let stored_state = "stored-state-from-session";

// Validate and exchange
let session = validate_auth_callback(&config, &query, stored_state).await?;

println!("Successfully authenticated: {}", session.shop.as_ref());
println!("Access token: {}", session.access_token);
```

### Custom State Data

You can embed custom data in the state parameter for advanced use cases:

```rust
use shopify_sdk::auth::oauth::{begin_auth, StateParam};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct FlowContext {
    return_url: String,
    install_source: String,
}

// Create state with custom data
let context = FlowContext {
    return_url: "/dashboard".to_string(),
    install_source: "app_store".to_string(),
};
let state = StateParam::with_data(&context);

// Use in begin_auth
let result = begin_auth(&config, &shop, "/callback", true, Some(state))?;

// Later, extract the data after callback validation
let extracted: Option<FlowContext> = result.state.extract_data();
```

## Client Credentials (Private/Organization Apps)

Client credentials flow is for private and organization apps that operate without user interaction.

> **Warning:** This flow requires `is_embedded(false)` (the default). It cannot be used with embedded apps.

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
use shopify_sdk::auth::oauth::exchange_client_credentials;

let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
    // is_embedded defaults to false
    .build()
    .unwrap();

let shop = ShopDomain::new("example-shop").unwrap();

let session = exchange_client_credentials(&config, &shop).await?;

println!("Access token: {}", session.access_token);
println!("Session ID: {}", session.id);  // "offline_example-shop.myshopify.com"
```

## Expiring Tokens

Shopify supports expiring offline tokens for enhanced security. These tokens have a limited lifetime and include a refresh token.

### Checking Token Expiration

```rust
use shopify_sdk::Session;

if session.expired() {
    println!("Token has expired, refresh required");
}

// Check when it expires
if let Some(expires_at) = session.expires_at {
    println!("Expires at: {}", expires_at);
}
```

### Refreshing Tokens

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
use shopify_sdk::auth::oauth::refresh_access_token;

if session.expired() {
    if let Some(refresh_token) = &session.refresh_token {
        let new_session = refresh_access_token(
            &config,
            &session.shop,
            refresh_token
        ).await?;

        println!("New access token: {}", new_session.access_token);
        println!("New refresh token: {:?}", new_session.refresh_token);

        // Store the new session
    }
}
```

### Migrating to Expiring Tokens

You can migrate existing non-expiring tokens to expiring tokens:

> **Warning:** This is a one-time, irreversible operation. Once migrated, you must use token refresh.

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
use shopify_sdk::auth::oauth::migrate_to_expiring_token;

let new_session = migrate_to_expiring_token(
    &config,
    &session.shop,
    &session.access_token
).await?;

println!("Migration successful!");
println!("New access token: {}", new_session.access_token);
println!("Refresh token: {:?}", new_session.refresh_token);
```

## Error Handling

All OAuth functions return `Result<_, OAuthError>`. Handle errors appropriately:

```rust
use shopify_sdk::auth::oauth::OAuthError;

match exchange_online_token(&config, &shop, session_token).await {
    Ok(session) => {
        println!("Success: {}", session.access_token);
    }
    Err(OAuthError::InvalidSessionToken { reason }) => {
        println!("Invalid session token: {}", reason);
    }
    Err(OAuthError::NotEmbeddedApp) => {
        println!("Token exchange requires is_embedded(true)");
    }
    Err(OAuthError::HmacValidationFailed) => {
        println!("HMAC signature validation failed");
    }
    Err(OAuthError::StateMismatch) => {
        println!("OAuth state doesn't match stored state");
    }
    Err(OAuthError::HttpError(e)) => {
        println!("HTTP error during OAuth: {}", e);
    }
    Err(OAuthError::TokenExchangeFailed { message }) => {
        println!("Token exchange failed: {}", message);
    }
    Err(e) => {
        println!("Other OAuth error: {}", e);
    }
}
```

### Common Error Variants

| Error | Cause | Solution |
|-------|-------|----------|
| `NotEmbeddedApp` | Token exchange with `is_embedded(false)` | Set `is_embedded(true)` |
| `IsEmbeddedApp` | Client credentials with `is_embedded(true)` | Set `is_embedded(false)` |
| `InvalidSessionToken` | Malformed or expired JWT | Request a new session token from App Bridge |
| `HmacValidationFailed` | Invalid HMAC signature | Check API secret key, possible tampering |
| `StateMismatch` | CSRF check failed | State expired or was modified |
| `HostNotConfigured` | Missing host URL | Set `host()` in config |
| `TokenExchangeFailed` | Shopify rejected the request | Check credentials and permissions |

## Security Features

The SDK includes several security features:

- **HMAC Validation** - All callbacks are verified using HMAC-SHA256
- **CSRF Protection** - State parameter prevents cross-site request forgery
- **Constant-Time Comparison** - Prevents timing attacks on sensitive comparisons
- **Key Rotation** - Configure `old_api_secret_key` for seamless key rotation
- **JWT Validation** - Session tokens are validated before exchange

```rust
// Key rotation example
let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("key").unwrap())
    .api_secret_key(ApiSecretKey::new("new-secret").unwrap())
    .old_api_secret_key(ApiSecretKey::new("old-secret").unwrap())
    .build()
    .unwrap();
```

## Next Steps

- [GraphQL Admin API](graphql.md) - Make authenticated API calls
- [Webhooks](webhooks.md) - Set up webhook subscriptions
- [Custom Apps](custom_apps.md) - Skip OAuth for custom apps

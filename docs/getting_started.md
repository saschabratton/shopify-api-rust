# Getting Started

This guide covers installation, configuration, and basic concepts for using the Shopify API Rust SDK.

## Requirements

- **Rust 1.70+** - The SDK uses features that require Rust 1.70 or later
- **Tokio runtime** - All async operations use Tokio
- **Shopify Partner account** - Required to create apps and obtain API credentials

## Installation

Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
shopify-sdk = "1.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1.0"
```

## Configuration

The SDK uses `ShopifyConfig` to hold all configuration. Create one using the builder pattern:

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion, HostUrl};

let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
    .host(HostUrl::new("https://your-app.example.com").unwrap())
    .scopes("read_products,write_orders".parse().unwrap())
    .api_version(ApiVersion::latest())
    .is_embedded(true)
    .build()
    .unwrap();
```

### Configuration Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `api_key` | `ApiKey` | Yes | - | Your app's API key from the Partner Dashboard |
| `api_secret_key` | `ApiSecretKey` | Yes | - | Your app's API secret key |
| `host` | `HostUrl` | No | None | Your app's public URL (required for OAuth redirects) |
| `scopes` | `AuthScopes` | No | Empty | OAuth scopes your app requires |
| `api_version` | `ApiVersion` | No | Latest | Shopify API version to use |
| `is_embedded` | `bool` | No | `true` | Whether your app is embedded in Shopify Admin |
| `old_api_secret_key` | `ApiSecretKey` | No | None | Previous secret key for rotation support |
| `user_agent_prefix` | `String` | No | None | Custom prefix for HTTP User-Agent header |

### Environment Variables

A common pattern is to load configuration from environment variables:

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, HostUrl};
use std::env;

let config = ShopifyConfig::builder()
    .api_key(ApiKey::new(&env::var("SHOPIFY_API_KEY").unwrap()).unwrap())
    .api_secret_key(ApiSecretKey::new(&env::var("SHOPIFY_API_SECRET").unwrap()).unwrap())
    .host(HostUrl::new(&env::var("HOST").unwrap()).unwrap())
    .scopes(env::var("SCOPES").unwrap().parse().unwrap())
    .build()
    .unwrap();
```

## Sessions

Sessions represent an authenticated connection to a Shopify store. They contain the access token and metadata needed to make API calls.

### Online vs Offline Sessions

Shopify supports two types of access tokens:

| Type | Use Case | Expiration | User Info |
|------|----------|------------|-----------|
| **Online** | User-facing operations | ~24 hours | Yes |
| **Offline** | Background tasks, webhooks | Never* | No |

*Unless using expiring tokens feature.

### Creating Sessions

Sessions are typically created through OAuth flows, but you can also create them directly for custom apps:

```rust
use shopify_sdk::{Session, ShopDomain, AuthScopes};

// Offline session (for background operations)
let session = Session::new(
    Session::generate_offline_id(&ShopDomain::new("my-store").unwrap()),
    ShopDomain::new("my-store").unwrap(),
    "your-access-token".to_string(),
    "read_products,write_orders".parse().unwrap(),
    false,  // is_online
    None,   // associated_user
);
```

### Session Serialization

Sessions can be serialized for storage in databases or caches:

```rust
use shopify_sdk::Session;

// Serialize to JSON
let json = serde_json::to_string(&session).unwrap();

// Deserialize from JSON
let session: Session = serde_json::from_str(&json).unwrap();
```

> **Tip:** Store offline sessions for webhook processing and background jobs. Online sessions should be stored in user session storage (like cookies) and refreshed as needed.

## API Versions

The SDK supports multiple Shopify API versions:

```rust
use shopify_sdk::ApiVersion;

// Use the latest stable version (recommended)
let version = ApiVersion::latest();

// Or specify a specific version
let version = ApiVersion::V2024_10;
let version = ApiVersion::V2025_01;
let version = ApiVersion::V2025_04;
let version = ApiVersion::V2025_07;
let version = ApiVersion::V2025_10;
```

> **Note:** Using `ApiVersion::latest()` ensures you always use the most recent stable API version.

## Next Steps

- [OAuth Authentication](usage/oauth.md) - Set up user authentication
- [GraphQL Admin API](usage/graphql.md) - Make GraphQL API calls
- [Webhooks](usage/webhooks.md) - Subscribe to Shopify events
- [Custom Apps](usage/custom_apps.md) - Direct token usage without OAuth

# shopify-sdk

[![Crates.io](https://img.shields.io/crates/v/shopify-sdk.svg)](https://crates.io/crates/shopify-sdk)
[![Documentation](https://docs.rs/shopify-sdk/badge.svg)](https://docs.rs/shopify-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)

A Rust SDK for the Shopify API, providing type-safe configuration, OAuth authentication, and clients for both GraphQL and REST APIs.

## Overview

This SDK provides everything you need to build Shopify apps in Rust:

- **OAuth 2.0 Authentication** - Token exchange, authorization code flow, and client credentials
- **GraphQL Admin API** - Modern, recommended API for Shopify development
- **Storefront API** - Build headless commerce experiences
- **REST Admin API** - Legacy REST resources with ActiveRecord-like patterns
- **Webhook System** - Registration and verification of webhook subscriptions
- **Session Management** - Online and offline access tokens with serialization support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
shopify-sdk = "1.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1.0"
```

## Quick Start

### Configuration

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ApiVersion, HostUrl};

let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
    .host(HostUrl::new("https://your-app.example.com").unwrap())
    .scopes("read_products,write_orders".parse().unwrap())
    .api_version(ApiVersion::latest())
    .build()
    .unwrap();
```

### OAuth Token Exchange (Embedded Apps)

```rust
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey, ShopDomain};
use shopify_sdk::auth::oauth::exchange_online_token;

// For embedded apps using App Bridge session tokens
let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
    .is_embedded(true)
    .build()
    .unwrap();

let shop = ShopDomain::new("example-shop").unwrap();
let session_token = "eyJ..."; // JWT from App Bridge

// Exchange for an access token
let session = exchange_online_token(&config, &shop, session_token).await?;
```

### GraphQL API (Recommended)

```rust
use shopify_sdk::{GraphqlClient, Session, ShopDomain, AuthScopes};
use serde_json::json;

// Create a client from a session
let client = GraphqlClient::new(&session, None);

// Query the shop
let response = client.query("query { shop { name } }", None, None, None).await?;
println!("Shop: {}", response.body["data"]["shop"]["name"]);

// Query with variables
let response = client.query(
    "query GetProduct($id: ID!) { product(id: $id) { title } }",
    Some(json!({ "id": "gid://shopify/Product/123" })),
    None,
    None
).await?;
```

### Webhook Registration

```rust
use shopify_sdk::{
    WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
};

let mut registry = WebhookRegistry::new();

// Register for order creation events
registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::OrdersCreate,
        WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks/orders".to_string(),
        },
    )
    .build()
);

// Sync with Shopify when you have a session
let results = registry.register_all(&session, &config).await?;
```

## Requirements

- Rust 1.70 or later
- A Shopify Partner account and app credentials

## Documentation

- [Getting Started](docs/getting_started.md) - Installation and configuration
- [OAuth Authentication](docs/usage/oauth.md) - All OAuth flows explained
- [GraphQL Admin API](docs/usage/graphql.md) - Modern API usage
- [Storefront API](docs/usage/graphql_storefront.md) - Headless commerce
- [Webhooks](docs/usage/webhooks.md) - Event subscriptions
- [REST Admin API](docs/usage/rest.md) - Legacy API (deprecated)
- [Custom Apps](docs/usage/custom_apps.md) - Direct token usage

See also the [API reference on docs.rs](https://docs.rs/shopify-sdk).

## Design Principles

- **No global state** - Configuration is instance-based and passed explicitly
- **Fail-fast validation** - All newtypes validate on construction
- **Thread-safe** - All types are `Send + Sync`
- **Async-first** - Designed for use with Tokio async runtime
- **Type-safe** - Leverages Rust's type system to prevent errors at compile time

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

This library draws heavily from [`shopify-api-ruby`](https://github.com/Shopify/shopify-api-ruby), Shopify's official Ruby SDK. The patterns in this library have followed its design, adapted for idiomatic Rust. Thank you to the Shopify team ❤️ We are grateful to the maintainers of that library for their work and for their support of the Shopify developer community.

## License

This library is licensed under the [MIT LICENSE](LICENSE).

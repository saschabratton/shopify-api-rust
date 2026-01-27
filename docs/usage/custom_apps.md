# Custom Apps

Custom apps are installed on a single Shopify store and use a direct Admin API access token instead of OAuth. This guide covers setting up and using the SDK with custom apps.

## Overview

Custom apps are ideal when:
- You're building an app for a single store you own or manage
- You need direct API access without OAuth complexity
- You're building internal tools or integrations

> **Note:** For apps distributed to multiple stores, use [OAuth authentication](oauth.md) instead.

## Getting Your Access Token

1. Go to your Shopify admin
2. Navigate to **Settings > Apps and sales channels > Develop apps**
3. Create a new app or select an existing one
4. Under **Configuration**, select the API scopes you need
5. Install the app to get your **Admin API access token**

> **Warning:** Keep your access token secret. Never commit it to version control or expose it in client-side code.

## Creating a Session

Create a session directly with your access token:

```rust
use shopify_sdk::{Session, ShopDomain, AuthScopes};

let session = Session::new(
    Session::generate_offline_id(&ShopDomain::new("my-store").unwrap()),
    ShopDomain::new("my-store").unwrap(),
    "your-admin-api-access-token".to_string(),
    "read_products,write_orders".parse().unwrap(),
    false,  // Custom app tokens are always offline
    None,   // No associated user for offline tokens
);
```

## Making GraphQL Requests

Use the session with the GraphQL client:

```rust
use shopify_sdk::{GraphqlClient, Session, ShopDomain, AuthScopes};
use serde_json::json;

// Create session
let session = Session::new(
    Session::generate_offline_id(&ShopDomain::new("my-store").unwrap()),
    ShopDomain::new("my-store").unwrap(),
    "your-access-token".to_string(),
    "read_products".parse().unwrap(),
    false,
    None,
);

// Create client
let client = GraphqlClient::new(&session, None);

// Query the shop
let response = client.query(
    "query { shop { name primaryDomain { url } } }",
    None,
    None,
    None
).await?;

println!("Shop: {}", response.body["data"]["shop"]["name"]);
```

## Making REST Requests

Use the session with the REST client:

```rust
use shopify_sdk::{RestClient, Session, ShopDomain, AuthScopes};

// Create session
let session = Session::new(
    Session::generate_offline_id(&ShopDomain::new("my-store").unwrap()),
    ShopDomain::new("my-store").unwrap(),
    "your-access-token".to_string(),
    "read_products,write_products".parse().unwrap(),
    false,
    None,
);

// Create client
let client = RestClient::new(&session, None)?;

// Get products
let response = client.get("products", None).await?;
println!("Products: {:?}", response.body);
```

## Webhook Configuration

Configure webhooks using the SDK's webhook registry:

```rust
use shopify_sdk::{
    ShopifyConfig, ApiKey, ApiSecretKey, Session, ShopDomain, AuthScopes,
    WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
};

// Configuration is still needed for webhook registration
let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-api-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-api-secret").unwrap())
    .build()
    .unwrap();

// Create session
let session = Session::new(
    Session::generate_offline_id(&ShopDomain::new("my-store").unwrap()),
    ShopDomain::new("my-store").unwrap(),
    "your-access-token".to_string(),
    "read_products".parse().unwrap(),
    false,
    None,
);

// Configure webhooks
let mut registry = WebhookRegistry::new();

registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::OrdersCreate,
        WebhookDeliveryMethod::Http {
            uri: "https://your-app.example.com/webhooks/orders".to_string(),
        },
    )
    .build()
);

// Register with Shopify
let results = registry.register_all(&session, &config).await?;
```

## Session Serialization

Store your session for reuse across application restarts:

```rust
use shopify_sdk::{Session, ShopDomain, AuthScopes};

// Create session
let session = Session::new(
    Session::generate_offline_id(&ShopDomain::new("my-store").unwrap()),
    ShopDomain::new("my-store").unwrap(),
    "your-access-token".to_string(),
    "read_products".parse().unwrap(),
    false,
    None,
);

// Serialize to JSON
let json = serde_json::to_string(&session).unwrap();

// Store in database, file, or environment
std::fs::write("session.json", &json).unwrap();

// Later, deserialize
let json = std::fs::read_to_string("session.json").unwrap();
let session: Session = serde_json::from_str(&json).unwrap();
```

## Environment-Based Configuration

A recommended pattern for custom apps:

```rust
use shopify_sdk::{Session, ShopDomain, AuthScopes, GraphqlClient};
use std::env;

fn create_session() -> Session {
    let shop = env::var("SHOPIFY_SHOP_DOMAIN")
        .expect("SHOPIFY_SHOP_DOMAIN must be set");
    let token = env::var("SHOPIFY_ACCESS_TOKEN")
        .expect("SHOPIFY_ACCESS_TOKEN must be set");
    let scopes = env::var("SHOPIFY_SCOPES")
        .unwrap_or_else(|_| "read_products".to_string());

    let shop_domain = ShopDomain::new(&shop).unwrap();

    Session::new(
        Session::generate_offline_id(&shop_domain),
        shop_domain,
        token,
        scopes.parse().unwrap(),
        false,
        None,
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = create_session();
    let client = GraphqlClient::new(&session, None);

    let response = client.query(
        "query { shop { name } }",
        None, None, None
    ).await?;

    println!("Connected to: {}", response.body["data"]["shop"]["name"]);
    Ok(())
}
```

## Best Practices

1. **Store tokens securely** - Use environment variables or secure secret management

2. **Use minimal scopes** - Only request the permissions your app needs

3. **Handle rate limits** - The SDK handles retries, but monitor your usage

4. **Rotate tokens periodically** - Generate new tokens and update your configuration

5. **Log API calls** - Track what operations your app performs for debugging

## Limitations

Custom apps have some limitations compared to OAuth apps:

- **Single store only** - Cannot be installed on multiple stores
- **No app store distribution** - Must be installed manually
- **No App Bridge** - Cannot use Shopify App Bridge features
- **Manual token management** - Tokens don't auto-refresh

## Next Steps

- [GraphQL Admin API](graphql.md) - Learn GraphQL operations
- [Webhooks](webhooks.md) - Set up event notifications
- [REST Admin API](rest.md) - Legacy REST operations (if needed)

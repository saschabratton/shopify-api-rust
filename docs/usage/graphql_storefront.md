# Storefront API

The Storefront API is a GraphQL API that provides access to a store's public data for building custom storefronts, mobile apps, and headless commerce experiences.

## Overview

Unlike the Admin API, the Storefront API:
- Provides read-only access to products, collections, and checkout
- Uses different authentication (public or private access tokens)
- Is designed for client-facing applications
- Has different rate limits optimized for high traffic

## Authentication Modes

The Storefront API supports three authentication modes:

| Mode | Token Type | Use Case | Security |
|------|------------|----------|----------|
| **Public** | Public access token | Client-side apps | Safe to expose |
| **Private** | Private access token | Server-side apps | Keep secret |
| **Tokenless** | None | Basic product access | Limited features |

## Creating a Client

### With Public Token

Public tokens are safe to expose in client-side code:

```rust
use shopify_sdk::{StorefrontClient, StorefrontToken, ShopDomain};

let shop = ShopDomain::new("my-store").unwrap();
let token = StorefrontToken::Public("public-access-token".to_string());

let client = StorefrontClient::new(&shop, Some(token), None);
```

### With Private Token

Private tokens have more capabilities but must be kept secret:

```rust
use shopify_sdk::{StorefrontClient, StorefrontToken, ShopDomain};

let shop = ShopDomain::new("my-store").unwrap();
let token = StorefrontToken::Private("private-access-token".to_string());

let client = StorefrontClient::new(&shop, Some(token), None);
```

### Tokenless Access

For basic product access without authentication:

```rust
use shopify_sdk::{StorefrontClient, ShopDomain};

let shop = ShopDomain::new("my-store").unwrap();

let client = StorefrontClient::new(&shop, None, None);
```

> **Note:** Tokenless access has limited functionality and may not be available for all stores.

## Querying Products

```rust
use shopify_sdk::{StorefrontClient, StorefrontToken, ShopDomain};
use serde_json::json;

let shop = ShopDomain::new("my-store").unwrap();
let token = StorefrontToken::Public("your-token".to_string());
let client = StorefrontClient::new(&shop, Some(token), None);

let query = r#"
    query GetProducts($first: Int!) {
        products(first: $first) {
            edges {
                node {
                    id
                    title
                    description
                    priceRange {
                        minVariantPrice {
                            amount
                            currencyCode
                        }
                    }
                    images(first: 1) {
                        edges {
                            node {
                                url
                                altText
                            }
                        }
                    }
                }
            }
        }
    }
"#;

let response = client.query(
    query,
    Some(json!({ "first": 10 })),
    None,
    None
).await?;

let products = &response.body["data"]["products"]["edges"];
for edge in products.as_array().unwrap_or(&vec![]) {
    let product = &edge["node"];
    println!("Product: {}", product["title"]);
}
```

## Working with Collections

```rust
use shopify_sdk::{StorefrontClient, StorefrontToken, ShopDomain};
use serde_json::json;

let client = StorefrontClient::new(&shop, Some(token), None);

let query = r#"
    query GetCollection($handle: String!) {
        collection(handle: $handle) {
            title
            description
            products(first: 20) {
                edges {
                    node {
                        id
                        title
                        availableForSale
                    }
                }
            }
        }
    }
"#;

let response = client.query(
    query,
    Some(json!({ "handle": "featured-products" })),
    None,
    None
).await?;

let collection = &response.body["data"]["collection"];
println!("Collection: {}", collection["title"]);
```

## Cart Operations

Create and manage shopping carts:

```rust
use shopify_sdk::{StorefrontClient, StorefrontToken, ShopDomain};
use serde_json::json;

let client = StorefrontClient::new(&shop, Some(token), None);

// Create a cart
let mutation = r#"
    mutation CreateCart($input: CartInput!) {
        cartCreate(input: $input) {
            cart {
                id
                checkoutUrl
                lines(first: 10) {
                    edges {
                        node {
                            id
                            quantity
                            merchandise {
                                ... on ProductVariant {
                                    id
                                    title
                                }
                            }
                        }
                    }
                }
            }
            userErrors {
                field
                message
            }
        }
    }
"#;

let variables = json!({
    "input": {
        "lines": [
            {
                "merchandiseId": "gid://shopify/ProductVariant/12345",
                "quantity": 1
            }
        ]
    }
});

let response = client.query(mutation, Some(variables), None, None).await?;

let cart = &response.body["data"]["cartCreate"]["cart"];
println!("Cart ID: {}", cart["id"]);
println!("Checkout URL: {}", cart["checkoutUrl"]);
```

## Storefront vs Admin API

| Feature | Storefront API | Admin API |
|---------|---------------|-----------|
| **Purpose** | Customer-facing apps | Store management |
| **Authentication** | Public/private tokens | OAuth access tokens |
| **Data Access** | Public store data | Full store data |
| **Write Access** | Cart, checkout only | Full CRUD operations |
| **Rate Limits** | Higher (50 req/sec) | Lower (2 req/sec) |
| **Client Type** | `StorefrontClient` | `GraphqlClient` |

## Custom API Version

Specify a custom API version:

```rust
use shopify_sdk::{StorefrontClient, StorefrontToken, ShopDomain, ApiVersion};

let client = StorefrontClient::new(
    &shop,
    Some(token),
    Some(ApiVersion::V2024_10)
);
```

## Error Handling

```rust
use shopify_sdk::{StorefrontClient, GraphqlError};

match client.query("query { shop { name } }", None, None, None).await {
    Ok(response) => {
        // Check for GraphQL errors in the response
        if let Some(errors) = response.body.get("errors") {
            println!("GraphQL errors: {:?}", errors);
        } else {
            println!("Success: {:?}", response.body["data"]);
        }
    }
    Err(GraphqlError::HttpError(e)) => {
        println!("HTTP error: {}", e);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## Best Practices

1. **Use public tokens client-side** - They're designed to be exposed safely

2. **Use private tokens server-side** - For enhanced functionality and security

3. **Cache product data** - Storefront API data changes infrequently

4. **Handle inventory carefully** - Product availability can change; check before checkout

5. **Use fragments** - Reuse field selections across queries:
   ```graphql
   fragment ProductCard on Product {
       id
       title
       handle
       featuredImage { url altText }
   }
   ```

## Next Steps

- [GraphQL Admin API](graphql.md) - For store management operations
- [Webhooks](webhooks.md) - Subscribe to store events

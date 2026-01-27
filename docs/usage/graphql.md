# GraphQL Admin API

The GraphQL Admin API is Shopify's recommended API for modern app development. It provides a single endpoint for all data operations with precise field selection.

## Overview

The `GraphqlClient` provides methods for querying and mutating data through Shopify's GraphQL Admin API. This is the preferred approach over REST for most use cases.

## Creating a Client

Create a `GraphqlClient` from an authenticated session:

```rust
use shopify_sdk::{GraphqlClient, Session, ShopDomain, AuthScopes};

// Assuming you have a session from OAuth
let session = Session::new(
    "session-id".to_string(),
    ShopDomain::new("my-store").unwrap(),
    "access-token".to_string(),
    "read_products".parse().unwrap(),
    false,
    None,
);

// Create the client
let client = GraphqlClient::new(&session, None);

// Or with a custom API version
use shopify_sdk::ApiVersion;
let client = GraphqlClient::new(&session, Some(ApiVersion::V2024_10));
```

## Simple Queries

Execute a basic GraphQL query:

```rust
use shopify_sdk::GraphqlClient;

let client = GraphqlClient::new(&session, None);

let response = client.query(
    "query { shop { name primaryDomain { url } } }",
    None,   // variables
    None,   // extra headers
    None,   // max retries
).await?;

// Access the response data
let shop_name = &response.body["data"]["shop"]["name"];
println!("Shop name: {}", shop_name);
```

## Queries with Variables

Use variables for dynamic queries:

```rust
use shopify_sdk::GraphqlClient;
use serde_json::json;

let client = GraphqlClient::new(&session, None);

let query = r#"
    query GetProduct($id: ID!) {
        product(id: $id) {
            title
            description
            status
            variants(first: 10) {
                edges {
                    node {
                        id
                        title
                        price
                    }
                }
            }
        }
    }
"#;

let variables = json!({
    "id": "gid://shopify/Product/123456789"
});

let response = client.query(query, Some(variables), None, None).await?;

let product = &response.body["data"]["product"];
println!("Product: {}", product["title"]);
```

## Mutations

Execute mutations to modify data:

```rust
use shopify_sdk::GraphqlClient;
use serde_json::json;

let client = GraphqlClient::new(&session, None);

let mutation = r#"
    mutation CreateProduct($input: ProductInput!) {
        productCreate(input: $input) {
            product {
                id
                title
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
        "title": "New Product",
        "productType": "Merchandise",
        "vendor": "My Store"
    }
});

let response = client.query(mutation, Some(variables), None, None).await?;

// Check for user errors
if let Some(errors) = response.body["data"]["productCreate"]["userErrors"].as_array() {
    if !errors.is_empty() {
        for error in errors {
            println!("Error on {}: {}", error["field"], error["message"]);
        }
    }
}

// Access created product
let product_id = &response.body["data"]["productCreate"]["product"]["id"];
println!("Created product: {}", product_id);
```

## Response Handling

The response contains the JSON body and metadata:

```rust
use shopify_sdk::GraphqlClient;

let response = client.query("query { shop { name } }", None, None, None).await?;

// Access the JSON body
let data = &response.body["data"];

// Check for GraphQL errors (returned with HTTP 200)
if let Some(errors) = response.body.get("errors") {
    if let Some(errors_array) = errors.as_array() {
        for error in errors_array {
            println!("GraphQL error: {}", error["message"]);
        }
    }
}

// Access HTTP response metadata
println!("API call limit: {:?}", response.api_call_limit);
```

## Debug Mode

Include extra information like query cost:

```rust
use shopify_sdk::GraphqlClient;
use std::collections::HashMap;

let client = GraphqlClient::new(&session, None);

let mut headers = HashMap::new();
headers.insert("X-GraphQL-Cost-Include-Fields".to_string(), "true".to_string());

let response = client.query(
    "query { shop { name } }",
    None,
    Some(headers),
    None
).await?;

// Query cost info is in extensions
if let Some(extensions) = response.body.get("extensions") {
    println!("Query cost: {:?}", extensions["cost"]);
}
```

## API Version Selection

Control which API version to use:

```rust
use shopify_sdk::{GraphqlClient, ApiVersion};

// Use the latest version (from session default)
let client = GraphqlClient::new(&session, None);

// Use a specific version
let client = GraphqlClient::new(&session, Some(ApiVersion::V2024_10));
```

## Error Handling

Handle errors appropriately:

```rust
use shopify_sdk::{GraphqlClient, GraphqlError};

match client.query("query { shop { name } }", None, None, None).await {
    Ok(response) => {
        println!("Success: {:?}", response.body);
    }
    Err(GraphqlError::HttpError(e)) => {
        println!("HTTP error: {}", e);
    }
    Err(e) => {
        println!("GraphQL error: {}", e);
    }
}
```

## Rate Limiting

The SDK automatically handles rate limiting with retries. You can customize the retry behavior:

```rust
use shopify_sdk::GraphqlClient;

let response = client.query(
    "query { shop { name } }",
    None,
    None,
    Some(5),  // max retries
).await?;
```

> **Tip:** Monitor the `api_call_limit` in responses to understand your rate limit usage.

## Best Practices

1. **Request only needed fields** - GraphQL allows precise field selection; use it to minimize response size

2. **Use fragments for repeated fields** - DRY up your queries:
   ```graphql
   fragment ProductFields on Product {
       id
       title
       status
   }
   query { products(first: 10) { edges { node { ...ProductFields } } } }
   ```

3. **Handle user errors** - Mutations return `userErrors` separately from GraphQL errors

4. **Use cursor-based pagination** - For listing resources:
   ```graphql
   query($cursor: String) {
       products(first: 50, after: $cursor) {
           pageInfo { hasNextPage endCursor }
           edges { node { id title } }
       }
   }
   ```

5. **Batch operations** - Use bulk operations for large data sets (see Shopify documentation)

## Next Steps

- [Storefront API](graphql_storefront.md) - Build headless commerce experiences
- [Webhooks](webhooks.md) - Subscribe to real-time events
- [REST Admin API](rest.md) - Legacy REST API (when needed)

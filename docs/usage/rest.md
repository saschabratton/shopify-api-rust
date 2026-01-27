# REST Admin API

> **Warning:** The REST Admin API is deprecated. Shopify recommends using the [GraphQL Admin API](graphql.md) for all new development. The REST API will continue to work but receives fewer updates.

This guide covers using the REST Admin API for legacy integrations and gradual migration scenarios.

## Table of Contents

- [REST Resources](#rest-resources)
- [REST Client](#rest-client)
- [Pagination](#pagination)
- [Available Resources](#available-resources)
- [Migration to GraphQL](#migration-to-graphql)

## REST Resources

REST resources provide an ActiveRecord-like pattern for CRUD operations with dirty tracking.

### Finding Resources

```rust
use shopify_sdk::{RestClient, Session, ShopDomain, AuthScopes};
use shopify_sdk::rest::{RestResource, ResourceResponse};
use shopify_sdk::rest::resources::v2025_10::Product;

// Create client
let session = Session::new(
    "session-id".to_string(),
    ShopDomain::new("my-store").unwrap(),
    "access-token".to_string(),
    "read_products".parse().unwrap(),
    false,
    None,
);
let client = RestClient::new(&session, None)?;

// Find a single resource
let response: ResourceResponse<Product> = Product::find(&client, 123456789, None).await?;
println!("Product: {}", response.title);  // Deref to Product

// List resources
let response: ResourceResponse<Vec<Product>> = Product::all(&client, None).await?;
for product in response.iter() {
    println!("- {}", product.title);
}

// Count resources
let count = Product::count(&client, None).await?;
println!("Total products: {}", count);
```

### Creating Resources

```rust
use shopify_sdk::rest::{RestResource, TrackedResource};
use shopify_sdk::rest::resources::v2025_10::Product;

let product = Product {
    id: None,
    title: "New Product".to_string(),
    body_html: Some("<p>Description</p>".to_string()),
    vendor: Some("My Store".to_string()),
    product_type: Some("Merchandise".to_string()),
    ..Default::default()
};

let mut tracked = TrackedResource::new(product);
let saved = tracked.save(&client).await?;

println!("Created product ID: {:?}", saved.id);
```

### Updating Resources

```rust
use shopify_sdk::rest::{RestResource, TrackedResource, ResourceResponse};
use shopify_sdk::rest::resources::v2025_10::Product;

// Fetch existing product
let response: ResourceResponse<Product> = Product::find(&client, 123456789, None).await?;
let mut tracked = TrackedResource::from_existing(response.into_inner());

// Modify fields
tracked.title = "Updated Title".to_string();
tracked.vendor = Some("New Vendor".to_string());

// Save changes (full update)
let saved = tracked.save(&client).await?;

// Or partial update (only changed fields)
if tracked.is_dirty() {
    let changes = tracked.changed_fields();
    let saved = tracked.save_partial(&client, changes).await?;
    tracked.mark_clean();
}
```

### Deleting Resources

```rust
use shopify_sdk::rest::{RestResource, ResourceResponse};
use shopify_sdk::rest::resources::v2025_10::Product;

let response: ResourceResponse<Product> = Product::find(&client, 123456789, None).await?;
let product = response.into_inner();

product.delete(&client).await?;
println!("Product deleted");
```

## REST Client

For lower-level control, use `RestClient` directly:

```rust
use shopify_sdk::{RestClient, Session, ShopDomain, AuthScopes};
use serde_json::json;

let client = RestClient::new(&session, None)?;

// GET request
let response = client.get("products", None).await?;
println!("Products: {:?}", response.body);

// GET with query parameters
use std::collections::HashMap;
let mut params = HashMap::new();
params.insert("limit".to_string(), "10".to_string());
params.insert("status".to_string(), "active".to_string());
let response = client.get("products", Some(&params)).await?;

// POST request
let body = json!({
    "product": {
        "title": "New Product",
        "product_type": "Merchandise"
    }
});
let response = client.post("products", body, None).await?;

// PUT request
let body = json!({
    "product": {
        "id": 123456789,
        "title": "Updated Title"
    }
});
let response = client.put("products/123456789", body, None).await?;

// DELETE request
client.delete("products/123456789", None).await?;
```

## Pagination

REST responses include pagination information:

```rust
use shopify_sdk::rest::{RestResource, ResourceResponse};
use shopify_sdk::rest::resources::v2025_10::Product;

let response: ResourceResponse<Vec<Product>> = Product::all(&client, None).await?;

// Check for next page
if response.has_next_page() {
    if let Some(page_info) = response.next_page_info() {
        // Use page_info for next request
        let mut params = std::collections::HashMap::new();
        params.insert("page_info".to_string(), page_info);
        params.insert("limit".to_string(), "50".to_string());

        let next_page = Product::all(&client, Some(&params)).await?;
    }
}

// Check for previous page
if response.has_prev_page() {
    if let Some(page_info) = response.prev_page_info() {
        // Similar handling
    }
}
```

### Iterating All Pages

```rust
use shopify_sdk::rest::{RestResource, ResourceResponse};
use shopify_sdk::rest::resources::v2025_10::Product;
use std::collections::HashMap;

let mut all_products = Vec::new();
let mut params: Option<HashMap<String, String>> = None;

loop {
    let response: ResourceResponse<Vec<Product>> = Product::all(
        &client,
        params.as_ref()
    ).await?;

    all_products.extend(response.into_inner());

    if !response.has_next_page() {
        break;
    }

    let mut next_params = HashMap::new();
    next_params.insert(
        "page_info".to_string(),
        response.next_page_info().unwrap()
    );
    params = Some(next_params);
}

println!("Total products fetched: {}", all_products.len());
```

## Available Resources

The SDK provides REST resources for API version 2025-10:

### Products & Inventory
- `Product` - Products
- `Variant` - Product variants
- `ProductImage` - Product images
- `InventoryItem` - Inventory items
- `InventoryLevel` - Inventory levels
- `Location` - Store locations
- `Collect` - Product-collection relationships

### Orders & Customers
- `Order` - Orders
- `DraftOrder` - Draft orders
- `Transaction` - Order transactions
- `Refund` - Refunds
- `Fulfillment` - Fulfillments
- `FulfillmentOrder` - Fulfillment orders
- `FulfillmentService` - Fulfillment services
- `Customer` - Customers
- `GiftCard` - Gift cards

### Collections
- `CustomCollection` - Manual collections
- `SmartCollection` - Automated collections

### Content
- `Page` - Store pages
- `Blog` - Blogs
- `Article` - Blog articles
- `Comment` - Blog comments
- `Theme` - Store themes
- `Asset` - Theme assets
- `Redirect` - URL redirects
- `ScriptTag` - Script tags

### Store Settings
- `Shop` - Shop details (read-only)
- `Policy` - Store policies (read-only)
- `Country` - Shipping countries
- `Province` - Provinces/states
- `Currency` - Currencies (read-only)

### Billing
- `ApplicationCharge` - One-time app charges
- `RecurringApplicationCharge` - Subscription charges
- `UsageCharge` - Usage-based charges

### Discounts
- `PriceRule` - Price rules
- `DiscountCode` - Discount codes

### Access & Metadata
- `AccessScope` - OAuth scopes (read-only)
- `StorefrontAccessToken` - Storefront tokens
- `Metafield` - Metafields
- `User` - Staff accounts (read-only)
- `Event` - Store events (read-only)
- `Webhook` - Webhook subscriptions

## Migration to GraphQL

We recommend migrating to GraphQL for better performance and more features.

### Example: Product Query

**REST:**
```rust
let response = client.get("products/123456789", None).await?;
let product = &response.body["product"];
```

**GraphQL:**
```rust
use shopify_sdk::GraphqlClient;
use serde_json::json;

let client = GraphqlClient::new(&session, None);
let response = client.query(
    "query { product(id: \"gid://shopify/Product/123456789\") { title } }",
    None, None, None
).await?;
let product = &response.body["data"]["product"];
```

### Example: Create Product

**REST:**
```rust
let body = json!({
    "product": {
        "title": "New Product",
        "product_type": "Merchandise"
    }
});
let response = client.post("products", body, None).await?;
```

**GraphQL:**
```rust
use shopify_sdk::GraphqlClient;
use serde_json::json;

let client = GraphqlClient::new(&session, None);
let response = client.query(
    r#"mutation CreateProduct($input: ProductInput!) {
        productCreate(input: $input) {
            product { id title }
            userErrors { field message }
        }
    }"#,
    Some(json!({
        "input": {
            "title": "New Product",
            "productType": "Merchandise"
        }
    })),
    None, None
).await?;
```

### Benefits of GraphQL

- **Precise data fetching** - Request exactly the fields you need
- **Fewer requests** - Get related data in a single query
- **Better typing** - Strong schema validation
- **Bulk operations** - Process large datasets asynchronously
- **Subscriptions** - Coming soon for real-time updates

## Error Handling

```rust
use shopify_sdk::{RestClient, RestError};

match client.get("products", None).await {
    Ok(response) => {
        println!("Products: {:?}", response.body);
    }
    Err(RestError::HttpError(e)) => {
        println!("HTTP error: {}", e);
    }
    Err(RestError::RateLimited { retry_after }) => {
        println!("Rate limited, retry after {:?}", retry_after);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## Next Steps

- [GraphQL Admin API](graphql.md) - Migrate to the recommended API
- [Webhooks](webhooks.md) - Real-time event notifications

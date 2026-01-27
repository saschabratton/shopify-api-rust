# Webhooks

Webhooks allow your app to receive real-time notifications when events occur in a Shopify store. This guide covers registration, delivery methods, and verification.

## Table of Contents

- [Overview](#overview)
- [Two-Phase Registration](#two-phase-registration)
- [Delivery Methods](#delivery-methods)
- [Registration Options](#registration-options)
- [Webhook Handlers](#webhook-handlers)
- [Verification](#verification)
- [Best Practices](#best-practices)

## Overview

The webhook system consists of two main components:

1. **Registration** - Configure which events to subscribe to and where to deliver them
2. **Handling** - Verify and process incoming webhook requests

## Two-Phase Registration

The SDK uses a two-phase registration pattern:

1. **Configure locally** - Add registrations at app startup
2. **Sync with Shopify** - Register with Shopify when you have a valid session

```rust
use shopify_sdk::{
    WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
};

// Phase 1: Configure at startup
let mut registry = WebhookRegistry::new();

registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::OrdersCreate,
        WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks/orders/create".to_string(),
        },
    )
    .build()
);

registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::ProductsUpdate,
        WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks/products/update".to_string(),
        },
    )
    .build()
);

// Phase 2: Register with Shopify (when session is available)
let results = registry.register_all(&session, &config).await?;

for result in &results {
    println!("Topic {:?}: success={}", result.topic, result.success);
}
```

## Delivery Methods

Webhooks can be delivered via HTTP, Amazon EventBridge, or Google Cloud Pub/Sub.

### HTTP Delivery

Standard HTTP POST to your endpoint:

```rust
use shopify_sdk::{
    WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
};

let mut registry = WebhookRegistry::new();

registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::OrdersCreate,
        WebhookDeliveryMethod::Http {
            uri: "https://example.com/api/webhooks/orders".to_string(),
        },
    )
    .build()
);
```

### Amazon EventBridge

Deliver webhooks to AWS EventBridge:

```rust
use shopify_sdk::{
    WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
};

let mut registry = WebhookRegistry::new();

registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::OrdersCreate,
        WebhookDeliveryMethod::EventBridge {
            arn: "arn:aws:events:us-east-1::event-source/aws.partner/shopify.com/123/source".to_string(),
        },
    )
    .build()
);
```

### Google Cloud Pub/Sub

Deliver webhooks to GCP Pub/Sub:

```rust
use shopify_sdk::{
    WebhookRegistry, WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod
};

let mut registry = WebhookRegistry::new();

registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::CustomersCreate,
        WebhookDeliveryMethod::PubSub {
            project_id: "my-gcp-project".to_string(),
            topic_id: "shopify-webhooks".to_string(),
        },
    )
    .build()
);
```

## Registration Options

The `WebhookRegistrationBuilder` supports additional options:

### Include Fields

Request specific fields in the webhook payload:

```rust
registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::OrdersCreate,
        WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks".to_string(),
        },
    )
    .include_fields(vec![
        "id".to_string(),
        "email".to_string(),
        "total_price".to_string(),
    ])
    .build()
);
```

### Metafield Namespaces

Include specific metafield namespaces:

```rust
registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::ProductsUpdate,
        WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks".to_string(),
        },
    )
    .metafield_namespaces(vec!["custom".to_string(), "my_app".to_string()])
    .build()
);
```

### Filters

Filter which events trigger the webhook:

```rust
registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::ProductsCreate,
        WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks".to_string(),
        },
    )
    .filter("vendor:MyVendor".to_string())
    .build()
);
```

## Webhook Handlers

Define handlers to process incoming webhooks:

```rust
use shopify_sdk::webhooks::{
    WebhookHandler, WebhookContext, WebhookError, WebhookRegistry,
    WebhookRegistrationBuilder, WebhookTopic, WebhookDeliveryMethod, BoxFuture
};
use serde_json::Value;

// Define a handler struct
struct OrderCreatedHandler;

impl WebhookHandler for OrderCreatedHandler {
    fn handle<'a>(
        &'a self,
        context: WebhookContext,
        payload: Value,
    ) -> BoxFuture<'a, Result<(), WebhookError>> {
        Box::pin(async move {
            let shop = context.shop_domain().unwrap_or("unknown");
            let topic = context.topic().unwrap_or("unknown");

            println!("Received {} from {}", topic, shop);
            println!("Order ID: {}", payload["id"]);

            // Process the order...

            Ok(())
        })
    }
}

// Register with a handler
let mut registry = WebhookRegistry::new();

registry.add_registration(
    WebhookRegistrationBuilder::new(
        WebhookTopic::OrdersCreate,
        WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks/orders".to_string(),
        },
    )
    .handler(OrderCreatedHandler)
    .build()
);
```

## Verification

Always verify incoming webhooks using HMAC signatures.

### High-Level Verification

Use `verify_webhook` for complete verification with key rotation support:

```rust
use shopify_sdk::webhooks::{WebhookRequest, verify_webhook};
use shopify_sdk::{ShopifyConfig, ApiKey, ApiSecretKey};

// Create config
let config = ShopifyConfig::builder()
    .api_key(ApiKey::new("your-key").unwrap())
    .api_secret_key(ApiSecretKey::new("your-secret").unwrap())
    .build()
    .unwrap();

// Extract from HTTP request headers and body
let request = WebhookRequest::new(
    body_bytes.to_vec(),           // Raw request body
    hmac_header.to_string(),       // X-Shopify-Hmac-SHA256 header
    Some(topic_header.to_string()),    // X-Shopify-Topic header
    Some(shop_header.to_string()),     // X-Shopify-Shop-Domain header
    Some(webhook_id.to_string()),      // X-Shopify-Webhook-Id header
    Some(api_version.to_string()),     // X-Shopify-API-Version header
);

// Verify
match verify_webhook(&config, &request) {
    Ok(context) => {
        println!("Verified webhook from: {:?}", context.shop_domain());
        // Process the webhook...
    }
    Err(e) => {
        println!("Verification failed: {}", e);
        // Reject the request
    }
}
```

### Low-Level Verification

For custom integrations:

```rust
use shopify_sdk::webhooks::verify_hmac;

let body = b"webhook payload";
let hmac_header = "base64-encoded-hmac";
let secret = "your-api-secret";

if verify_hmac(body, hmac_header, secret) {
    println!("Valid signature");
} else {
    println!("Invalid signature - reject request");
}
```

### Webhook Headers

The SDK provides constants for standard Shopify webhook headers:

```rust
use shopify_sdk::webhooks::{
    HEADER_HMAC,        // X-Shopify-Hmac-SHA256
    HEADER_TOPIC,       // X-Shopify-Topic
    HEADER_SHOP_DOMAIN, // X-Shopify-Shop-Domain
    HEADER_WEBHOOK_ID,  // X-Shopify-Webhook-Id
    HEADER_API_VERSION, // X-Shopify-API-Version
};
```

## Single Topic Registration

Register a single topic when needed:

```rust
let result = registry.register(&session, &config, WebhookTopic::OrdersCreate).await?;

if result.success {
    println!("Successfully registered orders/create webhook");
}
```

## Best Practices

1. **Respond quickly** - Return HTTP 200 within 5 seconds. Queue processing for later:

   ```rust
   // In your HTTP handler:
   // 1. Verify the webhook
   // 2. Queue for processing
   // 3. Return 200 immediately

   match verify_webhook(&config, &request) {
       Ok(context) => {
           // Queue for async processing
           job_queue.enqueue(WebhookJob {
               context,
               payload: body.clone(),
           });
           // Return 200 immediately
           HttpResponse::Ok()
       }
       Err(_) => HttpResponse::Unauthorized()
   }
   ```

2. **Implement idempotency** - Use `X-Shopify-Webhook-Id` to handle duplicates:

   ```rust
   let webhook_id = context.webhook_id();
   if already_processed(webhook_id) {
       return Ok(()); // Skip duplicate
   }
   process_and_mark_completed(webhook_id);
   ```

3. **Handle retries gracefully** - Shopify retries failed webhooks for 48 hours

4. **Use key rotation** - Configure `old_api_secret_key` during secret rotation:

   ```rust
   let config = ShopifyConfig::builder()
       .api_key(ApiKey::new("key").unwrap())
       .api_secret_key(ApiSecretKey::new("new-secret").unwrap())
       .old_api_secret_key(ApiSecretKey::new("old-secret").unwrap())
       .build()
       .unwrap();
   ```

5. **Log webhook metadata** - Include shop, topic, and ID for debugging

## Error Handling

```rust
use shopify_sdk::webhooks::WebhookError;

fn handle_webhook_error(error: WebhookError) {
    match error {
        WebhookError::InvalidHmac => {
            // Authentication failed - reject request
        }
        WebhookError::RegistrationNotFound { topic } => {
            // Topic not configured locally
        }
        WebhookError::GraphqlError(e) => {
            // API error during registration
        }
        WebhookError::ShopifyError { message } => {
            // Shopify returned an error
        }
        WebhookError::NoHandlerForTopic { topic } => {
            // No handler configured for topic
        }
        WebhookError::PayloadParseError { message } => {
            // Failed to parse webhook body as JSON
        }
        _ => {
            // Other errors
        }
    }
}
```

## Next Steps

- [GraphQL Admin API](graphql.md) - Make API calls in response to webhooks
- [Custom Apps](custom_apps.md) - Webhook handling for custom apps

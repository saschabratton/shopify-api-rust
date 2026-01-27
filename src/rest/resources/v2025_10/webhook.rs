//! Webhook resource implementation.
//!
//! This module provides the Webhook resource, which represents a webhook subscription
//! in a Shopify store. Webhooks allow apps to receive notifications when specific
//! events occur in the store.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{Webhook, WebhookListParams};
//! use shopify_sdk::rest::resources::v2025_10::common::{WebhookTopic, WebhookFormat};
//!
//! // Find a single webhook
//! let webhook = Webhook::find(&client, 123, None).await?;
//! println!("Webhook: {} -> {}", webhook.topic.map(|t| format!("{:?}", t)).unwrap_or_default(), webhook.address.as_deref().unwrap_or(""));
//!
//! // List webhooks with topic filter
//! let params = WebhookListParams {
//!     topic: Some("orders/create".to_string()),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let webhooks = Webhook::all(&client, Some(params)).await?;
//!
//! // Create a new webhook
//! let mut webhook = Webhook {
//!     topic: Some(WebhookTopic::OrdersCreate),
//!     address: Some("https://example.com/webhooks/orders".to_string()),
//!     format: Some(WebhookFormat::Json),
//!     ..Default::default()
//! };
//! let saved = webhook.save(&client).await?;
//!
//! // Count webhooks
//! let count = Webhook::count(&client, None).await?;
//! println!("Total webhooks: {}", count);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::{WebhookFormat, WebhookTopic};

/// A webhook subscription in a Shopify store.
///
/// Webhooks allow apps to receive HTTP POST notifications when specific events
/// occur in a Shopify store. When the subscribed event occurs, Shopify sends
/// a POST request to the webhook's address URL with details about the event.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the webhook
/// - `created_at` - When the webhook was created
/// - `updated_at` - When the webhook was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `address` - The URL where webhook payloads will be sent
/// - `topic` - The event that triggers the webhook (e.g., orders/create)
/// - `format` - The format of the webhook payload (json or xml)
/// - `api_version` - The API version used to serialize the webhook payload
/// - `fields` - Specific fields to include in the webhook payload
/// - `metafield_namespaces` - Metafield namespaces to include in the payload
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Webhook {
    /// The unique identifier of the webhook.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The URL where webhook payloads will be sent.
    /// Must be a valid HTTPS URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// The event that triggers the webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<WebhookTopic>,

    /// The format of the webhook payload.
    /// Defaults to JSON if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<WebhookFormat>,

    /// The API version used to serialize the webhook payload.
    /// If not specified, uses the API version of the request that created the webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,

    /// Specific fields to include in the webhook payload.
    /// If specified, only these fields will be included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,

    /// Metafield namespaces to include in the webhook payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metafield_namespaces: Option<Vec<String>>,

    /// When the webhook was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the webhook was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this webhook.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for Webhook {
    type Id = u64;
    type FindParams = WebhookFindParams;
    type AllParams = WebhookListParams;
    type CountParams = WebhookCountParams;

    const NAME: &'static str = "Webhook";
    const PLURAL: &'static str = "webhooks";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "webhooks/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "webhooks"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "webhooks/count",
        ),
        ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "webhooks"),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "webhooks/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "webhooks/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single webhook.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct WebhookFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing webhooks.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct WebhookListParams {
    /// Filter webhooks by address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Filter webhooks by topic.
    /// Can be a `WebhookTopic` value serialized as string (e.g., "orders/create").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,

    /// Show webhooks created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show webhooks created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show webhooks updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show webhooks updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return webhooks after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting webhooks.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct WebhookCountParams {
    /// Filter webhooks by address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Filter webhooks by topic.
    /// Can be a `WebhookTopic` value serialized as string (e.g., "orders/create").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_webhook_struct_serialization() {
        let webhook = Webhook {
            id: Some(12345),
            address: Some("https://example.com/webhooks".to_string()),
            topic: Some(WebhookTopic::OrdersCreate),
            format: Some(WebhookFormat::Json),
            api_version: Some("2025-10".to_string()),
            fields: Some(vec!["id".to_string(), "email".to_string()]),
            metafield_namespaces: Some(vec!["custom".to_string()]),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-20T15:45:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            admin_graphql_api_id: Some("gid://shopify/WebhookSubscription/12345".to_string()),
        };

        let json = serde_json::to_string(&webhook).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["address"], "https://example.com/webhooks");
        assert_eq!(parsed["topic"], "orders/create");
        assert_eq!(parsed["format"], "json");
        assert_eq!(parsed["api_version"], "2025-10");
        assert_eq!(parsed["fields"], serde_json::json!(["id", "email"]));
        assert_eq!(
            parsed["metafield_namespaces"],
            serde_json::json!(["custom"])
        );

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_webhook_deserialization_from_api_response() {
        let json = r#"{
            "id": 4759306,
            "address": "https://example.com/webhooks/orders",
            "topic": "orders/create",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "format": "json",
            "fields": ["id", "email", "total_price"],
            "metafield_namespaces": ["custom", "global"],
            "api_version": "2025-10",
            "admin_graphql_api_id": "gid://shopify/WebhookSubscription/4759306"
        }"#;

        let webhook: Webhook = serde_json::from_str(json).unwrap();

        assert_eq!(webhook.id, Some(4759306));
        assert_eq!(
            webhook.address,
            Some("https://example.com/webhooks/orders".to_string())
        );
        assert_eq!(webhook.topic, Some(WebhookTopic::OrdersCreate));
        assert_eq!(webhook.format, Some(WebhookFormat::Json));
        assert_eq!(webhook.api_version, Some("2025-10".to_string()));
        assert_eq!(
            webhook.fields,
            Some(vec![
                "id".to_string(),
                "email".to_string(),
                "total_price".to_string()
            ])
        );
        assert_eq!(
            webhook.metafield_namespaces,
            Some(vec!["custom".to_string(), "global".to_string()])
        );
        assert!(webhook.created_at.is_some());
        assert!(webhook.updated_at.is_some());
        assert_eq!(
            webhook.admin_graphql_api_id,
            Some("gid://shopify/WebhookSubscription/4759306".to_string())
        );
    }

    #[test]
    fn test_webhook_topic_enum_in_struct() {
        // Test all webhook topics can be serialized in a Webhook struct
        let topics = vec![
            (WebhookTopic::OrdersCreate, "orders/create"),
            (WebhookTopic::OrdersUpdated, "orders/updated"),
            (WebhookTopic::OrdersPaid, "orders/paid"),
            (WebhookTopic::ProductsCreate, "products/create"),
            (WebhookTopic::ProductsUpdate, "products/update"),
            (WebhookTopic::CustomersCreate, "customers/create"),
            (WebhookTopic::AppUninstalled, "app/uninstalled"),
        ];

        for (topic, expected_str) in topics {
            let webhook = Webhook {
                topic: Some(topic),
                address: Some("https://example.com".to_string()),
                ..Default::default()
            };

            let json = serde_json::to_value(&webhook).unwrap();
            assert_eq!(json["topic"], expected_str);
        }
    }

    #[test]
    fn test_webhook_format_enum_handling() {
        // Test JSON format
        let webhook_json = Webhook {
            format: Some(WebhookFormat::Json),
            address: Some("https://example.com".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&webhook_json).unwrap();
        assert_eq!(json["format"], "json");

        // Test XML format
        let webhook_xml = Webhook {
            format: Some(WebhookFormat::Xml),
            address: Some("https://example.com".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&webhook_xml).unwrap();
        assert_eq!(json["format"], "xml");

        // Test default format
        assert_eq!(WebhookFormat::default(), WebhookFormat::Json);
    }

    #[test]
    fn test_webhook_list_params_with_topic_filter() {
        let params = WebhookListParams {
            topic: Some("orders/create".to_string()),
            address: Some("https://example.com".to_string()),
            limit: Some(50),
            since_id: Some(100),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["topic"], "orders/create");
        assert_eq!(json["address"], "https://example.com");
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);

        // Fields not set should be omitted
        assert!(json.get("created_at_min").is_none());
        assert!(json.get("page_info").is_none());
    }

    #[test]
    fn test_webhook_count_params() {
        let params = WebhookCountParams {
            topic: Some("orders/create".to_string()),
            address: Some("https://example.com".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["topic"], "orders/create");
        assert_eq!(json["address"], "https://example.com");

        // Test empty params
        let empty_params = WebhookCountParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_webhook_path_constants_are_correct() {
        // Test Find path
        let find_path = get_path(Webhook::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "webhooks/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(Webhook::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "webhooks");
        assert_eq!(all_path.unwrap().http_method, HttpMethod::Get);

        // Test Count path
        let count_path = get_path(Webhook::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "webhooks/count");
        assert_eq!(count_path.unwrap().http_method, HttpMethod::Get);

        // Test Create path
        let create_path = get_path(Webhook::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "webhooks");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(Webhook::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "webhooks/{id}");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test Delete path
        let delete_path = get_path(Webhook::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "webhooks/{id}");
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        // Verify constants
        assert_eq!(Webhook::NAME, "Webhook");
        assert_eq!(Webhook::PLURAL, "webhooks");
    }

    #[test]
    fn test_webhook_get_id_returns_correct_value() {
        // Webhook with ID
        let webhook_with_id = Webhook {
            id: Some(123456789),
            address: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(webhook_with_id.get_id(), Some(123456789));

        // Webhook without ID (new webhook)
        let webhook_without_id = Webhook {
            id: None,
            address: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(webhook_without_id.get_id(), None);
    }
}

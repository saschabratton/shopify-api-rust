//! Webhook-related types for webhook configuration.
//!
//! This module provides types for webhook topics and formats.

use serde::{Deserialize, Serialize};

/// Represents a webhook topic that triggers webhook notifications.
///
/// Shopify sends webhook notifications when events occur that match
/// the topic you've subscribed to.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
///
/// let topic = WebhookTopic::OrdersCreate;
/// let json = serde_json::to_string(&topic).unwrap();
/// assert_eq!(json, "\"orders/create\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebhookTopic {
    // Order topics
    /// Triggered when an order is created.
    #[serde(rename = "orders/create")]
    OrdersCreate,
    /// Triggered when an order is updated.
    #[serde(rename = "orders/updated")]
    OrdersUpdated,
    /// Triggered when an order is paid.
    #[serde(rename = "orders/paid")]
    OrdersPaid,
    /// Triggered when an order is cancelled.
    #[serde(rename = "orders/cancelled")]
    OrdersCancelled,
    /// Triggered when an order is fulfilled.
    #[serde(rename = "orders/fulfilled")]
    OrdersFulfilled,
    /// Triggered when an order is partially fulfilled.
    #[serde(rename = "orders/partially_fulfilled")]
    OrdersPartiallyFulfilled,
    /// Triggered when an order is deleted.
    #[serde(rename = "orders/delete")]
    OrdersDelete,

    // Product topics
    /// Triggered when a product is created.
    #[serde(rename = "products/create")]
    ProductsCreate,
    /// Triggered when a product is updated.
    #[serde(rename = "products/update")]
    ProductsUpdate,
    /// Triggered when a product is deleted.
    #[serde(rename = "products/delete")]
    ProductsDelete,

    // Customer topics
    /// Triggered when a customer is created.
    #[serde(rename = "customers/create")]
    CustomersCreate,
    /// Triggered when a customer is updated.
    #[serde(rename = "customers/update")]
    CustomersUpdate,
    /// Triggered when a customer is deleted.
    #[serde(rename = "customers/delete")]
    CustomersDelete,
    /// Triggered when a customer is enabled.
    #[serde(rename = "customers/enable")]
    CustomersEnable,
    /// Triggered when a customer is disabled.
    #[serde(rename = "customers/disable")]
    CustomersDisable,

    // Collection topics
    /// Triggered when a collection is created.
    #[serde(rename = "collections/create")]
    CollectionsCreate,
    /// Triggered when a collection is updated.
    #[serde(rename = "collections/update")]
    CollectionsUpdate,
    /// Triggered when a collection is deleted.
    #[serde(rename = "collections/delete")]
    CollectionsDelete,

    // Checkout topics
    /// Triggered when a checkout is created.
    #[serde(rename = "checkouts/create")]
    CheckoutsCreate,
    /// Triggered when a checkout is updated.
    #[serde(rename = "checkouts/update")]
    CheckoutsUpdate,
    /// Triggered when a checkout is deleted.
    #[serde(rename = "checkouts/delete")]
    CheckoutsDelete,

    // Cart topics
    /// Triggered when a cart is created.
    #[serde(rename = "carts/create")]
    CartsCreate,
    /// Triggered when a cart is updated.
    #[serde(rename = "carts/update")]
    CartsUpdate,

    // Fulfillment topics
    /// Triggered when a fulfillment is created.
    #[serde(rename = "fulfillments/create")]
    FulfillmentsCreate,
    /// Triggered when a fulfillment is updated.
    #[serde(rename = "fulfillments/update")]
    FulfillmentsUpdate,

    // Refund topics
    /// Triggered when a refund is created.
    #[serde(rename = "refunds/create")]
    RefundsCreate,

    // App topics
    /// Triggered when the app is uninstalled.
    #[serde(rename = "app/uninstalled")]
    AppUninstalled,

    // Shop topics
    /// Triggered when the shop is updated.
    #[serde(rename = "shop/update")]
    ShopUpdate,

    // Theme topics
    /// Triggered when a theme is created.
    #[serde(rename = "themes/create")]
    ThemesCreate,
    /// Triggered when a theme is updated.
    #[serde(rename = "themes/update")]
    ThemesUpdate,
    /// Triggered when a theme is published.
    #[serde(rename = "themes/publish")]
    ThemesPublish,
    /// Triggered when a theme is deleted.
    #[serde(rename = "themes/delete")]
    ThemesDelete,

    // Inventory topics
    /// Triggered when inventory levels are updated.
    #[serde(rename = "inventory_levels/update")]
    InventoryLevelsUpdate,
    /// Triggered when inventory levels are connected.
    #[serde(rename = "inventory_levels/connect")]
    InventoryLevelsConnect,
    /// Triggered when inventory levels are disconnected.
    #[serde(rename = "inventory_levels/disconnect")]
    InventoryLevelsDisconnect,

    // Inventory item topics
    /// Triggered when an inventory item is created.
    #[serde(rename = "inventory_items/create")]
    InventoryItemsCreate,
    /// Triggered when an inventory item is updated.
    #[serde(rename = "inventory_items/update")]
    InventoryItemsUpdate,
    /// Triggered when an inventory item is deleted.
    #[serde(rename = "inventory_items/delete")]
    InventoryItemsDelete,
}

/// The format for webhook payloads.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::WebhookFormat;
///
/// let format = WebhookFormat::Json;
/// let json = serde_json::to_string(&format).unwrap();
/// assert_eq!(json, "\"json\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WebhookFormat {
    /// JSON format (default).
    #[default]
    Json,
    /// XML format.
    Xml,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_topic_serialization() {
        let topic = WebhookTopic::OrdersCreate;
        let json = serde_json::to_string(&topic).unwrap();
        assert_eq!(json, "\"orders/create\"");

        let topic = WebhookTopic::ProductsUpdate;
        let json = serde_json::to_string(&topic).unwrap();
        assert_eq!(json, "\"products/update\"");

        let topic = WebhookTopic::AppUninstalled;
        let json = serde_json::to_string(&topic).unwrap();
        assert_eq!(json, "\"app/uninstalled\"");
    }

    #[test]
    fn test_webhook_topic_deserialization() {
        let topic: WebhookTopic = serde_json::from_str("\"orders/create\"").unwrap();
        assert_eq!(topic, WebhookTopic::OrdersCreate);

        let topic: WebhookTopic = serde_json::from_str("\"customers/delete\"").unwrap();
        assert_eq!(topic, WebhookTopic::CustomersDelete);

        let topic: WebhookTopic = serde_json::from_str("\"app/uninstalled\"").unwrap();
        assert_eq!(topic, WebhookTopic::AppUninstalled);
    }

    #[test]
    fn test_webhook_format_serialization() {
        let format = WebhookFormat::Json;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, "\"json\"");

        let format = WebhookFormat::Xml;
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, "\"xml\"");
    }

    #[test]
    fn test_webhook_format_deserialization() {
        let format: WebhookFormat = serde_json::from_str("\"json\"").unwrap();
        assert_eq!(format, WebhookFormat::Json);

        let format: WebhookFormat = serde_json::from_str("\"xml\"").unwrap();
        assert_eq!(format, WebhookFormat::Xml);
    }

    #[test]
    fn test_webhook_format_default() {
        let format = WebhookFormat::default();
        assert_eq!(format, WebhookFormat::Json);
    }
}

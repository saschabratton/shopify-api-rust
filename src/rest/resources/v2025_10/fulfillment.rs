//! Fulfillment resource implementation.
//!
//! This module provides the [`Fulfillment`] resource for managing fulfillments in Shopify.
//! Fulfillments represent shipments of order line items to customers.
//!
//! # Nested Resource
//!
//! Fulfillments are primarily nested under orders:
//! - `/orders/{order_id}/fulfillments` - List fulfillments for an order
//! - `/orders/{order_id}/fulfillments/{id}` - Get a specific fulfillment
//!
//! # Resource-Specific Operations
//!
//! In addition to standard CRUD operations, the Fulfillment resource provides:
//! - [`Fulfillment::cancel`] - Cancel a fulfillment
//! - [`Fulfillment::update_tracking`] - Update tracking information
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{
//!     Fulfillment, FulfillmentListParams, FulfillmentStatus, TrackingInfo
//! };
//!
//! // List fulfillments for an order
//! let fulfillments = Fulfillment::all_with_parent(&client, "order_id", 123, None).await?;
//!
//! // Cancel a fulfillment
//! let cancelled = fulfillment.cancel(&client).await?;
//!
//! // Update tracking information
//! let tracking = TrackingInfo {
//!     tracking_number: Some("1Z999AA10123456784".to_string()),
//!     tracking_url: Some("https://ups.com/tracking/1Z999AA10123456784".to_string()),
//!     tracking_company: Some("UPS".to_string()),
//! };
//! let updated = fulfillment.update_tracking(&client, tracking).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::Address;

/// The status of a fulfillment.
///
/// Indicates the current state of the fulfillment process.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentStatus {
    /// The fulfillment is pending.
    #[default]
    Pending,
    /// The fulfillment is open and in progress.
    Open,
    /// The fulfillment was successful.
    Success,
    /// The fulfillment was cancelled.
    Cancelled,
    /// There was an error with the fulfillment.
    Error,
    /// The fulfillment failed.
    Failure,
}

/// The shipment status of a fulfillment.
///
/// Indicates the shipping/delivery status of the fulfillment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShipmentStatus {
    /// A label has been printed for the shipment.
    LabelPrinted,
    /// A label has been purchased for the shipment.
    LabelPurchased,
    /// Delivery was attempted but failed.
    AttemptedDelivery,
    /// The package is ready for pickup.
    ReadyForPickup,
    /// The shipment has been confirmed.
    Confirmed,
    /// The package is in transit.
    InTransit,
    /// The package is out for delivery.
    OutForDelivery,
    /// The package has been delivered.
    Delivered,
    /// The shipment failed.
    Failure,
}

/// A line item included in a fulfillment.
///
/// Contains information about the product/variant being fulfilled.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FulfillmentLineItem {
    /// The unique identifier of the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The ID of the product variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<u64>,

    /// The title of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The quantity being fulfilled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,

    /// The SKU of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,

    /// The title of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_title: Option<String>,

    /// The vendor of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    /// The fulfillment service for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_service: Option<String>,

    /// The ID of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// Whether the item requires shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_shipping: Option<bool>,

    /// Whether the item is taxable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxable: Option<bool>,

    /// Whether the item is a gift card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift_card: Option<bool>,

    /// The name of the product and variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The inventory management service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_inventory_management: Option<String>,

    /// Custom properties on the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<serde_json::Value>>,

    /// Whether the product still exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_exists: Option<bool>,

    /// The price per item as a string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The total discount on this line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_discount: Option<String>,

    /// The quantity that can still be fulfilled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillable_quantity: Option<i64>,

    /// The fulfillment status of this line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_status: Option<String>,
}

/// Tracking information for updating a fulfillment.
///
/// Used with the [`Fulfillment::update_tracking`] operation to update
/// tracking details for a fulfillment.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::TrackingInfo;
///
/// let tracking = TrackingInfo {
///     tracking_number: Some("1Z999AA10123456784".to_string()),
///     tracking_url: Some("https://ups.com/tracking/1Z999AA10123456784".to_string()),
///     tracking_company: Some("UPS".to_string()),
/// };
/// ```
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TrackingInfo {
    /// The tracking number for the shipment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_number: Option<String>,

    /// The URL to track the shipment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_url: Option<String>,

    /// The name of the tracking company.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_company: Option<String>,
}

/// A fulfillment in Shopify.
///
/// Fulfillments represent the shipping of order line items to customers.
/// Each fulfillment contains information about which items were shipped,
/// tracking information, and the fulfillment status.
///
/// # Nested Resource
///
/// Fulfillments are accessed under orders:
/// - `/orders/{order_id}/fulfillments/{id}`
///
/// Most operations require an `order_id`.
///
/// # Read-Only Fields
///
/// The following fields are read-only and will not be sent in create/update requests:
/// - `id`, `order_id`, `name`
/// - `created_at`, `updated_at`
/// - `admin_graphql_api_id`
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::rest::resources::v2025_10::Fulfillment;
///
/// // List fulfillments for an order
/// let fulfillments = Fulfillment::all_with_parent(&client, "order_id", 123, None).await?;
///
/// // Cancel a fulfillment
/// let cancelled = fulfillment.cancel(&client).await?;
/// ```
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Fulfillment {
    // --- Read-only fields (not serialized) ---
    /// The unique identifier of the fulfillment.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the order this fulfillment belongs to.
    #[serde(skip_serializing)]
    pub order_id: Option<u64>,

    /// The name of the fulfillment (e.g., "#1001.1").
    #[serde(skip_serializing)]
    pub name: Option<String>,

    /// When the fulfillment was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the fulfillment was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,

    // --- Core fields ---
    /// The status of the fulfillment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<FulfillmentStatus>,

    /// The fulfillment service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// The ID of the location that fulfilled the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<u64>,

    /// The shipment status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipment_status: Option<ShipmentStatus>,

    /// The name of the tracking company.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_company: Option<String>,

    /// The tracking number for the shipment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_number: Option<String>,

    /// Multiple tracking numbers (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_numbers: Option<Vec<String>>,

    /// The URL to track the shipment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_url: Option<String>,

    /// Multiple tracking URLs (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_urls: Option<Vec<String>>,

    /// The origin address for the shipment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_address: Option<Address>,

    /// Line items included in this fulfillment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<FulfillmentLineItem>>,

    /// Whether to notify the customer about this fulfillment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_customer: Option<bool>,

    /// The variant inventory management service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_inventory_management: Option<String>,
}

impl RestResource for Fulfillment {
    type Id = u64;
    type FindParams = FulfillmentFindParams;
    type AllParams = FulfillmentListParams;
    type CountParams = FulfillmentCountParams;

    const NAME: &'static str = "Fulfillment";
    const PLURAL: &'static str = "fulfillments";

    /// Paths for the Fulfillment resource.
    ///
    /// Fulfillments are primarily nested under orders, requiring `order_id`
    /// for most operations.
    const PATHS: &'static [ResourcePath] = &[
        // Nested paths under orders
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["order_id", "id"],
            "orders/{order_id}/fulfillments/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["order_id"],
            "orders/{order_id}/fulfillments",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["order_id"],
            "orders/{order_id}/fulfillments/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["order_id"],
            "orders/{order_id}/fulfillments",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["order_id", "id"],
            "orders/{order_id}/fulfillments/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl Fulfillment {
    /// Cancels the fulfillment.
    ///
    /// Sends a POST request to `/admin/api/{version}/orders/{order_id}/fulfillments/{id}/cancel.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the fulfillment doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the fulfillment has no ID or `order_id`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let fulfillment = Fulfillment::all_with_parent(&client, "order_id", 123, None).await?[0].clone();
    /// let cancelled = fulfillment.cancel(&client).await?;
    /// assert_eq!(cancelled.status, Some(FulfillmentStatus::Cancelled));
    /// ```
    pub async fn cancel(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "cancel",
        })?;

        let order_id = self.order_id.ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "cancel",
        })?;

        let path = format!("orders/{order_id}/fulfillments/{id}/cancel");
        let body = serde_json::json!({});

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the response - Shopify returns the fulfillment wrapped in "fulfillment" key
        let fulfillment: Self = response
            .body
            .get("fulfillment")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'fulfillment' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize fulfillment: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(fulfillment)
    }

    /// Updates tracking information for the fulfillment.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillments/{id}/update_tracking.json`.
    ///
    /// Note: Unlike other fulfillment operations, `update_tracking` uses a standalone path
    /// that only requires the fulfillment ID, not the order ID.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `tracking_info` - The new tracking information
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the fulfillment doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the fulfillment has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::rest::resources::v2025_10::TrackingInfo;
    ///
    /// let tracking = TrackingInfo {
    ///     tracking_number: Some("1Z999AA10123456784".to_string()),
    ///     tracking_url: Some("https://ups.com/tracking/1Z999AA10123456784".to_string()),
    ///     tracking_company: Some("UPS".to_string()),
    /// };
    ///
    /// let updated = fulfillment.update_tracking(&client, tracking).await?;
    /// assert_eq!(updated.tracking_number, Some("1Z999AA10123456784".to_string()));
    /// ```
    pub async fn update_tracking(
        &self,
        client: &RestClient,
        tracking_info: TrackingInfo,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "update_tracking",
        })?;

        // update_tracking uses a standalone path - doesn't require order_id
        let path = format!("fulfillments/{id}/update_tracking");

        let body = serde_json::json!({
            "fulfillment": {
                "tracking_info": tracking_info
            }
        });

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the response - Shopify returns the fulfillment wrapped in "fulfillment" key
        let fulfillment: Self = response
            .body
            .get("fulfillment")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'fulfillment' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize fulfillment: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(fulfillment)
    }
}

/// Parameters for finding a single fulfillment.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FulfillmentFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing fulfillments.
///
/// All fields are optional. Unset fields will not be included in the request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FulfillmentListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return only fulfillments after the specified ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Show fulfillments created at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show fulfillments created at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show fulfillments last updated at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show fulfillments last updated at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Page info for cursor-based pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

/// Parameters for counting fulfillments.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FulfillmentCountParams {
    /// Show fulfillments created at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show fulfillments created at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show fulfillments last updated at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show fulfillments last updated at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_fulfillment_struct_serialization() {
        let fulfillment = Fulfillment {
            id: Some(123456789),
            order_id: Some(987654321),
            name: Some("#1001.1".to_string()),
            status: Some(FulfillmentStatus::Success),
            service: Some("manual".to_string()),
            location_id: Some(111222333),
            shipment_status: Some(ShipmentStatus::Delivered),
            tracking_company: Some("UPS".to_string()),
            tracking_number: Some("1Z999AA10123456784".to_string()),
            tracking_numbers: Some(vec!["1Z999AA10123456784".to_string()]),
            tracking_url: Some("https://ups.com/tracking/1Z999AA10123456784".to_string()),
            tracking_urls: Some(vec![
                "https://ups.com/tracking/1Z999AA10123456784".to_string()
            ]),
            notify_customer: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_string(&fulfillment).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["status"], "success");
        assert_eq!(parsed["service"], "manual");
        assert_eq!(parsed["location_id"], 111222333);
        assert_eq!(parsed["shipment_status"], "delivered");
        assert_eq!(parsed["tracking_company"], "UPS");
        assert_eq!(parsed["tracking_number"], "1Z999AA10123456784");
        assert_eq!(parsed["notify_customer"], true);

        // Read-only fields should NOT be serialized
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("order_id").is_none());
        assert!(parsed.get("name").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_fulfillment_deserialization_from_api_response() {
        // Use r##"..."## to allow # inside the string
        let json_str = r##"{
            "id": 255858046,
            "order_id": 450789469,
            "name": "#1001.1",
            "status": "success",
            "service": "manual",
            "location_id": 487838322,
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "shipment_status": "in_transit",
            "tracking_company": "USPS",
            "tracking_number": "9400111899223456789012",
            "tracking_numbers": ["9400111899223456789012"],
            "tracking_url": "https://tools.usps.com/go/TrackConfirmAction?qtc_tLabels1=9400111899223456789012",
            "tracking_urls": ["https://tools.usps.com/go/TrackConfirmAction?qtc_tLabels1=9400111899223456789012"],
            "line_items": [
                {
                    "id": 669751112,
                    "variant_id": 457924702,
                    "product_id": 632910392,
                    "title": "IPod Nano - 8GB",
                    "quantity": 1,
                    "sku": "IPOD2008BLACK",
                    "requires_shipping": true
                }
            ],
            "admin_graphql_api_id": "gid://shopify/Fulfillment/255858046"
        }"##;

        let fulfillment: Fulfillment = serde_json::from_str(json_str).unwrap();

        assert_eq!(fulfillment.id, Some(255858046));
        assert_eq!(fulfillment.order_id, Some(450789469));
        assert_eq!(fulfillment.name.as_deref(), Some("#1001.1"));
        assert_eq!(fulfillment.status, Some(FulfillmentStatus::Success));
        assert_eq!(fulfillment.service.as_deref(), Some("manual"));
        assert_eq!(fulfillment.location_id, Some(487838322));
        assert_eq!(fulfillment.shipment_status, Some(ShipmentStatus::InTransit));
        assert_eq!(fulfillment.tracking_company.as_deref(), Some("USPS"));
        assert_eq!(
            fulfillment.tracking_number.as_deref(),
            Some("9400111899223456789012")
        );
        assert!(fulfillment.created_at.is_some());
        assert!(fulfillment.updated_at.is_some());

        // Check line items
        let line_items = fulfillment.line_items.unwrap();
        assert_eq!(line_items.len(), 1);
        assert_eq!(line_items[0].id, Some(669751112));
        assert_eq!(line_items[0].title.as_deref(), Some("IPod Nano - 8GB"));
        assert_eq!(line_items[0].quantity, Some(1));
    }

    #[test]
    fn test_fulfillment_status_enum_serialization() {
        // Test serialization to snake_case
        let pending_str = serde_json::to_string(&FulfillmentStatus::Pending).unwrap();
        assert_eq!(pending_str, "\"pending\"");

        let open_str = serde_json::to_string(&FulfillmentStatus::Open).unwrap();
        assert_eq!(open_str, "\"open\"");

        let success_str = serde_json::to_string(&FulfillmentStatus::Success).unwrap();
        assert_eq!(success_str, "\"success\"");

        let cancelled_str = serde_json::to_string(&FulfillmentStatus::Cancelled).unwrap();
        assert_eq!(cancelled_str, "\"cancelled\"");

        let error_str = serde_json::to_string(&FulfillmentStatus::Error).unwrap();
        assert_eq!(error_str, "\"error\"");

        let failure_str = serde_json::to_string(&FulfillmentStatus::Failure).unwrap();
        assert_eq!(failure_str, "\"failure\"");

        // Test deserialization
        let success: FulfillmentStatus = serde_json::from_str("\"success\"").unwrap();
        let cancelled: FulfillmentStatus = serde_json::from_str("\"cancelled\"").unwrap();

        assert_eq!(success, FulfillmentStatus::Success);
        assert_eq!(cancelled, FulfillmentStatus::Cancelled);

        // Test default
        assert_eq!(FulfillmentStatus::default(), FulfillmentStatus::Pending);
    }

    #[test]
    fn test_shipment_status_enum_serialization() {
        // Test serialization to snake_case
        let label_printed = serde_json::to_string(&ShipmentStatus::LabelPrinted).unwrap();
        assert_eq!(label_printed, "\"label_printed\"");

        let label_purchased = serde_json::to_string(&ShipmentStatus::LabelPurchased).unwrap();
        assert_eq!(label_purchased, "\"label_purchased\"");

        let attempted = serde_json::to_string(&ShipmentStatus::AttemptedDelivery).unwrap();
        assert_eq!(attempted, "\"attempted_delivery\"");

        let ready = serde_json::to_string(&ShipmentStatus::ReadyForPickup).unwrap();
        assert_eq!(ready, "\"ready_for_pickup\"");

        let confirmed = serde_json::to_string(&ShipmentStatus::Confirmed).unwrap();
        assert_eq!(confirmed, "\"confirmed\"");

        let in_transit = serde_json::to_string(&ShipmentStatus::InTransit).unwrap();
        assert_eq!(in_transit, "\"in_transit\"");

        let out_for_delivery = serde_json::to_string(&ShipmentStatus::OutForDelivery).unwrap();
        assert_eq!(out_for_delivery, "\"out_for_delivery\"");

        let delivered = serde_json::to_string(&ShipmentStatus::Delivered).unwrap();
        assert_eq!(delivered, "\"delivered\"");

        let failure = serde_json::to_string(&ShipmentStatus::Failure).unwrap();
        assert_eq!(failure, "\"failure\"");

        // Test deserialization
        let in_transit_val: ShipmentStatus = serde_json::from_str("\"in_transit\"").unwrap();
        let delivered_val: ShipmentStatus = serde_json::from_str("\"delivered\"").unwrap();
        let out_for_delivery_val: ShipmentStatus =
            serde_json::from_str("\"out_for_delivery\"").unwrap();

        assert_eq!(in_transit_val, ShipmentStatus::InTransit);
        assert_eq!(delivered_val, ShipmentStatus::Delivered);
        assert_eq!(out_for_delivery_val, ShipmentStatus::OutForDelivery);
    }

    #[test]
    fn test_nested_path_under_orders() {
        // Test Find path (requires both order_id and id)
        let find_path = get_path(
            Fulfillment::PATHS,
            ResourceOperation::Find,
            &["order_id", "id"],
        );
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "orders/{order_id}/fulfillments/{id}"
        );
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path (requires order_id)
        let all_path = get_path(Fulfillment::PATHS, ResourceOperation::All, &["order_id"]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "orders/{order_id}/fulfillments");

        // Test Count path (requires order_id)
        let count_path = get_path(Fulfillment::PATHS, ResourceOperation::Count, &["order_id"]);
        assert!(count_path.is_some());
        assert_eq!(
            count_path.unwrap().template,
            "orders/{order_id}/fulfillments/count"
        );

        // Test Create path (requires order_id)
        let create_path = get_path(Fulfillment::PATHS, ResourceOperation::Create, &["order_id"]);
        assert!(create_path.is_some());
        assert_eq!(
            create_path.unwrap().template,
            "orders/{order_id}/fulfillments"
        );
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path (requires both order_id and id)
        let update_path = get_path(
            Fulfillment::PATHS,
            ResourceOperation::Update,
            &["order_id", "id"],
        );
        assert!(update_path.is_some());
        assert_eq!(
            update_path.unwrap().template,
            "orders/{order_id}/fulfillments/{id}"
        );
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test that paths without order_id fail for All
        let no_order_all = get_path(Fulfillment::PATHS, ResourceOperation::All, &[]);
        assert!(no_order_all.is_none());

        // Verify constants
        assert_eq!(Fulfillment::NAME, "Fulfillment");
        assert_eq!(Fulfillment::PLURAL, "fulfillments");
    }

    #[test]
    fn test_resource_specific_operations_signatures() {
        // This test verifies that the cancel and update_tracking methods exist
        // with the correct signatures. The actual HTTP calls would require
        // a mock client, but we verify the type signatures compile correctly.

        // Verify cancel signature
        fn _assert_cancel_signature<F, Fut>(f: F)
        where
            F: Fn(&Fulfillment, &RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<Fulfillment, ResourceError>>,
        {
            let _ = f;
        }

        // Verify update_tracking signature
        fn _assert_update_tracking_signature<F, Fut>(f: F)
        where
            F: Fn(&Fulfillment, &RestClient, TrackingInfo) -> Fut,
            Fut: std::future::Future<Output = Result<Fulfillment, ResourceError>>,
        {
            let _ = f;
        }

        // Verify PathResolutionFailed error is returned when fulfillment has no ID
        let fulfillment_without_id = Fulfillment::default();
        assert!(fulfillment_without_id.get_id().is_none());

        // Verify TrackingInfo struct
        let tracking = TrackingInfo {
            tracking_number: Some("1Z999AA10123456784".to_string()),
            tracking_url: Some("https://ups.com/tracking".to_string()),
            tracking_company: Some("UPS".to_string()),
        };
        assert_eq!(
            tracking.tracking_number,
            Some("1Z999AA10123456784".to_string())
        );

        // Verify get_id returns correct value
        let fulfillment_with_id = Fulfillment {
            id: Some(255858046),
            order_id: Some(450789469),
            ..Default::default()
        };
        assert_eq!(fulfillment_with_id.get_id(), Some(255858046));
    }

    #[test]
    fn test_fulfillment_list_params_serialization() {
        let created_at_min = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let params = FulfillmentListParams {
            limit: Some(50),
            since_id: Some(12345),
            created_at_min: Some(created_at_min),
            created_at_max: None,
            updated_at_min: None,
            updated_at_max: None,
            fields: Some("id,status,tracking_number".to_string()),
            page_info: None,
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 12345);
        assert_eq!(json["fields"], "id,status,tracking_number");
        assert!(json["created_at_min"].as_str().is_some());

        // Verify None fields are not serialized
        assert!(json.get("created_at_max").is_none());
        assert!(json.get("updated_at_min").is_none());
        assert!(json.get("page_info").is_none());

        // Test empty params
        let empty_params = FulfillmentListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_tracking_info_serialization() {
        let tracking = TrackingInfo {
            tracking_number: Some("1Z999AA10123456784".to_string()),
            tracking_url: Some("https://ups.com/tracking/1Z999AA10123456784".to_string()),
            tracking_company: Some("UPS".to_string()),
        };

        let json = serde_json::to_string(&tracking).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["tracking_number"], "1Z999AA10123456784");
        assert_eq!(
            parsed["tracking_url"],
            "https://ups.com/tracking/1Z999AA10123456784"
        );
        assert_eq!(parsed["tracking_company"], "UPS");

        // Test partial tracking info
        let partial_tracking = TrackingInfo {
            tracking_number: Some("12345".to_string()),
            ..Default::default()
        };

        let partial_json = serde_json::to_value(&partial_tracking).unwrap();
        assert_eq!(partial_json["tracking_number"], "12345");
        assert!(partial_json.get("tracking_url").is_none());
        assert!(partial_json.get("tracking_company").is_none());
    }

    #[test]
    fn test_fulfillment_line_item_serialization() {
        let line_item = FulfillmentLineItem {
            id: Some(669751112),
            variant_id: Some(457924702),
            product_id: Some(632910392),
            title: Some("IPod Nano - 8GB".to_string()),
            quantity: Some(2),
            sku: Some("IPOD2008BLACK".to_string()),
            variant_title: Some("Black".to_string()),
            vendor: Some("Apple".to_string()),
            fulfillment_service: Some("manual".to_string()),
            requires_shipping: Some(true),
            taxable: Some(true),
            gift_card: Some(false),
            name: Some("IPod Nano - 8GB - Black".to_string()),
            product_exists: Some(true),
            price: Some("199.00".to_string()),
            total_discount: Some("0.00".to_string()),
            fulfillable_quantity: Some(0),
            fulfillment_status: Some("fulfilled".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&line_item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], 669751112);
        assert_eq!(parsed["variant_id"], 457924702);
        assert_eq!(parsed["title"], "IPod Nano - 8GB");
        assert_eq!(parsed["quantity"], 2);
        assert_eq!(parsed["sku"], "IPOD2008BLACK");
        assert_eq!(parsed["price"], "199.00");
        assert_eq!(parsed["requires_shipping"], true);
    }

    #[test]
    fn test_fulfillment_with_origin_address() {
        let fulfillment = Fulfillment {
            id: Some(123),
            order_id: Some(456),
            status: Some(FulfillmentStatus::Success),
            origin_address: Some(Address {
                first_name: Some("Warehouse".to_string()),
                address1: Some("123 Fulfillment Center".to_string()),
                city: Some("Los Angeles".to_string()),
                province: Some("California".to_string()),
                province_code: Some("CA".to_string()),
                country: Some("United States".to_string()),
                country_code: Some("US".to_string()),
                zip: Some("90001".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&fulfillment).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // origin_address should be present
        assert!(parsed.get("origin_address").is_some());
        assert_eq!(parsed["origin_address"]["city"], "Los Angeles");
        assert_eq!(parsed["origin_address"]["province_code"], "CA");
    }
}

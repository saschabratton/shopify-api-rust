//! FulfillmentOrder resource implementation.
//!
//! This module provides the [`FulfillmentOrder`] resource along with
//! [`FulfillmentRequest`] and [`CancellationRequest`] for managing
//! modern fulfillment workflows in Shopify.
//!
//! # Resource Overview
//!
//! FulfillmentOrders are automatically created by Shopify when an order is placed.
//! They represent the work required to fulfill a set of line items from a specific
//! location.
//!
//! # Read-Only Resource
//!
//! FulfillmentOrder is primarily a read-only resource - you cannot create, update,
//! or delete fulfillment orders directly. Instead, use the special operations to
//! manage their state.
//!
//! # Special Operations
//!
//! - [`FulfillmentOrder::cancel`] - Cancel the fulfillment order
//! - [`FulfillmentOrder::close`] - Close the fulfillment order
//! - [`FulfillmentOrder::hold`] - Place the fulfillment order on hold
//! - [`FulfillmentOrder::move_location`] - Move to a different location
//! - [`FulfillmentOrder::open`] - Re-open a closed fulfillment order
//! - [`FulfillmentOrder::release_hold`] - Release a hold
//! - [`FulfillmentOrder::reschedule`] - Reschedule the fulfillment
//!
//! # Related Structs
//!
//! - [`FulfillmentRequest`] - Request fulfillment from a fulfillment service
//! - [`CancellationRequest`] - Request cancellation of a fulfillment request
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{
//!     FulfillmentOrder, FulfillmentOrderListParams, FulfillmentOrderHoldParams,
//!     HoldReason, FulfillmentRequest, CancellationRequest
//! };
//!
//! // List fulfillment orders for an order
//! let fulfillment_orders = FulfillmentOrder::all_with_parent(&client, "order_id", 123, None).await?;
//!
//! // Place a fulfillment order on hold
//! let hold_params = FulfillmentOrderHoldParams {
//!     reason: HoldReason::AwaitingPayment,
//!     reason_notes: Some("Waiting for wire transfer".to_string()),
//!     ..Default::default()
//! };
//! let held = fulfillment_order.hold(&client, hold_params).await?;
//!
//! // Submit a fulfillment request
//! let fo = FulfillmentRequest::create(&client, 123, None, None).await?;
//!
//! // Request cancellation
//! let cancelled = CancellationRequest::create(&client, 123, Some("Customer changed mind")).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// The status of a fulfillment order.
///
/// Indicates the current state of the fulfillment order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentOrderStatus {
    /// The fulfillment order is open and ready to be fulfilled.
    #[default]
    Open,
    /// The fulfillment order is in progress.
    InProgress,
    /// The fulfillment order was cancelled.
    Cancelled,
    /// The fulfillment order is incomplete.
    Incomplete,
    /// The fulfillment order is closed.
    Closed,
    /// The fulfillment order is scheduled for a future date.
    Scheduled,
    /// The fulfillment order is on hold.
    OnHold,
}

/// The request status of a fulfillment order.
///
/// Indicates the status of fulfillment requests for this order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentOrderRequestStatus {
    /// No fulfillment request has been submitted.
    #[default]
    Unsubmitted,
    /// A fulfillment request has been submitted.
    Submitted,
    /// The fulfillment request was accepted.
    Accepted,
    /// The fulfillment request was rejected.
    Rejected,
    /// A cancellation has been requested.
    CancellationRequested,
    /// The cancellation was accepted.
    CancellationAccepted,
    /// The cancellation was rejected.
    CancellationRejected,
    /// The fulfillment request is closed.
    Closed,
}

/// The reason for placing a fulfillment order on hold.
///
/// Used with the [`FulfillmentOrder::hold`] operation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HoldReason {
    /// Awaiting payment from the customer.
    AwaitingPayment,
    /// The order has a high risk of fraud.
    HighRiskOfFraud,
    /// The shipping address is incorrect.
    IncorrectAddress,
    /// The inventory is out of stock.
    InventoryOutOfStock,
    /// Other reason (specify in reason_notes).
    Other,
}

/// A line item in a fulfillment order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FulfillmentOrderLineItem {
    /// The unique identifier of the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The ID of the shop.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shop_id: Option<u64>,

    /// The ID of the fulfillment order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_order_id: Option<u64>,

    /// The ID of the order line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_item_id: Option<u64>,

    /// The ID of the inventory item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_item_id: Option<u64>,

    /// The quantity to fulfill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,

    /// The fulfillable quantity remaining.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillable_quantity: Option<i64>,

    /// The ID of the product variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<u64>,
}

/// The destination for a fulfillment order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentOrderDestination {
    /// The unique identifier of the destination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The first name of the recipient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// The last name of the recipient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// The company name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,

    /// The first line of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,

    /// The second line of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,

    /// The city.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// The province or state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province: Option<String>,

    /// The country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// The ZIP or postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,

    /// The phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// The email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// A hold on a fulfillment order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentHold {
    /// The reason for the hold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Additional notes about the hold reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_notes: Option<String>,
}

/// The delivery method for a fulfillment order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DeliveryMethod {
    /// The unique identifier of the delivery method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The type of delivery method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_type: Option<String>,

    /// The minimum delivery date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_delivery_date_time: Option<DateTime<Utc>>,

    /// The maximum delivery date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_delivery_date_time: Option<DateTime<Utc>>,
}

/// A fulfillment order in Shopify.
///
/// Fulfillment orders are automatically created when an order is placed.
/// They represent the work required to fulfill line items from a specific location.
///
/// # Read-Only Resource
///
/// This is primarily a read-only resource. You cannot create, update, or delete
/// fulfillment orders directly. Use the special operations to manage their state.
///
/// # Nested Resource
///
/// Fulfillment orders are accessed under orders for listing:
/// - List: `/orders/{order_id}/fulfillment_orders`
/// - Find: `/fulfillment_orders/{id}` (standalone)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FulfillmentOrder {
    /// The unique identifier of the fulfillment order.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the order this fulfillment order belongs to.
    #[serde(skip_serializing)]
    pub order_id: Option<u64>,

    /// The ID of the shop.
    #[serde(skip_serializing)]
    pub shop_id: Option<u64>,

    /// The ID of the assigned location.
    #[serde(skip_serializing)]
    pub assigned_location_id: Option<u64>,

    /// The status of the fulfillment order.
    #[serde(skip_serializing)]
    pub status: Option<FulfillmentOrderStatus>,

    /// The request status.
    #[serde(skip_serializing)]
    pub request_status: Option<FulfillmentOrderRequestStatus>,

    /// Supported actions for this fulfillment order.
    #[serde(skip_serializing)]
    pub supported_actions: Option<Vec<String>>,

    /// The destination for this fulfillment order.
    #[serde(skip_serializing)]
    pub destination: Option<FulfillmentOrderDestination>,

    /// Line items in this fulfillment order.
    #[serde(skip_serializing)]
    pub line_items: Option<Vec<FulfillmentOrderLineItem>>,

    /// The scheduled fulfillment date.
    #[serde(skip_serializing)]
    pub fulfill_at: Option<DateTime<Utc>>,

    /// The deadline for fulfillment.
    #[serde(skip_serializing)]
    pub fulfill_by: Option<DateTime<Utc>>,

    /// International duties information.
    #[serde(skip_serializing)]
    pub international_duties: Option<serde_json::Value>,

    /// Holds placed on this fulfillment order.
    #[serde(skip_serializing)]
    pub fulfillment_holds: Option<Vec<FulfillmentHold>>,

    /// The delivery method for this fulfillment order.
    #[serde(skip_serializing)]
    pub delivery_method: Option<DeliveryMethod>,

    /// Assigned location information.
    #[serde(skip_serializing)]
    pub assigned_location: Option<serde_json::Value>,

    /// Merchant requests for this fulfillment order.
    #[serde(skip_serializing)]
    pub merchant_requests: Option<Vec<serde_json::Value>>,

    /// When the fulfillment order was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the fulfillment order was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for FulfillmentOrder {
    type Id = u64;
    type FindParams = FulfillmentOrderFindParams;
    type AllParams = FulfillmentOrderListParams;
    type CountParams = FulfillmentOrderCountParams;

    const NAME: &'static str = "FulfillmentOrder";
    const PLURAL: &'static str = "fulfillment_orders";

    /// Paths for the FulfillmentOrder resource.
    ///
    /// - Find: Standalone path `/fulfillment_orders/{id}`
    /// - All: Nested under orders `/orders/{order_id}/fulfillment_orders`
    /// - No Create, Update, Delete, or Count operations
    const PATHS: &'static [ResourcePath] = &[
        // Standalone find by ID
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "fulfillment_orders/{id}",
        ),
        // Nested all under orders
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["order_id"],
            "orders/{order_id}/fulfillment_orders",
        ),
        // No Create, Update, Delete, or Count paths
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl FulfillmentOrder {
    /// Cancels the fulfillment order.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/cancel.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Returns
    ///
    /// The updated fulfillment order with cancelled status.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if the fulfillment order has no ID.
    pub async fn cancel(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "cancel",
        })?;

        let path = format!("fulfillment_orders/{id}/cancel");
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

        Self::parse_response(&response)
    }

    /// Closes the fulfillment order.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/close.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `message` - Optional message explaining why the fulfillment order was closed
    ///
    /// # Returns
    ///
    /// The updated fulfillment order with closed status.
    pub async fn close(
        &self,
        client: &RestClient,
        message: Option<&str>,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "close",
        })?;

        let path = format!("fulfillment_orders/{id}/close");

        let body = if let Some(msg) = message {
            serde_json::json!({
                "fulfillment_order": {
                    "message": msg
                }
            })
        } else {
            serde_json::json!({})
        };

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

        Self::parse_response(&response)
    }

    /// Places the fulfillment order on hold.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/hold.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `params` - Hold parameters including reason and optional notes
    ///
    /// # Returns
    ///
    /// The updated fulfillment order with on_hold status.
    pub async fn hold(
        &self,
        client: &RestClient,
        params: FulfillmentOrderHoldParams,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "hold",
        })?;

        let path = format!("fulfillment_orders/{id}/hold");

        let body = serde_json::json!({
            "fulfillment_hold": params
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

        Self::parse_response(&response)
    }

    /// Moves the fulfillment order to a different location.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/move.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `params` - Move parameters including the new location ID
    ///
    /// # Returns
    ///
    /// The updated fulfillment order at the new location.
    pub async fn move_location(
        &self,
        client: &RestClient,
        params: FulfillmentOrderMoveParams,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "move",
        })?;

        let path = format!("fulfillment_orders/{id}/move");

        let body = serde_json::json!({
            "fulfillment_order": params
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

        // The move response may contain the fulfillment order in different locations
        // Check for original_fulfillment_order or moved_fulfillment_order
        let response_body = &response.body;

        if let Some(fo) = response_body.get("moved_fulfillment_order") {
            return serde_json::from_value(fo.clone()).map_err(|e| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: format!("Failed to deserialize fulfillment_order: {e}"),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            });
        }

        if let Some(fo) = response_body.get("original_fulfillment_order") {
            return serde_json::from_value(fo.clone()).map_err(|e| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: format!("Failed to deserialize fulfillment_order: {e}"),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            });
        }

        Self::parse_response(&response)
    }

    /// Re-opens a closed fulfillment order.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/open.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Returns
    ///
    /// The updated fulfillment order with open status.
    pub async fn open(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "open",
        })?;

        let path = format!("fulfillment_orders/{id}/open");
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

        Self::parse_response(&response)
    }

    /// Releases the hold on the fulfillment order.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/release_hold.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Returns
    ///
    /// The updated fulfillment order with the hold released.
    pub async fn release_hold(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "release_hold",
        })?;

        let path = format!("fulfillment_orders/{id}/release_hold");
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

        Self::parse_response(&response)
    }

    /// Reschedules the fulfillment order.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/reschedule.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `params` - Reschedule parameters including the new fulfillment date
    ///
    /// # Returns
    ///
    /// The updated fulfillment order with the new schedule.
    pub async fn reschedule(
        &self,
        client: &RestClient,
        params: FulfillmentOrderRescheduleParams,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "reschedule",
        })?;

        let path = format!("fulfillment_orders/{id}/reschedule");

        let body = serde_json::json!(params);

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

        Self::parse_response(&response)
    }

    /// Helper function to parse fulfillment order from response.
    fn parse_response(response: &crate::clients::HttpResponse) -> Result<Self, ResourceError> {
        let fulfillment_order: Self = response
            .body
            .get("fulfillment_order")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'fulfillment_order' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize fulfillment_order: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(fulfillment_order)
    }
}

/// Parameters for placing a fulfillment order on hold.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentOrderHoldParams {
    /// The reason for the hold.
    pub reason: HoldReason,

    /// Additional notes about the hold reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_notes: Option<String>,

    /// Whether to notify the merchant about the hold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_merchant: Option<bool>,

    /// Specific line items to hold (for partial holds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_order_line_items: Option<Vec<FulfillmentOrderLineItemInput>>,
}

impl Default for HoldReason {
    fn default() -> Self {
        HoldReason::Other
    }
}

/// Parameters for moving a fulfillment order to a different location.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FulfillmentOrderMoveParams {
    /// The ID of the new location.
    pub new_location_id: u64,

    /// Specific line items to move (for partial moves).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_order_line_items: Option<Vec<FulfillmentOrderLineItemInput>>,
}

/// Parameters for rescheduling a fulfillment order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentOrderRescheduleParams {
    /// The new scheduled fulfillment date.
    pub new_fulfill_at: DateTime<Utc>,
}

/// Input for specifying fulfillment order line items in operations.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentOrderLineItemInput {
    /// The ID of the fulfillment order line item.
    pub id: u64,

    /// The quantity to include.
    pub quantity: i64,
}

/// Parameters for finding a single fulfillment order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentOrderFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing fulfillment orders.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentOrderListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Page info for cursor-based pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

/// Parameters for counting fulfillment orders (not supported).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentOrderCountParams {
    // Count is not supported for FulfillmentOrder
}

/// A fulfillment request for a fulfillment order.
///
/// Used to request fulfillment from a fulfillment service.
/// All operations are associated functions taking a `fulfillment_order_id`.
#[derive(Debug, Clone, Default)]
pub struct FulfillmentRequest;

impl FulfillmentRequest {
    /// Creates a fulfillment request for a fulfillment order.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/fulfillment_request.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `fulfillment_order_id` - The ID of the fulfillment order
    /// * `message` - Optional message to the fulfillment service
    /// * `line_items` - Optional specific line items for partial fulfillment
    ///
    /// # Returns
    ///
    /// The parent fulfillment order with updated `request_status`.
    pub async fn create(
        client: &RestClient,
        fulfillment_order_id: u64,
        message: Option<&str>,
        line_items: Option<Vec<FulfillmentOrderLineItemInput>>,
    ) -> Result<FulfillmentOrder, ResourceError> {
        let path = format!("fulfillment_orders/{fulfillment_order_id}/fulfillment_request");

        let mut request_body = serde_json::Map::new();
        if let Some(msg) = message {
            request_body.insert("message".to_string(), serde_json::json!(msg));
        }
        if let Some(items) = line_items {
            request_body.insert(
                "fulfillment_order_line_items".to_string(),
                serde_json::json!(items),
            );
        }

        let body = serde_json::json!({
            "fulfillment_request": request_body
        });

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "FulfillmentRequest",
                Some(&fulfillment_order_id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the original_fulfillment_order from the response
        let fulfillment_order: FulfillmentOrder = response
            .body
            .get("original_fulfillment_order")
            .or_else(|| response.body.get("fulfillment_order"))
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing fulfillment_order in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize fulfillment_order: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(fulfillment_order)
    }

    /// Accepts a fulfillment request.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/fulfillment_request/accept.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `fulfillment_order_id` - The ID of the fulfillment order
    /// * `message` - Optional message explaining acceptance
    ///
    /// # Returns
    ///
    /// The parent fulfillment order with updated `request_status`.
    pub async fn accept(
        client: &RestClient,
        fulfillment_order_id: u64,
        message: Option<&str>,
    ) -> Result<FulfillmentOrder, ResourceError> {
        let path = format!("fulfillment_orders/{fulfillment_order_id}/fulfillment_request/accept");

        let body = if let Some(msg) = message {
            serde_json::json!({
                "fulfillment_request": {
                    "message": msg
                }
            })
        } else {
            serde_json::json!({})
        };

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "FulfillmentRequest",
                Some(&fulfillment_order_id.to_string()),
                response.request_id(),
            ));
        }

        FulfillmentOrder::parse_response(&response)
    }

    /// Rejects a fulfillment request.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/fulfillment_request/reject.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `fulfillment_order_id` - The ID of the fulfillment order
    /// * `message` - Optional message explaining rejection
    /// * `reason` - Optional rejection reason
    ///
    /// # Returns
    ///
    /// The parent fulfillment order with updated `request_status`.
    pub async fn reject(
        client: &RestClient,
        fulfillment_order_id: u64,
        message: Option<&str>,
        reason: Option<&str>,
    ) -> Result<FulfillmentOrder, ResourceError> {
        let path = format!("fulfillment_orders/{fulfillment_order_id}/fulfillment_request/reject");

        let mut request_body = serde_json::Map::new();
        if let Some(msg) = message {
            request_body.insert("message".to_string(), serde_json::json!(msg));
        }
        if let Some(r) = reason {
            request_body.insert("reason".to_string(), serde_json::json!(r));
        }

        let body = if request_body.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::json!({
                "fulfillment_request": request_body
            })
        };

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "FulfillmentRequest",
                Some(&fulfillment_order_id.to_string()),
                response.request_id(),
            ));
        }

        FulfillmentOrder::parse_response(&response)
    }
}

/// A cancellation request for a fulfillment order.
///
/// Used to request cancellation of a fulfillment request.
/// All operations are associated functions taking a `fulfillment_order_id`.
#[derive(Debug, Clone, Default)]
pub struct CancellationRequest;

impl CancellationRequest {
    /// Creates a cancellation request for a fulfillment order.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/cancellation_request.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `fulfillment_order_id` - The ID of the fulfillment order
    /// * `message` - Optional message explaining the cancellation request
    ///
    /// # Returns
    ///
    /// The parent fulfillment order with updated `request_status`.
    pub async fn create(
        client: &RestClient,
        fulfillment_order_id: u64,
        message: Option<&str>,
    ) -> Result<FulfillmentOrder, ResourceError> {
        let path = format!("fulfillment_orders/{fulfillment_order_id}/cancellation_request");

        let body = if let Some(msg) = message {
            serde_json::json!({
                "cancellation_request": {
                    "message": msg
                }
            })
        } else {
            serde_json::json!({})
        };

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "CancellationRequest",
                Some(&fulfillment_order_id.to_string()),
                response.request_id(),
            ));
        }

        FulfillmentOrder::parse_response(&response)
    }

    /// Accepts a cancellation request.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/cancellation_request/accept.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `fulfillment_order_id` - The ID of the fulfillment order
    /// * `message` - Optional message explaining acceptance
    ///
    /// # Returns
    ///
    /// The parent fulfillment order with updated `request_status`.
    pub async fn accept(
        client: &RestClient,
        fulfillment_order_id: u64,
        message: Option<&str>,
    ) -> Result<FulfillmentOrder, ResourceError> {
        let path =
            format!("fulfillment_orders/{fulfillment_order_id}/cancellation_request/accept");

        let body = if let Some(msg) = message {
            serde_json::json!({
                "cancellation_request": {
                    "message": msg
                }
            })
        } else {
            serde_json::json!({})
        };

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "CancellationRequest",
                Some(&fulfillment_order_id.to_string()),
                response.request_id(),
            ));
        }

        FulfillmentOrder::parse_response(&response)
    }

    /// Rejects a cancellation request.
    ///
    /// Sends a POST request to `/admin/api/{version}/fulfillment_orders/{id}/cancellation_request/reject.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `fulfillment_order_id` - The ID of the fulfillment order
    /// * `message` - Optional message explaining rejection
    ///
    /// # Returns
    ///
    /// The parent fulfillment order with updated `request_status`.
    pub async fn reject(
        client: &RestClient,
        fulfillment_order_id: u64,
        message: Option<&str>,
    ) -> Result<FulfillmentOrder, ResourceError> {
        let path =
            format!("fulfillment_orders/{fulfillment_order_id}/cancellation_request/reject");

        let body = if let Some(msg) = message {
            serde_json::json!({
                "cancellation_request": {
                    "message": msg
                }
            })
        } else {
            serde_json::json!({})
        };

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "CancellationRequest",
                Some(&fulfillment_order_id.to_string()),
                response.request_id(),
            ));
        }

        FulfillmentOrder::parse_response(&response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_fulfillment_order_deserialization() {
        let json_str = r##"{
            "id": 1046000778,
            "shop_id": 548380009,
            "order_id": 450789469,
            "assigned_location_id": 24826418,
            "status": "open",
            "request_status": "unsubmitted",
            "supported_actions": ["create_fulfillment", "request_fulfillment"],
            "destination": {
                "id": 1046000779,
                "first_name": "John",
                "last_name": "Doe",
                "address1": "123 Main St",
                "city": "New York",
                "province": "New York",
                "country": "United States",
                "zip": "10001",
                "email": "john@example.com"
            },
            "line_items": [
                {
                    "id": 1058737482,
                    "shop_id": 548380009,
                    "fulfillment_order_id": 1046000778,
                    "line_item_id": 669751112,
                    "inventory_item_id": 49148385,
                    "quantity": 1,
                    "fulfillable_quantity": 1
                }
            ],
            "fulfill_at": "2024-01-20T00:00:00Z",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "admin_graphql_api_id": "gid://shopify/FulfillmentOrder/1046000778"
        }"##;

        let fo: FulfillmentOrder = serde_json::from_str(json_str).unwrap();

        assert_eq!(fo.id, Some(1046000778));
        assert_eq!(fo.shop_id, Some(548380009));
        assert_eq!(fo.order_id, Some(450789469));
        assert_eq!(fo.assigned_location_id, Some(24826418));
        assert_eq!(fo.status, Some(FulfillmentOrderStatus::Open));
        assert_eq!(
            fo.request_status,
            Some(FulfillmentOrderRequestStatus::Unsubmitted)
        );

        // Check destination
        let dest = fo.destination.unwrap();
        assert_eq!(dest.first_name.as_deref(), Some("John"));
        assert_eq!(dest.city.as_deref(), Some("New York"));

        // Check line items
        let line_items = fo.line_items.unwrap();
        assert_eq!(line_items.len(), 1);
        assert_eq!(line_items[0].id, Some(1058737482));
        assert_eq!(line_items[0].quantity, Some(1));
    }

    #[test]
    fn test_fulfillment_order_status_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderStatus::Open).unwrap(),
            "\"open\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderStatus::InProgress).unwrap(),
            "\"in_progress\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderStatus::Cancelled).unwrap(),
            "\"cancelled\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderStatus::Incomplete).unwrap(),
            "\"incomplete\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderStatus::Closed).unwrap(),
            "\"closed\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderStatus::Scheduled).unwrap(),
            "\"scheduled\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderStatus::OnHold).unwrap(),
            "\"on_hold\""
        );

        // Test deserialization
        let open: FulfillmentOrderStatus = serde_json::from_str("\"open\"").unwrap();
        let in_progress: FulfillmentOrderStatus = serde_json::from_str("\"in_progress\"").unwrap();
        let on_hold: FulfillmentOrderStatus = serde_json::from_str("\"on_hold\"").unwrap();

        assert_eq!(open, FulfillmentOrderStatus::Open);
        assert_eq!(in_progress, FulfillmentOrderStatus::InProgress);
        assert_eq!(on_hold, FulfillmentOrderStatus::OnHold);

        assert_eq!(
            FulfillmentOrderStatus::default(),
            FulfillmentOrderStatus::Open
        );
    }

    #[test]
    fn test_fulfillment_order_request_status_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderRequestStatus::Unsubmitted).unwrap(),
            "\"unsubmitted\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderRequestStatus::Submitted).unwrap(),
            "\"submitted\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderRequestStatus::Accepted).unwrap(),
            "\"accepted\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderRequestStatus::Rejected).unwrap(),
            "\"rejected\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderRequestStatus::CancellationRequested).unwrap(),
            "\"cancellation_requested\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderRequestStatus::CancellationAccepted).unwrap(),
            "\"cancellation_accepted\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentOrderRequestStatus::CancellationRejected).unwrap(),
            "\"cancellation_rejected\""
        );

        // Test deserialization
        let cancellation_requested: FulfillmentOrderRequestStatus =
            serde_json::from_str("\"cancellation_requested\"").unwrap();
        assert_eq!(
            cancellation_requested,
            FulfillmentOrderRequestStatus::CancellationRequested
        );
    }

    #[test]
    fn test_hold_reason_enum_serialization() {
        assert_eq!(
            serde_json::to_string(&HoldReason::AwaitingPayment).unwrap(),
            "\"awaiting_payment\""
        );
        assert_eq!(
            serde_json::to_string(&HoldReason::HighRiskOfFraud).unwrap(),
            "\"high_risk_of_fraud\""
        );
        assert_eq!(
            serde_json::to_string(&HoldReason::IncorrectAddress).unwrap(),
            "\"incorrect_address\""
        );
        assert_eq!(
            serde_json::to_string(&HoldReason::InventoryOutOfStock).unwrap(),
            "\"inventory_out_of_stock\""
        );
        assert_eq!(
            serde_json::to_string(&HoldReason::Other).unwrap(),
            "\"other\""
        );

        // Test deserialization
        let awaiting_payment: HoldReason = serde_json::from_str("\"awaiting_payment\"").unwrap();
        assert_eq!(awaiting_payment, HoldReason::AwaitingPayment);
    }

    #[test]
    fn test_fulfillment_order_paths() {
        // Test standalone Find path
        let find_path = get_path(FulfillmentOrder::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "fulfillment_orders/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test nested All path under orders
        let all_path = get_path(
            FulfillmentOrder::PATHS,
            ResourceOperation::All,
            &["order_id"],
        );
        assert!(all_path.is_some());
        assert_eq!(
            all_path.unwrap().template,
            "orders/{order_id}/fulfillment_orders"
        );

        // Test that All without order_id fails
        let all_without_order = get_path(FulfillmentOrder::PATHS, ResourceOperation::All, &[]);
        assert!(all_without_order.is_none());

        // Test that there is no Create path
        let create_path = get_path(FulfillmentOrder::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_none());

        // Test that there is no Update path
        let update_path = get_path(FulfillmentOrder::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());

        // Test that there is no Delete path
        let delete_path = get_path(FulfillmentOrder::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());

        // Test that there is no Count path
        let count_path = get_path(FulfillmentOrder::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());

        // Verify constants
        assert_eq!(FulfillmentOrder::NAME, "FulfillmentOrder");
        assert_eq!(FulfillmentOrder::PLURAL, "fulfillment_orders");
    }

    #[test]
    fn test_special_operation_method_signatures() {
        // Verify cancel signature
        fn _assert_cancel_signature<F, Fut>(f: F)
        where
            F: Fn(&FulfillmentOrder, &RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify hold signature
        fn _assert_hold_signature<F, Fut>(f: F)
        where
            F: Fn(&FulfillmentOrder, &RestClient, FulfillmentOrderHoldParams) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify move_location signature
        fn _assert_move_signature<F, Fut>(f: F)
        where
            F: Fn(&FulfillmentOrder, &RestClient, FulfillmentOrderMoveParams) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify reschedule signature
        fn _assert_reschedule_signature<F, Fut>(f: F)
        where
            F: Fn(&FulfillmentOrder, &RestClient, FulfillmentOrderRescheduleParams) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify FulfillmentOrderHoldParams
        let hold_params = FulfillmentOrderHoldParams {
            reason: HoldReason::AwaitingPayment,
            reason_notes: Some("Waiting for wire transfer".to_string()),
            notify_merchant: Some(true),
            fulfillment_order_line_items: None,
        };

        let json = serde_json::to_value(&hold_params).unwrap();
        assert_eq!(json["reason"], "awaiting_payment");
        assert_eq!(json["reason_notes"], "Waiting for wire transfer");
        assert_eq!(json["notify_merchant"], true);

        // Verify FulfillmentOrderMoveParams
        let move_params = FulfillmentOrderMoveParams {
            new_location_id: 12345,
            fulfillment_order_line_items: Some(vec![FulfillmentOrderLineItemInput {
                id: 111,
                quantity: 2,
            }]),
        };
        assert_eq!(move_params.new_location_id, 12345);

        // Verify FulfillmentOrderRescheduleParams
        let reschedule_params = FulfillmentOrderRescheduleParams {
            new_fulfill_at: DateTime::parse_from_rfc3339("2024-02-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };
        assert!(reschedule_params.new_fulfill_at.timestamp() > 0);
    }

    #[test]
    fn test_fulfillment_request_signatures() {
        // Verify FulfillmentRequest::create signature
        fn _assert_create_signature<F, Fut>(f: F)
        where
            F: Fn(
                &RestClient,
                u64,
                Option<&str>,
                Option<Vec<FulfillmentOrderLineItemInput>>,
            ) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify FulfillmentRequest::accept signature
        fn _assert_accept_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient, u64, Option<&str>) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify FulfillmentRequest::reject signature
        fn _assert_reject_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient, u64, Option<&str>, Option<&str>) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }
    }

    #[test]
    fn test_cancellation_request_signatures() {
        // Verify CancellationRequest::create signature
        fn _assert_create_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient, u64, Option<&str>) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify CancellationRequest::accept signature
        fn _assert_accept_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient, u64, Option<&str>) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify CancellationRequest::reject signature
        fn _assert_reject_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient, u64, Option<&str>) -> Fut,
            Fut: std::future::Future<Output = Result<FulfillmentOrder, ResourceError>>,
        {
            let _ = f;
        }
    }

    #[test]
    fn test_fulfillment_order_get_id_returns_correct_value() {
        let fo_with_id = FulfillmentOrder {
            id: Some(1046000778),
            order_id: Some(450789469),
            ..Default::default()
        };
        assert_eq!(fo_with_id.get_id(), Some(1046000778));

        let fo_without_id = FulfillmentOrder {
            id: None,
            order_id: Some(450789469),
            ..Default::default()
        };
        assert_eq!(fo_without_id.get_id(), None);
    }
}

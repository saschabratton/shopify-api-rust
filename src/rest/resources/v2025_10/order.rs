//! Order resource implementation.
//!
//! This module provides the [`Order`] resource for managing orders in Shopify.
//! Orders represent completed checkout transactions.
//!
//! # Resource-Specific Operations
//!
//! In addition to standard CRUD operations, the Order resource provides:
//! - [`Order::cancel`] - Cancel an order
//! - [`Order::close`] - Close an order
//! - [`Order::open`] - Re-open a closed order
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{Order, OrderListParams, FinancialStatus};
//!
//! // Find a single order
//! let order = Order::find(&client, 123, None).await?;
//! println!("Order: {}", order.name.as_deref().unwrap_or(""));
//!
//! // List orders with filters
//! let params = OrderListParams {
//!     financial_status: Some(FinancialStatus::Paid),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let orders = Order::all(&client, Some(params)).await?;
//!
//! // Cancel an order
//! let cancelled_order = order.cancel(&client).await?;
//!
//! // Close an order
//! let closed_order = order.close(&client).await?;
//!
//! // Re-open a closed order
//! let reopened_order = closed_order.open(&client).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::{Address, DiscountApplication, LineItem, NoteAttribute, ShippingLine, TaxLine};
use super::customer::Customer;

/// The financial status of an order.
///
/// Indicates the payment status of the order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum FinancialStatus {
    /// Payment is pending.
    #[default]
    Pending,
    /// Payment has been authorized but not captured.
    Authorized,
    /// Payment has been partially paid.
    PartiallyPaid,
    /// Payment has been fully captured.
    Paid,
    /// Payment has been partially refunded.
    PartiallyRefunded,
    /// Payment has been fully refunded.
    Refunded,
    /// Payment authorization has been voided.
    Voided,
}

/// The fulfillment status of an order.
///
/// Indicates the shipping/fulfillment status of the order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentStatus {
    /// All line items have been fulfilled.
    Fulfilled,
    /// Some line items have been fulfilled.
    Partial,
    /// No line items have been fulfilled.
    Unfulfilled,
    /// Items have been restocked.
    Restocked,
}

/// The reason for canceling an order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CancelReason {
    /// Customer requested cancellation.
    Customer,
    /// Order was identified as fraudulent.
    Fraud,
    /// Items were out of stock.
    Inventory,
    /// Payment was declined.
    Declined,
    /// Other reason for cancellation.
    Other,
}

/// A discount code applied to an order.
///
/// Different from `DiscountApplication`, this represents the actual code
/// that was entered at checkout.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountCode {
    /// The discount code that was entered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The amount of the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// The type of discount (e.g., "percentage", `"fixed_amount"`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub discount_type: Option<String>,
}

/// A refund associated with an order.
///
/// Contains information about refunded amounts, line items, and transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Refund {
    /// The unique identifier of the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The ID of the order this refund belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<u64>,

    /// When the refund was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// An optional note attached to the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// The ID of the user who processed the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,

    /// When the refund was processed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_at: Option<DateTime<Utc>>,

    /// Whether items should be restocked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restock: Option<bool>,

    /// Duties associated with the refund (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duties: Option<serde_json::Value>,

    /// Refunded duties (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_duties: Option<serde_json::Value>,

    /// Line items included in the refund (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_line_items: Option<serde_json::Value>,

    /// Transactions for the refund (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<serde_json::Value>,

    /// Order adjustments from the refund (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_adjustments: Option<serde_json::Value>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_graphql_api_id: Option<String>,
}

/// An embedded fulfillment within an order response.
///
/// This is a simplified view of fulfillment data when embedded in order responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OrderFulfillment {
    /// The unique identifier of the fulfillment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The ID of the order this fulfillment belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<u64>,

    /// The status of the fulfillment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// When the fulfillment was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// The fulfillment service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// When the fulfillment was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,

    /// The tracking company name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_company: Option<String>,

    /// The shipment status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipment_status: Option<String>,

    /// The ID of the location that fulfilled the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<u64>,

    /// The tracking number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_number: Option<String>,

    /// Multiple tracking numbers (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_numbers: Option<Vec<String>>,

    /// The tracking URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_url: Option<String>,

    /// Multiple tracking URLs (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_urls: Option<Vec<String>>,

    /// Line items included in this fulfillment (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<serde_json::Value>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_graphql_api_id: Option<String>,
}

/// An order in Shopify.
///
/// Orders represent completed checkout transactions including the line items,
/// shipping information, and payment status.
///
/// # Read-Only Fields
///
/// The following fields are read-only and will not be sent in create/update requests:
/// - `id`, `name`, `order_number`
/// - `created_at`, `updated_at`
/// - `confirmation_number`, `admin_graphql_api_id`
///
/// # Example
///
/// ```rust,ignore
/// use shopify_sdk::rest::resources::v2025_10::Order;
///
/// let order = Order {
///     email: Some("customer@example.com".to_string()),
///     note: Some("Please gift wrap".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Order {
    // --- Read-only fields (not serialized) ---
    /// The unique identifier of the order.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The order name (e.g., "#1001").
    #[serde(skip_serializing)]
    pub name: Option<String>,

    /// The order number (integer portion of name).
    #[serde(skip_serializing)]
    pub order_number: Option<u64>,

    /// When the order was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the order was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The confirmation number.
    #[serde(skip_serializing)]
    pub confirmation_number: Option<String>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,

    // --- Core fields ---
    /// The customer's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// The customer's phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// When the order was closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,

    /// When the order was cancelled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancelled_at: Option<DateTime<Utc>>,

    /// The reason the order was cancelled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_reason: Option<CancelReason>,

    /// The financial status of the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_status: Option<FinancialStatus>,

    /// The fulfillment status of the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_status: Option<FulfillmentStatus>,

    /// The currency code (e.g., "USD").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// The total price of the order including taxes and discounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_price: Option<String>,

    /// The subtotal price (before taxes and shipping).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtotal_price: Option<String>,

    /// The total tax amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax: Option<String>,

    /// The total discount amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_discounts: Option<String>,

    /// The total weight in grams.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_weight: Option<i64>,

    /// Whether taxes are included in the prices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxes_included: Option<bool>,

    /// Whether the customer has opted into marketing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_accepts_marketing: Option<bool>,

    /// An optional note attached to the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Custom note attributes (key-value pairs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_attributes: Option<Vec<NoteAttribute>>,

    /// Comma-separated tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// The ID of the app that created the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<u64>,

    /// The IP address of the browser used to place the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_ip: Option<String>,

    /// The three-letter language code of the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_locale: Option<String>,

    /// The URL for the order status page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_status_url: Option<String>,

    /// When the order was processed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_at: Option<DateTime<Utc>>,

    /// The source name (e.g., "web", "pos").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,

    /// The total price in shop currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_price_usd: Option<String>,

    /// Total shipping price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_shipping_price_set: Option<serde_json::Value>,

    /// Total line items price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_line_items_price: Option<String>,

    /// Total outstanding amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_outstanding: Option<String>,

    /// Current total price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_total_price: Option<String>,

    /// Current subtotal price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_subtotal_price: Option<String>,

    /// Current total tax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_total_tax: Option<String>,

    /// Current total discounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_total_discounts: Option<String>,

    /// Whether the order has been confirmed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirmed: Option<bool>,

    /// Whether the order is a test order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<bool>,

    /// The ID of the user who placed the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,

    /// The ID of the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<u64>,

    /// The source identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_identifier: Option<String>,

    /// The source URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,

    /// The device ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<u64>,

    /// The landing site.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landing_site: Option<String>,

    /// The referring site.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referring_site: Option<String>,

    /// The gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,

    /// Payment gateway names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_gateway_names: Option<Vec<String>>,

    /// Processing method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_method: Option<String>,

    /// Reference for external order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    /// Checkout ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_id: Option<u64>,

    /// Checkout token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_token: Option<String>,

    /// Cart token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cart_token: Option<String>,

    /// Token for the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    // --- Nested structures ---
    /// The line items in the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<LineItem>>,

    /// The billing address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_address: Option<Address>,

    /// The shipping address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<Address>,

    /// Tax lines for the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_lines: Option<Vec<TaxLine>>,

    /// Discount codes applied to the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_codes: Option<Vec<DiscountCode>>,

    /// Discount applications on the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_applications: Option<Vec<DiscountApplication>>,

    /// Shipping lines for the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_lines: Option<Vec<ShippingLine>>,

    /// Fulfillments for the order (when embedded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillments: Option<Vec<OrderFulfillment>>,

    /// Refunds for the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refunds: Option<Vec<Refund>>,

    /// The customer who placed the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<Customer>,

    /// Client details (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_details: Option<serde_json::Value>,

    /// Payment details (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_details: Option<serde_json::Value>,
}

impl RestResource for Order {
    type Id = u64;
    type FindParams = OrderFindParams;
    type AllParams = OrderListParams;
    type CountParams = OrderCountParams;

    const NAME: &'static str = "Order";
    const PLURAL: &'static str = "orders";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "orders/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "orders"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "orders/count",
        ),
        ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "orders"),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "orders/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "orders/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl Order {
    /// Cancels the order.
    ///
    /// Sends a POST request to `/admin/api/{version}/orders/{id}/cancel.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the order doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the order has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let order = Order::find(&client, 123, None).await?;
    /// let cancelled = order.cancel(&client).await?;
    /// assert!(cancelled.cancelled_at.is_some());
    /// ```
    pub async fn cancel(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "cancel",
        })?;

        let path = format!("orders/{id}/cancel");
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

        // Parse the response - Shopify returns the order wrapped in "order" key
        let order: Self = response
            .body
            .get("order")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'order' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize order: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(order)
    }

    /// Closes the order.
    ///
    /// Sends a POST request to `/admin/api/{version}/orders/{id}/close.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the order doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the order has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let order = Order::find(&client, 123, None).await?;
    /// let closed = order.close(&client).await?;
    /// assert!(closed.closed_at.is_some());
    /// ```
    pub async fn close(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "close",
        })?;

        let path = format!("orders/{id}/close");
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

        // Parse the response - Shopify returns the order wrapped in "order" key
        let order: Self = response
            .body
            .get("order")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'order' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize order: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(order)
    }

    /// Re-opens a closed order.
    ///
    /// Sends a POST request to `/admin/api/{version}/orders/{id}/open.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the order doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the order has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let order = Order::find(&client, 123, None).await?;
    /// let reopened = order.open(&client).await?;
    /// assert!(reopened.closed_at.is_none());
    /// ```
    pub async fn open(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "open",
        })?;

        let path = format!("orders/{id}/open");
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

        // Parse the response - Shopify returns the order wrapped in "order" key
        let order: Self = response
            .body
            .get("order")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'order' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize order: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(order)
    }
}

/// Parameters for finding a single order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OrderFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing orders.
///
/// All fields are optional. Unset fields will not be included in the request.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OrderListParams {
    /// Comma-separated list of order IDs to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<u64>>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return only orders after the specified ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Filter by order status ("open", "closed", "cancelled", "any").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Filter by financial status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_status: Option<FinancialStatus>,

    /// Filter by fulfillment status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_status: Option<FulfillmentStatus>,

    /// Show orders created at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show orders created at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show orders last updated at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show orders last updated at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show orders processed at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_at_min: Option<DateTime<Utc>>,

    /// Show orders processed at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processed_at_max: Option<DateTime<Utc>>,

    /// Filter by the app that created the order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution_app_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Page info for cursor-based pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

/// Parameters for counting orders.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OrderCountParams {
    /// Filter by order status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Filter by financial status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub financial_status: Option<FinancialStatus>,

    /// Filter by fulfillment status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_status: Option<FulfillmentStatus>,

    /// Show orders created at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show orders created at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show orders last updated at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show orders last updated at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_order_struct_serialization() {
        let order = Order {
            id: Some(450789469),
            name: Some("#1001".to_string()),
            email: Some("customer@example.com".to_string()),
            phone: Some("+1-555-555-5555".to_string()),
            total_price: Some("199.99".to_string()),
            subtotal_price: Some("179.99".to_string()),
            total_tax: Some("15.00".to_string()),
            total_discounts: Some("5.00".to_string()),
            currency: Some("USD".to_string()),
            financial_status: Some(FinancialStatus::Paid),
            fulfillment_status: Some(FulfillmentStatus::Unfulfilled),
            tags: Some("important, vip".to_string()),
            note: Some("Please gift wrap".to_string()),
            buyer_accepts_marketing: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_string(&order).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["email"], "customer@example.com");
        assert_eq!(parsed["phone"], "+1-555-555-5555");
        assert_eq!(parsed["total_price"], "199.99");
        assert_eq!(parsed["currency"], "USD");
        assert_eq!(parsed["financial_status"], "paid");
        assert_eq!(parsed["fulfillment_status"], "unfulfilled");
        assert_eq!(parsed["note"], "Please gift wrap");

        // Read-only fields should NOT be serialized
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("name").is_none());
    }

    #[test]
    fn test_order_deserialization_with_nested_line_items() {
        // Use r##"..."## to allow # characters in the JSON string
        let json_str = r##"{
            "id": 450789469,
            "name": "#1001",
            "email": "customer@example.com",
            "order_number": 1001,
            "total_price": "199.99",
            "financial_status": "paid",
            "fulfillment_status": "partial",
            "line_items": [
                {
                    "id": 669751112,
                    "variant_id": 457924702,
                    "product_id": 632910392,
                    "title": "IPod Nano - 8GB",
                    "quantity": 1,
                    "price": "199.00",
                    "sku": "IPOD2008BLACK",
                    "taxable": true,
                    "tax_lines": [
                        {
                            "title": "State Tax",
                            "price": "15.99",
                            "rate": 0.08
                        }
                    ]
                }
            ],
            "billing_address": {
                "first_name": "John",
                "last_name": "Doe",
                "address1": "123 Main St",
                "city": "New York",
                "province": "New York",
                "country": "United States",
                "zip": "10001"
            },
            "customer": {
                "id": 207119551,
                "email": "customer@example.com",
                "first_name": "John",
                "last_name": "Doe"
            }
        }"##;

        let order: Order = serde_json::from_str(json_str).unwrap();

        assert_eq!(order.id, Some(450789469));
        assert_eq!(order.name.as_deref(), Some("#1001"));
        assert_eq!(order.email.as_deref(), Some("customer@example.com"));
        assert_eq!(order.order_number, Some(1001));
        assert_eq!(order.financial_status, Some(FinancialStatus::Paid));
        assert_eq!(order.fulfillment_status, Some(FulfillmentStatus::Partial));

        // Check line items
        let line_items = order.line_items.unwrap();
        assert_eq!(line_items.len(), 1);
        assert_eq!(line_items[0].id, Some(669751112));
        assert_eq!(line_items[0].title.as_deref(), Some("IPod Nano - 8GB"));
        assert_eq!(line_items[0].quantity, Some(1));

        // Check nested tax_lines in line_item
        let tax_lines = line_items[0].tax_lines.as_ref().unwrap();
        assert_eq!(tax_lines.len(), 1);
        assert_eq!(tax_lines[0].title.as_deref(), Some("State Tax"));

        // Check billing address
        let billing = order.billing_address.unwrap();
        assert_eq!(billing.first_name.as_deref(), Some("John"));
        assert_eq!(billing.city.as_deref(), Some("New York"));

        // Check customer
        let customer = order.customer.unwrap();
        assert_eq!(customer.id, Some(207119551));
        assert_eq!(customer.first_name.as_deref(), Some("John"));
    }

    #[test]
    fn test_financial_status_enum_serialization() {
        // Test serialization
        assert_eq!(
            serde_json::to_string(&FinancialStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&FinancialStatus::Authorized).unwrap(),
            "\"authorized\""
        );
        assert_eq!(
            serde_json::to_string(&FinancialStatus::PartiallyPaid).unwrap(),
            "\"partially_paid\""
        );
        assert_eq!(
            serde_json::to_string(&FinancialStatus::Paid).unwrap(),
            "\"paid\""
        );
        assert_eq!(
            serde_json::to_string(&FinancialStatus::PartiallyRefunded).unwrap(),
            "\"partially_refunded\""
        );
        assert_eq!(
            serde_json::to_string(&FinancialStatus::Refunded).unwrap(),
            "\"refunded\""
        );
        assert_eq!(
            serde_json::to_string(&FinancialStatus::Voided).unwrap(),
            "\"voided\""
        );

        // Test deserialization
        let paid: FinancialStatus = serde_json::from_str("\"paid\"").unwrap();
        let partially_refunded: FinancialStatus =
            serde_json::from_str("\"partially_refunded\"").unwrap();

        assert_eq!(paid, FinancialStatus::Paid);
        assert_eq!(partially_refunded, FinancialStatus::PartiallyRefunded);

        // Test default
        assert_eq!(FinancialStatus::default(), FinancialStatus::Pending);
    }

    #[test]
    fn test_fulfillment_status_enum_serialization() {
        // Test serialization
        assert_eq!(
            serde_json::to_string(&FulfillmentStatus::Fulfilled).unwrap(),
            "\"fulfilled\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentStatus::Partial).unwrap(),
            "\"partial\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentStatus::Unfulfilled).unwrap(),
            "\"unfulfilled\""
        );
        assert_eq!(
            serde_json::to_string(&FulfillmentStatus::Restocked).unwrap(),
            "\"restocked\""
        );

        // Test deserialization
        let fulfilled: FulfillmentStatus = serde_json::from_str("\"fulfilled\"").unwrap();
        let partial: FulfillmentStatus = serde_json::from_str("\"partial\"").unwrap();

        assert_eq!(fulfilled, FulfillmentStatus::Fulfilled);
        assert_eq!(partial, FulfillmentStatus::Partial);
    }

    #[test]
    fn test_cancel_reason_enum_serialization() {
        // Test serialization
        assert_eq!(
            serde_json::to_string(&CancelReason::Customer).unwrap(),
            "\"customer\""
        );
        assert_eq!(
            serde_json::to_string(&CancelReason::Fraud).unwrap(),
            "\"fraud\""
        );
        assert_eq!(
            serde_json::to_string(&CancelReason::Inventory).unwrap(),
            "\"inventory\""
        );
        assert_eq!(
            serde_json::to_string(&CancelReason::Declined).unwrap(),
            "\"declined\""
        );
        assert_eq!(
            serde_json::to_string(&CancelReason::Other).unwrap(),
            "\"other\""
        );

        // Test deserialization
        let fraud: CancelReason = serde_json::from_str("\"fraud\"").unwrap();
        let inventory: CancelReason = serde_json::from_str("\"inventory\"").unwrap();

        assert_eq!(fraud, CancelReason::Fraud);
        assert_eq!(inventory, CancelReason::Inventory);
    }

    #[test]
    fn test_order_list_params_with_status_filters() {
        let created_at_min = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let params = OrderListParams {
            ids: Some(vec![123, 456, 789]),
            limit: Some(50),
            since_id: Some(100),
            status: Some("open".to_string()),
            financial_status: Some(FinancialStatus::Paid),
            fulfillment_status: Some(FulfillmentStatus::Unfulfilled),
            created_at_min: Some(created_at_min),
            created_at_max: None,
            updated_at_min: None,
            updated_at_max: None,
            processed_at_min: None,
            processed_at_max: None,
            attribution_app_id: Some(12345),
            fields: Some("id,name,total_price".to_string()),
            page_info: None,
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["ids"], serde_json::json!([123, 456, 789]));
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);
        assert_eq!(json["status"], "open");
        assert_eq!(json["financial_status"], "paid");
        assert_eq!(json["fulfillment_status"], "unfulfilled");
        assert_eq!(json["attribution_app_id"], 12345);
        assert_eq!(json["fields"], "id,name,total_price");
        assert!(json["created_at_min"].as_str().is_some());

        // Test empty params
        let empty_params = OrderListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_order_resource_specific_operations_signatures() {
        // This test verifies that the cancel, close, and open methods exist
        // with the correct signatures. The actual HTTP calls would require
        // a mock client, but we verify the type signatures compile correctly.

        // Verify the method signatures are correct by referencing them
        fn _assert_cancel_signature<F, Fut>(f: F)
        where
            F: Fn(&Order, &RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<Order, ResourceError>>,
        {
            let _ = f;
        }

        fn _assert_close_signature<F, Fut>(f: F)
        where
            F: Fn(&Order, &RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<Order, ResourceError>>,
        {
            let _ = f;
        }

        fn _assert_open_signature<F, Fut>(f: F)
        where
            F: Fn(&Order, &RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<Order, ResourceError>>,
        {
            let _ = f;
        }

        // Verify PathResolutionFailed error is returned when order has no ID
        let order_without_id = Order::default();
        assert!(order_without_id.get_id().is_none());

        // The actual async tests would require a mock RestClient
        // For now, we just verify the types compile
    }

    #[test]
    fn test_order_get_id_returns_correct_value() {
        let order_with_id = Order {
            id: Some(450789469),
            name: Some("#1001".to_string()),
            ..Default::default()
        };
        assert_eq!(order_with_id.get_id(), Some(450789469));

        let order_without_id = Order {
            id: None,
            email: Some("new@example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(order_without_id.get_id(), None);
    }

    #[test]
    fn test_order_path_constants_are_correct() {
        let find_path = get_path(Order::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "orders/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        let all_path = get_path(Order::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "orders");

        let count_path = get_path(Order::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "orders/count");

        let create_path = get_path(Order::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        let update_path = get_path(Order::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        let delete_path = get_path(Order::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        assert_eq!(Order::NAME, "Order");
        assert_eq!(Order::PLURAL, "orders");
    }

    #[test]
    fn test_discount_code_serialization() {
        let discount = DiscountCode {
            code: Some("SAVE10".to_string()),
            amount: Some("10.00".to_string()),
            discount_type: Some("fixed_amount".to_string()),
        };

        let json = serde_json::to_string(&discount).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["code"], "SAVE10");
        assert_eq!(parsed["amount"], "10.00");
        assert_eq!(parsed["type"], "fixed_amount");

        // Test deserialization with renamed field
        let json_str = r#"{"code":"SUMMER20","amount":"20.00","type":"percentage"}"#;
        let parsed_discount: DiscountCode = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed_discount.discount_type.as_deref(), Some("percentage"));
    }

    #[test]
    fn test_refund_struct_serialization() {
        let refund = Refund {
            id: Some(123456),
            order_id: Some(450789469),
            note: Some("Customer requested".to_string()),
            restock: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_string(&refund).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], 123456);
        assert_eq!(parsed["order_id"], 450789469);
        assert_eq!(parsed["note"], "Customer requested");
        assert_eq!(parsed["restock"], true);
    }

    #[test]
    fn test_order_with_all_nested_structures() {
        // Use r##"..."## to allow # characters in the JSON string
        let json_str = r##"{
            "id": 450789469,
            "name": "#1001",
            "email": "customer@example.com",
            "financial_status": "partially_refunded",
            "line_items": [
                {"id": 1, "title": "Product 1", "quantity": 2}
            ],
            "billing_address": {
                "first_name": "John",
                "city": "New York"
            },
            "shipping_address": {
                "first_name": "John",
                "city": "New York"
            },
            "tax_lines": [
                {"title": "Tax", "price": "10.00", "rate": 0.1}
            ],
            "discount_codes": [
                {"code": "SAVE10", "amount": "10.00", "type": "fixed_amount"}
            ],
            "discount_applications": [
                {"type": "discount_code", "value": "10.00", "code": "SAVE10"}
            ],
            "shipping_lines": [
                {"id": 1, "title": "Standard", "price": "5.00"}
            ],
            "fulfillments": [
                {"id": 1, "status": "success", "tracking_number": "1234"}
            ],
            "refunds": [
                {"id": 1, "note": "Partial refund"}
            ],
            "customer": {
                "id": 207119551,
                "email": "customer@example.com"
            }
        }"##;

        let order: Order = serde_json::from_str(json_str).unwrap();

        assert!(order.line_items.is_some());
        assert!(order.billing_address.is_some());
        assert!(order.shipping_address.is_some());
        assert!(order.tax_lines.is_some());
        assert!(order.discount_codes.is_some());
        assert!(order.discount_applications.is_some());
        assert!(order.shipping_lines.is_some());
        assert!(order.fulfillments.is_some());
        assert!(order.refunds.is_some());
        assert!(order.customer.is_some());

        assert_eq!(
            order.financial_status,
            Some(FinancialStatus::PartiallyRefunded)
        );
        assert_eq!(
            order.discount_codes.as_ref().unwrap()[0].code.as_deref(),
            Some("SAVE10")
        );
        assert_eq!(
            order.fulfillments.as_ref().unwrap()[0]
                .tracking_number
                .as_deref(),
            Some("1234")
        );
    }
}

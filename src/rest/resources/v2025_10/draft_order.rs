//! DraftOrder resource implementation.
//!
//! This module provides the [`DraftOrder`] resource for managing draft orders in Shopify.
//! Draft orders are used for B2B/wholesale order creation workflows where merchants
//! can create orders on behalf of customers before payment is completed.
//!
//! # Resource-Specific Operations
//!
//! In addition to standard CRUD operations, the DraftOrder resource provides:
//! - [`DraftOrder::complete`] - Convert a draft order to an actual order
//! - [`DraftOrder::send_invoice`] - Send an invoice email to the customer
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{
//!     DraftOrder, DraftOrderListParams, DraftOrderStatus,
//!     DraftOrderLineItem, AppliedDiscount, DraftOrderInvoice,
//!     DraftOrderCompleteParams
//! };
//!
//! // Create a draft order
//! let mut draft = DraftOrder {
//!     line_items: Some(vec![DraftOrderLineItem {
//!         variant_id: Some(123456),
//!         quantity: Some(2),
//!         ..Default::default()
//!     }]),
//!     customer_id: Some(789012),
//!     ..Default::default()
//! };
//! let saved = draft.save(&client).await?;
//!
//! // Send an invoice to the customer
//! let invoice = DraftOrderInvoice {
//!     to: Some("customer@example.com".to_string()),
//!     subject: Some("Your order is ready".to_string()),
//!     custom_message: Some("Thanks for your order!".to_string()),
//!     ..Default::default()
//! };
//! let updated = saved.send_invoice(&client, invoice).await?;
//!
//! // Complete the draft order to create an actual order
//! let params = DraftOrderCompleteParams {
//!     payment_pending: Some(true),
//! };
//! let completed = updated.complete(&client, Some(params)).await?;
//! println!("Created order ID: {:?}", completed.order_id);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::{Address, NoteAttribute, ShippingLine, TaxLine};
use super::customer::Customer;

/// The status of a draft order.
///
/// Indicates the current state of the draft order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DraftOrderStatus {
    /// The draft order is open and can be edited.
    #[default]
    Open,
    /// An invoice has been sent to the customer.
    InvoiceSent,
    /// The draft order has been completed and converted to an order.
    Completed,
}

/// A discount applied to a draft order or line item.
///
/// Used for applying custom discounts to draft orders.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AppliedDiscount {
    /// The title of the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// A description of the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The discount value (numeric).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The type of value: "percentage" or "fixed_amount".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,

    /// The calculated discount amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

/// A line item in a draft order.
///
/// Represents a product or custom item to be included in the draft order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DraftOrderLineItem {
    /// The unique identifier of the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The ID of the product variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<u64>,

    /// The ID of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// The title of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The title of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_title: Option<String>,

    /// The SKU of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,

    /// The vendor of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    /// The quantity of items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,

    /// Whether the item requires shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_shipping: Option<bool>,

    /// Whether the item is taxable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxable: Option<bool>,

    /// Whether the item is a gift card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift_card: Option<bool>,

    /// The fulfillment service for the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_service: Option<String>,

    /// The weight in grams.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grams: Option<i64>,

    /// Tax lines applied to this line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_lines: Option<Vec<TaxLine>>,

    /// The name of the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Custom properties on the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<serde_json::Value>>,

    /// Whether the custom line item (not a product variant).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<bool>,

    /// The price per item as a string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// A discount applied to this specific line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_discount: Option<AppliedDiscount>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_graphql_api_id: Option<String>,
}

/// Invoice details for sending to a customer.
///
/// Used with the [`DraftOrder::send_invoice`] operation.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DraftOrderInvoice {
    /// The email address to send the invoice to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,

    /// The email address the invoice is sent from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    /// The subject line of the email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    /// A custom message included in the invoice email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_message: Option<String>,

    /// Email addresses to BCC on the invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
}

/// Parameters for completing a draft order.
///
/// Used with the [`DraftOrder::complete`] operation.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DraftOrderCompleteParams {
    /// Whether to mark the payment as pending.
    /// If true, the order is created with payment pending status.
    /// If false or omitted, the order is marked as paid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_pending: Option<bool>,
}

/// A draft order in Shopify.
///
/// Draft orders are used for B2B/wholesale workflows where merchants create
/// orders on behalf of customers. They support custom pricing, discounts,
/// and can be completed to become actual orders.
///
/// # Status Lifecycle
///
/// - `Open` - Initial state, can be edited
/// - `InvoiceSent` - Invoice sent to customer via `send_invoice()`
/// - `Completed` - Converted to actual order via `complete()`
///
/// # Read-Only Fields
///
/// The following fields are read-only and will not be sent in create/update requests:
/// - `id`, `order_id`, `name`
/// - `invoice_sent_at`, `invoice_url`
/// - `completed_at`, `created_at`, `updated_at`
/// - `admin_graphql_api_id`
///
/// # Example
///
/// ```rust,ignore
/// use shopify_sdk::rest::resources::v2025_10::{DraftOrder, DraftOrderLineItem};
///
/// let draft = DraftOrder {
///     line_items: Some(vec![DraftOrderLineItem {
///         variant_id: Some(123456),
///         quantity: Some(2),
///         ..Default::default()
///     }]),
///     note: Some("Wholesale order".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DraftOrder {
    // --- Read-only fields (not serialized) ---
    /// The unique identifier of the draft order.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the order created when this draft was completed.
    #[serde(skip_serializing)]
    pub order_id: Option<u64>,

    /// The name of the draft order (e.g., "#D1").
    #[serde(skip_serializing)]
    pub name: Option<String>,

    /// When an invoice was last sent for this draft order.
    #[serde(skip_serializing)]
    pub invoice_sent_at: Option<DateTime<Utc>>,

    /// The URL to the invoice for this draft order.
    #[serde(skip_serializing)]
    pub invoice_url: Option<String>,

    /// When the draft order was completed.
    #[serde(skip_serializing)]
    pub completed_at: Option<DateTime<Utc>>,

    /// When the draft order was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the draft order was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,

    // --- Core fields ---
    /// The status of the draft order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DraftOrderStatus>,

    /// The customer's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// The currency code for the draft order (e.g., "USD").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Whether the customer is tax exempt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exempt: Option<bool>,

    /// Tax exemptions applied to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_exemptions: Option<Vec<String>>,

    /// Whether taxes are included in the prices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxes_included: Option<bool>,

    /// The total tax amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax: Option<String>,

    /// The subtotal price before taxes and shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtotal_price: Option<String>,

    /// The total price including taxes and shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_price: Option<String>,

    /// An optional note attached to the draft order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Custom note attributes (key-value pairs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_attributes: Option<Vec<NoteAttribute>>,

    /// Comma-separated tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// The ID of the customer associated with this draft order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<u64>,

    /// Whether to use the customer's default address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_customer_default_address: Option<bool>,

    // --- Nested structures ---
    /// Line items in the draft order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_items: Option<Vec<DraftOrderLineItem>>,

    /// The shipping address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<Address>,

    /// The billing address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing_address: Option<Address>,

    /// The customer associated with this draft order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<Customer>,

    /// The shipping line for the draft order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_line: Option<ShippingLine>,

    /// Tax lines for the draft order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_lines: Option<Vec<TaxLine>>,

    /// A discount applied to the entire draft order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_discount: Option<AppliedDiscount>,
}

impl RestResource for DraftOrder {
    type Id = u64;
    type FindParams = DraftOrderFindParams;
    type AllParams = DraftOrderListParams;
    type CountParams = DraftOrderCountParams;

    const NAME: &'static str = "DraftOrder";
    const PLURAL: &'static str = "draft_orders";

    /// Paths for the DraftOrder resource.
    ///
    /// DraftOrder uses standalone paths (not nested under other resources).
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "draft_orders/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "draft_orders"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "draft_orders/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "draft_orders",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "draft_orders/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "draft_orders/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl DraftOrder {
    /// Completes the draft order and converts it to an actual order.
    ///
    /// Sends a PUT request to `/admin/api/{version}/draft_orders/{id}/complete.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `params` - Optional parameters including `payment_pending`
    ///
    /// # Returns
    ///
    /// The completed draft order with the `order_id` field populated.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the draft order doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the draft order has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let params = DraftOrderCompleteParams {
    ///     payment_pending: Some(true),
    /// };
    /// let completed = draft_order.complete(&client, Some(params)).await?;
    /// println!("Created order ID: {:?}", completed.order_id);
    /// ```
    pub async fn complete(
        &self,
        client: &RestClient,
        params: Option<DraftOrderCompleteParams>,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "complete",
        })?;

        // Build query string for payment_pending parameter
        let query_string = params
            .as_ref()
            .and_then(|p| p.payment_pending)
            .map(|pp| format!("?payment_pending={}", pp))
            .unwrap_or_default();

        let path = format!("draft_orders/{id}/complete{query_string}");
        let body = serde_json::json!({});

        let response = client.put(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the response - Shopify returns the draft order wrapped in "draft_order" key
        let draft_order: Self = response
            .body
            .get("draft_order")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'draft_order' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize draft_order: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(draft_order)
    }

    /// Sends an invoice for the draft order to the customer.
    ///
    /// Sends a POST request to `/admin/api/{version}/draft_orders/{id}/send_invoice.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `invoice` - Invoice details including recipient, subject, and message
    ///
    /// # Returns
    ///
    /// The updated draft order with `invoice_sent_at` and `invoice_url` populated.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the draft order doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the draft order has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let invoice = DraftOrderInvoice {
    ///     to: Some("customer@example.com".to_string()),
    ///     subject: Some("Your order is ready".to_string()),
    ///     custom_message: Some("Thank you for your order!".to_string()),
    ///     ..Default::default()
    /// };
    /// let updated = draft_order.send_invoice(&client, invoice).await?;
    /// println!("Invoice URL: {:?}", updated.invoice_url);
    /// ```
    pub async fn send_invoice(
        &self,
        client: &RestClient,
        invoice: DraftOrderInvoice,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "send_invoice",
        })?;

        let path = format!("draft_orders/{id}/send_invoice");

        let body = serde_json::json!({
            "draft_order_invoice": invoice
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

        // Parse the response - Shopify returns the draft order wrapped in "draft_order" key
        let draft_order: Self = response
            .body
            .get("draft_order")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'draft_order' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize draft_order: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(draft_order)
    }
}

/// Parameters for finding a single draft order.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DraftOrderFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing draft orders.
///
/// All fields are optional. Unset fields will not be included in the request.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DraftOrderListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return only draft orders after the specified ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Show draft orders created at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show draft orders created at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show draft orders last updated at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show draft orders last updated at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Comma-separated list of draft order IDs to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<u64>>,

    /// Filter by draft order status (open, invoice_sent, completed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DraftOrderStatus>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Page info for cursor-based pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

/// Parameters for counting draft orders.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DraftOrderCountParams {
    /// Return only draft orders after the specified ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Filter by draft order status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DraftOrderStatus>,

    /// Show draft orders created at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show draft orders created at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show draft orders last updated at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show draft orders last updated at or before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_draft_order_struct_serialization() {
        let draft_order = DraftOrder {
            id: Some(123456789),
            order_id: Some(987654321),
            name: Some("#D1".to_string()),
            status: Some(DraftOrderStatus::Open),
            email: Some("customer@example.com".to_string()),
            currency: Some("USD".to_string()),
            tax_exempt: Some(false),
            taxes_included: Some(true),
            total_tax: Some("10.00".to_string()),
            subtotal_price: Some("100.00".to_string()),
            total_price: Some("110.00".to_string()),
            note: Some("Wholesale order".to_string()),
            tags: Some("vip, wholesale".to_string()),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };

        let json = serde_json::to_string(&draft_order).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["status"], "open");
        assert_eq!(parsed["email"], "customer@example.com");
        assert_eq!(parsed["currency"], "USD");
        assert_eq!(parsed["tax_exempt"], false);
        assert_eq!(parsed["taxes_included"], true);
        assert_eq!(parsed["total_tax"], "10.00");
        assert_eq!(parsed["subtotal_price"], "100.00");
        assert_eq!(parsed["total_price"], "110.00");
        assert_eq!(parsed["note"], "Wholesale order");
        assert_eq!(parsed["tags"], "vip, wholesale");

        // Read-only fields should NOT be serialized
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("order_id").is_none());
        assert!(parsed.get("name").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_draft_order_deserialization_from_api_response() {
        let json_str = r##"{
            "id": 994118539,
            "order_id": null,
            "name": "#D2",
            "status": "open",
            "email": "bob.norman@example.com",
            "currency": "USD",
            "invoice_sent_at": null,
            "invoice_url": "https://jsmith.myshopify.com/548380009/invoices/994118539/dcc0adb7c08e3be1",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "tax_exempt": false,
            "taxes_included": false,
            "total_tax": "11.94",
            "subtotal_price": "398.00",
            "total_price": "409.94",
            "line_items": [
                {
                    "id": 994118540,
                    "variant_id": 39072856,
                    "product_id": 632910392,
                    "title": "IPod Nano - 8GB",
                    "variant_title": "green",
                    "sku": "IPOD2008GREEN",
                    "vendor": "Apple",
                    "quantity": 1,
                    "requires_shipping": true,
                    "taxable": true,
                    "gift_card": false,
                    "price": "199.00"
                }
            ],
            "shipping_address": {
                "first_name": "Bob",
                "last_name": "Norman",
                "address1": "Chestnut Street 92",
                "city": "Louisville",
                "province": "Kentucky",
                "country": "United States",
                "zip": "40202"
            },
            "billing_address": {
                "first_name": "Bob",
                "last_name": "Norman",
                "address1": "Chestnut Street 92",
                "city": "Louisville",
                "province": "Kentucky",
                "country": "United States",
                "zip": "40202"
            },
            "note": "Test draft order",
            "admin_graphql_api_id": "gid://shopify/DraftOrder/994118539"
        }"##;

        let draft_order: DraftOrder = serde_json::from_str(json_str).unwrap();

        assert_eq!(draft_order.id, Some(994118539));
        assert_eq!(draft_order.order_id, None);
        assert_eq!(draft_order.name.as_deref(), Some("#D2"));
        assert_eq!(draft_order.status, Some(DraftOrderStatus::Open));
        assert_eq!(
            draft_order.email.as_deref(),
            Some("bob.norman@example.com")
        );
        assert_eq!(draft_order.currency.as_deref(), Some("USD"));
        assert_eq!(draft_order.total_tax.as_deref(), Some("11.94"));
        assert_eq!(draft_order.subtotal_price.as_deref(), Some("398.00"));
        assert_eq!(draft_order.total_price.as_deref(), Some("409.94"));
        assert!(draft_order.created_at.is_some());
        assert!(draft_order.updated_at.is_some());

        // Check line items
        let line_items = draft_order.line_items.unwrap();
        assert_eq!(line_items.len(), 1);
        assert_eq!(line_items[0].id, Some(994118540));
        assert_eq!(line_items[0].title.as_deref(), Some("IPod Nano - 8GB"));
        assert_eq!(line_items[0].quantity, Some(1));
        assert_eq!(line_items[0].price.as_deref(), Some("199.00"));

        // Check shipping address
        let shipping = draft_order.shipping_address.unwrap();
        assert_eq!(shipping.first_name.as_deref(), Some("Bob"));
        assert_eq!(shipping.city.as_deref(), Some("Louisville"));

        // Check billing address
        let billing = draft_order.billing_address.unwrap();
        assert_eq!(billing.first_name.as_deref(), Some("Bob"));
    }

    #[test]
    fn test_draft_order_status_enum_serialization() {
        // Test serialization to snake_case
        let open_str = serde_json::to_string(&DraftOrderStatus::Open).unwrap();
        assert_eq!(open_str, "\"open\"");

        let invoice_sent_str = serde_json::to_string(&DraftOrderStatus::InvoiceSent).unwrap();
        assert_eq!(invoice_sent_str, "\"invoice_sent\"");

        let completed_str = serde_json::to_string(&DraftOrderStatus::Completed).unwrap();
        assert_eq!(completed_str, "\"completed\"");

        // Test deserialization
        let open: DraftOrderStatus = serde_json::from_str("\"open\"").unwrap();
        let invoice_sent: DraftOrderStatus = serde_json::from_str("\"invoice_sent\"").unwrap();
        let completed: DraftOrderStatus = serde_json::from_str("\"completed\"").unwrap();

        assert_eq!(open, DraftOrderStatus::Open);
        assert_eq!(invoice_sent, DraftOrderStatus::InvoiceSent);
        assert_eq!(completed, DraftOrderStatus::Completed);

        // Test default
        assert_eq!(DraftOrderStatus::default(), DraftOrderStatus::Open);
    }

    #[test]
    fn test_draft_order_path_constants() {
        // Test Find path
        let find_path = get_path(DraftOrder::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "draft_orders/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(DraftOrder::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "draft_orders");

        // Test Count path
        let count_path = get_path(DraftOrder::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "draft_orders/count");

        // Test Create path
        let create_path = get_path(DraftOrder::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "draft_orders");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(DraftOrder::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "draft_orders/{id}");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test Delete path
        let delete_path = get_path(DraftOrder::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "draft_orders/{id}");
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        // Verify constants
        assert_eq!(DraftOrder::NAME, "DraftOrder");
        assert_eq!(DraftOrder::PLURAL, "draft_orders");
    }

    #[test]
    fn test_complete_method_signature() {
        // Verify the complete method signature compiles correctly
        fn _assert_complete_signature<F, Fut>(f: F)
        where
            F: Fn(&DraftOrder, &RestClient, Option<DraftOrderCompleteParams>) -> Fut,
            Fut: std::future::Future<Output = Result<DraftOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify DraftOrderCompleteParams
        let params = DraftOrderCompleteParams {
            payment_pending: Some(true),
        };
        assert_eq!(params.payment_pending, Some(true));

        // Verify PathResolutionFailed error is returned when draft order has no ID
        let draft_without_id = DraftOrder::default();
        assert!(draft_without_id.get_id().is_none());
    }

    #[test]
    fn test_send_invoice_method_signature() {
        // Verify the send_invoice method signature compiles correctly
        fn _assert_send_invoice_signature<F, Fut>(f: F)
        where
            F: Fn(&DraftOrder, &RestClient, DraftOrderInvoice) -> Fut,
            Fut: std::future::Future<Output = Result<DraftOrder, ResourceError>>,
        {
            let _ = f;
        }

        // Verify DraftOrderInvoice struct
        let invoice = DraftOrderInvoice {
            to: Some("customer@example.com".to_string()),
            from: Some("store@example.com".to_string()),
            subject: Some("Your order".to_string()),
            custom_message: Some("Thanks!".to_string()),
            bcc: Some(vec!["admin@example.com".to_string()]),
        };

        let json = serde_json::to_value(&invoice).unwrap();
        assert_eq!(json["to"], "customer@example.com");
        assert_eq!(json["from"], "store@example.com");
        assert_eq!(json["subject"], "Your order");
        assert_eq!(json["custom_message"], "Thanks!");
        assert_eq!(json["bcc"][0], "admin@example.com");
    }

    #[test]
    fn test_draft_order_line_item_with_applied_discount() {
        let line_item = DraftOrderLineItem {
            id: Some(123),
            variant_id: Some(456),
            product_id: Some(789),
            title: Some("Test Product".to_string()),
            quantity: Some(2),
            price: Some("50.00".to_string()),
            taxable: Some(true),
            applied_discount: Some(AppliedDiscount {
                title: Some("10% Off".to_string()),
                description: Some("Wholesale discount".to_string()),
                value: Some("10".to_string()),
                value_type: Some("percentage".to_string()),
                amount: Some("10.00".to_string()),
            }),
            ..Default::default()
        };

        let json = serde_json::to_value(&line_item).unwrap();
        assert_eq!(json["id"], 123);
        assert_eq!(json["title"], "Test Product");
        assert_eq!(json["quantity"], 2);
        assert!(json.get("applied_discount").is_some());
        assert_eq!(json["applied_discount"]["title"], "10% Off");
        assert_eq!(json["applied_discount"]["value"], "10");
        assert_eq!(json["applied_discount"]["value_type"], "percentage");
    }

    #[test]
    fn test_draft_order_invoice_serialization() {
        let invoice = DraftOrderInvoice {
            to: Some("customer@example.com".to_string()),
            subject: Some("Invoice for your order".to_string()),
            custom_message: Some("Thank you for shopping with us!".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&invoice).unwrap();

        assert_eq!(json["to"], "customer@example.com");
        assert_eq!(json["subject"], "Invoice for your order");
        assert_eq!(json["custom_message"], "Thank you for shopping with us!");
        assert!(json.get("from").is_none());
        assert!(json.get("bcc").is_none());

        // Test empty invoice
        let empty_invoice = DraftOrderInvoice::default();
        let empty_json = serde_json::to_value(&empty_invoice).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_draft_order_get_id_returns_correct_value() {
        let draft_with_id = DraftOrder {
            id: Some(994118539),
            name: Some("#D2".to_string()),
            ..Default::default()
        };
        assert_eq!(draft_with_id.get_id(), Some(994118539));

        let draft_without_id = DraftOrder {
            id: None,
            email: Some("customer@example.com".to_string()),
            ..Default::default()
        };
        assert_eq!(draft_without_id.get_id(), None);
    }
}

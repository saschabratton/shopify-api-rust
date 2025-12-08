//! Refund resource implementation.
//!
//! This module provides the `RefundResource` resource, which represents a refund
//! associated with an order in Shopify.
//!
//! # Note on Naming
//!
//! This module uses `RefundResource` as the struct name to avoid conflicts with
//! the embedded `Refund` struct in the Order module. The embedded `Refund` struct
//! is used when refunds appear within order responses, while `RefundResource` is
//! the full resource for direct refund operations.
//!
//! # Nested Path Pattern
//!
//! Refunds are always accessed under an order:
//! - List: `/orders/{order_id}/refunds`
//! - Find: `/orders/{order_id}/refunds/{id}`
//! - Create: `/orders/{order_id}/refunds`
//! - Calculate: `/orders/{order_id}/refunds/calculate`
//!
//! Use `RefundResource::all_with_parent()` to list refunds under a specific order.
//!
//! # Special Operations
//!
//! - [`RefundResource::calculate`] - Calculate refund amounts without creating a refund
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{RefundResource, RefundListParams, RefundCalculateParams};
//!
//! // List refunds under a specific order
//! let refunds = RefundResource::all_with_parent(&client, "order_id", 450789469, None).await?;
//! for refund in refunds.iter() {
//!     println!("Refund ID: {} - Note: {:?}", refund.id.unwrap_or(0), refund.note);
//! }
//!
//! // Calculate a potential refund without creating it
//! let calc_params = RefundCalculateParams {
//!     shipping: Some(RefundShipping { full_refund: Some(true), ..Default::default() }),
//!     refund_line_items: Some(vec![
//!         RefundLineItemInput { line_item_id: 669751112, quantity: 1, restock_type: None },
//!     ]),
//!     ..Default::default()
//! };
//! let calculation = RefundResource::calculate(&client, 450789469, calc_params).await?;
//! println!("Estimated refund: {:?}", calculation);
//!
//! // Create an actual refund
//! let mut refund = RefundResource {
//!     order_id: Some(450789469),
//!     note: Some("Customer requested refund".to_string()),
//!     notify: Some(true),
//!     refund_line_items: Some(serde_json::json!([
//!         {"line_item_id": 669751112, "quantity": 1, "restock_type": "return"}
//!     ])),
//!     ..Default::default()
//! };
//! let saved = refund.save(&client).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A refund line item in a refund.
///
/// Represents a line item that is being refunded.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RefundLineItem {
    /// The unique identifier of the refund line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The quantity being refunded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i32>,

    /// The line item ID being refunded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_item_id: Option<u64>,

    /// The location ID for restocking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<u64>,

    /// The restock type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restock_type: Option<String>,

    /// The subtotal amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtotal: Option<String>,

    /// The total tax amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax: Option<String>,

    /// The subtotal set (multi-currency).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtotal_set: Option<serde_json::Value>,

    /// The total tax set (multi-currency).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tax_set: Option<serde_json::Value>,

    /// The line item (embedded when returned from API).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_item: Option<serde_json::Value>,
}

/// An order adjustment from a refund.
///
/// Represents adjustments made to the order totals due to a refund.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct OrderAdjustment {
    /// The unique identifier of the adjustment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The order ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<u64>,

    /// The refund ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_id: Option<u64>,

    /// The kind of adjustment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    /// The reason for the adjustment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// The adjustment amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// The tax amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_amount: Option<String>,

    /// The amount set (multi-currency).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_set: Option<serde_json::Value>,

    /// The tax amount set (multi-currency).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_amount_set: Option<serde_json::Value>,
}

/// Shipping refund information.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RefundShipping {
    /// Whether to fully refund shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_refund: Option<bool>,

    /// The amount to refund for shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
}

/// Input for a refund line item when calculating or creating a refund.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RefundLineItemInput {
    /// The line item ID to refund.
    pub line_item_id: u64,

    /// The quantity to refund.
    pub quantity: i32,

    /// The restock type (e.g., "return", "cancel", "no_restock").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restock_type: Option<String>,
}

/// A refund shipping line.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RefundShippingLine {
    /// The ID of the shipping line being refunded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// Whether this is a full refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_refund: Option<bool>,

    /// The refund amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// The amount set (multi-currency).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_set: Option<serde_json::Value>,
}

/// A refund resource for an order.
///
/// This is the full resource for direct refund operations. It is named
/// `RefundResource` to avoid conflicts with the embedded `Refund` struct
/// in the Order module.
///
/// Refunds are nested under orders and can only be created, not updated
/// or deleted after creation.
///
/// # Nested Resource
///
/// Refunds follow the nested path pattern under orders:
/// - All operations require `order_id` context
/// - Use `all_with_parent()` to list refunds under an order
/// - The `order_id` field is required for creating new refunds
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the refund
/// - `created_at` - When the refund was created
/// - `processed_at` - When the refund was processed
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `order_id` - The ID of the order this refund belongs to
/// - `note` - An optional note explaining the refund
/// - `notify` - Whether to notify the customer
/// - `shipping` - Shipping refund information
/// - `refund_line_items` - Line items to refund
/// - `transactions` - Transactions for the refund
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RefundResource {
    /// The unique identifier of the refund.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the order this refund belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<u64>,

    /// An optional note explaining the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// The ID of the user who processed the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,

    /// Whether to restock the items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restock: Option<bool>,

    /// Whether to notify the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify: Option<bool>,

    /// When the refund was processed.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub processed_at: Option<DateTime<Utc>>,

    /// When the refund was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// Duties associated with the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duties: Option<serde_json::Value>,

    /// Refunded duties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_duties: Option<serde_json::Value>,

    /// Line items included in the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_line_items: Option<serde_json::Value>,

    /// Refund shipping lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_shipping_lines: Option<Vec<RefundShippingLine>>,

    /// Transactions for the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transactions: Option<serde_json::Value>,

    /// Order adjustments from the refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_adjustments: Option<Vec<OrderAdjustment>>,

    /// Shipping refund information (for create).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<RefundShipping>,

    /// Currency for the refund (for create).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// The admin GraphQL API ID.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

/// Parameters for calculating a refund.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RefundCalculateParams {
    /// Shipping refund information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<RefundShipping>,

    /// Line items to refund.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refund_line_items: Option<Vec<RefundLineItemInput>>,

    /// Currency for the calculation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

impl RefundResource {
    /// Calculates a refund without creating it.
    ///
    /// Sends a POST request to `/admin/api/{version}/orders/{order_id}/refunds/calculate.json`
    /// to calculate refund amounts without actually creating the refund.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `order_id` - The ID of the order to calculate refund for
    /// * `params` - Parameters for the calculation (shipping, line_items, currency)
    ///
    /// # Returns
    ///
    /// A `RefundResource` populated with calculated amounts.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the order doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let calc_params = RefundCalculateParams {
    ///     shipping: Some(RefundShipping { full_refund: Some(true), ..Default::default() }),
    ///     refund_line_items: Some(vec![
    ///         RefundLineItemInput { line_item_id: 669751112, quantity: 1, restock_type: None },
    ///     ]),
    ///     ..Default::default()
    /// };
    /// let calculation = RefundResource::calculate(&client, 450789469, calc_params).await?;
    /// println!("Calculated refund: {:?}", calculation);
    /// ```
    pub async fn calculate(
        client: &RestClient,
        order_id: u64,
        params: RefundCalculateParams,
    ) -> Result<RefundResource, ResourceError> {
        let path = format!("orders/{order_id}/refunds/calculate");

        // Wrap params in refund key
        let body = serde_json::json!({
            "refund": params
        });

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&order_id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the response - Shopify returns the refund wrapped in "refund" key
        let refund: RefundResource = response
            .body
            .get("refund")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'refund' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize refund: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(refund)
    }

    /// Counts refunds under a specific order.
    ///
    /// Note: The Shopify API does not provide a count endpoint for refunds.
    /// This method is provided for API consistency but will return an error.
    ///
    /// Use `all_with_parent()` and count the results instead.
    pub async fn count_with_parent<ParentId: std::fmt::Display + Send>(
        _client: &RestClient,
        _parent_id_name: &str,
        _parent_id: ParentId,
        _params: Option<RefundCountParams>,
    ) -> Result<u64, ResourceError> {
        Err(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "count",
        })
    }
}

impl RestResource for RefundResource {
    type Id = u64;
    type FindParams = RefundFindParams;
    type AllParams = RefundListParams;
    type CountParams = RefundCountParams;

    const NAME: &'static str = "Refund";
    const PLURAL: &'static str = "refunds";

    /// Paths for the RefundResource.
    ///
    /// Refunds are NESTED under orders. All operations require `order_id`.
    /// Note: Refunds cannot be updated or deleted. No count endpoint available.
    const PATHS: &'static [ResourcePath] = &[
        // All paths require order_id
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["order_id", "id"],
            "orders/{order_id}/refunds/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["order_id"],
            "orders/{order_id}/refunds",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["order_id"],
            "orders/{order_id}/refunds",
        ),
        // No Count, Update, or Delete paths
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single refund.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RefundFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Whether to return the amount in shop currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_shop_currency: Option<bool>,
}

/// Parameters for listing refunds.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RefundListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Whether to return the amount in shop currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_shop_currency: Option<bool>,
}

/// Parameters for counting refunds.
///
/// Note: The Shopify API does not provide a count endpoint for refunds.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RefundCountParams {
    // No count endpoint available for refunds
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_refund_nested_paths_require_order_id() {
        // All paths should require order_id (nested under orders)

        // Find requires both order_id and id
        let find_path =
            get_path(RefundResource::PATHS, ResourceOperation::Find, &["order_id", "id"]);
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "orders/{order_id}/refunds/{id}"
        );

        // Find with only id should fail (no standalone path)
        let find_without_order = get_path(RefundResource::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_without_order.is_none());

        // All requires order_id
        let all_path = get_path(RefundResource::PATHS, ResourceOperation::All, &["order_id"]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "orders/{order_id}/refunds");

        // All without order_id should fail
        let all_without_order = get_path(RefundResource::PATHS, ResourceOperation::All, &[]);
        assert!(all_without_order.is_none());

        // Create requires order_id
        let create_path = get_path(RefundResource::PATHS, ResourceOperation::Create, &["order_id"]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "orders/{order_id}/refunds");

        // No Count path
        let count_path = get_path(RefundResource::PATHS, ResourceOperation::Count, &["order_id"]);
        assert!(count_path.is_none());

        // No Update path
        let update_path = get_path(
            RefundResource::PATHS,
            ResourceOperation::Update,
            &["order_id", "id"],
        );
        assert!(update_path.is_none());

        // No Delete path
        let delete_path = get_path(
            RefundResource::PATHS,
            ResourceOperation::Delete,
            &["order_id", "id"],
        );
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_refund_calculate_path_construction() {
        // Verify the calculate path format
        let order_id = 450789469u64;
        let expected_path = format!("orders/{order_id}/refunds/calculate");
        assert_eq!(expected_path, "orders/450789469/refunds/calculate");
    }

    #[test]
    fn test_refund_struct_serialization() {
        let refund = RefundResource {
            id: Some(123456),
            order_id: Some(450789469),
            note: Some("Customer requested refund".to_string()),
            user_id: Some(799407056),
            restock: Some(true),
            notify: Some(true),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            processed_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            admin_graphql_api_id: Some("gid://shopify/Refund/123456".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&refund).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["order_id"], 450789469);
        assert_eq!(parsed["note"], "Customer requested refund");
        assert_eq!(parsed["user_id"], 799407056);
        assert_eq!(parsed["restock"], true);
        assert_eq!(parsed["notify"], true);

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("processed_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_refund_deserialization_with_complex_nested_structures() {
        let json = r#"{
            "id": 123456,
            "order_id": 450789469,
            "note": "Customer requested refund",
            "user_id": 799407056,
            "restock": true,
            "processed_at": "2024-01-15T10:30:00Z",
            "created_at": "2024-01-15T10:30:00Z",
            "refund_line_items": [
                {
                    "id": 1,
                    "quantity": 1,
                    "line_item_id": 669751112,
                    "location_id": 655441491,
                    "restock_type": "return",
                    "subtotal": "199.99",
                    "total_tax": "15.00",
                    "line_item": {
                        "id": 669751112,
                        "title": "IPod Nano - 8GB"
                    }
                }
            ],
            "transactions": [
                {
                    "id": 389404469,
                    "order_id": 450789469,
                    "kind": "refund",
                    "amount": "214.99",
                    "status": "success"
                }
            ],
            "order_adjustments": [
                {
                    "id": 1,
                    "order_id": 450789469,
                    "refund_id": 123456,
                    "kind": "refund_discrepancy",
                    "reason": "Refund discrepancy",
                    "amount": "-0.01"
                }
            ],
            "refund_shipping_lines": [
                {
                    "id": 1,
                    "full_refund": true,
                    "amount": "5.00"
                }
            ],
            "admin_graphql_api_id": "gid://shopify/Refund/123456"
        }"#;

        let refund: RefundResource = serde_json::from_str(json).unwrap();

        assert_eq!(refund.id, Some(123456));
        assert_eq!(refund.order_id, Some(450789469));
        assert_eq!(refund.note, Some("Customer requested refund".to_string()));
        assert_eq!(refund.user_id, Some(799407056));
        assert_eq!(refund.restock, Some(true));
        assert!(refund.processed_at.is_some());
        assert!(refund.created_at.is_some());

        // Check nested structures
        assert!(refund.refund_line_items.is_some());
        assert!(refund.transactions.is_some());

        // Check order adjustments
        assert!(refund.order_adjustments.is_some());
        let adjustments = refund.order_adjustments.unwrap();
        assert_eq!(adjustments.len(), 1);
        assert_eq!(adjustments[0].kind, Some("refund_discrepancy".to_string()));

        // Check refund shipping lines
        assert!(refund.refund_shipping_lines.is_some());
        let shipping_lines = refund.refund_shipping_lines.unwrap();
        assert_eq!(shipping_lines.len(), 1);
        assert_eq!(shipping_lines[0].full_refund, Some(true));
        assert_eq!(shipping_lines[0].amount, Some("5.00".to_string()));
    }

    #[test]
    fn test_refund_calculate_params_serialization() {
        let params = RefundCalculateParams {
            shipping: Some(RefundShipping {
                full_refund: Some(true),
                amount: None,
            }),
            refund_line_items: Some(vec![
                RefundLineItemInput {
                    line_item_id: 669751112,
                    quantity: 1,
                    restock_type: Some("return".to_string()),
                },
            ]),
            currency: Some("USD".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert!(json["shipping"]["full_refund"].as_bool().unwrap());
        assert_eq!(json["refund_line_items"][0]["line_item_id"], 669751112);
        assert_eq!(json["refund_line_items"][0]["quantity"], 1);
        assert_eq!(json["refund_line_items"][0]["restock_type"], "return");
        assert_eq!(json["currency"], "USD");
    }

    #[test]
    fn test_refund_list_params_serialization() {
        let params = RefundListParams {
            limit: Some(50),
            fields: Some("id,note,created_at".to_string()),
            in_shop_currency: Some(true),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["fields"], "id,note,created_at");
        assert_eq!(json["in_shop_currency"], true);

        // Test empty params
        let empty_params = RefundListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_refund_get_id_returns_correct_value() {
        // Refund with ID
        let refund_with_id = RefundResource {
            id: Some(123456),
            order_id: Some(450789469),
            note: Some("Test refund".to_string()),
            ..Default::default()
        };
        assert_eq!(refund_with_id.get_id(), Some(123456));

        // Refund without ID (new refund)
        let refund_without_id = RefundResource {
            id: None,
            order_id: Some(450789469),
            note: Some("New refund".to_string()),
            ..Default::default()
        };
        assert_eq!(refund_without_id.get_id(), None);
    }

    #[test]
    fn test_refund_constants() {
        assert_eq!(RefundResource::NAME, "Refund");
        assert_eq!(RefundResource::PLURAL, "refunds");
    }

    #[test]
    fn test_refund_line_item_serialization() {
        let line_item = RefundLineItem {
            id: Some(1),
            quantity: Some(1),
            line_item_id: Some(669751112),
            location_id: Some(655441491),
            restock_type: Some("return".to_string()),
            subtotal: Some("199.99".to_string()),
            total_tax: Some("15.00".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&line_item).unwrap();

        assert_eq!(json["id"], 1);
        assert_eq!(json["quantity"], 1);
        assert_eq!(json["line_item_id"], 669751112);
        assert_eq!(json["location_id"], 655441491);
        assert_eq!(json["restock_type"], "return");
        assert_eq!(json["subtotal"], "199.99");
        assert_eq!(json["total_tax"], "15.00");
    }

    #[test]
    fn test_order_adjustment_serialization() {
        let adjustment = OrderAdjustment {
            id: Some(1),
            order_id: Some(450789469),
            refund_id: Some(123456),
            kind: Some("refund_discrepancy".to_string()),
            reason: Some("Refund discrepancy".to_string()),
            amount: Some("-0.01".to_string()),
            tax_amount: Some("0.00".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&adjustment).unwrap();

        assert_eq!(json["id"], 1);
        assert_eq!(json["order_id"], 450789469);
        assert_eq!(json["refund_id"], 123456);
        assert_eq!(json["kind"], "refund_discrepancy");
        assert_eq!(json["reason"], "Refund discrepancy");
        assert_eq!(json["amount"], "-0.01");
    }
}

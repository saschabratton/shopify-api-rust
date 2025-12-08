//! Transaction resource implementation.
//!
//! This module provides the Transaction resource, which represents a payment
//! transaction associated with an order in Shopify.
//!
//! # Nested Path Pattern
//!
//! Transactions are always accessed under an order:
//! - List: `/orders/{order_id}/transactions`
//! - Find: `/orders/{order_id}/transactions/{id}`
//! - Create: `/orders/{order_id}/transactions`
//! - Count: `/orders/{order_id}/transactions/count`
//!
//! Use `Transaction::all_with_parent()` to list transactions under a specific order.
//!
//! # Note
//!
//! Transactions cannot be updated or deleted. They represent immutable records
//! of payment events.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{Transaction, TransactionKind, TransactionListParams};
//!
//! // List transactions under a specific order
//! let transactions = Transaction::all_with_parent(&client, "order_id", 450789469, None).await?;
//! for txn in transactions.iter() {
//!     println!("Transaction: {} - {:?}", txn.amount.as_deref().unwrap_or("0"), txn.kind);
//! }
//!
//! // Create a capture transaction
//! let mut transaction = Transaction {
//!     order_id: Some(450789469),
//!     kind: Some(TransactionKind::Capture),
//!     amount: Some("199.99".to_string()),
//!     ..Default::default()
//! };
//! let saved = transaction.save(&client).await?;
//!
//! // Count transactions for an order
//! let count = Transaction::count_with_parent(&client, "order_id", 450789469, None).await?;
//! println!("Total transactions: {}", count);
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{
    build_path, get_path, ResourceError, ResourceOperation, ResourcePath, RestResource,
};
use crate::HttpMethod;

/// The kind of transaction.
///
/// Represents the type of payment operation performed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionKind {
    /// Initial authorization of payment.
    #[default]
    Authorization,
    /// Capture of previously authorized payment.
    Capture,
    /// Combined authorization and capture in one step.
    Sale,
    /// Cancellation of an authorization.
    Void,
    /// Return of funds to customer.
    Refund,
}

/// The status of a transaction.
///
/// Indicates whether the transaction succeeded or failed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    /// Transaction is pending completion.
    #[default]
    Pending,
    /// Transaction failed.
    Failure,
    /// Transaction completed successfully.
    Success,
    /// Transaction encountered an error.
    Error,
}

/// Payment details for a transaction.
///
/// Contains information about the payment method used.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PaymentDetails {
    /// The credit card bin number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_card_bin: Option<String>,

    /// AVS result code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avs_result_code: Option<String>,

    /// CVV result code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cvv_result_code: Option<String>,

    /// The credit card number (masked).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_card_number: Option<String>,

    /// The credit card company.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_card_company: Option<String>,

    /// The name on the credit card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_card_name: Option<String>,

    /// The credit card wallet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_card_wallet: Option<String>,

    /// The credit card expiration month.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_card_expiration_month: Option<i32>,

    /// The credit card expiration year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_card_expiration_year: Option<i32>,

    /// The buyer action info (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_action_info: Option<serde_json::Value>,
}

/// Currency exchange adjustment for multi-currency transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CurrencyExchangeAdjustment {
    /// The ID of the adjustment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The original amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_amount: Option<String>,

    /// The final amount after adjustment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_amount: Option<String>,

    /// The currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// The adjustment amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment: Option<String>,
}

/// A payment transaction for an order.
///
/// Transactions represent payment events such as authorizations, captures,
/// refunds, and voids. They are nested under orders and cannot be updated
/// or deleted after creation.
///
/// # Nested Resource
///
/// Transactions follow the nested path pattern under orders:
/// - All operations require `order_id` context
/// - Use `all_with_parent()` to list transactions under an order
/// - The `order_id` field is required for creating new transactions
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the transaction
/// - `created_at` - When the transaction was created
/// - `processed_at` - When the transaction was processed
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `order_id` - The ID of the order this transaction belongs to
/// - `kind` - The type of transaction (authorization, capture, sale, void, refund)
/// - `amount` - The transaction amount
/// - `currency` - The currency code
/// - `gateway` - The payment gateway used
/// - `parent_id` - The ID of the parent transaction (for captures/refunds)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Transaction {
    /// The unique identifier of the transaction.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the order this transaction belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<u64>,

    /// The kind of transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<TransactionKind>,

    /// The transaction amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// The status of the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TransactionStatus>,

    /// The payment gateway used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,

    /// A message describing the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// The error code if the transaction failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,

    /// The authorization code from the payment gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,

    /// When the authorization expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_expires_at: Option<DateTime<Utc>>,

    /// The currency code (e.g., "USD").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// Whether this is a test transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<bool>,

    /// The ID of the parent transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<u64>,

    /// The ID of the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<u64>,

    /// The ID of the device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<u64>,

    /// The ID of the user who processed the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,

    /// The source name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_name: Option<String>,

    /// When the transaction was processed.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub processed_at: Option<DateTime<Utc>>,

    /// When the transaction was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// The receipt from the payment gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<serde_json::Value>,

    /// Payment details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_details: Option<PaymentDetails>,

    /// Currency exchange adjustment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_exchange_adjustment: Option<CurrencyExchangeAdjustment>,

    /// Total unsettled set (complex structure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_unsettled_set: Option<serde_json::Value>,

    /// Whether this is a manual payment gateway.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_payment_gateway: Option<bool>,

    /// Amount rounding information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_rounding: Option<serde_json::Value>,

    /// Payments refund attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments_refund_attributes: Option<serde_json::Value>,

    /// Extended authorization attributes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_authorization_attributes: Option<serde_json::Value>,

    /// The admin GraphQL API ID for this transaction.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl Transaction {
    /// Counts transactions under a specific order.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `parent_id_name` - The name of the parent ID parameter (should be `order_id`)
    /// * `parent_id` - The order ID
    /// * `params` - Optional parameters for filtering
    ///
    /// # Returns
    ///
    /// The count of matching transactions as a `u64`.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no count path exists.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = Transaction::count_with_parent(&client, "order_id", 450789469, None).await?;
    /// println!("Transactions in order: {}", count);
    /// ```
    pub async fn count_with_parent<ParentId: std::fmt::Display + Send>(
        client: &RestClient,
        parent_id_name: &str,
        parent_id: ParentId,
        params: Option<TransactionCountParams>,
    ) -> Result<u64, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert(parent_id_name, parent_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Count, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "count",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Build query params
        let query = params
            .map(|p| {
                let value = serde_json::to_value(&p).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: 400,
                            message: format!("Failed to serialize params: {e}"),
                            error_reference: None,
                        },
                    ))
                })?;

                let mut query = HashMap::new();
                if let serde_json::Value::Object(map) = value {
                    for (key, val) in map {
                        match val {
                            serde_json::Value::String(s) => {
                                query.insert(key, s);
                            }
                            serde_json::Value::Number(n) => {
                                query.insert(key, n.to_string());
                            }
                            serde_json::Value::Bool(b) => {
                                query.insert(key, b.to_string());
                            }
                            _ => {}
                        }
                    }
                }
                Ok::<_, ResourceError>(query)
            })
            .transpose()?
            .filter(|q| !q.is_empty());

        let response = client.get(&url, query).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        // Extract count from response
        let count = response
            .body
            .get("count")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'count' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        Ok(count)
    }
}

impl RestResource for Transaction {
    type Id = u64;
    type FindParams = TransactionFindParams;
    type AllParams = TransactionListParams;
    type CountParams = TransactionCountParams;

    const NAME: &'static str = "Transaction";
    const PLURAL: &'static str = "transactions";

    /// Paths for the Transaction resource.
    ///
    /// Transactions are NESTED under orders. All operations require `order_id`.
    /// Note: Transactions cannot be updated or deleted.
    const PATHS: &'static [ResourcePath] = &[
        // All paths require order_id
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["order_id", "id"],
            "orders/{order_id}/transactions/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["order_id"],
            "orders/{order_id}/transactions",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["order_id"],
            "orders/{order_id}/transactions/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["order_id"],
            "orders/{order_id}/transactions",
        ),
        // No Update or Delete paths - transactions are immutable
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single transaction.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TransactionFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Whether to return the amount in shop currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_shop_currency: Option<bool>,
}

/// Parameters for listing transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TransactionListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return transactions after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Whether to return the amount in shop currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_shop_currency: Option<bool>,
}

/// Parameters for counting transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TransactionCountParams {
    // No specific count params for transactions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_transaction_kind_enum_serialization() {
        // Test serialization to snake_case
        assert_eq!(
            serde_json::to_string(&TransactionKind::Authorization).unwrap(),
            "\"authorization\""
        );
        assert_eq!(
            serde_json::to_string(&TransactionKind::Capture).unwrap(),
            "\"capture\""
        );
        assert_eq!(
            serde_json::to_string(&TransactionKind::Sale).unwrap(),
            "\"sale\""
        );
        assert_eq!(
            serde_json::to_string(&TransactionKind::Void).unwrap(),
            "\"void\""
        );
        assert_eq!(
            serde_json::to_string(&TransactionKind::Refund).unwrap(),
            "\"refund\""
        );

        // Test deserialization from snake_case
        let auth: TransactionKind = serde_json::from_str("\"authorization\"").unwrap();
        let capture: TransactionKind = serde_json::from_str("\"capture\"").unwrap();
        let sale: TransactionKind = serde_json::from_str("\"sale\"").unwrap();
        let void_txn: TransactionKind = serde_json::from_str("\"void\"").unwrap();
        let refund: TransactionKind = serde_json::from_str("\"refund\"").unwrap();

        assert_eq!(auth, TransactionKind::Authorization);
        assert_eq!(capture, TransactionKind::Capture);
        assert_eq!(sale, TransactionKind::Sale);
        assert_eq!(void_txn, TransactionKind::Void);
        assert_eq!(refund, TransactionKind::Refund);

        // Test default
        assert_eq!(TransactionKind::default(), TransactionKind::Authorization);
    }

    #[test]
    fn test_transaction_status_enum_serialization() {
        // Test serialization
        assert_eq!(
            serde_json::to_string(&TransactionStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&TransactionStatus::Failure).unwrap(),
            "\"failure\""
        );
        assert_eq!(
            serde_json::to_string(&TransactionStatus::Success).unwrap(),
            "\"success\""
        );
        assert_eq!(
            serde_json::to_string(&TransactionStatus::Error).unwrap(),
            "\"error\""
        );

        // Test deserialization
        let success: TransactionStatus = serde_json::from_str("\"success\"").unwrap();
        let failure: TransactionStatus = serde_json::from_str("\"failure\"").unwrap();

        assert_eq!(success, TransactionStatus::Success);
        assert_eq!(failure, TransactionStatus::Failure);

        // Test default
        assert_eq!(TransactionStatus::default(), TransactionStatus::Pending);
    }

    #[test]
    fn test_transaction_nested_paths_require_order_id() {
        // All paths should require order_id (nested under orders)

        // Find requires both order_id and id
        let find_path = get_path(Transaction::PATHS, ResourceOperation::Find, &["order_id", "id"]);
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "orders/{order_id}/transactions/{id}"
        );

        // Find with only id should fail (no standalone path)
        let find_without_order = get_path(Transaction::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_without_order.is_none());

        // All requires order_id
        let all_path = get_path(Transaction::PATHS, ResourceOperation::All, &["order_id"]);
        assert!(all_path.is_some());
        assert_eq!(
            all_path.unwrap().template,
            "orders/{order_id}/transactions"
        );

        // All without order_id should fail
        let all_without_order = get_path(Transaction::PATHS, ResourceOperation::All, &[]);
        assert!(all_without_order.is_none());

        // Count requires order_id
        let count_path = get_path(Transaction::PATHS, ResourceOperation::Count, &["order_id"]);
        assert!(count_path.is_some());
        assert_eq!(
            count_path.unwrap().template,
            "orders/{order_id}/transactions/count"
        );

        // Create requires order_id
        let create_path = get_path(Transaction::PATHS, ResourceOperation::Create, &["order_id"]);
        assert!(create_path.is_some());
        assert_eq!(
            create_path.unwrap().template,
            "orders/{order_id}/transactions"
        );

        // No Update path
        let update_path = get_path(
            Transaction::PATHS,
            ResourceOperation::Update,
            &["order_id", "id"],
        );
        assert!(update_path.is_none());

        // No Delete path
        let delete_path = get_path(
            Transaction::PATHS,
            ResourceOperation::Delete,
            &["order_id", "id"],
        );
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_transaction_struct_serialization() {
        let transaction = Transaction {
            id: Some(389404469),
            order_id: Some(450789469),
            kind: Some(TransactionKind::Capture),
            amount: Some("199.99".to_string()),
            status: Some(TransactionStatus::Success),
            gateway: Some("bogus".to_string()),
            message: Some("Transaction successful".to_string()),
            currency: Some("USD".to_string()),
            test: Some(true),
            parent_id: Some(389404468),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            admin_graphql_api_id: Some("gid://shopify/OrderTransaction/389404469".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&transaction).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["order_id"], 450789469);
        assert_eq!(parsed["kind"], "capture");
        assert_eq!(parsed["amount"], "199.99");
        assert_eq!(parsed["status"], "success");
        assert_eq!(parsed["gateway"], "bogus");
        assert_eq!(parsed["message"], "Transaction successful");
        assert_eq!(parsed["currency"], "USD");
        assert_eq!(parsed["test"], true);
        assert_eq!(parsed["parent_id"], 389404468);

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("processed_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_transaction_deserialization_from_api_response() {
        let json = r#"{
            "id": 389404469,
            "order_id": 450789469,
            "kind": "capture",
            "amount": "199.99",
            "status": "success",
            "gateway": "bogus",
            "message": "Bogus Gateway: Forced success",
            "error_code": null,
            "authorization": "ch_1234567890",
            "authorization_expires_at": "2024-01-22T10:30:00Z",
            "currency": "USD",
            "test": true,
            "parent_id": 389404468,
            "location_id": 655441491,
            "user_id": 799407056,
            "source_name": "web",
            "processed_at": "2024-01-15T10:30:00Z",
            "created_at": "2024-01-15T10:30:00Z",
            "payment_details": {
                "credit_card_bin": "424242",
                "credit_card_number": "xxxx xxxx xxxx 4242",
                "credit_card_company": "Visa",
                "credit_card_name": "John Doe"
            },
            "receipt": {
                "testcase": true,
                "authorization": "ch_1234567890"
            },
            "admin_graphql_api_id": "gid://shopify/OrderTransaction/389404469"
        }"#;

        let transaction: Transaction = serde_json::from_str(json).unwrap();

        assert_eq!(transaction.id, Some(389404469));
        assert_eq!(transaction.order_id, Some(450789469));
        assert_eq!(transaction.kind, Some(TransactionKind::Capture));
        assert_eq!(transaction.amount, Some("199.99".to_string()));
        assert_eq!(transaction.status, Some(TransactionStatus::Success));
        assert_eq!(transaction.gateway, Some("bogus".to_string()));
        assert_eq!(
            transaction.authorization,
            Some("ch_1234567890".to_string())
        );
        assert!(transaction.authorization_expires_at.is_some());
        assert_eq!(transaction.currency, Some("USD".to_string()));
        assert_eq!(transaction.test, Some(true));
        assert_eq!(transaction.parent_id, Some(389404468));
        assert_eq!(transaction.location_id, Some(655441491));
        assert_eq!(transaction.user_id, Some(799407056));
        assert!(transaction.processed_at.is_some());
        assert!(transaction.created_at.is_some());
        assert!(transaction.payment_details.is_some());
        assert!(transaction.receipt.is_some());

        let payment_details = transaction.payment_details.unwrap();
        assert_eq!(payment_details.credit_card_bin, Some("424242".to_string()));
        assert_eq!(payment_details.credit_card_company, Some("Visa".to_string()));
    }

    #[test]
    fn test_transaction_list_params_serialization() {
        let params = TransactionListParams {
            limit: Some(50),
            since_id: Some(100),
            fields: Some("id,kind,amount".to_string()),
            in_shop_currency: Some(true),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);
        assert_eq!(json["fields"], "id,kind,amount");
        assert_eq!(json["in_shop_currency"], true);

        // Test empty params
        let empty_params = TransactionListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_transaction_find_params_serialization() {
        let params = TransactionFindParams {
            fields: Some("id,kind,amount".to_string()),
            in_shop_currency: Some(true),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["fields"], "id,kind,amount");
        assert_eq!(json["in_shop_currency"], true);
    }

    #[test]
    fn test_transaction_get_id_returns_correct_value() {
        // Transaction with ID
        let txn_with_id = Transaction {
            id: Some(389404469),
            order_id: Some(450789469),
            kind: Some(TransactionKind::Capture),
            ..Default::default()
        };
        assert_eq!(txn_with_id.get_id(), Some(389404469));

        // Transaction without ID (new transaction)
        let txn_without_id = Transaction {
            id: None,
            order_id: Some(450789469),
            kind: Some(TransactionKind::Capture),
            ..Default::default()
        };
        assert_eq!(txn_without_id.get_id(), None);
    }

    #[test]
    fn test_transaction_constants() {
        assert_eq!(Transaction::NAME, "Transaction");
        assert_eq!(Transaction::PLURAL, "transactions");
    }
}

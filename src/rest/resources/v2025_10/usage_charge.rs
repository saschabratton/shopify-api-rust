//! UsageCharge resource implementation.
//!
//! This module provides the [`UsageCharge`] resource for managing usage-based
//! charges in Shopify apps. Usage charges are created under a parent
//! `RecurringApplicationCharge` that has a `capped_amount` set.
//!
//! # Nested Resource
//!
//! UsageCharges are nested under RecurringApplicationCharges:
//! - `GET /recurring_application_charges/{charge_id}/usage_charges.json`
//! - `POST /recurring_application_charges/{charge_id}/usage_charges.json`
//! - `GET /recurring_application_charges/{charge_id}/usage_charges/{id}.json`
//!
//! Note: Usage charges cannot be updated or deleted after creation.
//!
//! # Usage-Based Billing
//!
//! To implement usage-based billing:
//!
//! 1. Create a `RecurringApplicationCharge` with a `capped_amount`
//! 2. As the merchant uses resources, create `UsageCharge` records
//! 3. Charges accumulate up to the `capped_amount` per billing period
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{UsageCharge, UsageChargeListParams};
//!
//! // Create a usage charge under a recurring charge
//! let usage = UsageCharge {
//!     recurring_application_charge_id: Some(455696195),
//!     description: Some("100 emails sent".to_string()),
//!     price: Some("1.00".to_string()),
//!     ..Default::default()
//! };
//! let saved = usage.save(&client).await?;
//!
//! // List all usage charges for a recurring charge
//! let usages = UsageCharge::all_with_parent(
//!     &client,
//!     "recurring_application_charge_id",
//!     455696195,
//!     None
//! ).await?;
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{
    build_path, get_path, ResourceError, ResourceOperation, ResourcePath, ResourceResponse,
    RestResource,
};
use crate::HttpMethod;

use super::common::ChargeCurrency;

/// A usage-based charge under a recurring application charge.
///
/// Usage charges allow apps to bill merchants based on resource consumption.
/// They require a parent `RecurringApplicationCharge` with a `capped_amount`.
///
/// # Nested Resource
///
/// This is a nested resource under `RecurringApplicationCharge`. All operations
/// require the parent `recurring_application_charge_id`.
///
/// Use `UsageCharge::all_with_parent()` to list charges under a specific
/// recurring charge.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the usage charge
/// - `currency` - The currency object with the currency code
/// - `created_at` - When the charge was created
/// - `updated_at` - When the charge was last updated
///
/// ## Writable Fields
/// - `recurring_application_charge_id` - The parent charge ID (required)
/// - `description` - Description shown to merchant
/// - `price` - The price of this usage
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct UsageCharge {
    /// The unique identifier of the usage charge.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the parent recurring application charge.
    /// Required for creating new usage charges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurring_application_charge_id: Option<u64>,

    /// The description of the usage charge.
    /// Displayed to the merchant on their invoice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The price of the usage charge.
    /// Must be a string representing the monetary amount (e.g., "1.00").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The currency information for the charge.
    /// Read-only field containing the currency code.
    #[serde(skip_serializing)]
    pub currency: Option<ChargeCurrency>,

    /// When the charge was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the charge was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
}

impl UsageCharge {
    /// Counts usage charges under a specific recurring application charge.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `recurring_application_charge_id` - The parent charge ID
    /// * `params` - Optional parameters for filtering
    ///
    /// # Returns
    ///
    /// The count of matching usage charges as a `u64`.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no count path exists.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = UsageCharge::count_with_parent(&client, 455696195, None).await?;
    /// println!("Total usage charges: {}", count);
    /// ```
    pub async fn count_with_parent(
        client: &RestClient,
        recurring_application_charge_id: u64,
        _params: Option<()>,
    ) -> Result<u64, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert(
            "recurring_application_charge_id",
            recurring_application_charge_id.to_string(),
        );

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Count, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "count",
            },
        )?;

        let url = build_path(path.template, &ids);
        let response = client.get(&url, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

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

    /// Finds a single usage charge by ID under a recurring application charge.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `recurring_application_charge_id` - The parent charge ID
    /// * `id` - The usage charge ID to find
    /// * `params` - Optional parameters for the request
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the usage charge doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let usage = UsageCharge::find_with_parent(&client, 455696195, 123, None).await?;
    /// ```
    pub async fn find_with_parent(
        client: &RestClient,
        recurring_application_charge_id: u64,
        id: u64,
        _params: Option<UsageChargeFindParams>,
    ) -> Result<ResourceResponse<Self>, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert(
            "recurring_application_charge_id",
            recurring_application_charge_id.to_string(),
        );
        ids.insert("id", id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Find, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "find",
            },
        )?;

        let url = build_path(path.template, &ids);
        let response = client.get(&url, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }
}

impl RestResource for UsageCharge {
    type Id = u64;
    type FindParams = UsageChargeFindParams;
    type AllParams = UsageChargeListParams;
    type CountParams = ();

    const NAME: &'static str = "UsageCharge";
    const PLURAL: &'static str = "usage_charges";

    /// Paths for the UsageCharge resource.
    ///
    /// All paths require `recurring_application_charge_id` as UsageCharges
    /// are nested under RecurringApplicationCharges.
    ///
    /// Note: No Update or Delete paths - usage charges cannot be modified.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["recurring_application_charge_id", "id"],
            "recurring_application_charges/{recurring_application_charge_id}/usage_charges/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["recurring_application_charge_id"],
            "recurring_application_charges/{recurring_application_charge_id}/usage_charges",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["recurring_application_charge_id"],
            "recurring_application_charges/{recurring_application_charge_id}/usage_charges",
        ),
        // Note: Count path not officially documented but following pattern
        // No Update or Delete paths - usage charges cannot be modified after creation
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single usage charge.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct UsageChargeFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing usage charges.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct UsageChargeListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return charges after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_usage_charge_serialization() {
        let charge = UsageCharge {
            id: Some(12345),
            recurring_application_charge_id: Some(455696195),
            description: Some("100 emails sent".to_string()),
            price: Some("1.00".to_string()),
            currency: Some(ChargeCurrency::new("USD")),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:35:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_string(&charge).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["recurring_application_charge_id"], 455696195);
        assert_eq!(parsed["description"], "100 emails sent");
        assert_eq!(parsed["price"], "1.00");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("currency").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_usage_charge_deserialization() {
        let json = r#"{
            "id": 1034618207,
            "recurring_application_charge_id": 455696195,
            "description": "Super Mega Plan 1000 emails",
            "price": "1.00",
            "currency": {
                "currency": "USD"
            },
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:35:00Z"
        }"#;

        let charge: UsageCharge = serde_json::from_str(json).unwrap();

        assert_eq!(charge.id, Some(1034618207));
        assert_eq!(charge.recurring_application_charge_id, Some(455696195));
        assert_eq!(
            charge.description,
            Some("Super Mega Plan 1000 emails".to_string())
        );
        assert_eq!(charge.price, Some("1.00".to_string()));
        assert_eq!(charge.currency.as_ref().unwrap().code(), Some("USD"));
        assert!(charge.created_at.is_some());
        assert!(charge.updated_at.is_some());
    }

    #[test]
    fn test_usage_charge_nested_paths() {
        // All paths should require recurring_application_charge_id

        // Find requires both recurring_application_charge_id and id
        let find_path = get_path(
            UsageCharge::PATHS,
            ResourceOperation::Find,
            &["recurring_application_charge_id", "id"],
        );
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "recurring_application_charges/{recurring_application_charge_id}/usage_charges/{id}"
        );

        // Find with only id should fail (no standalone path)
        let find_without_parent = get_path(UsageCharge::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_without_parent.is_none());

        // All requires recurring_application_charge_id
        let all_path = get_path(
            UsageCharge::PATHS,
            ResourceOperation::All,
            &["recurring_application_charge_id"],
        );
        assert!(all_path.is_some());
        assert_eq!(
            all_path.unwrap().template,
            "recurring_application_charges/{recurring_application_charge_id}/usage_charges"
        );

        // All without parent should fail
        let all_without_parent = get_path(UsageCharge::PATHS, ResourceOperation::All, &[]);
        assert!(all_without_parent.is_none());

        // Create requires recurring_application_charge_id
        let create_path = get_path(
            UsageCharge::PATHS,
            ResourceOperation::Create,
            &["recurring_application_charge_id"],
        );
        assert!(create_path.is_some());
        assert_eq!(
            create_path.unwrap().template,
            "recurring_application_charges/{recurring_application_charge_id}/usage_charges"
        );
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // No Update path
        let update_path = get_path(
            UsageCharge::PATHS,
            ResourceOperation::Update,
            &["recurring_application_charge_id", "id"],
        );
        assert!(update_path.is_none());

        // No Delete path
        let delete_path = get_path(
            UsageCharge::PATHS,
            ResourceOperation::Delete,
            &["recurring_application_charge_id", "id"],
        );
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_usage_charge_list_params() {
        let params = UsageChargeListParams {
            limit: Some(50),
            since_id: Some(100),
            fields: Some("id,description,price".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);
        assert_eq!(json["fields"], "id,description,price");

        // Empty params should serialize to empty object
        let empty_params = UsageChargeListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_usage_charge_constants() {
        assert_eq!(UsageCharge::NAME, "UsageCharge");
        assert_eq!(UsageCharge::PLURAL, "usage_charges");
    }

    #[test]
    fn test_usage_charge_get_id() {
        let charge_with_id = UsageCharge {
            id: Some(12345),
            ..Default::default()
        };
        assert_eq!(charge_with_id.get_id(), Some(12345));

        let charge_without_id = UsageCharge::default();
        assert_eq!(charge_without_id.get_id(), None);
    }

    #[test]
    fn test_usage_charge_with_currency_nested_object() {
        // Test that currency deserializes correctly as a nested object
        let json = r#"{
            "id": 123,
            "recurring_application_charge_id": 456,
            "description": "Test charge",
            "price": "5.00",
            "currency": {
                "currency": "EUR"
            }
        }"#;

        let charge: UsageCharge = serde_json::from_str(json).unwrap();
        assert!(charge.currency.is_some());
        let currency = charge.currency.unwrap();
        assert_eq!(currency.code(), Some("EUR"));
    }
}

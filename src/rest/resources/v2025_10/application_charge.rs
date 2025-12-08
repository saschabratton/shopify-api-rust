//! ApplicationCharge resource implementation.
//!
//! This module provides the [`ApplicationCharge`] resource for managing one-time
//! charges in Shopify apps. Application charges are used to bill merchants
//! for one-time purchases within an app.
//!
//! # API Endpoints
//!
//! - `GET /application_charges.json` - List all application charges
//! - `POST /application_charges.json` - Create a new application charge
//! - `GET /application_charges/{id}.json` - Retrieve a single application charge
//!
//! Note: Application charges cannot be updated or deleted after creation.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{ApplicationCharge, ApplicationChargeListParams};
//!
//! // Create a new application charge
//! let charge = ApplicationCharge {
//!     name: Some("Super Widget".to_string()),
//!     price: Some("9.99".to_string()),
//!     return_url: Some("https://myapp.com/charge-callback".to_string()),
//!     test: Some(true), // Test charges don't actually bill
//!     ..Default::default()
//! };
//! let saved = charge.save(&client).await?;
//!
//! // Redirect merchant to the confirmation_url for approval
//! if let Some(url) = saved.confirmation_url.as_ref() {
//!     println!("Redirect merchant to: {}", url);
//! }
//!
//! // Check charge status
//! if saved.is_active() {
//!     println!("Charge was accepted!");
//! }
//!
//! // List all charges
//! let charges = ApplicationCharge::all(&client, None).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::{ChargeCurrency, ChargeStatus};

/// A one-time application charge.
///
/// Application charges allow apps to bill merchants for one-time purchases.
/// After creating a charge, redirect the merchant to the `confirmation_url`
/// for them to approve the charge.
///
/// # Charge Lifecycle
///
/// 1. App creates a charge via POST
/// 2. App redirects merchant to `confirmation_url`
/// 3. Merchant approves or declines the charge
/// 4. Merchant is redirected back to the app's `return_url`
/// 5. App checks the charge status
///
/// # Test Charges
///
/// Set `test: true` to create test charges that don't actually bill.
/// Test charges are automatically approved when created on development stores.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the charge
/// - `confirmation_url` - The URL to redirect merchant for approval
/// - `status` - The current status of the charge
/// - `currency` - The currency object with the currency code
/// - `created_at` - When the charge was created
/// - `updated_at` - When the charge was last updated
///
/// ## Writable Fields
/// - `name` - The name of the charge (displayed to merchant)
/// - `price` - The price of the charge
/// - `return_url` - The URL to redirect after merchant action
/// - `test` - Whether this is a test charge
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ApplicationCharge {
    /// The unique identifier of the application charge.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The name of the application charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The price of the application charge.
    /// Must be a string representing the monetary amount (e.g., "9.99").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The current status of the charge.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub status: Option<ChargeStatus>,

    /// Whether this is a test charge.
    /// Test charges don't actually bill the merchant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<bool>,

    /// The URL to redirect the merchant after they approve/decline the charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<String>,

    /// The URL where the merchant can approve/decline the charge.
    /// Read-only field, populated after creating the charge.
    #[serde(skip_serializing)]
    pub confirmation_url: Option<String>,

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

impl ApplicationCharge {
    /// Returns `true` if the charge is active (approved and billed).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if charge.is_active() {
    ///     println!("Charge has been paid!");
    /// }
    /// ```
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status.as_ref().map_or(false, ChargeStatus::is_active)
    }

    /// Returns `true` if the charge is pending merchant approval.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if charge.is_pending() {
    ///     println!("Waiting for merchant approval");
    /// }
    /// ```
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status.as_ref().map_or(false, ChargeStatus::is_pending)
    }

    /// Returns `true` if this is a test charge.
    ///
    /// Test charges don't actually bill the merchant and are
    /// automatically approved on development stores.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if charge.is_test() {
    ///     println!("This is a test charge");
    /// }
    /// ```
    #[must_use]
    pub fn is_test(&self) -> bool {
        self.test.unwrap_or(false)
    }
}

impl RestResource for ApplicationCharge {
    type Id = u64;
    type FindParams = ApplicationChargeFindParams;
    type AllParams = ApplicationChargeListParams;
    type CountParams = ();

    const NAME: &'static str = "ApplicationCharge";
    const PLURAL: &'static str = "application_charges";

    /// Paths for the ApplicationCharge resource.
    ///
    /// Note: ApplicationCharge has limited CRUD - no Update or Delete operations.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "application_charges/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "application_charges",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "application_charges",
        ),
        // No Update or Delete paths - charges cannot be modified after creation
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single application charge.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ApplicationChargeFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing application charges.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ApplicationChargeListParams {
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
    fn test_application_charge_serialization() {
        let charge = ApplicationCharge {
            id: Some(12345),
            name: Some("Pro Widget".to_string()),
            price: Some("19.99".to_string()),
            status: Some(ChargeStatus::Pending),
            test: Some(true),
            return_url: Some("https://myapp.com/callback".to_string()),
            confirmation_url: Some("https://shop.myshopify.com/confirm".to_string()),
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
        assert_eq!(parsed["name"], "Pro Widget");
        assert_eq!(parsed["price"], "19.99");
        assert_eq!(parsed["test"], true);
        assert_eq!(parsed["return_url"], "https://myapp.com/callback");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("status").is_none());
        assert!(parsed.get("confirmation_url").is_none());
        assert!(parsed.get("currency").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_application_charge_deserialization() {
        let json = r#"{
            "id": 675931192,
            "name": "Super Duper Expensive action",
            "price": "100.00",
            "status": "active",
            "test": false,
            "return_url": "https://super-duper.shopifyapps.com/",
            "confirmation_url": "https://jsmith.myshopify.com/admin/charges/675931192/confirm_application_charge",
            "currency": {
                "currency": "USD"
            },
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:35:00Z"
        }"#;

        let charge: ApplicationCharge = serde_json::from_str(json).unwrap();

        assert_eq!(charge.id, Some(675931192));
        assert_eq!(
            charge.name,
            Some("Super Duper Expensive action".to_string())
        );
        assert_eq!(charge.price, Some("100.00".to_string()));
        assert_eq!(charge.status, Some(ChargeStatus::Active));
        assert_eq!(charge.test, Some(false));
        assert_eq!(
            charge.return_url,
            Some("https://super-duper.shopifyapps.com/".to_string())
        );
        assert!(charge.confirmation_url.is_some());
        assert_eq!(charge.currency.as_ref().unwrap().code(), Some("USD"));
        assert!(charge.created_at.is_some());
        assert!(charge.updated_at.is_some());
    }

    #[test]
    fn test_application_charge_convenience_methods() {
        // Test is_active
        let active_charge = ApplicationCharge {
            status: Some(ChargeStatus::Active),
            ..Default::default()
        };
        assert!(active_charge.is_active());
        assert!(!active_charge.is_pending());

        // Test is_pending
        let pending_charge = ApplicationCharge {
            status: Some(ChargeStatus::Pending),
            ..Default::default()
        };
        assert!(pending_charge.is_pending());
        assert!(!pending_charge.is_active());

        // Test is_test
        let test_charge = ApplicationCharge {
            test: Some(true),
            ..Default::default()
        };
        assert!(test_charge.is_test());

        let non_test_charge = ApplicationCharge {
            test: Some(false),
            ..Default::default()
        };
        assert!(!non_test_charge.is_test());

        // Test default (no test field)
        let default_charge = ApplicationCharge::default();
        assert!(!default_charge.is_test());
        assert!(!default_charge.is_active());
        assert!(!default_charge.is_pending());
    }

    #[test]
    fn test_application_charge_paths() {
        // Find path
        let find_path = get_path(
            ApplicationCharge::PATHS,
            ResourceOperation::Find,
            &["id"],
        );
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "application_charges/{id}");

        // All path
        let all_path = get_path(ApplicationCharge::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "application_charges");

        // Create path
        let create_path = get_path(ApplicationCharge::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "application_charges");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // No Update path
        let update_path = get_path(
            ApplicationCharge::PATHS,
            ResourceOperation::Update,
            &["id"],
        );
        assert!(update_path.is_none());

        // No Delete path
        let delete_path = get_path(
            ApplicationCharge::PATHS,
            ResourceOperation::Delete,
            &["id"],
        );
        assert!(delete_path.is_none());

        // No Count path
        let count_path = get_path(ApplicationCharge::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());
    }

    #[test]
    fn test_application_charge_list_params() {
        let params = ApplicationChargeListParams {
            limit: Some(50),
            since_id: Some(100),
            fields: Some("id,name,price".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);
        assert_eq!(json["fields"], "id,name,price");

        // Empty params should serialize to empty object
        let empty_params = ApplicationChargeListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_application_charge_constants() {
        assert_eq!(ApplicationCharge::NAME, "ApplicationCharge");
        assert_eq!(ApplicationCharge::PLURAL, "application_charges");
    }

    #[test]
    fn test_application_charge_get_id() {
        let charge_with_id = ApplicationCharge {
            id: Some(12345),
            ..Default::default()
        };
        assert_eq!(charge_with_id.get_id(), Some(12345));

        let charge_without_id = ApplicationCharge::default();
        assert_eq!(charge_without_id.get_id(), None);
    }
}

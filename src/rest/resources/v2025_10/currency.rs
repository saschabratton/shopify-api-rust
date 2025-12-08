//! Currency resource implementation.
//!
//! This module provides the [`Currency`] resource for retrieving the currencies
//! enabled on a shop.
//!
//! # Read-Only Resource
//!
//! Currencies implement [`ReadOnlyResource`](crate::rest::ReadOnlyResource) - they
//! can only be listed, not created, updated, or deleted through the API.
//! Currency settings are managed through the Shopify admin.
//!
//! # No ID Field
//!
//! Unlike most resources, Currency does not have a numeric ID field.
//! Currencies are identified by their currency code (e.g., "USD", "CAD").
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::Currency;
//!
//! // List all enabled currencies
//! let currencies = Currency::all(&client, None).await?;
//! for currency in currencies.iter() {
//!     println!("{} - rate updated: {:?}",
//!         currency.currency.as_deref().unwrap_or(""),
//!         currency.rate_updated_at);
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::rest::{ReadOnlyResource, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A currency enabled on a Shopify store.
///
/// Currencies represent the different currencies a store can accept.
/// They are read-only through the API.
///
/// # Read-Only Resource
///
/// This resource implements [`ReadOnlyResource`] - only GET operations are
/// available. Currency settings are managed through the Shopify admin.
///
/// # No ID Field
///
/// This resource does not have a numeric `id` field. Currencies are
/// identified by their currency code.
///
/// # Fields
///
/// All fields are read-only:
/// - `currency` - The three-letter ISO 4217 currency code
/// - `rate_updated_at` - When the exchange rate was last updated
/// - `enabled` - Whether the currency is enabled
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Currency {
    /// The three-letter ISO 4217 currency code (e.g., "USD", "CAD", "EUR").
    #[serde(skip_serializing)]
    pub currency: Option<String>,

    /// When the exchange rate was last updated.
    #[serde(skip_serializing)]
    pub rate_updated_at: Option<String>,

    /// Whether this currency is enabled on the shop.
    #[serde(skip_serializing)]
    pub enabled: Option<bool>,
}

impl RestResource for Currency {
    type Id = String;
    type FindParams = ();
    type AllParams = ();
    type CountParams = ();

    const NAME: &'static str = "Currency";
    const PLURAL: &'static str = "currencies";

    /// Paths for the Currency resource.
    ///
    /// Only list operation - no Find, Count, Create, Update, or Delete.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "currencies"),
        // Note: No Find by ID - currencies identified by code
        // Note: No Count endpoint
        // Note: No Create, Update, or Delete - read-only resource
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.currency.clone()
    }
}

impl ReadOnlyResource for Currency {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ReadOnlyResource, ResourceOperation, RestResource};

    #[test]
    fn test_currency_implements_read_only_resource() {
        fn assert_read_only<T: ReadOnlyResource>() {}
        assert_read_only::<Currency>();
    }

    #[test]
    fn test_currency_deserialization() {
        let json = r#"{
            "currency": "CAD",
            "rate_updated_at": "2024-01-15T10:30:00-05:00",
            "enabled": true
        }"#;

        let currency: Currency = serde_json::from_str(json).unwrap();

        assert_eq!(currency.currency, Some("CAD".to_string()));
        assert_eq!(
            currency.rate_updated_at,
            Some("2024-01-15T10:30:00-05:00".to_string())
        );
        assert_eq!(currency.enabled, Some(true));
    }

    #[test]
    fn test_currency_list_only_paths() {
        // All (list)
        let all_path = get_path(Currency::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "currencies");

        // No Find path
        let find_path = get_path(Currency::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_none());

        // No Count path
        let count_path = get_path(Currency::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());

        // No Create path
        let create_path = get_path(Currency::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_none());

        // No Update path
        let update_path = get_path(Currency::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());

        // No Delete path
        let delete_path = get_path(Currency::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_currency_constants() {
        assert_eq!(Currency::NAME, "Currency");
        assert_eq!(Currency::PLURAL, "currencies");
    }

    #[test]
    fn test_currency_get_id_returns_code() {
        let currency_with_code = Currency {
            currency: Some("USD".to_string()),
            rate_updated_at: None,
            enabled: Some(true),
        };
        assert_eq!(currency_with_code.get_id(), Some("USD".to_string()));

        let currency_without_code = Currency::default();
        assert_eq!(currency_without_code.get_id(), None);
    }

    #[test]
    fn test_currency_all_fields_are_read_only() {
        // All fields should be skipped during serialization
        let currency = Currency {
            currency: Some("EUR".to_string()),
            rate_updated_at: Some("2024-01-15T10:30:00-05:00".to_string()),
            enabled: Some(true),
        };

        let json = serde_json::to_value(&currency).unwrap();
        // All fields should be omitted (empty object)
        assert_eq!(json, serde_json::json!({}));
    }
}

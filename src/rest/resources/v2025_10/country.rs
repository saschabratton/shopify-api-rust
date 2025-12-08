//! Country resource implementation.
//!
//! This module provides the [`Country`] resource for managing countries
//! that a store ships to, including their tax rates and embedded provinces.
//!
//! # Embedded Provinces
//!
//! Countries contain an embedded `Vec<Province>` with all provinces/states
//! for that country. Use the separate `Province` resource for individual
//! province operations.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{Country, CountryListParams};
//!
//! // List all shipping countries
//! let countries = Country::all(&client, None).await?;
//! for country in countries.iter() {
//!     println!("{} ({})", country.name.as_deref().unwrap_or(""),
//!         country.code.as_deref().unwrap_or(""));
//!     if let Some(provinces) = &country.provinces {
//!         println!("  Provinces: {}", provinces.len());
//!     }
//! }
//!
//! // Create a shipping country
//! let country = Country {
//!     code: Some("CA".to_string()),
//!     tax: Some("0.13".to_string()),
//!     ..Default::default()
//! };
//! let saved = country.save(&client).await?;
//! ```

use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::Province;

/// A country that a store ships to.
///
/// Countries define shipping destinations with tax rates. Each country
/// can contain multiple provinces/states with their own tax rates.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `name` - The full country name (derived from code)
/// - `provinces` - Embedded array of provinces
///
/// ## Writable Fields
/// - `code` - The two-letter ISO 3166-1 alpha-2 country code
/// - `tax` - The national tax rate as a decimal
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Country {
    /// The unique identifier of the country.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The full name of the country.
    /// Read-only field - derived from the country code.
    #[serde(skip_serializing)]
    pub name: Option<String>,

    /// The two-letter ISO 3166-1 alpha-2 country code (e.g., "US", "CA").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The national tax rate as a decimal string (e.g., "0.13" for 13%).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax: Option<String>,

    /// The provinces/states within this country.
    /// Read-only field - use Province resource for modifications.
    #[serde(skip_serializing)]
    pub provinces: Option<Vec<Province>>,
}

impl RestResource for Country {
    type Id = u64;
    type FindParams = CountryFindParams;
    type AllParams = CountryListParams;
    type CountParams = CountryCountParams;

    const NAME: &'static str = "Country";
    const PLURAL: &'static str = "countries";

    /// Paths for the Country resource.
    ///
    /// Full CRUD operations are supported.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "countries/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "countries"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "countries/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "countries",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "countries/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "countries/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single country.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CountryFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing countries.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CountryListParams {
    /// Return countries after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting countries.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CountryCountParams {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_country_serialization() {
        let country = Country {
            id: Some(879921427),
            name: Some("Canada".to_string()),
            code: Some("CA".to_string()),
            tax: Some("0.05".to_string()),
            provinces: Some(vec![]),
        };

        let json = serde_json::to_string(&country).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["code"], "CA");
        assert_eq!(parsed["tax"], "0.05");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("name").is_none());
        assert!(parsed.get("provinces").is_none());
    }

    #[test]
    fn test_country_deserialization_with_provinces() {
        let json = r#"{
            "id": 879921427,
            "name": "Canada",
            "code": "CA",
            "tax": "0.05",
            "provinces": [
                {
                    "id": 224293623,
                    "country_id": 879921427,
                    "name": "Ontario",
                    "code": "ON",
                    "tax": 0.08,
                    "tax_name": "HST",
                    "tax_type": "compounded",
                    "tax_percentage": 8.0
                },
                {
                    "id": 702530425,
                    "country_id": 879921427,
                    "name": "Quebec",
                    "code": "QC",
                    "tax": 0.0975,
                    "tax_name": "QST",
                    "tax_type": "compounded",
                    "tax_percentage": 9.75
                }
            ]
        }"#;

        let country: Country = serde_json::from_str(json).unwrap();

        assert_eq!(country.id, Some(879921427));
        assert_eq!(country.name, Some("Canada".to_string()));
        assert_eq!(country.code, Some("CA".to_string()));
        assert_eq!(country.tax, Some("0.05".to_string()));
        assert!(country.provinces.is_some());
        let provinces = country.provinces.unwrap();
        assert_eq!(provinces.len(), 2);
        assert_eq!(provinces[0].name, Some("Ontario".to_string()));
        assert_eq!(provinces[1].name, Some("Quebec".to_string()));
    }

    #[test]
    fn test_country_full_crud_paths() {
        // Find by ID
        let find_path = get_path(Country::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "countries/{id}");

        // List all
        let all_path = get_path(Country::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "countries");

        // Count
        let count_path = get_path(Country::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "countries/count");

        // Create
        let create_path = get_path(Country::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "countries");

        // Update
        let update_path = get_path(Country::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "countries/{id}");

        // Delete
        let delete_path = get_path(Country::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "countries/{id}");
    }

    #[test]
    fn test_country_constants() {
        assert_eq!(Country::NAME, "Country");
        assert_eq!(Country::PLURAL, "countries");
    }

    #[test]
    fn test_country_get_id() {
        let country_with_id = Country {
            id: Some(879921427),
            code: Some("CA".to_string()),
            ..Default::default()
        };
        assert_eq!(country_with_id.get_id(), Some(879921427));

        let country_without_id = Country::default();
        assert_eq!(country_without_id.get_id(), None);
    }
}

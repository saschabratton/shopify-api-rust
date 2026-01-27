//! Province resource implementation.
//!
//! This module provides the [`Province`] resource for managing provinces/states
//! within a country. Provinces are nested under countries and have limited
//! operations.
//!
//! # Nested Resource
//!
//! Provinces are nested under Countries:
//! - `GET /countries/{country_id}/provinces.json`
//! - `GET /countries/{country_id}/provinces/{id}.json`
//! - `PUT /countries/{country_id}/provinces/{id}.json`
//! - `GET /countries/{country_id}/provinces/count.json`
//!
//! Note: Provinces cannot be created or deleted directly. They are managed
//! through the Country resource.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{Province, ProvinceListParams};
//!
//! // List provinces for a country
//! let provinces = Province::all_with_parent(&client, "country_id", 879921427, None).await?;
//! for province in provinces.iter() {
//!     println!("{} ({}) - tax: {:?}",
//!         province.name.as_deref().unwrap_or(""),
//!         province.code.as_deref().unwrap_or(""),
//!         province.tax);
//! }
//!
//! // Update a province's tax
//! let mut province = Province::find_with_parent(&client, 879921427, 224293623, None).await?.into_inner();
//! province.tax = Some(0.13);
//! // Note: Updates require saving through the nested path
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{
    build_path, get_path, ResourceError, ResourceOperation, ResourcePath, ResourceResponse,
    RestResource,
};
use crate::HttpMethod;

/// A province or state within a country.
///
/// Provinces define regional tax rates within a country. They are nested
/// resources under Country and have limited operations (no create/delete).
///
/// # Nested Resource
///
/// This is a nested resource under `Country`. All operations require the
/// parent `country_id`.
///
/// # Limited Operations
///
/// - **No Create**: Provinces are created when the country is created
/// - **No Delete**: Provinces can only be removed by deleting the country
/// - **Update**: Tax rates can be modified
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `country_id` - The parent country ID
/// - `name` - The province name (derived from code)
/// - `shipping_zone_id` - The associated shipping zone
///
/// ## Writable Fields
/// - `code` - The province code (e.g., "ON", "CA")
/// - `tax` - The provincial tax rate as a decimal
/// - `tax_name` - The name of the tax (e.g., "HST", "PST")
/// - `tax_type` - The tax calculation type
/// - `tax_percentage` - The tax rate as a percentage
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Province {
    /// The unique identifier of the province.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the parent country.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub country_id: Option<u64>,

    /// The full name of the province.
    /// Read-only field - derived from the province code.
    #[serde(skip_serializing)]
    pub name: Option<String>,

    /// The province code (e.g., "ON" for Ontario, "CA" for California).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The provincial tax rate as a decimal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax: Option<f64>,

    /// The name of the tax (e.g., "HST", "PST", "Sales Tax").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_name: Option<String>,

    /// The tax calculation type: "normal", "compounded", or "harmonized".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_type: Option<String>,

    /// The tax rate as a percentage (e.g., 13.0 for 13%).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_percentage: Option<f64>,

    /// The ID of the shipping zone this province belongs to.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub shipping_zone_id: Option<u64>,
}

impl Province {
    /// Counts provinces under a specific country.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client
    /// * `country_id` - The parent country ID
    /// * `params` - Optional count parameters
    pub async fn count_with_parent(
        client: &RestClient,
        country_id: u64,
        _params: Option<ProvinceCountParams>,
    ) -> Result<u64, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("country_id", country_id.to_string());

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

    /// Finds a single province by ID under a country.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client
    /// * `country_id` - The parent country ID
    /// * `id` - The province ID to find
    /// * `params` - Optional parameters
    pub async fn find_with_parent(
        client: &RestClient,
        country_id: u64,
        id: u64,
        _params: Option<ProvinceFindParams>,
    ) -> Result<ResourceResponse<Self>, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("country_id", country_id.to_string());
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

impl RestResource for Province {
    type Id = u64;
    type FindParams = ProvinceFindParams;
    type AllParams = ProvinceListParams;
    type CountParams = ProvinceCountParams;

    const NAME: &'static str = "Province";
    const PLURAL: &'static str = "provinces";

    /// Paths for the Province resource.
    ///
    /// Limited operations - no Create or Delete.
    /// All paths require country_id.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["country_id", "id"],
            "countries/{country_id}/provinces/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["country_id"],
            "countries/{country_id}/provinces",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["country_id"],
            "countries/{country_id}/provinces/count",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["country_id", "id"],
            "countries/{country_id}/provinces/{id}",
        ),
        // Note: No Create or Delete paths - provinces managed via Country
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single province.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProvinceFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing provinces.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProvinceListParams {
    /// Return provinces after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting provinces.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProvinceCountParams {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_province_serialization() {
        let province = Province {
            id: Some(224293623),
            country_id: Some(879921427),
            name: Some("Ontario".to_string()),
            code: Some("ON".to_string()),
            tax: Some(0.08),
            tax_name: Some("HST".to_string()),
            tax_type: Some("compounded".to_string()),
            tax_percentage: Some(8.0),
            shipping_zone_id: Some(123),
        };

        let json = serde_json::to_string(&province).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["code"], "ON");
        assert_eq!(parsed["tax"], 0.08);
        assert_eq!(parsed["tax_name"], "HST");
        assert_eq!(parsed["tax_type"], "compounded");
        assert_eq!(parsed["tax_percentage"], 8.0);

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("country_id").is_none());
        assert!(parsed.get("name").is_none());
        assert!(parsed.get("shipping_zone_id").is_none());
    }

    #[test]
    fn test_province_deserialization() {
        let json = r#"{
            "id": 224293623,
            "country_id": 879921427,
            "name": "Ontario",
            "code": "ON",
            "tax": 0.08,
            "tax_name": "HST",
            "tax_type": "compounded",
            "tax_percentage": 8.0,
            "shipping_zone_id": 123456
        }"#;

        let province: Province = serde_json::from_str(json).unwrap();

        assert_eq!(province.id, Some(224293623));
        assert_eq!(province.country_id, Some(879921427));
        assert_eq!(province.name, Some("Ontario".to_string()));
        assert_eq!(province.code, Some("ON".to_string()));
        assert_eq!(province.tax, Some(0.08));
        assert_eq!(province.tax_name, Some("HST".to_string()));
        assert_eq!(province.tax_type, Some("compounded".to_string()));
        assert_eq!(province.tax_percentage, Some(8.0));
        assert_eq!(province.shipping_zone_id, Some(123456));
    }

    #[test]
    fn test_province_nested_paths_no_create_delete() {
        // Find requires both country_id and id
        let find_path = get_path(
            Province::PATHS,
            ResourceOperation::Find,
            &["country_id", "id"],
        );
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "countries/{country_id}/provinces/{id}"
        );

        // All requires country_id
        let all_path = get_path(Province::PATHS, ResourceOperation::All, &["country_id"]);
        assert!(all_path.is_some());
        assert_eq!(
            all_path.unwrap().template,
            "countries/{country_id}/provinces"
        );

        // Count requires country_id
        let count_path = get_path(Province::PATHS, ResourceOperation::Count, &["country_id"]);
        assert!(count_path.is_some());
        assert_eq!(
            count_path.unwrap().template,
            "countries/{country_id}/provinces/count"
        );

        // Update requires both country_id and id
        let update_path = get_path(
            Province::PATHS,
            ResourceOperation::Update,
            &["country_id", "id"],
        );
        assert!(update_path.is_some());
        assert_eq!(
            update_path.unwrap().template,
            "countries/{country_id}/provinces/{id}"
        );

        // No Create path (provinces managed via Country)
        let create_path = get_path(Province::PATHS, ResourceOperation::Create, &["country_id"]);
        assert!(create_path.is_none());

        // No Delete path (provinces managed via Country)
        let delete_path = get_path(
            Province::PATHS,
            ResourceOperation::Delete,
            &["country_id", "id"],
        );
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_province_constants() {
        assert_eq!(Province::NAME, "Province");
        assert_eq!(Province::PLURAL, "provinces");
    }

    #[test]
    fn test_province_get_id() {
        let province_with_id = Province {
            id: Some(224293623),
            code: Some("ON".to_string()),
            ..Default::default()
        };
        assert_eq!(province_with_id.get_id(), Some(224293623));

        let province_without_id = Province::default();
        assert_eq!(province_without_id.get_id(), None);
    }
}

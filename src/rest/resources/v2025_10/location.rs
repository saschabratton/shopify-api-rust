//! Location resource implementation.
//!
//! This module provides the [`Location`] resource for accessing store locations
//! in a Shopify store. Locations represent physical places where a merchant
//! stores, sells, or ships inventory.
//!
//! # Read-Only Resource
//!
//! Location is a read-only resource that implements the [`ReadOnlyResource`] marker trait.
//! It only supports GET operations (find, all, count) and does not have Create, Update,
//! or Delete capabilities through the REST API.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse, ReadOnlyResource};
//! use shopify_sdk::rest::resources::v2025_10::{Location, LocationListParams};
//!
//! // Find a single location
//! let location = Location::find(&client, 123, None).await?;
//! println!("Location: {}", location.name.as_deref().unwrap_or(""));
//!
//! // List all locations
//! let locations = Location::all(&client, None).await?;
//! for location in locations.iter() {
//!     println!("Location: {} - {}", location.name.as_deref().unwrap_or(""), location.city.as_deref().unwrap_or(""));
//! }
//!
//! // Count locations
//! let count = Location::count(&client, None).await?;
//! println!("Total locations: {}", count);
//!
//! // Get inventory levels at a location
//! let levels = location.inventory_levels(&client, None).await?;
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ReadOnlyResource, ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A location in a Shopify store.
///
/// Locations represent physical places where a merchant stores, sells, or ships
/// inventory. This includes retail stores, warehouses, and fulfillment centers.
///
/// # Read-Only Resource
///
/// This resource implements the [`ReadOnlyResource`] marker trait, indicating
/// that it only supports read operations. Locations cannot be created, updated,
/// or deleted through the REST API.
///
/// # Fields
///
/// All fields are read-only:
/// - `id` - The unique identifier of the location
/// - `name` - The name of the location
/// - `address1`, `address2` - Street address lines
/// - `city`, `province`, `province_code` - City and province/state
/// - `country`, `country_code` - Country information
/// - `zip` - Postal/ZIP code
/// - `phone` - Phone number
/// - `active` - Whether the location is active
/// - `legacy` - Whether the location is a legacy location
/// - `localized_country_name`, `localized_province_name` - Localized names
/// - `created_at`, `updated_at` - Timestamps
/// - `admin_graphql_api_id` - GraphQL API ID
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Location {
    /// The unique identifier of the location.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The name of the location.
    #[serde(skip_serializing)]
    pub name: Option<String>,

    /// The first line of the address.
    #[serde(skip_serializing)]
    pub address1: Option<String>,

    /// The second line of the address.
    #[serde(skip_serializing)]
    pub address2: Option<String>,

    /// The city.
    #[serde(skip_serializing)]
    pub city: Option<String>,

    /// The province or state name.
    #[serde(skip_serializing)]
    pub province: Option<String>,

    /// The two-letter province/state code.
    #[serde(skip_serializing)]
    pub province_code: Option<String>,

    /// The country name.
    #[serde(skip_serializing)]
    pub country: Option<String>,

    /// The two-letter country code (ISO 3166-1 alpha-2).
    #[serde(skip_serializing)]
    pub country_code: Option<String>,

    /// The localized country name.
    #[serde(skip_serializing)]
    pub localized_country_name: Option<String>,

    /// The localized province name.
    #[serde(skip_serializing)]
    pub localized_province_name: Option<String>,

    /// The ZIP or postal code.
    #[serde(skip_serializing)]
    pub zip: Option<String>,

    /// The phone number.
    #[serde(skip_serializing)]
    pub phone: Option<String>,

    /// Whether the location is active.
    #[serde(skip_serializing)]
    pub active: Option<bool>,

    /// Whether this is a legacy location.
    #[serde(skip_serializing)]
    pub legacy: Option<bool>,

    /// When the location was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the location was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this location.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl Location {
    /// Retrieves inventory levels at this location.
    ///
    /// Sends a GET request to `/admin/api/{version}/locations/{id}/inventory_levels.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `params` - Optional parameters for the request
    ///
    /// # Returns
    ///
    /// A vector of inventory levels at this location.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if the location has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let location = Location::find(&client, 123, None).await?.into_inner();
    /// let levels = location.inventory_levels(&client, None).await?;
    /// for level in &levels {
    ///     println!("Item {} has {} available", level.inventory_item_id.unwrap_or(0), level.available.unwrap_or(0));
    /// }
    /// ```
    pub async fn inventory_levels(
        &self,
        client: &RestClient,
        params: Option<LocationInventoryLevelsParams>,
    ) -> Result<Vec<super::InventoryLevel>, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "inventory_levels",
        })?;

        let path = format!("locations/{id}/inventory_levels");

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

        let response = client.get(&path, query).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the response - inventory levels are wrapped in "inventory_levels" key
        let levels: Vec<super::InventoryLevel> = response
            .body
            .get("inventory_levels")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'inventory_levels' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize inventory levels: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(levels)
    }
}

impl RestResource for Location {
    type Id = u64;
    type FindParams = LocationFindParams;
    type AllParams = LocationListParams;
    type CountParams = LocationCountParams;

    const NAME: &'static str = "Location";
    const PLURAL: &'static str = "locations";

    /// Paths for the Location resource.
    ///
    /// Location is a READ-ONLY resource. Only GET operations are available:
    /// - Find: GET `/locations/{id}`
    /// - All: GET `/locations`
    /// - Count: GET `/locations/count`
    ///
    /// No Create, Update, or Delete paths are defined.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "locations/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "locations"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "locations/count",
        ),
        // No Create, Update, or Delete paths - read-only resource
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Marker trait implementation indicating Location is read-only.
impl ReadOnlyResource for Location {}

/// Parameters for finding a single location.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LocationFindParams {
    // No specific find params for locations
}

/// Parameters for listing locations.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LocationListParams {
    // No specific list params for locations
}

/// Parameters for counting locations.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LocationCountParams {
    // No specific count params for locations
}

/// Parameters for getting inventory levels at a location.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LocationInventoryLevelsParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_location_implements_read_only_resource() {
        // Test that Location implements ReadOnlyResource marker trait
        fn assert_read_only<T: ReadOnlyResource>() {}
        assert_read_only::<Location>();
    }

    #[test]
    fn test_location_has_only_get_paths() {
        // Should have Find, All, Count paths
        assert!(get_path(Location::PATHS, ResourceOperation::Find, &["id"]).is_some());
        assert!(get_path(Location::PATHS, ResourceOperation::All, &[]).is_some());
        assert!(get_path(Location::PATHS, ResourceOperation::Count, &[]).is_some());

        // Should NOT have Create, Update, Delete paths
        assert!(get_path(Location::PATHS, ResourceOperation::Create, &[]).is_none());
        assert!(get_path(Location::PATHS, ResourceOperation::Update, &["id"]).is_none());
        assert!(get_path(Location::PATHS, ResourceOperation::Delete, &["id"]).is_none());
    }

    #[test]
    fn test_location_deserialization() {
        let json = r#"{
            "id": 655441491,
            "name": "Main Warehouse",
            "address1": "123 Main St",
            "address2": "Suite 100",
            "city": "New York",
            "province": "New York",
            "province_code": "NY",
            "country": "United States",
            "country_code": "US",
            "localized_country_name": "United States",
            "localized_province_name": "New York",
            "zip": "10001",
            "phone": "555-555-5555",
            "active": true,
            "legacy": false,
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/Location/655441491"
        }"#;

        let location: Location = serde_json::from_str(json).unwrap();

        assert_eq!(location.id, Some(655441491));
        assert_eq!(location.name, Some("Main Warehouse".to_string()));
        assert_eq!(location.address1, Some("123 Main St".to_string()));
        assert_eq!(location.address2, Some("Suite 100".to_string()));
        assert_eq!(location.city, Some("New York".to_string()));
        assert_eq!(location.province, Some("New York".to_string()));
        assert_eq!(location.province_code, Some("NY".to_string()));
        assert_eq!(location.country, Some("United States".to_string()));
        assert_eq!(location.country_code, Some("US".to_string()));
        assert_eq!(
            location.localized_country_name,
            Some("United States".to_string())
        );
        assert_eq!(
            location.localized_province_name,
            Some("New York".to_string())
        );
        assert_eq!(location.zip, Some("10001".to_string()));
        assert_eq!(location.phone, Some("555-555-5555".to_string()));
        assert_eq!(location.active, Some(true));
        assert_eq!(location.legacy, Some(false));
        assert!(location.created_at.is_some());
        assert!(location.updated_at.is_some());
        assert_eq!(
            location.admin_graphql_api_id,
            Some("gid://shopify/Location/655441491".to_string())
        );
    }

    #[test]
    fn test_location_serialization_is_empty() {
        // Since all fields are read-only (skip_serializing), serialization should produce empty object
        let location = Location {
            id: Some(655441491),
            name: Some("Main Warehouse".to_string()),
            city: Some("New York".to_string()),
            active: Some(true),
            ..Default::default()
        };

        let json = serde_json::to_string(&location).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // All fields should be omitted since they are read-only
        assert_eq!(parsed, serde_json::json!({}));
    }

    #[test]
    fn test_location_get_id_returns_correct_value() {
        let location_with_id = Location {
            id: Some(655441491),
            name: Some("Warehouse".to_string()),
            ..Default::default()
        };
        assert_eq!(location_with_id.get_id(), Some(655441491));

        let location_without_id = Location {
            id: None,
            name: Some("New Location".to_string()),
            ..Default::default()
        };
        assert_eq!(location_without_id.get_id(), None);
    }

    #[test]
    fn test_location_path_constants() {
        let find_path = get_path(Location::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "locations/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        let all_path = get_path(Location::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "locations");

        let count_path = get_path(Location::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "locations/count");
    }

    #[test]
    fn test_location_constants() {
        assert_eq!(Location::NAME, "Location");
        assert_eq!(Location::PLURAL, "locations");
    }
}

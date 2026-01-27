//! `InventoryLevel` resource implementation.
//!
//! This module provides the [`InventoryLevel`] resource for managing inventory levels
//! at locations in a Shopify store. Inventory levels represent the quantity of an
//! inventory item available at a specific location.
//!
//! # Composite Key
//!
//! Unlike most resources, `InventoryLevel` does NOT have an `id` field. Instead, it uses
//! a composite key of `inventory_item_id` + `location_id` to uniquely identify a record.
//!
//! # Special Operations
//!
//! Due to the composite key nature, inventory levels have special operations that are
//! implemented as associated functions rather than instance methods:
//!
//! - [`InventoryLevel::adjust`] - Adjust available quantity by a relative amount
//! - [`InventoryLevel::connect`] - Connect an inventory item to a location
//! - [`InventoryLevel::set`] - Set the available quantity to an absolute value
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::v2025_10::{InventoryLevel, InventoryLevelListParams};
//!
//! // List inventory levels
//! let params = InventoryLevelListParams {
//!     inventory_item_ids: Some("808950810,808950811".to_string()),
//!     location_ids: Some("655441491".to_string()),
//!     ..Default::default()
//! };
//! let levels = InventoryLevel::all(&client, Some(params)).await?;
//!
//! // Adjust inventory by a relative amount
//! let adjusted = InventoryLevel::adjust(&client, 808950810, 655441491, -5).await?;
//!
//! // Set inventory to an absolute value
//! let set_level = InventoryLevel::set(&client, 808950810, 655441491, 100, None).await?;
//!
//! // Connect an inventory item to a location
//! let connected = InventoryLevel::connect(&client, 808950810, 655441491, None).await?;
//!
//! // Delete inventory level at a location
//! InventoryLevel::delete_at_location(&client, 808950810, 655441491).await?;
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// An inventory level in a Shopify store.
///
/// Inventory levels represent the quantity of an inventory item available at
/// a specific location. This is a special resource that uses a composite key
/// (`inventory_item_id` + `location_id`) instead of a single `id` field.
///
/// # Composite Key
///
/// This resource does NOT have an `id` field. It is uniquely identified by:
/// - `inventory_item_id` - The ID of the inventory item
/// - `location_id` - The ID of the location
///
/// # Fields
///
/// - `inventory_item_id` - The ID of the inventory item
/// - `location_id` - The ID of the location
/// - `available` - The quantity available for sale
/// - `updated_at` - When the level was last updated
/// - `admin_graphql_api_id` - GraphQL API ID
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct InventoryLevel {
    /// The ID of the inventory item.
    /// Part of the composite key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_item_id: Option<u64>,

    /// The ID of the location.
    /// Part of the composite key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<u64>,

    /// The quantity available for sale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available: Option<i64>,

    /// When the inventory level was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this inventory level.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl InventoryLevel {
    /// Adjusts the inventory level by a relative amount.
    ///
    /// Sends a POST request to `/admin/api/{version}/inventory_levels/adjust.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `inventory_item_id` - The ID of the inventory item
    /// * `location_id` - The ID of the location
    /// * `available_adjustment` - The amount to adjust by (positive to add, negative to subtract)
    ///
    /// # Returns
    ///
    /// The updated inventory level.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Decrease inventory by 5
    /// let level = InventoryLevel::adjust(&client, 808950810, 655441491, -5).await?;
    ///
    /// // Increase inventory by 10
    /// let level = InventoryLevel::adjust(&client, 808950810, 655441491, 10).await?;
    /// ```
    pub async fn adjust(
        client: &RestClient,
        inventory_item_id: u64,
        location_id: u64,
        available_adjustment: i64,
    ) -> Result<Self, ResourceError> {
        let path = "inventory_levels/adjust";
        let body = serde_json::json!({
            "inventory_item_id": inventory_item_id,
            "location_id": location_id,
            "available_adjustment": available_adjustment
        });

        let response = client.post(path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        // Parse the response - inventory level is wrapped in "inventory_level" key
        let level: Self = response
            .body
            .get("inventory_level")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'inventory_level' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize inventory level: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(level)
    }

    /// Connects an inventory item to a location.
    ///
    /// Sends a POST request to `/admin/api/{version}/inventory_levels/connect.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `inventory_item_id` - The ID of the inventory item
    /// * `location_id` - The ID of the location
    /// * `relocate_if_necessary` - If true and the item is stocked at another location,
    ///   the stock will be moved to the new location. If false, the connection will fail
    ///   if the item is already stocked elsewhere.
    ///
    /// # Returns
    ///
    /// The created inventory level.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Connect item to location, relocating if necessary
    /// let level = InventoryLevel::connect(&client, 808950810, 655441491, Some(true)).await?;
    /// ```
    pub async fn connect(
        client: &RestClient,
        inventory_item_id: u64,
        location_id: u64,
        relocate_if_necessary: Option<bool>,
    ) -> Result<Self, ResourceError> {
        let path = "inventory_levels/connect";
        let mut body = serde_json::json!({
            "inventory_item_id": inventory_item_id,
            "location_id": location_id
        });

        if let Some(relocate) = relocate_if_necessary {
            body["relocate_if_necessary"] = serde_json::json!(relocate);
        }

        let response = client.post(path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        // Parse the response
        let level: Self = response
            .body
            .get("inventory_level")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'inventory_level' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize inventory level: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(level)
    }

    /// Sets the inventory level to an absolute value.
    ///
    /// Sends a POST request to `/admin/api/{version}/inventory_levels/set.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `inventory_item_id` - The ID of the inventory item
    /// * `location_id` - The ID of the location
    /// * `available` - The absolute quantity to set
    /// * `disconnect_if_necessary` - If true and the available quantity is 0,
    ///   the inventory item will be disconnected from the location.
    ///
    /// # Returns
    ///
    /// The updated inventory level.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Set inventory to 100 units
    /// let level = InventoryLevel::set(&client, 808950810, 655441491, 100, None).await?;
    ///
    /// // Set inventory to 0 and disconnect
    /// let level = InventoryLevel::set(&client, 808950810, 655441491, 0, Some(true)).await?;
    /// ```
    pub async fn set(
        client: &RestClient,
        inventory_item_id: u64,
        location_id: u64,
        available: i64,
        disconnect_if_necessary: Option<bool>,
    ) -> Result<Self, ResourceError> {
        let path = "inventory_levels/set";
        let mut body = serde_json::json!({
            "inventory_item_id": inventory_item_id,
            "location_id": location_id,
            "available": available
        });

        if let Some(disconnect) = disconnect_if_necessary {
            body["disconnect_if_necessary"] = serde_json::json!(disconnect);
        }

        let response = client.post(path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        // Parse the response
        let level: Self = response
            .body
            .get("inventory_level")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'inventory_level' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize inventory level: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(level)
    }

    /// Deletes an inventory level at a specific location.
    ///
    /// Sends a DELETE request to `/admin/api/{version}/inventory_levels.json`
    /// with query parameters for `inventory_item_id` and `location_id`.
    ///
    /// Note: This is different from most resources where DELETE uses a path parameter.
    /// For inventory levels, the composite key is passed as query parameters.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `inventory_item_id` - The ID of the inventory item
    /// * `location_id` - The ID of the location
    ///
    /// # Errors
    ///
    /// Returns a [`ResourceError`] if the deletion fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Delete inventory level at a location
    /// InventoryLevel::delete_at_location(&client, 808950810, 655441491).await?;
    /// ```
    pub async fn delete_at_location(
        client: &RestClient,
        inventory_item_id: u64,
        location_id: u64,
    ) -> Result<(), ResourceError> {
        let path = "inventory_levels";
        let mut query = HashMap::new();
        query.insert("inventory_item_id".to_string(), inventory_item_id.to_string());
        query.insert("location_id".to_string(), location_id.to_string());

        let response = client.delete(path, Some(query)).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        Ok(())
    }
}

impl RestResource for InventoryLevel {
    // Using String as ID type since we don't have a single ID field
    // This is a workaround for the composite key nature of this resource
    type Id = String;
    type FindParams = ();
    type AllParams = InventoryLevelListParams;
    type CountParams = ();

    const NAME: &'static str = "InventoryLevel";
    const PLURAL: &'static str = "inventory_levels";

    /// Paths for the `InventoryLevel` resource.
    ///
    /// Note: `InventoryLevel` has limited standard REST operations due to its
    /// composite key nature. Most operations are handled through special
    /// associated functions (adjust, connect, set, `delete_at_location`).
    const PATHS: &'static [ResourcePath] = &[
        // List all inventory levels (requires inventory_item_ids or location_ids param)
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "inventory_levels",
        ),
        // Note: Delete is handled by delete_at_location with query params
        // No Find, Create, Update, or Count paths
    ];

    fn get_id(&self) -> Option<Self::Id> {
        // Composite key - return None since there's no single ID
        // Use the special operations (adjust, connect, set, delete_at_location) instead
        None
    }
}

/// Parameters for listing inventory levels.
///
/// At least one of `inventory_item_ids` or `location_ids` must be provided.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct InventoryLevelListParams {
    /// Comma-separated list of inventory item IDs to retrieve levels for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_item_ids: Option<String>,

    /// Comma-separated list of location IDs to retrieve levels for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_ids: Option<String>,

    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Show inventory levels updated at or after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_inventory_level_has_no_id_field() {
        // InventoryLevel uses composite key (inventory_item_id + location_id)
        let level = InventoryLevel {
            inventory_item_id: Some(808950810),
            location_id: Some(655441491),
            available: Some(100),
            updated_at: None,
            admin_graphql_api_id: None,
        };

        // get_id should return None since there's no single ID field
        assert!(level.get_id().is_none());
    }

    #[test]
    fn test_inventory_level_serialization() {
        let level = InventoryLevel {
            inventory_item_id: Some(808950810),
            location_id: Some(655441491),
            available: Some(100),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            admin_graphql_api_id: Some("gid://shopify/InventoryLevel/123".to_string()),
        };

        let json = serde_json::to_string(&level).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["inventory_item_id"], 808950810);
        assert_eq!(parsed["location_id"], 655441491);
        assert_eq!(parsed["available"], 100);

        // Read-only fields should be omitted
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_inventory_level_deserialization() {
        let json = r#"{
            "inventory_item_id": 808950810,
            "location_id": 655441491,
            "available": 42,
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/InventoryLevel/808950810?inventory_item_id=808950810"
        }"#;

        let level: InventoryLevel = serde_json::from_str(json).unwrap();

        assert_eq!(level.inventory_item_id, Some(808950810));
        assert_eq!(level.location_id, Some(655441491));
        assert_eq!(level.available, Some(42));
        assert!(level.updated_at.is_some());
        assert!(level.admin_graphql_api_id.is_some());
    }

    #[test]
    fn test_inventory_level_special_operations_path_construction() {
        // Verify the paths used by special operations
        // These are NOT in the PATHS constant but are used by the associated functions

        // adjust -> inventory_levels/adjust
        assert_eq!(format!("inventory_levels/adjust"), "inventory_levels/adjust");

        // connect -> inventory_levels/connect
        assert_eq!(
            format!("inventory_levels/connect"),
            "inventory_levels/connect"
        );

        // set -> inventory_levels/set
        assert_eq!(format!("inventory_levels/set"), "inventory_levels/set");

        // delete_at_location -> inventory_levels with query params
        assert_eq!(format!("inventory_levels"), "inventory_levels");
    }

    #[test]
    fn test_inventory_level_list_params_serialization() {
        let params = InventoryLevelListParams {
            inventory_item_ids: Some("808950810,808950811".to_string()),
            location_ids: Some("655441491,655441492".to_string()),
            limit: Some(50),
            updated_at_min: Some(
                DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["inventory_item_ids"], "808950810,808950811");
        assert_eq!(json["location_ids"], "655441491,655441492");
        assert_eq!(json["limit"], 50);
        assert!(json["updated_at_min"].as_str().is_some());

        // Test empty params
        let empty_params = InventoryLevelListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_inventory_level_paths() {
        // Should only have All path
        let all_path = get_path(InventoryLevel::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "inventory_levels");
        assert_eq!(all_path.unwrap().http_method, HttpMethod::Get);

        // Should NOT have Find, Create, Update, Delete, Count paths
        assert!(get_path(InventoryLevel::PATHS, ResourceOperation::Find, &["id"]).is_none());
        assert!(get_path(InventoryLevel::PATHS, ResourceOperation::Create, &[]).is_none());
        assert!(get_path(InventoryLevel::PATHS, ResourceOperation::Update, &["id"]).is_none());
        assert!(get_path(InventoryLevel::PATHS, ResourceOperation::Delete, &["id"]).is_none());
        assert!(get_path(InventoryLevel::PATHS, ResourceOperation::Count, &[]).is_none());
    }

    #[test]
    fn test_inventory_level_constants() {
        assert_eq!(InventoryLevel::NAME, "InventoryLevel");
        assert_eq!(InventoryLevel::PLURAL, "inventory_levels");
    }
}

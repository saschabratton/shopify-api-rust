//! `InventoryItem` resource implementation.
//!
//! This module provides the `InventoryItem` resource, which represents an inventory item
//! in a Shopify store. Inventory items are linked to product variants via the
//! `inventory_item_id` field on variants.
//!
//! # Important Notes
//!
//! - `InventoryItem` uses standalone paths only (`/inventory_items/{id}`)
//! - The list operation requires the `ids` parameter (comma-separated inventory item IDs)
//! - `InventoryItem` is linked to Variant via `Variant.inventory_item_id`
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{InventoryItem, InventoryItemListParams};
//!
//! // Find a single inventory item
//! let inventory_item = InventoryItem::find(&client, 123, None).await?;
//! println!("SKU: {}", inventory_item.sku.as_deref().unwrap_or(""));
//!
//! // List inventory items by IDs (ids parameter is required)
//! let params = InventoryItemListParams {
//!     ids: Some(vec![123, 456, 789]),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let inventory_items = InventoryItem::all(&client, Some(params)).await?;
//!
//! // Update an inventory item
//! let mut item = InventoryItem::find(&client, 123, None).await?.into_inner();
//! item.cost = Some("15.99".to_string());
//! item.tracked = Some(true);
//! let saved = item.save(&client).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// Country-specific harmonized system code for customs.
///
/// Used for international shipping and customs declarations.
/// Each country may have its own harmonized system code for a product.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CountryHarmonizedSystemCode {
    /// The harmonized system code for this country.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harmonized_system_code: Option<String>,

    /// The ISO 3166-1 alpha-2 country code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

/// An inventory item in a Shopify store.
///
/// Inventory items are associated with product variants and contain
/// information about tracking, cost, and customs declarations.
///
/// # Relationship to Variants
///
/// Each variant has an `inventory_item_id` field that links to its
/// corresponding inventory item. The inventory item tracks:
/// - Cost of goods
/// - Whether inventory is tracked
/// - Customs/HS codes for international shipping
///
/// # Fields
///
/// ## Writable Fields
/// - `cost` - The cost of the item (for profit calculations)
/// - `sku` - Stock keeping unit identifier
/// - `country_code_of_origin` - ISO country code where item originated
/// - `province_code_of_origin` - Province/state code where item originated
/// - `harmonized_system_code` - HS code for customs
/// - `tracked` - Whether inventory levels are tracked
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `requires_shipping` - Whether the item requires shipping
/// - `created_at` - When the inventory item was created
/// - `updated_at` - When the inventory item was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct InventoryItem {
    /// The unique identifier of the inventory item.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The stock keeping unit (SKU) of the inventory item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,

    /// The unit cost of the inventory item.
    /// Stored as a string to preserve decimal precision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<String>,

    /// When the inventory item was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the inventory item was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// Whether the inventory item requires shipping.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub requires_shipping: Option<bool>,

    /// Whether inventory levels are tracked for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracked: Option<bool>,

    /// The ISO 3166-1 alpha-2 country code of origin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code_of_origin: Option<String>,

    /// The province/state code of origin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province_code_of_origin: Option<String>,

    /// The harmonized system code for customs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harmonized_system_code: Option<String>,

    /// Country-specific harmonized system codes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_harmonized_system_codes: Option<Vec<CountryHarmonizedSystemCode>>,

    /// The admin GraphQL API ID for this inventory item.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for InventoryItem {
    type Id = u64;
    type FindParams = InventoryItemFindParams;
    type AllParams = InventoryItemListParams;
    type CountParams = ();

    const NAME: &'static str = "InventoryItem";
    const PLURAL: &'static str = "inventory_items";

    /// Paths for the `InventoryItem` resource.
    ///
    /// `InventoryItem` uses STANDALONE PATHS only:
    /// - `/inventory_items/{id}` for individual item access
    /// - `/inventory_items.json` for listing (requires `ids` parameter)
    ///
    /// Note: There is no count endpoint for inventory items.
    /// Note: Inventory items cannot be created or deleted directly;
    ///       they are managed through variants.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "inventory_items/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "inventory_items",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "inventory_items/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single inventory item.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct InventoryItemFindParams {
    // No specific find params for inventory items
}

/// Parameters for listing inventory items.
///
/// # Important
///
/// The `ids` parameter is required when listing inventory items.
/// The API will return an error if `ids` is not provided.
///
/// # Example
///
/// ```rust,ignore
/// let params = InventoryItemListParams {
///     ids: Some(vec![123, 456, 789]),
///     limit: Some(50),
///     ..Default::default()
/// };
/// let items = InventoryItem::all(&client, Some(params)).await?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct InventoryItemListParams {
    /// Comma-separated list of inventory item IDs to retrieve.
    /// **Required** - the API requires this parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<u64>>,

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
    fn test_inventory_item_struct_serialization() {
        let item = InventoryItem {
            id: Some(12345), // Read-only, should be skipped
            sku: Some("SKU-001".to_string()),
            cost: Some("15.99".to_string()),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ), // Read-only
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-20T15:45:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ), // Read-only
            requires_shipping: Some(true), // Read-only
            tracked: Some(true),
            country_code_of_origin: Some("US".to_string()),
            province_code_of_origin: Some("CA".to_string()),
            harmonized_system_code: Some("6109.10".to_string()),
            country_harmonized_system_codes: Some(vec![CountryHarmonizedSystemCode {
                harmonized_system_code: Some("6109.10.0000".to_string()),
                country_code: Some("CA".to_string()),
            }]),
            admin_graphql_api_id: Some("gid://shopify/InventoryItem/12345".to_string()), // Read-only
        };

        let json = serde_json::to_string(&item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["sku"], "SKU-001");
        assert_eq!(parsed["cost"], "15.99");
        assert_eq!(parsed["tracked"], true);
        assert_eq!(parsed["country_code_of_origin"], "US");
        assert_eq!(parsed["province_code_of_origin"], "CA");
        assert_eq!(parsed["harmonized_system_code"], "6109.10");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("requires_shipping").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());

        // Nested country harmonized system codes should be present
        let codes = parsed["country_harmonized_system_codes"]
            .as_array()
            .unwrap();
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0]["harmonized_system_code"], "6109.10.0000");
        assert_eq!(codes[0]["country_code"], "CA");
    }

    #[test]
    fn test_inventory_item_deserialization() {
        let json = r#"{
            "id": 808950810,
            "sku": "IPOD-342-N",
            "cost": "25.00",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "requires_shipping": true,
            "tracked": true,
            "country_code_of_origin": "US",
            "province_code_of_origin": "CA",
            "harmonized_system_code": "8523.29.90",
            "country_harmonized_system_codes": [
                {
                    "harmonized_system_code": "8523.29.9000",
                    "country_code": "CA"
                },
                {
                    "harmonized_system_code": "8523.29.9090",
                    "country_code": "GB"
                }
            ],
            "admin_graphql_api_id": "gid://shopify/InventoryItem/808950810"
        }"#;

        let item: InventoryItem = serde_json::from_str(json).unwrap();

        // Verify all fields are deserialized correctly
        assert_eq!(item.id, Some(808950810));
        assert_eq!(item.sku, Some("IPOD-342-N".to_string()));
        assert_eq!(item.cost, Some("25.00".to_string()));
        assert!(item.created_at.is_some());
        assert!(item.updated_at.is_some());
        assert_eq!(item.requires_shipping, Some(true));
        assert_eq!(item.tracked, Some(true));
        assert_eq!(item.country_code_of_origin, Some("US".to_string()));
        assert_eq!(item.province_code_of_origin, Some("CA".to_string()));
        assert_eq!(item.harmonized_system_code, Some("8523.29.90".to_string()));
        assert_eq!(
            item.admin_graphql_api_id,
            Some("gid://shopify/InventoryItem/808950810".to_string())
        );

        // Verify nested country harmonized system codes
        let codes = item.country_harmonized_system_codes.unwrap();
        assert_eq!(codes.len(), 2);
        assert_eq!(
            codes[0].harmonized_system_code,
            Some("8523.29.9000".to_string())
        );
        assert_eq!(codes[0].country_code, Some("CA".to_string()));
        assert_eq!(
            codes[1].harmonized_system_code,
            Some("8523.29.9090".to_string())
        );
        assert_eq!(codes[1].country_code, Some("GB".to_string()));
    }

    #[test]
    fn test_inventory_item_list_params_with_ids() {
        let params = InventoryItemListParams {
            ids: Some(vec![808950810, 808950811, 808950812]),
            limit: Some(50),
            page_info: None,
        };

        let json = serde_json::to_value(&params).unwrap();

        // IDs should serialize as an array (converted to comma-separated by serialize_to_query)
        assert_eq!(
            json["ids"],
            serde_json::json!([808950810, 808950811, 808950812])
        );
        assert_eq!(json["limit"], 50);

        // page_info should be omitted when None
        assert!(json.get("page_info").is_none());

        // Test with minimal params
        let empty_params = InventoryItemListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_inventory_item_get_id_returns_correct_value() {
        // Inventory item with ID
        let item_with_id = InventoryItem {
            id: Some(808950810),
            sku: Some("TEST-SKU".to_string()),
            ..Default::default()
        };
        assert_eq!(item_with_id.get_id(), Some(808950810));

        // Inventory item without ID (should not normally happen since items are auto-created)
        let item_without_id = InventoryItem {
            id: None,
            sku: Some("NEW-SKU".to_string()),
            ..Default::default()
        };
        assert_eq!(item_without_id.get_id(), None);

        // Verify trait constants
        assert_eq!(InventoryItem::NAME, "InventoryItem");
        assert_eq!(InventoryItem::PLURAL, "inventory_items");
    }

    #[test]
    fn test_inventory_item_path_constants_are_correct() {
        // Test Find path (standalone only)
        let find_path = get_path(InventoryItem::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "inventory_items/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path (standalone)
        let all_path = get_path(InventoryItem::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "inventory_items");
        assert_eq!(all_path.unwrap().http_method, HttpMethod::Get);

        // Test Update path
        let update_path = get_path(InventoryItem::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "inventory_items/{id}");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // No Create path (inventory items are auto-created with variants)
        let create_path = get_path(InventoryItem::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_none());

        // No Delete path (inventory items are auto-deleted with variants)
        let delete_path = get_path(InventoryItem::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());

        // No Count path for inventory items
        let count_path = get_path(InventoryItem::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());
    }

    #[test]
    fn test_country_harmonized_system_code_struct() {
        let code = CountryHarmonizedSystemCode {
            harmonized_system_code: Some("8523.29.9000".to_string()),
            country_code: Some("CA".to_string()),
        };

        // Test serialization
        let json = serde_json::to_value(&code).unwrap();
        assert_eq!(json["harmonized_system_code"], "8523.29.9000");
        assert_eq!(json["country_code"], "CA");

        // Test deserialization
        let json_str = r#"{"harmonized_system_code": "1234.56.7890", "country_code": "US"}"#;
        let parsed: CountryHarmonizedSystemCode = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            parsed.harmonized_system_code,
            Some("1234.56.7890".to_string())
        );
        assert_eq!(parsed.country_code, Some("US".to_string()));

        // Test with optional fields omitted
        let empty_code = CountryHarmonizedSystemCode::default();
        let empty_json = serde_json::to_value(&empty_code).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }
}

//! Variant resource implementation.
//!
//! This module provides the Variant resource, which represents a variant of a product
//! in a Shopify store. Variants are different versions of a product (e.g., size, color).
//!
//! # Dual Path Support
//!
//! The Variant resource supports both nested and standalone paths:
//! - Nested: `/products/{product_id}/variants/{id}` - when `product_id` is available
//! - Standalone: `/variants/{id}` - fallback when only variant id is available
//!
//! The path selection automatically chooses the most specific path based on available IDs.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{Variant, VariantListParams, WeightUnit};
//!
//! // Find a variant by ID (standalone path)
//! let variant = Variant::find(&client, 123, None).await?;
//! println!("Variant: {}", variant.title.as_deref().unwrap_or(""));
//!
//! // List variants under a product (nested path)
//! let variants = Variant::all_with_parent(&client, "product_id", 456, None).await?;
//!
//! // Create a new variant under a product
//! let mut variant = Variant {
//!     product_id: Some(456),
//!     title: Some("Large / Blue".to_string()),
//!     price: Some("29.99".to_string()),
//!     sku: Some("PROD-LG-BL".to_string()),
//!     weight: Some(1.5),
//!     weight_unit: Some(WeightUnit::Kg),
//!     ..Default::default()
//! };
//! let saved = variant.save(&client).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// The unit of measurement for variant weight.
///
/// Used to specify whether the weight is in kilograms, grams, pounds, or ounces.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum WeightUnit {
    /// Kilograms
    #[default]
    Kg,
    /// Grams
    G,
    /// Pounds
    Lb,
    /// Ounces
    Oz,
}

/// A product variant in a Shopify store.
///
/// Variants represent different versions of a product, typically distinguished by
/// attributes like size, color, or material. Each variant can have its own price,
/// SKU, inventory, and weight settings.
///
/// # Dual Path Access
///
/// Variants can be accessed through two path patterns:
/// - Nested under a product: `/products/{product_id}/variants/{id}`
/// - Standalone: `/variants/{id}`
///
/// The nested path is preferred when `product_id` is available.
///
/// # Fields
///
/// ## Writable Fields
/// - `product_id` - The ID of the product this variant belongs to
/// - `title` - The title of the variant
/// - `price` - The price of the variant
/// - `compare_at_price` - The original price for comparison (sale pricing)
/// - `sku` - Stock keeping unit identifier
/// - `barcode` - The barcode, UPC, or ISBN number
/// - `position` - The position in the variant list
/// - `grams` - The weight in grams (deprecated, use `weight`/`weight_unit`)
/// - `weight` - The weight value
/// - `weight_unit` - The unit of measurement for weight
/// - `inventory_management` - The fulfillment service tracking inventory
/// - `inventory_policy` - Whether to allow purchases when out of stock
/// - `fulfillment_service` - The fulfillment service for this variant
/// - `option1`, `option2`, `option3` - Option values
/// - `image_id` - The ID of the associated image
/// - `taxable` - Whether the variant is taxable
/// - `tax_code` - The tax code for the variant
/// - `requires_shipping` - Whether the variant requires shipping
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `inventory_item_id` - The ID of the associated inventory item
/// - `inventory_quantity` - The available quantity
/// - `created_at` - When the variant was created
/// - `updated_at` - When the variant was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Variant {
    /// The unique identifier of the variant.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the product this variant belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// The title of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The price of the variant.
    /// Stored as a string to preserve decimal precision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The original price of the variant for comparison.
    /// Used to show sale pricing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compare_at_price: Option<String>,

    /// The stock keeping unit (SKU) of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,

    /// The barcode, UPC, or ISBN number of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barcode: Option<String>,

    /// The position of the variant in the product's variant list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,

    /// The weight of the variant in grams.
    /// Deprecated: Use `weight` and `weight_unit` instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grams: Option<i64>,

    /// The weight of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,

    /// The unit of measurement for the variant's weight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight_unit: Option<WeightUnit>,

    /// The ID of the inventory item associated with this variant.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub inventory_item_id: Option<u64>,

    /// The available quantity of the variant.
    /// Read-only field - use Inventory API to modify.
    #[serde(skip_serializing)]
    pub inventory_quantity: Option<i64>,

    /// The fulfillment service that tracks inventory for this variant.
    /// Valid values: "shopify" or the handle of a fulfillment service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_management: Option<String>,

    /// Whether customers can purchase the variant when it's out of stock.
    /// Valid values: "deny" or "continue".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_policy: Option<String>,

    /// The fulfillment service for this variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_service: Option<String>,

    /// The value of the first option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option1: Option<String>,

    /// The value of the second option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option2: Option<String>,

    /// The value of the third option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option3: Option<String>,

    /// The ID of the image associated with this variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<u64>,

    /// Whether the variant is taxable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxable: Option<bool>,

    /// The tax code for the variant (Shopify Plus feature).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_code: Option<String>,

    /// Whether the variant requires shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_shipping: Option<bool>,

    /// When the variant was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the variant was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this variant.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for Variant {
    type Id = u64;
    type FindParams = VariantFindParams;
    type AllParams = VariantListParams;
    type CountParams = VariantCountParams;

    const NAME: &'static str = "Variant";
    const PLURAL: &'static str = "variants";

    /// Paths for the Variant resource.
    ///
    /// The Variant resource supports DUAL PATHS:
    /// 1. Nested paths under products (more specific, preferred when `product_id` available)
    /// 2. Standalone paths (fallback when only variant id available)
    ///
    /// Path selection chooses the most specific path based on available IDs.
    const PATHS: &'static [ResourcePath] = &[
        // Nested paths (more specific - preferred when product_id is available)
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["product_id", "id"],
            "products/{product_id}/variants/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["product_id"],
            "products/{product_id}/variants",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["product_id"],
            "products/{product_id}/variants/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["product_id"],
            "products/{product_id}/variants",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["product_id", "id"],
            "products/{product_id}/variants/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["product_id", "id"],
            "products/{product_id}/variants/{id}",
        ),
        // Standalone paths (fallback - used when only variant id is available)
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "variants/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "variants/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single variant.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct VariantFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing variants.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct VariantListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return variants after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

/// Parameters for counting variants.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct VariantCountParams {
    // No specific count params for variants beyond the product_id in the path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_variant_struct_serialization() {
        let variant = Variant {
            id: Some(12345),         // Read-only, should be skipped
            product_id: Some(67890), // Writable
            title: Some("Large / Blue".to_string()),
            price: Some("29.99".to_string()),
            compare_at_price: Some("39.99".to_string()),
            sku: Some("PROD-LG-BL".to_string()),
            barcode: Some("1234567890123".to_string()),
            position: Some(2),
            grams: Some(500),
            weight: Some(0.5),
            weight_unit: Some(WeightUnit::Kg),
            inventory_item_id: Some(111222), // Read-only
            inventory_quantity: Some(100),   // Read-only
            inventory_management: Some("shopify".to_string()),
            inventory_policy: Some("deny".to_string()),
            fulfillment_service: Some("manual".to_string()),
            option1: Some("Large".to_string()),
            option2: Some("Blue".to_string()),
            option3: None,
            image_id: Some(999888),
            taxable: Some(true),
            tax_code: None,
            requires_shipping: Some(true),
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
            admin_graphql_api_id: Some("gid://shopify/ProductVariant/12345".to_string()), // Read-only
        };

        let json = serde_json::to_string(&variant).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["product_id"], 67890);
        assert_eq!(parsed["title"], "Large / Blue");
        assert_eq!(parsed["price"], "29.99");
        assert_eq!(parsed["compare_at_price"], "39.99");
        assert_eq!(parsed["sku"], "PROD-LG-BL");
        assert_eq!(parsed["barcode"], "1234567890123");
        assert_eq!(parsed["position"], 2);
        assert_eq!(parsed["grams"], 500);
        assert_eq!(parsed["weight"], 0.5);
        assert_eq!(parsed["weight_unit"], "kg");
        assert_eq!(parsed["inventory_management"], "shopify");
        assert_eq!(parsed["inventory_policy"], "deny");
        assert_eq!(parsed["fulfillment_service"], "manual");
        assert_eq!(parsed["option1"], "Large");
        assert_eq!(parsed["option2"], "Blue");
        assert_eq!(parsed["image_id"], 999888);
        assert_eq!(parsed["taxable"], true);
        assert_eq!(parsed["requires_shipping"], true);

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("inventory_item_id").is_none());
        assert!(parsed.get("inventory_quantity").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());

        // Optional fields that are None should be omitted
        assert!(parsed.get("option3").is_none());
        assert!(parsed.get("tax_code").is_none());
    }

    #[test]
    fn test_variant_deserialization_from_api_response() {
        let json = r#"{
            "id": 39072856,
            "product_id": 788032119674292922,
            "title": "Large / Blue",
            "price": "29.99",
            "compare_at_price": "39.99",
            "sku": "PROD-LG-BL",
            "barcode": "1234567890123",
            "position": 2,
            "grams": 500,
            "weight": 0.5,
            "weight_unit": "kg",
            "inventory_item_id": 111222333,
            "inventory_quantity": 100,
            "inventory_management": "shopify",
            "inventory_policy": "deny",
            "fulfillment_service": "manual",
            "option1": "Large",
            "option2": "Blue",
            "option3": null,
            "image_id": 999888777,
            "taxable": true,
            "tax_code": null,
            "requires_shipping": true,
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/ProductVariant/39072856"
        }"#;

        let variant: Variant = serde_json::from_str(json).unwrap();

        // Verify all fields are deserialized correctly
        assert_eq!(variant.id, Some(39072856));
        assert_eq!(variant.product_id, Some(788032119674292922));
        assert_eq!(variant.title, Some("Large / Blue".to_string()));
        assert_eq!(variant.price, Some("29.99".to_string()));
        assert_eq!(variant.compare_at_price, Some("39.99".to_string()));
        assert_eq!(variant.sku, Some("PROD-LG-BL".to_string()));
        assert_eq!(variant.barcode, Some("1234567890123".to_string()));
        assert_eq!(variant.position, Some(2));
        assert_eq!(variant.grams, Some(500));
        assert_eq!(variant.weight, Some(0.5));
        assert_eq!(variant.weight_unit, Some(WeightUnit::Kg));
        assert_eq!(variant.inventory_item_id, Some(111222333));
        assert_eq!(variant.inventory_quantity, Some(100));
        assert_eq!(variant.inventory_management, Some("shopify".to_string()));
        assert_eq!(variant.inventory_policy, Some("deny".to_string()));
        assert_eq!(variant.fulfillment_service, Some("manual".to_string()));
        assert_eq!(variant.option1, Some("Large".to_string()));
        assert_eq!(variant.option2, Some("Blue".to_string()));
        assert_eq!(variant.option3, None);
        assert_eq!(variant.image_id, Some(999888777));
        assert_eq!(variant.taxable, Some(true));
        assert_eq!(variant.tax_code, None);
        assert_eq!(variant.requires_shipping, Some(true));
        assert!(variant.created_at.is_some());
        assert!(variant.updated_at.is_some());
        assert_eq!(
            variant.admin_graphql_api_id,
            Some("gid://shopify/ProductVariant/39072856".to_string())
        );
    }

    #[test]
    fn test_dual_path_patterns() {
        // Test nested path (with both product_id and id) - most specific for Find
        let nested_find_path = get_path(
            Variant::PATHS,
            ResourceOperation::Find,
            &["product_id", "id"],
        );
        assert!(nested_find_path.is_some());
        assert_eq!(
            nested_find_path.unwrap().template,
            "products/{product_id}/variants/{id}"
        );

        // Test standalone path (with only id) - fallback for Find
        let standalone_find_path = get_path(Variant::PATHS, ResourceOperation::Find, &["id"]);
        assert!(standalone_find_path.is_some());
        assert_eq!(standalone_find_path.unwrap().template, "variants/{id}");

        // Test nested All path (requires product_id)
        let nested_all_path = get_path(Variant::PATHS, ResourceOperation::All, &["product_id"]);
        assert!(nested_all_path.is_some());
        assert_eq!(
            nested_all_path.unwrap().template,
            "products/{product_id}/variants"
        );

        // Test that All without product_id fails (no standalone All path)
        let standalone_all_path = get_path(Variant::PATHS, ResourceOperation::All, &[]);
        assert!(standalone_all_path.is_none());

        // Test nested Update path (with both product_id and id)
        let nested_update_path = get_path(
            Variant::PATHS,
            ResourceOperation::Update,
            &["product_id", "id"],
        );
        assert!(nested_update_path.is_some());
        assert_eq!(
            nested_update_path.unwrap().template,
            "products/{product_id}/variants/{id}"
        );

        // Test standalone Update path (with only id)
        let standalone_update_path = get_path(Variant::PATHS, ResourceOperation::Update, &["id"]);
        assert!(standalone_update_path.is_some());
        assert_eq!(standalone_update_path.unwrap().template, "variants/{id}");

        // Test Create path (requires product_id)
        let create_path = get_path(Variant::PATHS, ResourceOperation::Create, &["product_id"]);
        assert!(create_path.is_some());
        assert_eq!(
            create_path.unwrap().template,
            "products/{product_id}/variants"
        );

        // Test Delete path (requires both product_id and id)
        let delete_path = get_path(
            Variant::PATHS,
            ResourceOperation::Delete,
            &["product_id", "id"],
        );
        assert!(delete_path.is_some());
        assert_eq!(
            delete_path.unwrap().template,
            "products/{product_id}/variants/{id}"
        );

        // Test Count path (requires product_id)
        let count_path = get_path(Variant::PATHS, ResourceOperation::Count, &["product_id"]);
        assert!(count_path.is_some());
        assert_eq!(
            count_path.unwrap().template,
            "products/{product_id}/variants/count"
        );

        // Verify constants
        assert_eq!(Variant::NAME, "Variant");
        assert_eq!(Variant::PLURAL, "variants");
    }

    #[test]
    fn test_weight_unit_enum_serialization() {
        // Test serialization to lowercase
        assert_eq!(serde_json::to_string(&WeightUnit::Kg).unwrap(), "\"kg\"");
        assert_eq!(serde_json::to_string(&WeightUnit::G).unwrap(), "\"g\"");
        assert_eq!(serde_json::to_string(&WeightUnit::Lb).unwrap(), "\"lb\"");
        assert_eq!(serde_json::to_string(&WeightUnit::Oz).unwrap(), "\"oz\"");

        // Test deserialization from lowercase
        let kg: WeightUnit = serde_json::from_str("\"kg\"").unwrap();
        let g: WeightUnit = serde_json::from_str("\"g\"").unwrap();
        let lb: WeightUnit = serde_json::from_str("\"lb\"").unwrap();
        let oz: WeightUnit = serde_json::from_str("\"oz\"").unwrap();

        assert_eq!(kg, WeightUnit::Kg);
        assert_eq!(g, WeightUnit::G);
        assert_eq!(lb, WeightUnit::Lb);
        assert_eq!(oz, WeightUnit::Oz);

        // Test default value
        assert_eq!(WeightUnit::default(), WeightUnit::Kg);
    }

    #[test]
    fn test_variant_list_params_serialization() {
        let params = VariantListParams {
            limit: Some(50),
            since_id: Some(12345),
            fields: Some("id,title,price,sku".to_string()),
            page_info: Some("eyJsYXN0X2lkIjoxMjM0NTY3ODkwfQ".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 12345);
        assert_eq!(json["fields"], "id,title,price,sku");
        assert_eq!(json["page_info"], "eyJsYXN0X2lkIjoxMjM0NTY3ODkwfQ");

        // Test with minimal params (all None)
        let empty_params = VariantListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();

        // Empty object when all fields are None
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_variant_get_id_returns_correct_value() {
        // Variant with ID
        let variant_with_id = Variant {
            id: Some(123456789),
            product_id: Some(987654321),
            title: Some("Test Variant".to_string()),
            ..Default::default()
        };
        assert_eq!(variant_with_id.get_id(), Some(123456789));

        // Variant without ID (new variant)
        let variant_without_id = Variant {
            id: None,
            product_id: Some(987654321),
            title: Some("New Variant".to_string()),
            ..Default::default()
        };
        assert_eq!(variant_without_id.get_id(), None);
    }
}

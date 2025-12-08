//! Product-related embedded types.
//!
//! This module provides types for product images and options that are
//! embedded within Product resources.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An image associated with a product.
///
/// Product images can be associated with specific variants or with the
/// product as a whole.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::ProductImage;
///
/// let image = ProductImage {
///     position: Some(1),
///     src: Some("https://cdn.shopify.com/s/files/1/0/products/image.jpg".to_string()),
///     alt: Some("Product front view".to_string()),
///     width: Some(800),
///     height: Some(600),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductImage {
    /// The unique identifier of the image.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the product this image belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// The position of the image in the product's image list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,

    /// The source URL of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// The width of the image in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,

    /// The height of the image in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,

    /// Alternative text for the image (for accessibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,

    /// IDs of variants that use this image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_ids: Option<Vec<u64>>,

    /// When the image was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the image was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

/// A product option (e.g., Size, Color).
///
/// Product options define the customizable attributes of a product
/// that create variants.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::ProductOption;
///
/// let option = ProductOption {
///     name: Some("Size".to_string()),
///     position: Some(1),
///     values: Some(vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()]),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductOption {
    /// The unique identifier of the option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The ID of the product this option belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// The name of the option (e.g., "Size", "Color").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The position of the option in the product's option list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,

    /// The possible values for this option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_image_serialization() {
        let image = ProductImage {
            id: Some(12345), // Should not be serialized
            product_id: Some(67890),
            position: Some(1),
            src: Some("https://cdn.shopify.com/image.jpg".to_string()),
            width: Some(1024),
            height: Some(768),
            alt: Some("Product image".to_string()),
            variant_ids: Some(vec![111, 222, 333]),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ), // Should not be serialized
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-20T15:45:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ), // Should not be serialized
            admin_graphql_api_id: Some("gid://shopify/ProductImage/12345".to_string()), // Should not be serialized
        };

        let json = serde_json::to_string(&image).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["product_id"], 67890);
        assert_eq!(parsed["position"], 1);
        assert_eq!(parsed["src"], "https://cdn.shopify.com/image.jpg");
        assert_eq!(parsed["width"], 1024);
        assert_eq!(parsed["height"], 768);
        assert_eq!(parsed["alt"], "Product image");
        assert_eq!(parsed["variant_ids"], serde_json::json!([111, 222, 333]));

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_product_image_deserialization() {
        let json = r#"{
            "id": 99999,
            "product_id": 88888,
            "position": 2,
            "src": "https://example.com/product.png",
            "width": 500,
            "height": 400,
            "alt": "Alt text",
            "variant_ids": [1, 2],
            "created_at": "2024-03-10T08:00:00Z",
            "updated_at": "2024-03-15T12:00:00Z",
            "admin_graphql_api_id": "gid://shopify/ProductImage/99999"
        }"#;

        let image: ProductImage = serde_json::from_str(json).unwrap();

        // All fields should be deserialized
        assert_eq!(image.id, Some(99999));
        assert_eq!(image.product_id, Some(88888));
        assert_eq!(image.position, Some(2));
        assert_eq!(
            image.src,
            Some("https://example.com/product.png".to_string())
        );
        assert_eq!(image.width, Some(500));
        assert_eq!(image.height, Some(400));
        assert_eq!(image.variant_ids, Some(vec![1, 2]));
        assert!(image.created_at.is_some());
        assert!(image.updated_at.is_some());
        assert_eq!(
            image.admin_graphql_api_id,
            Some("gid://shopify/ProductImage/99999".to_string())
        );
    }

    #[test]
    fn test_product_option_serialization() {
        let option = ProductOption {
            id: Some(54321),
            product_id: Some(11111),
            name: Some("Color".to_string()),
            position: Some(1),
            values: Some(vec![
                "Red".to_string(),
                "Blue".to_string(),
                "Green".to_string(),
            ]),
        };

        let json = serde_json::to_string(&option).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], 54321);
        assert_eq!(parsed["product_id"], 11111);
        assert_eq!(parsed["name"], "Color");
        assert_eq!(parsed["position"], 1);
        assert_eq!(
            parsed["values"],
            serde_json::json!(["Red", "Blue", "Green"])
        );
    }

    #[test]
    fn test_product_option_deserialization() {
        let json = r#"{
            "id": 777,
            "product_id": 888,
            "name": "Size",
            "position": 2,
            "values": ["XS", "S", "M", "L", "XL"]
        }"#;

        let option: ProductOption = serde_json::from_str(json).unwrap();

        assert_eq!(option.id, Some(777));
        assert_eq!(option.product_id, Some(888));
        assert_eq!(option.name, Some("Size".to_string()));
        assert_eq!(option.position, Some(2));
        assert_eq!(
            option.values,
            Some(vec![
                "XS".to_string(),
                "S".to_string(),
                "M".to_string(),
                "L".to_string(),
                "XL".to_string()
            ])
        );
    }

    #[test]
    fn test_product_option_with_empty_values() {
        let option = ProductOption {
            name: Some("Material".to_string()),
            values: Some(vec![]),
            ..Default::default()
        };

        let json = serde_json::to_string(&option).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["name"], "Material");
        assert_eq!(parsed["values"], serde_json::json!([]));
    }
}

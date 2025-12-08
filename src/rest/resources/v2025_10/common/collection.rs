//! Collection-related types used by `CustomCollection` and `SmartCollection`.
//!
//! This module provides shared types for collection resources.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An image associated with a collection.
///
/// This struct is used by both `CustomCollection` and `SmartCollection`
/// to represent the collection's featured image.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::CollectionImage;
///
/// let image = CollectionImage {
///     src: Some("https://cdn.shopify.com/s/files/1/collection.jpg".to_string()),
///     alt: Some("Summer collection".to_string()),
///     width: Some(1200),
///     height: Some(800),
///     created_at: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CollectionImage {
    /// The source URL of the collection image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// Alternative text for the image (for accessibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,

    /// The width of the image in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,

    /// The height of the image in pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i64>,

    /// When the image was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,
}

/// A rule for automatic product inclusion in a `SmartCollection`.
///
/// Smart collections use rules to automatically include products
/// that match specified criteria.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::SmartCollectionRule;
///
/// // Match products with "summer" tag
/// let rule = SmartCollectionRule {
///     column: "tag".to_string(),
///     relation: "equals".to_string(),
///     condition: "summer".to_string(),
/// };
///
/// // Match products from vendor "Nike"
/// let vendor_rule = SmartCollectionRule {
///     column: "vendor".to_string(),
///     relation: "equals".to_string(),
///     condition: "Nike".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SmartCollectionRule {
    /// The property to match against.
    ///
    /// Common values: "title", "type", "vendor", "variant\_price",
    /// "tag", "variant\_compare\_at\_price", "variant\_weight",
    /// "variant\_inventory", "variant\_title"
    pub column: String,

    /// The relationship between the column and condition.
    ///
    /// Common values: "equals", "not\_equals", "greater\_than",
    /// "less\_than", "starts\_with", "ends\_with", "contains",
    /// "not\_contains"
    pub relation: String,

    /// The value to match against the column.
    pub condition: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_image_serialization() {
        let image = CollectionImage {
            src: Some("https://cdn.shopify.com/collection.jpg".to_string()),
            alt: Some("Featured collection".to_string()),
            width: Some(1024),
            height: Some(768),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_string(&image).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["src"], "https://cdn.shopify.com/collection.jpg");
        assert_eq!(parsed["alt"], "Featured collection");
        assert_eq!(parsed["width"], 1024);
        assert_eq!(parsed["height"], 768);

        // Read-only fields should be omitted during serialization
        assert!(parsed.get("created_at").is_none());
    }

    #[test]
    fn test_collection_image_deserialization() {
        let json = r#"{
            "src": "https://example.com/image.png",
            "alt": "Alt text",
            "width": 500,
            "height": 400,
            "created_at": "2024-01-20T08:00:00Z"
        }"#;

        let image: CollectionImage = serde_json::from_str(json).unwrap();

        assert_eq!(image.src, Some("https://example.com/image.png".to_string()));
        assert_eq!(image.alt, Some("Alt text".to_string()));
        assert_eq!(image.width, Some(500));
        assert_eq!(image.height, Some(400));
        assert!(image.created_at.is_some());
    }

    #[test]
    fn test_collection_image_with_optional_fields_omitted() {
        let image = CollectionImage {
            src: Some("https://example.com/simple.jpg".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&image).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["src"], "https://example.com/simple.jpg");
        assert!(parsed.get("alt").is_none());
        assert!(parsed.get("width").is_none());
        assert!(parsed.get("height").is_none());
    }

    #[test]
    fn test_smart_collection_rule_serialization() {
        let rule = SmartCollectionRule {
            column: "tag".to_string(),
            relation: "equals".to_string(),
            condition: "summer".to_string(),
        };

        let json = serde_json::to_string(&rule).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["column"], "tag");
        assert_eq!(parsed["relation"], "equals");
        assert_eq!(parsed["condition"], "summer");
    }

    #[test]
    fn test_smart_collection_rule_deserialization() {
        let json = r#"{
            "column": "vendor",
            "relation": "contains",
            "condition": "Nike"
        }"#;

        let rule: SmartCollectionRule = serde_json::from_str(json).unwrap();

        assert_eq!(rule.column, "vendor");
        assert_eq!(rule.relation, "contains");
        assert_eq!(rule.condition, "Nike");
    }

    #[test]
    fn test_smart_collection_rule_with_price_condition() {
        let rule = SmartCollectionRule {
            column: "variant_price".to_string(),
            relation: "greater_than".to_string(),
            condition: "100".to_string(),
        };

        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: SmartCollectionRule = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, rule);
    }
}

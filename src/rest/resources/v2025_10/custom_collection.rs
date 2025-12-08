//! `CustomCollection` resource implementation.
//!
//! This module provides the [`CustomCollection`] resource for managing manually curated
//! product collections in Shopify.
//!
//! # Overview
//!
//! Custom collections are collections where products are manually added by the merchant.
//! They contrast with smart collections, which automatically include products based on rules.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::resources::v2025_10::{CustomCollection, CustomCollectionListParams};
//! use shopify_api::rest::RestResource;
//!
//! // Find a single custom collection
//! let collection = CustomCollection::find(&client, 123, None).await?;
//! println!("Collection: {}", collection.title.as_deref().unwrap_or(""));
//!
//! // List custom collections
//! let collections = CustomCollection::all(&client, None).await?;
//! for coll in collections.iter() {
//!     println!("- {}", coll.title.as_deref().unwrap_or(""));
//! }
//!
//! // Create a new custom collection
//! let collection = CustomCollection {
//!     title: Some("Summer Sale".to_string()),
//!     body_html: Some("<p>Best summer products</p>".to_string()),
//!     ..Default::default()
//! };
//! let saved = collection.save(&client).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::CollectionImage;

/// A manually curated collection of products.
///
/// Custom collections allow merchants to manually select which products to include
/// in the collection. Products are added to custom collections through "collects",
/// which are join records linking products to collections.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `handle` - The URL-friendly name (auto-generated from title)
/// - `created_at` - When the collection was created
/// - `updated_at` - When the collection was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `title` - The name of the collection
/// - `body_html` - The description in HTML format
/// - `published_at` - When the collection was/will be published
/// - `published_scope` - Where the collection is published ("web", "global")
/// - `sort_order` - How products are sorted in the collection
/// - `template_suffix` - The template suffix for the collection page
/// - `image` - The collection's featured image
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::rest::resources::v2025_10::CustomCollection;
///
/// let collection = CustomCollection {
///     title: Some("Featured Products".to_string()),
///     body_html: Some("<p>Our best-selling items</p>".to_string()),
///     sort_order: Some("best-selling".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CustomCollection {
    // --- Read-only fields ---
    /// The unique identifier of the custom collection.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The URL-friendly name of the collection.
    /// Automatically generated from the title if not specified.
    #[serde(skip_serializing)]
    pub handle: Option<String>,

    /// When the collection was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the collection was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this collection.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,

    // --- Writable fields ---
    /// The name of the collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The description of the collection in HTML format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,

    /// When the collection was or will be published.
    /// Set to null to unpublish.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,

    /// Where the collection is published.
    /// Valid values: "web", "global".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_scope: Option<String>,

    /// The order in which products appear in the collection.
    ///
    /// Valid values:
    /// - `alpha-asc` - Alphabetically, A-Z
    /// - `alpha-desc` - Alphabetically, Z-A
    /// - `best-selling` - By best-selling products
    /// - `created` - By date created, newest first
    /// - `created-desc` - By date created, oldest first
    /// - `manual` - Manual ordering
    /// - `price-asc` - By price, lowest to highest
    /// - `price-desc` - By price, highest to lowest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,

    /// The suffix of the Liquid template used for the collection page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_suffix: Option<String>,

    /// The collection's featured image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<CollectionImage>,

    /// Whether the collection is published.
    /// Convenience field returned by the API.
    #[serde(skip_serializing)]
    pub published: Option<bool>,
}

impl RestResource for CustomCollection {
    type Id = u64;
    type FindParams = CustomCollectionFindParams;
    type AllParams = CustomCollectionListParams;
    type CountParams = CustomCollectionCountParams;

    const NAME: &'static str = "CustomCollection";
    const PLURAL: &'static str = "custom_collections";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "custom_collections/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "custom_collections",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "custom_collections/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "custom_collections",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "custom_collections/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "custom_collections/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single custom collection.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CustomCollectionFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing custom collections.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CustomCollectionListParams {
    /// Return only collections with the given IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<u64>>,

    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return collections after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Filter by collection title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Filter by collection handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// Filter to collections containing this product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// Show collections updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show collections updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Filter by published status.
    /// Valid values: "published", "unpublished", "any".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_status: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

/// Parameters for counting custom collections.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CustomCollectionCountParams {
    /// Filter by collection title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Filter to collections containing this product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// Show collections updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show collections updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Filter by published status.
    /// Valid values: "published", "unpublished", "any".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_custom_collection_struct_serialization() {
        let collection = CustomCollection {
            id: Some(841564295),
            title: Some("Summer Collection".to_string()),
            body_html: Some("<p>Best summer products</p>".to_string()),
            handle: Some("summer-collection".to_string()),
            published_at: None,
            published_scope: Some("web".to_string()),
            sort_order: Some("best-selling".to_string()),
            template_suffix: Some("custom".to_string()),
            image: Some(CollectionImage {
                src: Some("https://cdn.shopify.com/collection.jpg".to_string()),
                alt: Some("Summer".to_string()),
                ..Default::default()
            }),
            created_at: None,
            updated_at: None,
            admin_graphql_api_id: Some("gid://shopify/Collection/841564295".to_string()),
            published: Some(true),
        };

        let json = serde_json::to_string(&collection).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["title"], "Summer Collection");
        assert_eq!(parsed["body_html"], "<p>Best summer products</p>");
        assert_eq!(parsed["published_scope"], "web");
        assert_eq!(parsed["sort_order"], "best-selling");
        assert_eq!(parsed["template_suffix"], "custom");
        assert!(parsed.get("image").is_some());

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("handle").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
        assert!(parsed.get("published").is_none());
    }

    #[test]
    fn test_custom_collection_deserialization_from_api_response() {
        let json = r#"{
            "id": 841564295,
            "handle": "ipods",
            "title": "IPods",
            "updated_at": "2024-01-02T09:28:43-05:00",
            "body_html": "<p>The best iPods</p>",
            "published_at": "2008-02-01T19:00:00-05:00",
            "sort_order": "manual",
            "template_suffix": null,
            "published_scope": "web",
            "admin_graphql_api_id": "gid://shopify/Collection/841564295",
            "image": {
                "src": "https://cdn.shopify.com/s/files/ipods.jpg",
                "alt": "iPods collection",
                "width": 1024,
                "height": 768,
                "created_at": "2024-01-01T10:00:00-05:00"
            }
        }"#;

        let collection: CustomCollection = serde_json::from_str(json).unwrap();

        assert_eq!(collection.id, Some(841564295));
        assert_eq!(collection.handle.as_deref(), Some("ipods"));
        assert_eq!(collection.title.as_deref(), Some("IPods"));
        assert_eq!(
            collection.body_html.as_deref(),
            Some("<p>The best iPods</p>")
        );
        assert_eq!(collection.sort_order.as_deref(), Some("manual"));
        assert_eq!(collection.published_scope.as_deref(), Some("web"));
        assert!(collection.published_at.is_some());
        assert!(collection.updated_at.is_some());
        assert_eq!(
            collection.admin_graphql_api_id.as_deref(),
            Some("gid://shopify/Collection/841564295")
        );

        // Check image
        let image = collection.image.unwrap();
        assert_eq!(
            image.src.as_deref(),
            Some("https://cdn.shopify.com/s/files/ipods.jpg")
        );
        assert_eq!(image.alt.as_deref(), Some("iPods collection"));
        assert_eq!(image.width, Some(1024));
        assert_eq!(image.height, Some(768));
    }

    #[test]
    fn test_custom_collection_path_constants_are_correct() {
        // Test Find path
        let find_path = get_path(CustomCollection::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "custom_collections/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(CustomCollection::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "custom_collections");

        // Test Count path
        let count_path = get_path(CustomCollection::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "custom_collections/count");

        // Test Create path
        let create_path = get_path(CustomCollection::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(CustomCollection::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test Delete path
        let delete_path = get_path(CustomCollection::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        // Verify constants
        assert_eq!(CustomCollection::NAME, "CustomCollection");
        assert_eq!(CustomCollection::PLURAL, "custom_collections");
    }

    #[test]
    fn test_custom_collection_get_id_returns_correct_value() {
        let collection_with_id = CustomCollection {
            id: Some(841564295),
            title: Some("Test Collection".to_string()),
            ..Default::default()
        };
        assert_eq!(collection_with_id.get_id(), Some(841564295));

        let collection_without_id = CustomCollection {
            id: None,
            title: Some("New Collection".to_string()),
            ..Default::default()
        };
        assert_eq!(collection_without_id.get_id(), None);
    }

    #[test]
    fn test_custom_collection_list_params_serialization() {
        let params = CustomCollectionListParams {
            ids: Some(vec![123, 456, 789]),
            limit: Some(50),
            since_id: Some(100),
            title: Some("Summer".to_string()),
            handle: Some("summer-sale".to_string()),
            product_id: Some(999),
            published_status: Some("published".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["ids"], serde_json::json!([123, 456, 789]));
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);
        assert_eq!(json["title"], "Summer");
        assert_eq!(json["handle"], "summer-sale");
        assert_eq!(json["product_id"], 999);
        assert_eq!(json["published_status"], "published");

        // Test empty params
        let empty_params = CustomCollectionListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_collection_image_handling() {
        // Test collection with image
        let collection = CustomCollection {
            title: Some("Image Test".to_string()),
            image: Some(CollectionImage {
                src: Some("https://example.com/image.jpg".to_string()),
                alt: Some("Collection image".to_string()),
                width: Some(800),
                height: Some(600),
                created_at: None,
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&collection).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let image = &parsed["image"];
        assert_eq!(image["src"], "https://example.com/image.jpg");
        assert_eq!(image["alt"], "Collection image");
        assert_eq!(image["width"], 800);
        assert_eq!(image["height"], 600);
        // created_at should be skipped in serialization
        assert!(image.get("created_at").is_none());
    }

    #[test]
    fn test_sort_order_field() {
        // Test various sort_order values
        let sort_orders = vec![
            "alpha-asc",
            "alpha-desc",
            "best-selling",
            "created",
            "created-desc",
            "manual",
            "price-asc",
            "price-desc",
        ];

        for sort_order in sort_orders {
            let collection = CustomCollection {
                title: Some("Test".to_string()),
                sort_order: Some(sort_order.to_string()),
                ..Default::default()
            };

            let json = serde_json::to_string(&collection).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

            assert_eq!(parsed["sort_order"], sort_order);
        }
    }

    #[test]
    fn test_custom_collection_count_params_serialization() {
        let params = CustomCollectionCountParams {
            title: Some("Summer".to_string()),
            product_id: Some(12345),
            published_status: Some("published".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["title"], "Summer");
        assert_eq!(json["product_id"], 12345);
        assert_eq!(json["published_status"], "published");

        let empty_params = CustomCollectionCountParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }
}

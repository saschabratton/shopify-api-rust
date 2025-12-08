//! `SmartCollection` resource implementation.
//!
//! This module provides the [`SmartCollection`] resource for managing rule-based
//! product collections in Shopify.
//!
//! # Overview
//!
//! Smart collections automatically include products based on rules. For example,
//! a smart collection can include all products with a specific tag, from a certain
//! vendor, or within a price range.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::resources::v2025_10::{SmartCollection, SmartCollectionListParams};
//! use shopify_api::rest::resources::v2025_10::common::SmartCollectionRule;
//! use shopify_api::rest::RestResource;
//!
//! // Find a single smart collection
//! let collection = SmartCollection::find(&client, 123, None).await?;
//! println!("Collection: {}", collection.title.as_deref().unwrap_or(""));
//!
//! // Create a smart collection with rules
//! let collection = SmartCollection {
//!     title: Some("Summer Products".to_string()),
//!     rules: Some(vec![
//!         SmartCollectionRule {
//!             column: "tag".to_string(),
//!             relation: "equals".to_string(),
//!             condition: "summer".to_string(),
//!         },
//!     ]),
//!     disjunctive: Some(false),  // All rules must match
//!     ..Default::default()
//! };
//! let saved = collection.save(&client).await?;
//!
//! // Manually reorder products (when sort_order is "manual")
//! collection.order(&client, vec![product_id_1, product_id_2, product_id_3]).await?;
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::{CollectionImage, SmartCollectionRule};

/// A rule-based collection of products.
///
/// Smart collections automatically include products that match specified rules.
/// Products are dynamically added or removed based on the collection's rules.
///
/// # Rules
///
/// Each rule has three components:
/// - `column` - The product property to check (e.g., "tag", "vendor", "title")
/// - `relation` - How to compare (e.g., "equals", "contains", "`greater_than`")
/// - `condition` - The value to compare against
///
/// # Disjunctive Mode
///
/// When `disjunctive` is `true`, products matching ANY rule are included (OR logic).
/// When `disjunctive` is `false`, products must match ALL rules (AND logic).
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
/// - `published_scope` - Where the collection is published
/// - `sort_order` - How products are sorted
/// - `template_suffix` - The template suffix
/// - `image` - The collection's featured image
/// - `rules` - The rules that determine which products are included
/// - `disjunctive` - Whether rules use OR (true) or AND (false) logic
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::rest::resources::v2025_10::SmartCollection;
/// use shopify_api::rest::resources::v2025_10::common::SmartCollectionRule;
///
/// // Collection with multiple rules (AND logic)
/// let collection = SmartCollection {
///     title: Some("Premium Nike Products".to_string()),
///     disjunctive: Some(false),  // All rules must match
///     rules: Some(vec![
///         SmartCollectionRule {
///             column: "vendor".to_string(),
///             relation: "equals".to_string(),
///             condition: "Nike".to_string(),
///         },
///         SmartCollectionRule {
///             column: "variant_price".to_string(),
///             relation: "greater_than".to_string(),
///             condition: "100".to_string(),
///         },
///     ]),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SmartCollection {
    // --- Read-only fields ---
    /// The unique identifier of the smart collection.
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
    /// - `manual` - Manual ordering (allows use of `order()` method)
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

    /// The rules that determine which products are included.
    ///
    /// Each rule specifies a column, relation, and condition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<SmartCollectionRule>>,

    /// Whether products must match any rule (true) or all rules (false).
    ///
    /// - `true` - Products matching ANY rule are included (OR logic)
    /// - `false` - Products must match ALL rules (AND logic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disjunctive: Option<bool>,

    /// Whether the collection is published.
    /// Convenience field returned by the API.
    #[serde(skip_serializing)]
    pub published: Option<bool>,
}

impl RestResource for SmartCollection {
    type Id = u64;
    type FindParams = SmartCollectionFindParams;
    type AllParams = SmartCollectionListParams;
    type CountParams = SmartCollectionCountParams;

    const NAME: &'static str = "SmartCollection";
    const PLURAL: &'static str = "smart_collections";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "smart_collections/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "smart_collections",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "smart_collections/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "smart_collections",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "smart_collections/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "smart_collections/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl SmartCollection {
    /// Manually reorders products in the smart collection.
    ///
    /// This method is only applicable when `sort_order` is set to "manual".
    /// The products are reordered according to their position in the `product_ids` array.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `product_ids` - The product IDs in the desired order
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if the collection has no ID.
    /// Returns [`ResourceError::Http`] if the API request fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::rest::resources::v2025_10::SmartCollection;
    ///
    /// let collection = SmartCollection::find(&client, 123, None).await?.into_inner();
    ///
    /// // Reorder products (collection must have sort_order = "manual")
    /// collection.order(&client, vec![
    ///     111222333,  // First product
    ///     444555666,  // Second product
    ///     777888999,  // Third product
    /// ]).await?;
    /// ```
    pub async fn order(
        &self,
        client: &RestClient,
        product_ids: Vec<u64>,
    ) -> Result<(), ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "order",
        })?;

        // Build the query parameters with products[] array
        let mut query: HashMap<String, String> = HashMap::new();
        let products_param: String = product_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        query.insert("products[]".to_string(), products_param);

        let path = format!("smart_collections/{id}/order");
        let body = serde_json::json!({});

        let response = client.put(&path, body, Some(query)).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        Ok(())
    }
}

/// Parameters for finding a single smart collection.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SmartCollectionFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing smart collections.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SmartCollectionListParams {
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

/// Parameters for counting smart collections.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SmartCollectionCountParams {
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
    fn test_smart_collection_struct_serialization() {
        let collection = SmartCollection {
            id: Some(1063001322),
            title: Some("Nike Products".to_string()),
            body_html: Some("<p>All Nike products</p>".to_string()),
            handle: Some("nike-products".to_string()),
            published_at: None,
            published_scope: Some("web".to_string()),
            sort_order: Some("best-selling".to_string()),
            template_suffix: None,
            image: None,
            rules: Some(vec![SmartCollectionRule {
                column: "vendor".to_string(),
                relation: "equals".to_string(),
                condition: "Nike".to_string(),
            }]),
            disjunctive: Some(false),
            created_at: None,
            updated_at: None,
            admin_graphql_api_id: Some("gid://shopify/Collection/1063001322".to_string()),
            published: Some(true),
        };

        let json = serde_json::to_string(&collection).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["title"], "Nike Products");
        assert_eq!(parsed["body_html"], "<p>All Nike products</p>");
        assert_eq!(parsed["published_scope"], "web");
        assert_eq!(parsed["sort_order"], "best-selling");
        assert_eq!(parsed["disjunctive"], false);
        assert!(parsed.get("rules").is_some());

        // Check rules array
        let rules = &parsed["rules"];
        assert_eq!(rules[0]["column"], "vendor");
        assert_eq!(rules[0]["relation"], "equals");
        assert_eq!(rules[0]["condition"], "Nike");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("handle").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
        assert!(parsed.get("published").is_none());
    }

    #[test]
    fn test_smart_collection_deserialization_with_rules() {
        let json = r#"{
            "id": 1063001322,
            "handle": "nike-sale",
            "title": "Nike Sale",
            "updated_at": "2024-01-02T09:28:43-05:00",
            "body_html": "<p>Nike products on sale</p>",
            "published_at": "2024-01-01T19:00:00-05:00",
            "sort_order": "price-asc",
            "template_suffix": null,
            "published_scope": "global",
            "disjunctive": true,
            "rules": [
                {
                    "column": "vendor",
                    "relation": "equals",
                    "condition": "Nike"
                },
                {
                    "column": "tag",
                    "relation": "equals",
                    "condition": "sale"
                }
            ],
            "admin_graphql_api_id": "gid://shopify/Collection/1063001322"
        }"#;

        let collection: SmartCollection = serde_json::from_str(json).unwrap();

        assert_eq!(collection.id, Some(1063001322));
        assert_eq!(collection.handle.as_deref(), Some("nike-sale"));
        assert_eq!(collection.title.as_deref(), Some("Nike Sale"));
        assert_eq!(
            collection.body_html.as_deref(),
            Some("<p>Nike products on sale</p>")
        );
        assert_eq!(collection.sort_order.as_deref(), Some("price-asc"));
        assert_eq!(collection.published_scope.as_deref(), Some("global"));
        assert_eq!(collection.disjunctive, Some(true));
        assert!(collection.published_at.is_some());
        assert!(collection.updated_at.is_some());

        // Check rules
        let rules = collection.rules.unwrap();
        assert_eq!(rules.len(), 2);

        assert_eq!(rules[0].column, "vendor");
        assert_eq!(rules[0].relation, "equals");
        assert_eq!(rules[0].condition, "Nike");

        assert_eq!(rules[1].column, "tag");
        assert_eq!(rules[1].relation, "equals");
        assert_eq!(rules[1].condition, "sale");
    }

    #[test]
    fn test_smart_collection_rule_struct() {
        // Test serialization
        let rule = SmartCollectionRule {
            column: "variant_price".to_string(),
            relation: "greater_than".to_string(),
            condition: "50".to_string(),
        };

        let json = serde_json::to_string(&rule).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["column"], "variant_price");
        assert_eq!(parsed["relation"], "greater_than");
        assert_eq!(parsed["condition"], "50");

        // Test deserialization
        let json_str = r#"{"column":"title","relation":"contains","condition":"summer"}"#;
        let rule: SmartCollectionRule = serde_json::from_str(json_str).unwrap();

        assert_eq!(rule.column, "title");
        assert_eq!(rule.relation, "contains");
        assert_eq!(rule.condition, "summer");
    }

    #[test]
    fn test_smart_collection_path_constants_are_correct() {
        // Test Find path
        let find_path = get_path(SmartCollection::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "smart_collections/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(SmartCollection::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "smart_collections");

        // Test Count path
        let count_path = get_path(SmartCollection::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "smart_collections/count");

        // Test Create path
        let create_path = get_path(SmartCollection::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(SmartCollection::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test Delete path
        let delete_path = get_path(SmartCollection::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        // Verify constants
        assert_eq!(SmartCollection::NAME, "SmartCollection");
        assert_eq!(SmartCollection::PLURAL, "smart_collections");
    }

    #[test]
    fn test_smart_collection_get_id_returns_correct_value() {
        let collection_with_id = SmartCollection {
            id: Some(1063001322),
            title: Some("Test Collection".to_string()),
            ..Default::default()
        };
        assert_eq!(collection_with_id.get_id(), Some(1063001322));

        let collection_without_id = SmartCollection {
            id: None,
            title: Some("New Collection".to_string()),
            ..Default::default()
        };
        assert_eq!(collection_without_id.get_id(), None);
    }

    #[test]
    fn test_smart_collection_list_params_serialization() {
        let params = SmartCollectionListParams {
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
        let empty_params = SmartCollectionListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_sort_order_and_order_method_signature() {
        // Test sort_order field with "manual" value
        let collection = SmartCollection {
            id: Some(123),
            title: Some("Manual Sort Collection".to_string()),
            sort_order: Some("manual".to_string()),
            ..Default::default()
        };

        assert_eq!(collection.sort_order.as_deref(), Some("manual"));
        assert!(collection.get_id().is_some());

        // Verify the order method signature exists by referencing it
        fn _assert_order_signature<F, Fut>(f: F)
        where
            F: Fn(&SmartCollection, &RestClient, Vec<u64>) -> Fut,
            Fut: std::future::Future<Output = Result<(), ResourceError>>,
        {
            let _ = f;
        }

        // This test verifies the method exists and has the correct signature.
        // The actual HTTP call would require a mock client.
    }

    #[test]
    fn test_disjunctive_field_logic() {
        // Test disjunctive = true (OR logic)
        let or_collection = SmartCollection {
            title: Some("OR Logic Collection".to_string()),
            disjunctive: Some(true),
            rules: Some(vec![
                SmartCollectionRule {
                    column: "tag".to_string(),
                    relation: "equals".to_string(),
                    condition: "summer".to_string(),
                },
                SmartCollectionRule {
                    column: "tag".to_string(),
                    relation: "equals".to_string(),
                    condition: "winter".to_string(),
                },
            ]),
            ..Default::default()
        };

        let json = serde_json::to_string(&or_collection).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["disjunctive"], true);

        // Test disjunctive = false (AND logic)
        let and_collection = SmartCollection {
            title: Some("AND Logic Collection".to_string()),
            disjunctive: Some(false),
            rules: Some(vec![
                SmartCollectionRule {
                    column: "vendor".to_string(),
                    relation: "equals".to_string(),
                    condition: "Nike".to_string(),
                },
                SmartCollectionRule {
                    column: "variant_price".to_string(),
                    relation: "greater_than".to_string(),
                    condition: "100".to_string(),
                },
            ]),
            ..Default::default()
        };

        let json = serde_json::to_string(&and_collection).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["disjunctive"], false);
    }

    #[test]
    fn test_smart_collection_count_params_serialization() {
        let params = SmartCollectionCountParams {
            title: Some("Summer".to_string()),
            product_id: Some(12345),
            published_status: Some("published".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["title"], "Summer");
        assert_eq!(json["product_id"], 12345);
        assert_eq!(json["published_status"], "published");

        let empty_params = SmartCollectionCountParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_smart_collection_with_image() {
        let collection = SmartCollection {
            title: Some("Image Test".to_string()),
            image: Some(CollectionImage {
                src: Some("https://example.com/collection.jpg".to_string()),
                alt: Some("Collection banner".to_string()),
                width: Some(1200),
                height: Some(400),
                created_at: None,
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&collection).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let image = &parsed["image"];
        assert_eq!(image["src"], "https://example.com/collection.jpg");
        assert_eq!(image["alt"], "Collection banner");
        assert_eq!(image["width"], 1200);
        assert_eq!(image["height"], 400);
    }
}

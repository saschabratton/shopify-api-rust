//! Collect resource implementation.
//!
//! This module provides the [`Collect`] resource for managing the connections
//! between products and custom collections.
//!
//! # What is a Collect?
//!
//! A Collect is a join table that connects a product to a custom collection.
//! Each collect represents one product-collection relationship.
//!
//! # Limited Operations
//!
//! Collect has limited operations:
//! - **Create**: Add a product to a collection
//! - **Find**: Get a specific collect
//! - **List**: List collects (filterable by product or collection)
//! - **Count**: Count collects
//! - **Delete**: Remove a product from a collection
//!
//! **No Update operation** - to change a collect, delete and recreate it.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{Collect, CollectListParams};
//!
//! // Add a product to a collection
//! let collect = Collect {
//!     product_id: Some(632910392),
//!     collection_id: Some(841564295),
//!     ..Default::default()
//! };
//! let saved = collect.save(&client).await?;
//!
//! // List all products in a collection
//! let params = CollectListParams {
//!     collection_id: Some(841564295),
//!     ..Default::default()
//! };
//! let collects = Collect::all(&client, Some(params)).await?;
//!
//! // Remove a product from a collection
//! Collect::delete(&client, collect_id).await?;
//! ```

use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A connection between a product and a custom collection.
///
/// Collects represent the many-to-many relationship between products
/// and custom collections. Smart collections manage their own products
/// automatically based on conditions.
///
/// # Limited Operations
///
/// - **Create**: Yes - add product to collection
/// - **Find**: Yes - get specific collect
/// - **List**: Yes - filterable by product_id or collection_id
/// - **Count**: Yes - filterable by product_id or collection_id
/// - **Update**: No - delete and recreate instead
/// - **Delete**: Yes - remove product from collection
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `created_at` - When the collect was created
/// - `updated_at` - When the collect was last updated
///
/// ## Writable Fields
/// - `product_id` - The ID of the product
/// - `collection_id` - The ID of the custom collection
/// - `position` - The position of the product in the collection
/// - `sort_value` - The sort value for the product
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Collect {
    /// The unique identifier of the collect.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the product in this collect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// The ID of the custom collection containing the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<u64>,

    /// The position of the product in the collection.
    /// Products are sorted by position in ascending order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,

    /// A string used for sorting when the collection is sorted manually.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_value: Option<String>,

    /// When the collect was created.
    #[serde(skip_serializing)]
    pub created_at: Option<String>,

    /// When the collect was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<String>,
}

impl RestResource for Collect {
    type Id = u64;
    type FindParams = CollectFindParams;
    type AllParams = CollectListParams;
    type CountParams = CollectCountParams;

    const NAME: &'static str = "Collect";
    const PLURAL: &'static str = "collects";

    /// Paths for the Collect resource.
    ///
    /// Limited operations: No Update.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "collects/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "collects"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "collects/count",
        ),
        ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "collects"),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "collects/{id}",
        ),
        // Note: No Update path - collects cannot be modified, only created or deleted
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single collect.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CollectFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing collects.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CollectListParams {
    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return collects after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Filter by product ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// Filter by collection ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting collects.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CollectCountParams {
    /// Filter by product ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// Filter by collection ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation, RestResource};

    #[test]
    fn test_collect_serialization() {
        let collect = Collect {
            id: Some(455204334),
            product_id: Some(632910392),
            collection_id: Some(841564295),
            position: Some(1),
            sort_value: Some("0000000001".to_string()),
            created_at: Some("2024-01-15T10:30:00-05:00".to_string()),
            updated_at: Some("2024-01-15T10:30:00-05:00".to_string()),
        };

        let json = serde_json::to_string(&collect).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["product_id"], 632910392);
        assert_eq!(parsed["collection_id"], 841564295);
        assert_eq!(parsed["position"], 1);
        assert_eq!(parsed["sort_value"], "0000000001");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_collect_deserialization() {
        let json = r#"{
            "id": 455204334,
            "product_id": 632910392,
            "collection_id": 841564295,
            "position": 1,
            "sort_value": "0000000001",
            "created_at": "2024-01-15T10:30:00-05:00",
            "updated_at": "2024-01-15T10:30:00-05:00"
        }"#;

        let collect: Collect = serde_json::from_str(json).unwrap();

        assert_eq!(collect.id, Some(455204334));
        assert_eq!(collect.product_id, Some(632910392));
        assert_eq!(collect.collection_id, Some(841564295));
        assert_eq!(collect.position, Some(1));
        assert_eq!(collect.sort_value, Some("0000000001".to_string()));
        assert_eq!(
            collect.created_at,
            Some("2024-01-15T10:30:00-05:00".to_string())
        );
    }

    #[test]
    fn test_collect_limited_paths_no_update() {
        // Find
        let find_path = get_path(Collect::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "collects/{id}");

        // All
        let all_path = get_path(Collect::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "collects");

        // Count
        let count_path = get_path(Collect::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "collects/count");

        // Create
        let create_path = get_path(Collect::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "collects");

        // Delete
        let delete_path = get_path(Collect::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "collects/{id}");

        // No Update (the key differentiator)
        let update_path = get_path(Collect::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());
    }

    #[test]
    fn test_collect_constants() {
        assert_eq!(Collect::NAME, "Collect");
        assert_eq!(Collect::PLURAL, "collects");
    }

    #[test]
    fn test_collect_get_id() {
        let collect_with_id = Collect {
            id: Some(455204334),
            product_id: Some(632910392),
            collection_id: Some(841564295),
            ..Default::default()
        };
        assert_eq!(collect_with_id.get_id(), Some(455204334));

        let collect_without_id = Collect::default();
        assert_eq!(collect_without_id.get_id(), None);
    }

    #[test]
    fn test_collect_list_params() {
        let params = CollectListParams {
            limit: Some(50),
            since_id: Some(1000),
            product_id: Some(632910392),
            collection_id: Some(841564295),
            fields: Some("id,product_id,collection_id".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 1000);
        assert_eq!(json["product_id"], 632910392);
        assert_eq!(json["collection_id"], 841564295);
        assert_eq!(json["fields"], "id,product_id,collection_id");
    }

    #[test]
    fn test_collect_count_params() {
        let params = CollectCountParams {
            product_id: Some(632910392),
            collection_id: None,
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["product_id"], 632910392);
        assert!(json.get("collection_id").is_none());
    }
}

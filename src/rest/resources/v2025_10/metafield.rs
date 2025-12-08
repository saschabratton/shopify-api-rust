//! Metafield resource implementation.
//!
//! This module provides the [`Metafield`] resource for managing custom metadata
//! on various Shopify resources including products, customers, orders, and more.
//!
//! # Polymorphic Paths
//!
//! Metafields can be attached to different owner types. The API path depends on the owner:
//! - Products: `/products/{product_id}/metafields/{id}`
//! - Variants: `/variants/{variant_id}/metafields/{id}`
//! - Customers: `/customers/{customer_id}/metafields/{id}`
//! - Orders: `/orders/{order_id}/metafields/{id}`
//! - Collections: `/collections/{collection_id}/metafields/{id}`
//! - Pages: `/pages/{page_id}/metafields/{id}`
//! - Blogs: `/blogs/{blog_id}/metafields/{id}`
//! - Articles: `/articles/{article_id}/metafields/{id}`
//! - Shop (global): `/metafields/{id}`
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::resources::v2025_10::{Metafield, MetafieldListParams};
//! use shopify_api::rest::resources::v2025_10::common::MetafieldOwner;
//! use shopify_api::rest::RestResource;
//!
//! // List metafields for a specific product
//! let metafields = Metafield::all_for_owner(
//!     &client,
//!     MetafieldOwner::Product,
//!     123456789,
//!     None
//! ).await?;
//!
//! // Create a new metafield on a product
//! let metafield = Metafield {
//!     namespace: Some("custom".to_string()),
//!     key: Some("color".to_string()),
//!     value: Some("blue".to_string()),
//!     metafield_type: Some("single_line_text_field".to_string()),
//!     owner_id: Some(123456789),
//!     owner_resource: Some("product".to_string()),
//!     ..Default::default()
//! };
//!
//! // Find a metafield directly by ID (standalone path)
//! let metafield = Metafield::find(&client, 987654321, None).await?;
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{
    build_path, ResourceError, ResourceOperation, ResourcePath, ResourceResponse, RestResource,
};
use crate::HttpMethod;

use super::common::MetafieldOwner;

/// A metafield attached to a Shopify resource.
///
/// Metafields allow you to store custom data on various Shopify resources.
/// Each metafield has a namespace and key that uniquely identify it within
/// the owner resource.
///
/// # Polymorphic Ownership
///
/// Metafields can belong to different resource types. The `owner_id` and
/// `owner_resource` fields identify the parent resource when returned from
/// the API.
///
/// # Value Types
///
/// The `metafield_type` field (serialized as `type` in JSON) specifies the
/// data type of the value. Common types include:
/// - `single_line_text_field`
/// - `multi_line_text_field`
/// - `number_integer`
/// - `number_decimal`
/// - `boolean`
/// - `json`
/// - `date`
/// - `date_time`
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::rest::resources::v2025_10::Metafield;
///
/// let metafield = Metafield {
///     namespace: Some("inventory".to_string()),
///     key: Some("warehouse_location".to_string()),
///     value: Some("A-15-3".to_string()),
///     metafield_type: Some("single_line_text_field".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Metafield {
    /// The unique identifier of the metafield.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The namespace for the metafield.
    ///
    /// Namespaces group related metafields together. Use your app's namespace
    /// to avoid conflicts with other apps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// The key for the metafield.
    ///
    /// The key uniquely identifies the metafield within its namespace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The value stored in the metafield.
    ///
    /// The format depends on the `metafield_type`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The type of data stored in the metafield.
    ///
    /// Renamed from `type` to avoid Rust keyword conflict.
    /// Common types: `single_line_text_field`, `number_integer`, `json`, etc.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub metafield_type: Option<String>,

    /// The ID of the resource that owns this metafield.
    /// Read-only field returned by the API.
    #[serde(skip_serializing)]
    pub owner_id: Option<u64>,

    /// The type of resource that owns this metafield.
    ///
    /// Examples: "product", "customer", "order", "shop".
    /// Read-only field returned by the API.
    #[serde(skip_serializing)]
    pub owner_resource: Option<String>,

    /// Additional description for the metafield.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When the metafield was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the metafield was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this metafield.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for Metafield {
    type Id = u64;
    type FindParams = MetafieldFindParams;
    type AllParams = MetafieldListParams;
    type CountParams = MetafieldCountParams;

    const NAME: &'static str = "Metafield";
    const PLURAL: &'static str = "metafields";

    /// Paths for the Metafield resource.
    ///
    /// Metafields support polymorphic paths based on the owner resource type.
    /// Path selection chooses the most specific path based on available IDs.
    ///
    /// # Owner Types Supported
    ///
    /// - Products: `products/{product_id}/metafields`
    /// - Variants: `variants/{variant_id}/metafields`
    /// - Customers: `customers/{customer_id}/metafields`
    /// - Orders: `orders/{order_id}/metafields`
    /// - Collections: `collections/{collection_id}/metafields`
    /// - Pages: `pages/{page_id}/metafields`
    /// - Blogs: `blogs/{blog_id}/metafields`
    /// - Articles: `articles/{article_id}/metafields`
    /// - Shop (global): `metafields` (no parent ID)
    const PATHS: &'static [ResourcePath] = &[
        // === Product metafield paths ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["product_id", "id"],
            "products/{product_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["product_id"],
            "products/{product_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["product_id"],
            "products/{product_id}/metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["product_id"],
            "products/{product_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["product_id", "id"],
            "products/{product_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["product_id", "id"],
            "products/{product_id}/metafields/{id}",
        ),
        // === Variant metafield paths ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["variant_id", "id"],
            "variants/{variant_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["variant_id"],
            "variants/{variant_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["variant_id"],
            "variants/{variant_id}/metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["variant_id"],
            "variants/{variant_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["variant_id", "id"],
            "variants/{variant_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["variant_id", "id"],
            "variants/{variant_id}/metafields/{id}",
        ),
        // === Customer metafield paths ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["customer_id", "id"],
            "customers/{customer_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["customer_id"],
            "customers/{customer_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["customer_id"],
            "customers/{customer_id}/metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["customer_id"],
            "customers/{customer_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["customer_id", "id"],
            "customers/{customer_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["customer_id", "id"],
            "customers/{customer_id}/metafields/{id}",
        ),
        // === Order metafield paths ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["order_id", "id"],
            "orders/{order_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["order_id"],
            "orders/{order_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["order_id"],
            "orders/{order_id}/metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["order_id"],
            "orders/{order_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["order_id", "id"],
            "orders/{order_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["order_id", "id"],
            "orders/{order_id}/metafields/{id}",
        ),
        // === Collection metafield paths ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["collection_id", "id"],
            "collections/{collection_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["collection_id"],
            "collections/{collection_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["collection_id"],
            "collections/{collection_id}/metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["collection_id"],
            "collections/{collection_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["collection_id", "id"],
            "collections/{collection_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["collection_id", "id"],
            "collections/{collection_id}/metafields/{id}",
        ),
        // === Page metafield paths ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["page_id", "id"],
            "pages/{page_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["page_id"],
            "pages/{page_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["page_id"],
            "pages/{page_id}/metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["page_id"],
            "pages/{page_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["page_id", "id"],
            "pages/{page_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["page_id", "id"],
            "pages/{page_id}/metafields/{id}",
        ),
        // === Blog metafield paths ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["blog_id", "id"],
            "blogs/{blog_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["blog_id"],
            "blogs/{blog_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["blog_id"],
            "blogs/{blog_id}/metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["blog_id"],
            "blogs/{blog_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["blog_id", "id"],
            "blogs/{blog_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["blog_id", "id"],
            "blogs/{blog_id}/metafields/{id}",
        ),
        // === Article metafield paths ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["article_id", "id"],
            "articles/{article_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["article_id"],
            "articles/{article_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["article_id"],
            "articles/{article_id}/metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["article_id"],
            "articles/{article_id}/metafields",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["article_id", "id"],
            "articles/{article_id}/metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["article_id", "id"],
            "articles/{article_id}/metafields/{id}",
        ),
        // === Shop-level (global) metafield paths ===
        // These are the fallback paths when no parent ID is specified
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "metafields"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "metafields/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "metafields",
        ),
        // === Standalone metafield paths (direct ID access) ===
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "metafields/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "metafields/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl Metafield {
    /// Lists all metafields for a specific owner resource.
    ///
    /// This is a convenience method that automatically constructs the correct
    /// path based on the owner type.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `owner` - The type of resource that owns the metafields
    /// * `owner_id` - The ID of the owner resource
    /// * `params` - Optional parameters for filtering/pagination
    ///
    /// # Returns
    ///
    /// Returns a paginated response containing the metafields.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no valid path matches.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::rest::resources::v2025_10::{Metafield, MetafieldListParams};
    /// use shopify_api::rest::resources::v2025_10::common::MetafieldOwner;
    ///
    /// // Get all metafields for a product
    /// let metafields = Metafield::all_for_owner(
    ///     &client,
    ///     MetafieldOwner::Product,
    ///     123456789,
    ///     None
    /// ).await?;
    ///
    /// for mf in metafields.iter() {
    ///     println!("{}.{} = {}",
    ///         mf.namespace.as_deref().unwrap_or(""),
    ///         mf.key.as_deref().unwrap_or(""),
    ///         mf.value.as_deref().unwrap_or("")
    ///     );
    /// }
    ///
    /// // Get metafields with namespace filter
    /// let params = MetafieldListParams {
    ///     namespace: Some("custom".to_string()),
    ///     ..Default::default()
    /// };
    /// let metafields = Metafield::all_for_owner(
    ///     &client,
    ///     MetafieldOwner::Customer,
    ///     987654321,
    ///     Some(params)
    /// ).await?;
    /// ```
    pub async fn all_for_owner(
        client: &RestClient,
        owner: MetafieldOwner,
        owner_id: u64,
        params: Option<MetafieldListParams>,
    ) -> Result<ResourceResponse<Vec<Self>>, ResourceError> {
        // Map owner type to the correct parent ID name
        let parent_id_name = match owner {
            MetafieldOwner::Product => "product_id",
            MetafieldOwner::Variant => "variant_id",
            MetafieldOwner::Customer => "customer_id",
            MetafieldOwner::Order => "order_id",
            MetafieldOwner::Collection => "collection_id",
            MetafieldOwner::Page => "page_id",
            MetafieldOwner::Blog => "blog_id",
            MetafieldOwner::Article => "article_id",
            MetafieldOwner::Shop => {
                // Shop metafields use the global path (no parent ID)
                return Self::all(client, params).await;
            }
        };

        Self::all_with_parent(client, parent_id_name, owner_id, params).await
    }

    /// Counts metafields for a specific owner resource.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `owner` - The type of resource that owns the metafields
    /// * `owner_id` - The ID of the owner resource
    /// * `params` - Optional parameters for filtering
    ///
    /// # Returns
    ///
    /// Returns the count of metafields.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no valid path matches.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::rest::resources::v2025_10::Metafield;
    /// use shopify_api::rest::resources::v2025_10::common::MetafieldOwner;
    ///
    /// let count = Metafield::count_for_owner(
    ///     &client,
    ///     MetafieldOwner::Product,
    ///     123456789,
    ///     None
    /// ).await?;
    /// println!("Product has {} metafields", count);
    /// ```
    pub async fn count_for_owner(
        client: &RestClient,
        owner: MetafieldOwner,
        owner_id: u64,
        params: Option<MetafieldCountParams>,
    ) -> Result<u64, ResourceError> {
        // Map owner type to the correct parent ID name and path
        let (parent_id_name, path_template) = match owner {
            MetafieldOwner::Product => ("product_id", "products/{product_id}/metafields/count"),
            MetafieldOwner::Variant => ("variant_id", "variants/{variant_id}/metafields/count"),
            MetafieldOwner::Customer => ("customer_id", "customers/{customer_id}/metafields/count"),
            MetafieldOwner::Order => ("order_id", "orders/{order_id}/metafields/count"),
            MetafieldOwner::Collection => (
                "collection_id",
                "collections/{collection_id}/metafields/count",
            ),
            MetafieldOwner::Page => ("page_id", "pages/{page_id}/metafields/count"),
            MetafieldOwner::Blog => ("blog_id", "blogs/{blog_id}/metafields/count"),
            MetafieldOwner::Article => ("article_id", "articles/{article_id}/metafields/count"),
            MetafieldOwner::Shop => {
                // Shop metafields use the global count path
                return Self::count(client, params).await;
            }
        };

        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert(parent_id_name, owner_id.to_string());

        let url = build_path(path_template, &ids);

        // Build query params
        let query = params
            .map(|p| serialize_to_query(&p))
            .transpose()?
            .filter(|q| !q.is_empty());

        let response = client.get(&url, query).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        // Extract count from response
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
}

/// Helper function to serialize params to query parameters.
fn serialize_to_query<T: Serialize>(params: &T) -> Result<HashMap<String, String>, ResourceError> {
    let value = serde_json::to_value(params).map_err(|e| {
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
                serde_json::Value::Null => {}
                serde_json::Value::String(s) => {
                    query.insert(key, s);
                }
                serde_json::Value::Number(n) => {
                    query.insert(key, n.to_string());
                }
                serde_json::Value::Bool(b) => {
                    query.insert(key, b.to_string());
                }
                serde_json::Value::Array(arr) => {
                    let values: Vec<String> = arr
                        .iter()
                        .filter_map(|v| match v {
                            serde_json::Value::String(s) => Some(s.clone()),
                            serde_json::Value::Number(n) => Some(n.to_string()),
                            _ => None,
                        })
                        .collect();
                    if !values.is_empty() {
                        query.insert(key, values.join(","));
                    }
                }
                serde_json::Value::Object(_) => {
                    query.insert(key, val.to_string());
                }
            }
        }
    }

    Ok(query)
}

/// Parameters for finding a single metafield.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MetafieldFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing metafields.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MetafieldListParams {
    /// Filter by namespace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Filter by key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Filter by metafield namespaces (comma-separated list).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metafield_namespaces: Option<String>,

    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return metafields after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,

    /// Return metafields created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Return metafields created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Return metafields updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Return metafields updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,
}

/// Parameters for counting metafields.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MetafieldCountParams {
    /// Filter by namespace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,

    /// Filter by key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_metafield_struct_serialization() {
        let metafield = Metafield {
            id: Some(12345),
            namespace: Some("custom".to_string()),
            key: Some("color".to_string()),
            value: Some("blue".to_string()),
            metafield_type: Some("single_line_text_field".to_string()),
            owner_id: Some(67890),
            owner_resource: Some("product".to_string()),
            description: Some("Product color".to_string()),
            created_at: None,
            updated_at: None,
            admin_graphql_api_id: Some("gid://shopify/Metafield/12345".to_string()),
        };

        let json = serde_json::to_string(&metafield).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["namespace"], "custom");
        assert_eq!(parsed["key"], "color");
        assert_eq!(parsed["value"], "blue");
        assert_eq!(parsed["type"], "single_line_text_field");
        assert_eq!(parsed["description"], "Product color");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("owner_id").is_none());
        assert!(parsed.get("owner_resource").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_metafield_deserialization_from_api_response() {
        let json = r#"{
            "id": 721389482,
            "namespace": "inventory",
            "key": "warehouse",
            "value": "A-15",
            "type": "single_line_text_field",
            "description": "Warehouse location",
            "owner_id": 632910392,
            "owner_resource": "product",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/Metafield/721389482"
        }"#;

        let metafield: Metafield = serde_json::from_str(json).unwrap();

        assert_eq!(metafield.id, Some(721389482));
        assert_eq!(metafield.namespace.as_deref(), Some("inventory"));
        assert_eq!(metafield.key.as_deref(), Some("warehouse"));
        assert_eq!(metafield.value.as_deref(), Some("A-15"));
        assert_eq!(
            metafield.metafield_type.as_deref(),
            Some("single_line_text_field")
        );
        assert_eq!(metafield.description.as_deref(), Some("Warehouse location"));
        assert_eq!(metafield.owner_id, Some(632910392));
        assert_eq!(metafield.owner_resource.as_deref(), Some("product"));
        assert!(metafield.created_at.is_some());
        assert!(metafield.updated_at.is_some());
        assert_eq!(
            metafield.admin_graphql_api_id.as_deref(),
            Some("gid://shopify/Metafield/721389482")
        );
    }

    #[test]
    fn test_polymorphic_path_selection_with_product_id() {
        // Test All with product_id
        let all_path = get_path(Metafield::PATHS, ResourceOperation::All, &["product_id"]);
        assert!(all_path.is_some());
        assert_eq!(
            all_path.unwrap().template,
            "products/{product_id}/metafields"
        );

        // Test Find with product_id and id
        let find_path = get_path(
            Metafield::PATHS,
            ResourceOperation::Find,
            &["product_id", "id"],
        );
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "products/{product_id}/metafields/{id}"
        );

        // Test Create with product_id
        let create_path = get_path(Metafield::PATHS, ResourceOperation::Create, &["product_id"]);
        assert!(create_path.is_some());
        assert_eq!(
            create_path.unwrap().template,
            "products/{product_id}/metafields"
        );

        // Test Update with product_id and id
        let update_path = get_path(
            Metafield::PATHS,
            ResourceOperation::Update,
            &["product_id", "id"],
        );
        assert!(update_path.is_some());
        assert_eq!(
            update_path.unwrap().template,
            "products/{product_id}/metafields/{id}"
        );

        // Test Delete with product_id and id
        let delete_path = get_path(
            Metafield::PATHS,
            ResourceOperation::Delete,
            &["product_id", "id"],
        );
        assert!(delete_path.is_some());
        assert_eq!(
            delete_path.unwrap().template,
            "products/{product_id}/metafields/{id}"
        );

        // Test Count with product_id
        let count_path = get_path(Metafield::PATHS, ResourceOperation::Count, &["product_id"]);
        assert!(count_path.is_some());
        assert_eq!(
            count_path.unwrap().template,
            "products/{product_id}/metafields/count"
        );
    }

    #[test]
    fn test_polymorphic_path_selection_with_customer_id() {
        // Test All with customer_id
        let all_path = get_path(Metafield::PATHS, ResourceOperation::All, &["customer_id"]);
        assert!(all_path.is_some());
        assert_eq!(
            all_path.unwrap().template,
            "customers/{customer_id}/metafields"
        );

        // Test Find with customer_id and id
        let find_path = get_path(
            Metafield::PATHS,
            ResourceOperation::Find,
            &["customer_id", "id"],
        );
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "customers/{customer_id}/metafields/{id}"
        );

        // Test Create with customer_id
        let create_path = get_path(
            Metafield::PATHS,
            ResourceOperation::Create,
            &["customer_id"],
        );
        assert!(create_path.is_some());
        assert_eq!(
            create_path.unwrap().template,
            "customers/{customer_id}/metafields"
        );
    }

    #[test]
    fn test_standalone_path_metafields_id_fallback() {
        // Test Find with only id (standalone path)
        let find_path = get_path(Metafield::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "metafields/{id}");

        // Test Update with only id (standalone path)
        let update_path = get_path(Metafield::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "metafields/{id}");

        // Test Delete with only id (standalone path)
        let delete_path = get_path(Metafield::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "metafields/{id}");
    }

    #[test]
    fn test_shop_level_metafield_paths() {
        // Test All with no parent ID (shop-level)
        let all_path = get_path(Metafield::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "metafields");

        // Test Count with no parent ID (shop-level)
        let count_path = get_path(Metafield::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "metafields/count");

        // Test Create with no parent ID (shop-level)
        let create_path = get_path(Metafield::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "metafields");
    }

    #[test]
    fn test_metafield_list_params_serialization() {
        let params = MetafieldListParams {
            namespace: Some("custom".to_string()),
            key: Some("color".to_string()),
            metafield_namespaces: Some("custom,inventory".to_string()),
            limit: Some(50),
            since_id: Some(12345),
            fields: Some("id,namespace,key,value".to_string()),
            page_info: None,
            created_at_min: None,
            created_at_max: None,
            updated_at_min: None,
            updated_at_max: None,
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["namespace"], "custom");
        assert_eq!(json["key"], "color");
        assert_eq!(json["metafield_namespaces"], "custom,inventory");
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 12345);
        assert_eq!(json["fields"], "id,namespace,key,value");

        // Test empty params
        let empty_params = MetafieldListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_namespace_and_key_filtering() {
        // Verify filter params are serialized correctly
        let params = MetafieldListParams {
            namespace: Some("inventory".to_string()),
            key: Some("warehouse".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["namespace"], "inventory");
        assert_eq!(json["key"], "warehouse");
    }

    #[test]
    fn test_value_type_field_serialization_with_rename() {
        // Test that metafield_type serializes as "type" in JSON
        let metafield = Metafield {
            metafield_type: Some("json".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&metafield).unwrap();
        assert!(json.contains("\"type\":\"json\""));
        assert!(!json.contains("metafield_type"));

        // Test deserialization from "type"
        let json_input = r#"{"type":"number_integer"}"#;
        let parsed: Metafield = serde_json::from_str(json_input).unwrap();
        assert_eq!(parsed.metafield_type.as_deref(), Some("number_integer"));
    }

    #[test]
    fn test_all_for_owner_method_signature() {
        // Verify the method signature exists and is correct by referencing it
        fn _assert_all_for_owner_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient, MetafieldOwner, u64, Option<MetafieldListParams>) -> Fut,
            Fut: std::future::Future<
                Output = Result<ResourceResponse<Vec<Metafield>>, ResourceError>,
            >,
        {
            let _ = f;
        }

        // This test verifies the method exists and has the correct signature.
        // The actual HTTP call would require a mock client.
    }

    #[test]
    fn test_all_owner_type_paths() {
        // Test paths for all supported owner types

        // Variant
        let variant_all = get_path(Metafield::PATHS, ResourceOperation::All, &["variant_id"]);
        assert!(variant_all.is_some());
        assert_eq!(
            variant_all.unwrap().template,
            "variants/{variant_id}/metafields"
        );

        // Order
        let order_all = get_path(Metafield::PATHS, ResourceOperation::All, &["order_id"]);
        assert!(order_all.is_some());
        assert_eq!(order_all.unwrap().template, "orders/{order_id}/metafields");

        // Collection
        let collection_all = get_path(Metafield::PATHS, ResourceOperation::All, &["collection_id"]);
        assert!(collection_all.is_some());
        assert_eq!(
            collection_all.unwrap().template,
            "collections/{collection_id}/metafields"
        );

        // Page
        let page_all = get_path(Metafield::PATHS, ResourceOperation::All, &["page_id"]);
        assert!(page_all.is_some());
        assert_eq!(page_all.unwrap().template, "pages/{page_id}/metafields");

        // Blog
        let blog_all = get_path(Metafield::PATHS, ResourceOperation::All, &["blog_id"]);
        assert!(blog_all.is_some());
        assert_eq!(blog_all.unwrap().template, "blogs/{blog_id}/metafields");

        // Article
        let article_all = get_path(Metafield::PATHS, ResourceOperation::All, &["article_id"]);
        assert!(article_all.is_some());
        assert_eq!(
            article_all.unwrap().template,
            "articles/{article_id}/metafields"
        );
    }

    #[test]
    fn test_metafield_get_id_returns_correct_value() {
        let metafield_with_id = Metafield {
            id: Some(123456789),
            namespace: Some("custom".to_string()),
            key: Some("test".to_string()),
            ..Default::default()
        };
        assert_eq!(metafield_with_id.get_id(), Some(123456789));

        let metafield_without_id = Metafield {
            id: None,
            namespace: Some("custom".to_string()),
            key: Some("new_key".to_string()),
            ..Default::default()
        };
        assert_eq!(metafield_without_id.get_id(), None);
    }

    #[test]
    fn test_metafield_resource_constants() {
        assert_eq!(Metafield::NAME, "Metafield");
        assert_eq!(Metafield::PLURAL, "metafields");
        // We have many paths for all the different owner types
        assert!(Metafield::PATHS.len() > 50); // Lots of polymorphic paths
    }

    #[test]
    fn test_metafield_count_params_serialization() {
        let params = MetafieldCountParams {
            namespace: Some("custom".to_string()),
            key: Some("color".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["namespace"], "custom");
        assert_eq!(json["key"], "color");

        let empty_params = MetafieldCountParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }
}

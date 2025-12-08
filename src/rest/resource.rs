//! REST Resource trait for CRUD operations.
//!
//! This module defines the [`RestResource`] trait, which provides a standardized
//! interface for interacting with Shopify REST API resources. Resources that
//! implement this trait gain `find()`, `all()`, `save()`, `delete()`, and
//! `count()` methods.
//!
//! # Implementing a Resource
//!
//! To implement a REST resource:
//!
//! 1. Define a struct with serde derives
//! 2. Implement the `RestResource` trait with associated types and constants
//! 3. The trait provides default implementations for CRUD operations
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourcePath, ResourceOperation, ResourceResponse, ResourceError};
//! use shopify_api::{HttpMethod, RestClient};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! pub struct Product {
//!     pub id: Option<u64>,
//!     pub title: String,
//!     #[serde(skip_serializing_if = "Option::is_none")]
//!     pub vendor: Option<String>,
//! }
//!
//! impl RestResource for Product {
//!     type Id = u64;
//!     type FindParams = ();
//!     type AllParams = ProductListParams;
//!     type CountParams = ();
//!
//!     const NAME: &'static str = "Product";
//!     const PLURAL: &'static str = "products";
//!     const PATHS: &'static [ResourcePath] = &[
//!         ResourcePath::new(HttpMethod::Get, ResourceOperation::Find, &["id"], "products/{id}"),
//!         ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "products"),
//!         ResourcePath::new(HttpMethod::Get, ResourceOperation::Count, &[], "products/count"),
//!         ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "products"),
//!         ResourcePath::new(HttpMethod::Put, ResourceOperation::Update, &["id"], "products/{id}"),
//!         ResourcePath::new(HttpMethod::Delete, ResourceOperation::Delete, &["id"], "products/{id}"),
//!     ];
//!
//!     fn get_id(&self) -> Option<Self::Id> {
//!         self.id
//!     }
//! }
//!
//! // Usage:
//! let product = Product::find(&client, 123, None).await?;
//! let products = Product::all(&client, None).await?;
//! ```

use std::collections::HashMap;
use std::fmt::Display;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::clients::RestClient;
use crate::rest::{
    build_path, get_path, ResourceError, ResourceOperation, ResourcePath, ResourceResponse,
};

/// A REST resource that can be fetched, created, updated, and deleted.
///
/// This trait provides a standardized interface for CRUD operations on
/// Shopify REST API resources. Implementors define the resource's paths,
/// name, and parameter types, and get default implementations for all
/// CRUD methods.
///
/// # Associated Types
///
/// - `Id`: The type of the resource's identifier (usually `u64` or `String`)
/// - `FindParams`: Parameters for `find()` operations (use `()` if none)
/// - `AllParams`: Parameters for `all()` operations (pagination, filters, etc.)
/// - `CountParams`: Parameters for `count()` operations
///
/// # Associated Constants
///
/// - `NAME`: The singular resource name (e.g., "Product")
/// - `PLURAL`: The plural form used in URLs (e.g., "products")
/// - `PATHS`: Available paths for different operations
/// - `PREFIX`: Optional path prefix for nested resources
///
/// # Required Bounds
///
/// Resources must be serializable, deserializable, cloneable, and thread-safe.
#[allow(async_fn_in_trait)]
pub trait RestResource: Serialize + DeserializeOwned + Clone + Send + Sync + Sized {
    /// The type of the resource's identifier.
    type Id: Display + Clone + Send + Sync;

    /// Parameters for `find()` operations.
    ///
    /// Use `()` if no parameters are needed.
    type FindParams: Serialize + Default + Send + Sync;

    /// Parameters for `all()` operations (filtering, pagination, etc.).
    ///
    /// Use `()` if no parameters are needed.
    type AllParams: Serialize + Default + Send + Sync;

    /// Parameters for `count()` operations.
    ///
    /// Use `()` if no parameters are needed.
    type CountParams: Serialize + Default + Send + Sync;

    /// The singular name of the resource (e.g., "Product").
    ///
    /// Used in error messages and as the response body key for single resources.
    const NAME: &'static str;

    /// The plural name used in URL paths (e.g., "products").
    ///
    /// Used as the response body key for collection operations.
    const PLURAL: &'static str;

    /// Available paths for this resource.
    ///
    /// Define paths for each operation the resource supports. The path
    /// selection logic will choose the most specific path that matches
    /// the available IDs.
    const PATHS: &'static [ResourcePath];

    /// Optional path prefix for nested resources.
    ///
    /// Override this for resources that always appear under a parent path.
    const PREFIX: Option<&'static str> = None;

    /// Returns the resource's ID if it exists.
    ///
    /// Returns `None` for new resources that haven't been saved yet.
    fn get_id(&self) -> Option<Self::Id>;

    /// Returns the lowercase key used in JSON request/response bodies.
    #[must_use]
    fn resource_key() -> String {
        Self::NAME.to_lowercase()
    }

    /// Finds a single resource by ID.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `id` - The resource ID to find
    /// * `params` - Optional parameters for the request
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the resource doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if no valid path matches.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let product = Product::find(&client, 123, None).await?;
    /// println!("Found: {}", product.title);
    /// ```
    async fn find(
        client: &RestClient,
        id: Self::Id,
        params: Option<Self::FindParams>,
    ) -> Result<ResourceResponse<Self>, ResourceError> {
        // Build the path
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("id", id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Find, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "find",
            },
        )?;

        let url = build_path(path.template, &ids);
        let full_path = Self::build_full_path(&url);

        // Build query params from FindParams
        let query = params
            .map(|p| serialize_to_query(&p))
            .transpose()?
            .filter(|q| !q.is_empty());

        // Make the request
        let response = client.get(&full_path, query).await?;

        // Check for error status codes
        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the response
        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }

    /// Lists all resources matching the given parameters.
    ///
    /// Returns a paginated response. Use `has_next_page()` and `next_page_info()`
    /// to navigate through pages.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `params` - Optional parameters for filtering/pagination
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no valid path matches.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let response = Product::all(&client, None).await?;
    /// for product in response.iter() {
    ///     println!("Product: {}", product.title);
    /// }
    ///
    /// if response.has_next_page() {
    ///     // Fetch next page...
    /// }
    /// ```
    async fn all(
        client: &RestClient,
        params: Option<Self::AllParams>,
    ) -> Result<ResourceResponse<Vec<Self>>, ResourceError> {
        let path = get_path(Self::PATHS, ResourceOperation::All, &[]).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "all",
            },
        )?;

        let url = path.template;
        let full_path = Self::build_full_path(url);

        // Build query params from AllParams
        let query = params
            .map(|p| serialize_to_query(&p))
            .transpose()?
            .filter(|q| !q.is_empty());

        // Make the request
        let response = client.get(&full_path, query).await?;

        // Check for error status codes
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
        ResourceResponse::from_http_response(response, Self::PLURAL)
    }

    /// Lists resources with a specific parent resource ID.
    ///
    /// For nested resources that require a parent ID (e.g., variants under products).
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    /// * `parent_id_name` - The name of the parent ID parameter (e.g., `product_id`)
    /// * `parent_id` - The parent resource ID
    /// * `params` - Optional parameters for filtering/pagination
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no valid path matches.
    async fn all_with_parent<ParentId: Display + Send>(
        client: &RestClient,
        parent_id_name: &str,
        parent_id: ParentId,
        params: Option<Self::AllParams>,
    ) -> Result<ResourceResponse<Vec<Self>>, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert(parent_id_name, parent_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::All, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "all",
            },
        )?;

        let url = build_path(path.template, &ids);
        let full_path = Self::build_full_path(&url);

        let query = params
            .map(|p| serialize_to_query(&p))
            .transpose()?
            .filter(|q| !q.is_empty());

        let response = client.get(&full_path, query).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        ResourceResponse::from_http_response(response, Self::PLURAL)
    }

    /// Saves the resource (create or update).
    ///
    /// For new resources (no ID), sends a POST request to create.
    /// For existing resources (has ID), sends a PUT request to update.
    ///
    /// When updating, only changed fields are sent if dirty tracking is used.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    ///
    /// # Returns
    ///
    /// The saved resource with any server-generated fields populated.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::ValidationFailed`] if validation fails (422).
    /// Returns [`ResourceError::NotFound`] if updating a non-existent resource.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Create new
    /// let mut product = Product { id: None, title: "New".to_string(), vendor: None };
    /// let saved = product.save(&client).await?;
    ///
    /// // Update existing
    /// let mut product = Product::find(&client, 123, None).await?.into_inner();
    /// product.title = "Updated".to_string();
    /// let saved = product.save(&client).await?;
    /// ```
    async fn save(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let is_new = self.get_id().is_none();
        let key = Self::resource_key();

        if is_new {
            // Create (POST)
            let path = get_path(Self::PATHS, ResourceOperation::Create, &[]).ok_or(
                ResourceError::PathResolutionFailed {
                    resource: Self::NAME,
                    operation: "create",
                },
            )?;

            let url = path.template;
            let full_path = Self::build_full_path(url);

            // Wrap in resource key - use a map to avoid move issues
            let mut body_map = serde_json::Map::new();
            body_map.insert(
                key.clone(),
                serde_json::to_value(self).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: 400,
                            message: format!("Failed to serialize resource: {e}"),
                            error_reference: None,
                        },
                    ))
                })?,
            );
            let body = Value::Object(body_map);

            let response = client.post(&full_path, body, None).await?;

            if !response.is_ok() {
                return Err(ResourceError::from_http_response(
                    response.code,
                    &response.body,
                    Self::NAME,
                    None,
                    response.request_id(),
                ));
            }

            let result: ResourceResponse<Self> =
                ResourceResponse::from_http_response(response, &key)?;
            Ok(result.into_inner())
        } else {
            // Update (PUT)
            let id = self.get_id().unwrap();

            let mut ids: HashMap<&str, String> = HashMap::new();
            ids.insert("id", id.to_string());

            let available_ids: Vec<&str> = ids.keys().copied().collect();
            let path = get_path(Self::PATHS, ResourceOperation::Update, &available_ids).ok_or(
                ResourceError::PathResolutionFailed {
                    resource: Self::NAME,
                    operation: "update",
                },
            )?;

            let url = build_path(path.template, &ids);
            let full_path = Self::build_full_path(&url);

            // Wrap in resource key - use a map to avoid move issues
            let mut body_map = serde_json::Map::new();
            body_map.insert(
                key.clone(),
                serde_json::to_value(self).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: 400,
                            message: format!("Failed to serialize resource: {e}"),
                            error_reference: None,
                        },
                    ))
                })?,
            );
            let body = Value::Object(body_map);

            let response = client.put(&full_path, body, None).await?;

            if !response.is_ok() {
                return Err(ResourceError::from_http_response(
                    response.code,
                    &response.body,
                    Self::NAME,
                    Some(&id.to_string()),
                    response.request_id(),
                ));
            }

            let result: ResourceResponse<Self> =
                ResourceResponse::from_http_response(response, &key)?;
            Ok(result.into_inner())
        }
    }

    /// Saves the resource with dirty tracking for partial updates.
    ///
    /// For existing resources, only sends changed fields in the PUT request.
    /// This is more efficient than `save()` for large resources.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    /// * `changed_fields` - JSON value containing only the changed fields
    ///
    /// # Returns
    ///
    /// The saved resource with server-generated fields populated.
    async fn save_partial(
        &self,
        client: &RestClient,
        changed_fields: Value,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "update",
        })?;

        let key = Self::resource_key();

        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("id", id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Update, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "update",
            },
        )?;

        let url = build_path(path.template, &ids);
        let full_path = Self::build_full_path(&url);

        // Wrap changed fields in resource key - use a map to avoid move issues
        let mut body_map = serde_json::Map::new();
        body_map.insert(key.clone(), changed_fields);
        let body = Value::Object(body_map);

        let response = client.put(&full_path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        let result: ResourceResponse<Self> = ResourceResponse::from_http_response(response, &key)?;
        Ok(result.into_inner())
    }

    /// Deletes the resource.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the resource doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if no delete path exists.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let product = Product::find(&client, 123, None).await?;
    /// product.delete(&client).await?;
    /// ```
    async fn delete(&self, client: &RestClient) -> Result<(), ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "delete",
        })?;

        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("id", id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Delete, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "delete",
            },
        )?;

        let url = build_path(path.template, &ids);
        let full_path = Self::build_full_path(&url);

        let response = client.delete(&full_path, None).await?;

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

    /// Counts resources matching the given parameters.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    /// * `params` - Optional parameters for filtering
    ///
    /// # Returns
    ///
    /// The count of matching resources as a `u64`.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no count path exists.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = Product::count(&client, None).await?;
    /// println!("Total products: {}", count);
    /// ```
    async fn count(
        client: &RestClient,
        params: Option<Self::CountParams>,
    ) -> Result<u64, ResourceError> {
        let path = get_path(Self::PATHS, ResourceOperation::Count, &[]).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "count",
            },
        )?;

        let url = path.template;
        let full_path = Self::build_full_path(url);

        let query = params
            .map(|p| serialize_to_query(&p))
            .transpose()?
            .filter(|q| !q.is_empty());

        let response = client.get(&full_path, query).await?;

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

    /// Builds the full path including any prefix.
    #[must_use]
    fn build_full_path(path: &str) -> String {
        Self::PREFIX.map_or_else(|| path.to_string(), |prefix| format!("{prefix}/{path}"))
    }
}

/// Serializes a params struct to a query parameter map.
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

    if let Value::Object(map) = value {
        for (key, val) in map {
            match val {
                Value::Null => {} // Skip null values
                Value::String(s) => {
                    query.insert(key, s);
                }
                Value::Number(n) => {
                    query.insert(key, n.to_string());
                }
                Value::Bool(b) => {
                    query.insert(key, b.to_string());
                }
                Value::Array(arr) => {
                    // Convert arrays to comma-separated values
                    let values: Vec<String> = arr
                        .iter()
                        .filter_map(|v| match v {
                            Value::String(s) => Some(s.clone()),
                            Value::Number(n) => Some(n.to_string()),
                            _ => None,
                        })
                        .collect();
                    if !values.is_empty() {
                        query.insert(key, values.join(","));
                    }
                }
                Value::Object(_) => {
                    // For complex objects, serialize as JSON string
                    query.insert(key, val.to_string());
                }
            }
        }
    }

    Ok(query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{ResourceOperation, ResourcePath};
    use crate::HttpMethod;
    use serde::{Deserialize, Serialize};

    // Test resource implementation
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct MockProduct {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        title: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        vendor: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct MockProductParams {
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        page_info: Option<String>,
    }

    impl RestResource for MockProduct {
        type Id = u64;
        type FindParams = ();
        type AllParams = MockProductParams;
        type CountParams = ();

        const NAME: &'static str = "Product";
        const PLURAL: &'static str = "products";
        const PATHS: &'static [ResourcePath] = &[
            ResourcePath::new(
                HttpMethod::Get,
                ResourceOperation::Find,
                &["id"],
                "products/{id}",
            ),
            ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "products"),
            ResourcePath::new(
                HttpMethod::Get,
                ResourceOperation::Count,
                &[],
                "products/count",
            ),
            ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "products"),
            ResourcePath::new(
                HttpMethod::Put,
                ResourceOperation::Update,
                &["id"],
                "products/{id}",
            ),
            ResourcePath::new(
                HttpMethod::Delete,
                ResourceOperation::Delete,
                &["id"],
                "products/{id}",
            ),
        ];

        fn get_id(&self) -> Option<Self::Id> {
            self.id
        }
    }

    // Nested resource for testing
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct MockVariant {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        product_id: Option<u64>,
        title: String,
    }

    impl RestResource for MockVariant {
        type Id = u64;
        type FindParams = ();
        type AllParams = ();
        type CountParams = ();

        const NAME: &'static str = "Variant";
        const PLURAL: &'static str = "variants";
        const PATHS: &'static [ResourcePath] = &[
            // Nested paths (more specific)
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
            // Standalone paths (less specific)
            ResourcePath::new(
                HttpMethod::Get,
                ResourceOperation::Find,
                &["id"],
                "variants/{id}",
            ),
        ];

        fn get_id(&self) -> Option<Self::Id> {
            self.id
        }
    }

    #[test]
    fn test_resource_defines_name_and_paths() {
        assert_eq!(MockProduct::NAME, "Product");
        assert_eq!(MockProduct::PLURAL, "products");
        assert!(!MockProduct::PATHS.is_empty());
    }

    #[test]
    fn test_get_id_returns_none_for_new_resource() {
        let product = MockProduct {
            id: None,
            title: "New".to_string(),
            vendor: None,
        };
        assert!(product.get_id().is_none());
    }

    #[test]
    fn test_get_id_returns_some_for_existing_resource() {
        let product = MockProduct {
            id: Some(123),
            title: "Existing".to_string(),
            vendor: None,
        };
        assert_eq!(product.get_id(), Some(123));
    }

    #[test]
    fn test_build_full_path_without_prefix() {
        let path = MockProduct::build_full_path("products/123");
        assert_eq!(path, "products/123");
    }

    #[test]
    fn test_serialize_to_query_handles_basic_types() {
        #[derive(Serialize)]
        struct Params {
            limit: u32,
            title: String,
            active: bool,
        }

        let params = Params {
            limit: 50,
            title: "Test".to_string(),
            active: true,
        };

        let query = serialize_to_query(&params).unwrap();
        assert_eq!(query.get("limit"), Some(&"50".to_string()));
        assert_eq!(query.get("title"), Some(&"Test".to_string()));
        assert_eq!(query.get("active"), Some(&"true".to_string()));
    }

    #[test]
    fn test_serialize_to_query_skips_none() {
        #[derive(Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            limit: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            page_info: Option<String>,
        }

        let params = Params {
            limit: Some(50),
            page_info: None,
        };

        let query = serialize_to_query(&params).unwrap();
        assert_eq!(query.get("limit"), Some(&"50".to_string()));
        assert!(!query.contains_key("page_info"));
    }

    #[test]
    fn test_serialize_to_query_handles_arrays() {
        #[derive(Serialize)]
        struct Params {
            ids: Vec<u64>,
        }

        let params = Params { ids: vec![1, 2, 3] };

        let query = serialize_to_query(&params).unwrap();
        assert_eq!(query.get("ids"), Some(&"1,2,3".to_string()));
    }

    #[test]
    fn test_nested_resource_path_selection() {
        // With product_id available, should select nested path for All
        let path = get_path(MockVariant::PATHS, ResourceOperation::All, &["product_id"]);
        assert!(path.is_some());
        assert_eq!(path.unwrap().template, "products/{product_id}/variants");

        // With both product_id and id, should select nested Find path
        let path = get_path(
            MockVariant::PATHS,
            ResourceOperation::Find,
            &["product_id", "id"],
        );
        assert!(path.is_some());
        assert_eq!(
            path.unwrap().template,
            "products/{product_id}/variants/{id}"
        );

        // With only id, should select standalone Find path
        let path = get_path(MockVariant::PATHS, ResourceOperation::Find, &["id"]);
        assert!(path.is_some());
        assert_eq!(path.unwrap().template, "variants/{id}");
    }

    #[test]
    fn test_resource_trait_bounds() {
        fn assert_trait_bounds<T: RestResource>() {}
        assert_trait_bounds::<MockProduct>();
        assert_trait_bounds::<MockVariant>();
    }

    #[test]
    fn test_resource_key_lowercase() {
        assert_eq!(MockProduct::resource_key(), "product");
        assert_eq!(MockVariant::resource_key(), "variant");
    }
}

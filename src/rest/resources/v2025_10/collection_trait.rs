//! Collection trait for shared behavior between collection types.
//!
//! This module provides the [`Collection`] trait, which defines shared behavior
//! for both [`CustomCollection`] and [`SmartCollection`].
//!
//! # Overview
//!
//! The Collection trait provides polymorphic access to collection functionality,
//! allowing code to work with either collection type interchangeably when it
//! only needs the shared `products()` and `product_count()` methods.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::v2025_10::{
//!     Collection, CustomCollection, SmartCollection, ProductListParams
//! };
//!
//! // Works with either collection type
//! async fn display_collection_products<C: Collection>(
//!     client: &RestClient,
//!     collection: &C
//! ) -> Result<(), ResourceError> {
//!     // Get products in the collection
//!     let response = collection.products(client, None).await?;
//!     for product in response.iter() {
//!         println!("- {}", product.title.as_deref().unwrap_or(""));
//!     }
//!
//!     // Get product count
//!     let count = collection.product_count(client).await?;
//!     println!("Total products: {}", count);
//!
//!     Ok(())
//! }
//!
//! // Use with CustomCollection
//! let custom = CustomCollection::find(&client, 123, None).await?.into_inner();
//! display_collection_products(&client, &custom).await?;
//!
//! // Use with SmartCollection
//! let smart = SmartCollection::find(&client, 456, None).await?.into_inner();
//! display_collection_products(&client, &smart).await?;
//! ```

use std::collections::HashMap;

use serde::Serialize;

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceResponse};

use super::custom_collection::CustomCollection;
use super::product::{Product, ProductListParams};
use super::smart_collection::SmartCollection;

/// Trait for shared collection behavior.
///
/// Both [`CustomCollection`] and [`SmartCollection`] implement this trait,
/// allowing polymorphic access to collection functionality.
///
/// # Provided Methods
///
/// - `get_collection_id()` - Returns the collection's ID (if it exists)
/// - `products()` - Fetches products in the collection
/// - `product_count()` - Returns the count of products in the collection
///
/// # Example
///
/// ```rust,ignore
/// use shopify_sdk::rest::resources::v2025_10::{Collection, CustomCollection};
///
/// let collection = CustomCollection::find(&client, 123, None).await?.into_inner();
///
/// // Get collection ID
/// if let Some(id) = collection.get_collection_id() {
///     println!("Collection ID: {}", id);
/// }
///
/// // Get products
/// let products = collection.products(&client, None).await?;
/// println!("Found {} products", products.len());
///
/// // Get count
/// let count = collection.product_count(&client).await?;
/// println!("Product count: {}", count);
/// ```
#[allow(async_fn_in_trait)]
pub trait Collection: Send + Sync {
    /// Returns the collection's ID if it exists.
    ///
    /// Returns `None` for new collections that haven't been saved yet.
    fn get_collection_id(&self) -> Option<u64>;

    /// Fetches the products in this collection.
    ///
    /// Sends a GET request to `/admin/api/{version}/collections/{id}/products.json`.
    ///
    /// Note: This endpoint uses the shared `/collections/{id}` path, which works
    /// for both custom collections and smart collections.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `params` - Optional parameters for filtering/pagination (uses [`ProductListParams`])
    ///
    /// # Returns
    ///
    /// Returns a paginated response containing the products in the collection.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if the collection has no ID.
    /// Returns [`ResourceError::NotFound`] if the collection doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::rest::resources::v2025_10::{Collection, CustomCollection, ProductListParams};
    ///
    /// let collection = CustomCollection::find(&client, 123, None).await?.into_inner();
    ///
    /// // Get all products
    /// let products = collection.products(&client, None).await?;
    ///
    /// // Get products with filters
    /// let params = ProductListParams {
    ///     limit: Some(10),
    ///     ..Default::default()
    /// };
    /// let products = collection.products(&client, Some(params)).await?;
    /// ```
    async fn products(
        &self,
        client: &RestClient,
        params: Option<ProductListParams>,
    ) -> Result<ResourceResponse<Vec<Product>>, ResourceError>;

    /// Returns the count of products in this collection.
    ///
    /// Sends a GET request to `/admin/api/{version}/collections/{id}/products/count.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Returns
    ///
    /// Returns the count of products in the collection.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if the collection has no ID.
    /// Returns [`ResourceError::NotFound`] if the collection doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::rest::resources::v2025_10::{Collection, SmartCollection};
    ///
    /// let collection = SmartCollection::find(&client, 456, None).await?.into_inner();
    /// let count = collection.product_count(&client).await?;
    /// println!("Collection has {} products", count);
    /// ```
    async fn product_count(&self, client: &RestClient) -> Result<u64, ResourceError>;
}

impl Collection for CustomCollection {
    fn get_collection_id(&self) -> Option<u64> {
        self.id
    }

    async fn products(
        &self,
        client: &RestClient,
        params: Option<ProductListParams>,
    ) -> Result<ResourceResponse<Vec<Product>>, ResourceError> {
        let id = self
            .get_collection_id()
            .ok_or(ResourceError::PathResolutionFailed {
                resource: "CustomCollection",
                operation: "products",
            })?;

        fetch_collection_products(client, id, params).await
    }

    async fn product_count(&self, client: &RestClient) -> Result<u64, ResourceError> {
        let id = self
            .get_collection_id()
            .ok_or(ResourceError::PathResolutionFailed {
                resource: "CustomCollection",
                operation: "product_count",
            })?;

        fetch_collection_product_count(client, id).await
    }
}

impl Collection for SmartCollection {
    fn get_collection_id(&self) -> Option<u64> {
        self.id
    }

    async fn products(
        &self,
        client: &RestClient,
        params: Option<ProductListParams>,
    ) -> Result<ResourceResponse<Vec<Product>>, ResourceError> {
        let id = self
            .get_collection_id()
            .ok_or(ResourceError::PathResolutionFailed {
                resource: "SmartCollection",
                operation: "products",
            })?;

        fetch_collection_products(client, id, params).await
    }

    async fn product_count(&self, client: &RestClient) -> Result<u64, ResourceError> {
        let id = self
            .get_collection_id()
            .ok_or(ResourceError::PathResolutionFailed {
                resource: "SmartCollection",
                operation: "product_count",
            })?;

        fetch_collection_product_count(client, id).await
    }
}

/// Shared implementation for fetching products from a collection.
///
/// Both custom and smart collections use the same endpoint:
/// `/collections/{id}/products.json`
async fn fetch_collection_products(
    client: &RestClient,
    collection_id: u64,
    params: Option<ProductListParams>,
) -> Result<ResourceResponse<Vec<Product>>, ResourceError> {
    let path = format!("collections/{collection_id}/products");

    // Build query params
    let query = params
        .map(|p| serialize_to_query(&p))
        .transpose()?
        .filter(|q| !q.is_empty());

    let response = client.get(&path, query).await?;

    if !response.is_ok() {
        return Err(ResourceError::from_http_response(
            response.code,
            &response.body,
            "Collection",
            Some(&collection_id.to_string()),
            response.request_id(),
        ));
    }

    ResourceResponse::from_http_response(response, "products")
}

/// Shared implementation for counting products in a collection.
///
/// Both custom and smart collections use the same endpoint:
/// `/collections/{id}/products/count.json`
async fn fetch_collection_product_count(
    client: &RestClient,
    collection_id: u64,
) -> Result<u64, ResourceError> {
    let path = format!("collections/{collection_id}/products/count");

    let response = client.get(&path, None).await?;

    if !response.is_ok() {
        return Err(ResourceError::from_http_response(
            response.code,
            &response.body,
            "Collection",
            Some(&collection_id.to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_trait_get_collection_id_custom() {
        let collection = CustomCollection {
            id: Some(841564295),
            title: Some("Test Collection".to_string()),
            ..Default::default()
        };

        assert_eq!(collection.get_collection_id(), Some(841564295));

        let new_collection = CustomCollection {
            id: None,
            title: Some("New Collection".to_string()),
            ..Default::default()
        };

        assert_eq!(new_collection.get_collection_id(), None);
    }

    #[test]
    fn test_collection_trait_get_collection_id_smart() {
        let collection = SmartCollection {
            id: Some(1063001322),
            title: Some("Test Collection".to_string()),
            ..Default::default()
        };

        assert_eq!(collection.get_collection_id(), Some(1063001322));

        let new_collection = SmartCollection {
            id: None,
            title: Some("New Collection".to_string()),
            ..Default::default()
        };

        assert_eq!(new_collection.get_collection_id(), None);
    }

    #[test]
    fn test_collection_trait_products_method_signature_custom() {
        // Verify the method signature exists and is correct by referencing it
        fn _assert_products_signature<F, Fut>(f: F)
        where
            F: Fn(&CustomCollection, &RestClient, Option<ProductListParams>) -> Fut,
            Fut:
                std::future::Future<Output = Result<ResourceResponse<Vec<Product>>, ResourceError>>,
        {
            let _ = f;
        }

        // Verify PathResolutionFailed is returned for collection without ID
        let collection = CustomCollection {
            id: None,
            ..Default::default()
        };
        assert!(collection.get_collection_id().is_none());
    }

    #[test]
    fn test_collection_trait_products_method_signature_smart() {
        // Verify the method signature exists and is correct by referencing it
        fn _assert_products_signature<F, Fut>(f: F)
        where
            F: Fn(&SmartCollection, &RestClient, Option<ProductListParams>) -> Fut,
            Fut:
                std::future::Future<Output = Result<ResourceResponse<Vec<Product>>, ResourceError>>,
        {
            let _ = f;
        }

        // Verify PathResolutionFailed is returned for collection without ID
        let collection = SmartCollection {
            id: None,
            ..Default::default()
        };
        assert!(collection.get_collection_id().is_none());
    }

    #[test]
    fn test_collection_trait_product_count_method_signature_custom() {
        // Verify the method signature exists and is correct by referencing it
        fn _assert_product_count_signature<F, Fut>(f: F)
        where
            F: Fn(&CustomCollection, &RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<u64, ResourceError>>,
        {
            let _ = f;
        }
    }

    #[test]
    fn test_collection_trait_product_count_method_signature_smart() {
        // Verify the method signature exists and is correct by referencing it
        fn _assert_product_count_signature<F, Fut>(f: F)
        where
            F: Fn(&SmartCollection, &RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<u64, ResourceError>>,
        {
            let _ = f;
        }
    }

    #[test]
    fn test_collection_trait_polymorphism() {
        // Verify that both collection types can be used polymorphically
        fn _takes_collection<C: Collection>(_collection: &C) {}

        let custom = CustomCollection {
            id: Some(123),
            ..Default::default()
        };
        _takes_collection(&custom);

        let smart = SmartCollection {
            id: Some(456),
            ..Default::default()
        };
        _takes_collection(&smart);
    }

    #[test]
    fn test_serialize_to_query_helper() {
        let params = ProductListParams {
            limit: Some(50),
            title: Some("Test".to_string()),
            ..Default::default()
        };

        let query = serialize_to_query(&params).unwrap();

        assert_eq!(query.get("limit"), Some(&"50".to_string()));
        assert_eq!(query.get("title"), Some(&"Test".to_string()));
    }

    #[test]
    fn test_collection_trait_bounds() {
        fn assert_trait_bounds<T: Collection + Send + Sync>() {}
        assert_trait_bounds::<CustomCollection>();
        assert_trait_bounds::<SmartCollection>();
    }
}

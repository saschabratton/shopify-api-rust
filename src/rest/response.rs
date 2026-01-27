//! Response wrapper for REST resource operations.
//!
//! This module provides [`ResourceResponse<T>`], a wrapper that combines
//! resource data with metadata like pagination and rate limit information.
//! The wrapper implements `Deref` for ergonomic access to the inner data.
//!
//! # Deref Pattern
//!
//! `ResourceResponse<T>` implements `Deref<Target = T>`, which means you can
//! use it like the inner type directly:
//!
//! ```rust,ignore
//! let response: ResourceResponse<Vec<Product>> = Product::all(&client, None).await?;
//!
//! // Iterate directly (Vec method via Deref)
//! for product in response.iter() {
//!     println!("{}", product.title);
//! }
//!
//! // Access length (Vec method via Deref)
//! println!("Count: {}", response.len());
//!
//! // Index access (Vec trait via Deref)
//! let first = &response[0];
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//!
//! // Fetch products with pagination
//! let response: ResourceResponse<Vec<Product>> = Product::all(&client, None).await?;
//!
//! // Access products directly via Deref
//! for product in response.iter() {
//!     println!("Product: {}", product.title);
//! }
//!
//! // Check pagination
//! if response.has_next_page() {
//!     let page_info = response.next_page_info().unwrap();
//!     // Fetch next page using page_info...
//! }
//!
//! // Check rate limits
//! if let Some(limit) = response.rate_limit() {
//!     println!("API calls: {}/{}", limit.request_count, limit.bucket_size);
//! }
//!
//! // Take ownership of inner data
//! let products: Vec<Product> = response.into_inner();
//! ```

use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;

use crate::clients::{ApiCallLimit, HttpResponse, PaginationInfo};
use crate::rest::ResourceError;

/// A response from a REST resource operation.
///
/// This wrapper combines the resource data with metadata from the HTTP
/// response, including pagination information and rate limit data.
///
/// The struct implements `Deref<Target = T>` for transparent access to
/// the inner data. This allows calling methods on `T` directly through
/// the response wrapper.
///
/// # Type Parameters
///
/// * `T` - The type of data contained in the response. For single resources
///   this is the resource type (e.g., `Product`). For collections, this is
///   `Vec<ResourceType>` (e.g., `Vec<Product>`).
///
/// # Example
///
/// ```rust
/// use shopify_sdk::rest::ResourceResponse;
/// use shopify_sdk::clients::{ApiCallLimit, PaginationInfo};
///
/// // Create a response with a vector of items
/// let response = ResourceResponse::new(
///     vec!["item1", "item2", "item3"],
///     Some(PaginationInfo {
///         prev_page_info: None,
///         next_page_info: Some("eyJsYXN0X2lkIjo0fQ".to_string()),
///     }),
///     Some(ApiCallLimit { request_count: 1, bucket_size: 40 }),
///     Some("req-123".to_string()),
/// );
///
/// // Access items via Deref
/// assert_eq!(response.len(), 3);
/// assert_eq!(response[0], "item1");
///
/// // Access metadata
/// assert!(response.has_next_page());
/// assert!(!response.has_prev_page());
/// ```
#[derive(Debug, Clone)]
pub struct ResourceResponse<T> {
    /// The resource data.
    data: T,
    /// Pagination information from the Link header.
    pagination: Option<PaginationInfo>,
    /// Rate limit information from the API call limit header.
    rate_limit: Option<ApiCallLimit>,
    /// Request ID from the X-Request-Id header.
    request_id: Option<String>,
}

impl<T> ResourceResponse<T> {
    /// Creates a new `ResourceResponse` with the given data and metadata.
    ///
    /// # Arguments
    ///
    /// * `data` - The resource data
    /// * `pagination` - Pagination info from Link header
    /// * `rate_limit` - Rate limit info from API call limit header
    /// * `request_id` - Request ID from X-Request-Id header
    #[must_use]
    pub const fn new(
        data: T,
        pagination: Option<PaginationInfo>,
        rate_limit: Option<ApiCallLimit>,
        request_id: Option<String>,
    ) -> Self {
        Self {
            data,
            pagination,
            rate_limit,
            request_id,
        }
    }

    /// Consumes the response and returns the inner data.
    ///
    /// Use this when you need ownership of the data and no longer
    /// need the response metadata.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::rest::ResourceResponse;
    ///
    /// let response = ResourceResponse::new(
    ///     vec![1, 2, 3],
    ///     None,
    ///     None,
    ///     None,
    /// );
    /// let data: Vec<i32> = response.into_inner();
    /// assert_eq!(data, vec![1, 2, 3]);
    /// ```
    #[must_use]
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Returns a reference to the inner data.
    ///
    /// Note: In most cases, you can use Deref coercion instead of
    /// calling this method explicitly.
    #[must_use]
    pub const fn data(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the inner data.
    ///
    /// Note: In most cases, you can use `DerefMut` coercion instead of
    /// calling this method explicitly.
    #[must_use]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Returns `true` if there is a next page of results.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::rest::ResourceResponse;
    /// use shopify_sdk::clients::PaginationInfo;
    ///
    /// let response = ResourceResponse::new(
    ///     vec!["item"],
    ///     Some(PaginationInfo {
    ///         prev_page_info: None,
    ///         next_page_info: Some("token".to_string()),
    ///     }),
    ///     None,
    ///     None,
    /// );
    /// assert!(response.has_next_page());
    /// ```
    #[must_use]
    pub fn has_next_page(&self) -> bool {
        self.pagination
            .as_ref()
            .is_some_and(|p| p.next_page_info.is_some())
    }

    /// Returns `true` if there is a previous page of results.
    #[must_use]
    pub fn has_prev_page(&self) -> bool {
        self.pagination
            .as_ref()
            .is_some_and(|p| p.prev_page_info.is_some())
    }

    /// Returns the page info token for the next page, if available.
    ///
    /// Use this token with the `page_info` query parameter to fetch
    /// the next page of results.
    #[must_use]
    pub fn next_page_info(&self) -> Option<&str> {
        self.pagination
            .as_ref()
            .and_then(|p| p.next_page_info.as_deref())
    }

    /// Returns the page info token for the previous page, if available.
    #[must_use]
    pub fn prev_page_info(&self) -> Option<&str> {
        self.pagination
            .as_ref()
            .and_then(|p| p.prev_page_info.as_deref())
    }

    /// Returns the pagination info, if available.
    #[must_use]
    pub const fn pagination(&self) -> Option<&PaginationInfo> {
        self.pagination.as_ref()
    }

    /// Returns the rate limit information, if available.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::rest::ResourceResponse;
    /// use shopify_sdk::clients::ApiCallLimit;
    ///
    /// let response = ResourceResponse::new(
    ///     "data",
    ///     None,
    ///     Some(ApiCallLimit { request_count: 5, bucket_size: 40 }),
    ///     None,
    /// );
    ///
    /// let limit = response.rate_limit().unwrap();
    /// assert_eq!(limit.request_count, 5);
    /// assert_eq!(limit.bucket_size, 40);
    /// ```
    #[must_use]
    pub const fn rate_limit(&self) -> Option<&ApiCallLimit> {
        self.rate_limit.as_ref()
    }

    /// Returns the request ID from the response headers.
    ///
    /// Useful for debugging and error reporting.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Maps the inner data to a new type.
    ///
    /// Useful for transforming the response data while preserving metadata.
    #[must_use]
    pub fn map<U, F>(self, f: F) -> ResourceResponse<U>
    where
        F: FnOnce(T) -> U,
    {
        ResourceResponse {
            data: f(self.data),
            pagination: self.pagination,
            rate_limit: self.rate_limit,
            request_id: self.request_id,
        }
    }
}

impl<T: DeserializeOwned> ResourceResponse<T> {
    /// Creates a `ResourceResponse` from an HTTP response.
    ///
    /// Extracts the data from the response body under the given key,
    /// along with pagination and rate limit information.
    ///
    /// # Arguments
    ///
    /// * `response` - The HTTP response
    /// * `key` - The key in the response body containing the data
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::Http`] if the data cannot be deserialized.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::rest::ResourceResponse;
    ///
    /// // Assuming response.body = {"product": {"id": 123, "title": "Test"}}
    /// let response: ResourceResponse<Product> = ResourceResponse::from_http_response(
    ///     http_response,
    ///     "product",
    /// )?;
    /// ```
    pub fn from_http_response(response: HttpResponse, key: &str) -> Result<Self, ResourceError> {
        // Extract request_id before any potential moves
        let request_id = response.request_id().map(ToString::to_string);

        // Extract the data from the response body
        let data_value = response.body.get(key).ok_or_else(|| {
            ResourceError::Http(crate::clients::HttpError::Response(
                crate::clients::HttpResponseError {
                    code: response.code,
                    message: format!("Missing key '{key}' in response body"),
                    error_reference: request_id.clone(),
                },
            ))
        })?;

        // Deserialize the data
        let data: T = serde_json::from_value(data_value.clone()).map_err(|e| {
            ResourceError::Http(crate::clients::HttpError::Response(
                crate::clients::HttpResponseError {
                    code: response.code,
                    message: format!("Failed to deserialize '{key}': {e}"),
                    error_reference: request_id.clone(),
                },
            ))
        })?;

        // Build pagination info
        let pagination = if response.prev_page_info.is_some() || response.next_page_info.is_some() {
            Some(PaginationInfo {
                prev_page_info: response.prev_page_info,
                next_page_info: response.next_page_info,
            })
        } else {
            None
        };

        Ok(Self {
            data,
            pagination,
            rate_limit: response.api_call_limit,
            request_id,
        })
    }
}

/// Provides transparent access to the inner data.
///
/// This allows methods of `T` to be called directly on `ResourceResponse<T>`.
impl<T> Deref for ResourceResponse<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// Provides mutable access to the inner data.
impl<T> DerefMut for ResourceResponse<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

// Verify ResourceResponse is Send + Sync when T is Send + Sync
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ResourceResponse<String>>();
    assert_send_sync::<ResourceResponse<Vec<String>>>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestProduct {
        id: u64,
        title: String,
    }

    #[test]
    fn test_resource_response_stores_data_and_metadata() {
        let pagination = PaginationInfo {
            prev_page_info: Some("prev".to_string()),
            next_page_info: Some("next".to_string()),
        };
        let rate_limit = ApiCallLimit {
            request_count: 5,
            bucket_size: 40,
        };

        let response = ResourceResponse::new(
            vec!["item1", "item2"],
            Some(pagination),
            Some(rate_limit),
            Some("req-123".to_string()),
        );

        assert_eq!(response.data.len(), 2);
        assert!(response.pagination.is_some());
        assert!(response.rate_limit.is_some());
        assert_eq!(response.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_deref_allows_direct_access_to_inner_data() {
        let response = ResourceResponse::new(vec!["item1", "item2", "item3"], None, None, None);

        // Vec methods via Deref
        assert_eq!(response.len(), 3);
        assert!(!response.is_empty());
        assert_eq!(response.first(), Some(&"item1"));
    }

    #[test]
    fn test_deref_mut_allows_mutable_access() {
        let mut response = ResourceResponse::new(vec!["item1", "item2"], None, None, None);

        // Mutate via DerefMut
        response.push("item3");
        assert_eq!(response.len(), 3);

        response[0] = "modified";
        assert_eq!(response[0], "modified");
    }

    #[test]
    fn test_into_inner_returns_owned_data() {
        let response = ResourceResponse::new(vec![1, 2, 3], None, None, None);

        let data: Vec<i32> = response.into_inner();
        assert_eq!(data, vec![1, 2, 3]);
    }

    #[test]
    fn test_has_next_page_returns_correct_boolean() {
        let response_with_next = ResourceResponse::new(
            "data",
            Some(PaginationInfo {
                prev_page_info: None,
                next_page_info: Some("token".to_string()),
            }),
            None,
            None,
        );
        assert!(response_with_next.has_next_page());

        let response_without_next = ResourceResponse::new(
            "data",
            Some(PaginationInfo {
                prev_page_info: Some("prev".to_string()),
                next_page_info: None,
            }),
            None,
            None,
        );
        assert!(!response_without_next.has_next_page());

        let response_no_pagination: ResourceResponse<&str> =
            ResourceResponse::new("data", None, None, None);
        assert!(!response_no_pagination.has_next_page());
    }

    #[test]
    fn test_has_prev_page_returns_correct_boolean() {
        let response_with_prev = ResourceResponse::new(
            "data",
            Some(PaginationInfo {
                prev_page_info: Some("token".to_string()),
                next_page_info: None,
            }),
            None,
            None,
        );
        assert!(response_with_prev.has_prev_page());

        let response_without_prev = ResourceResponse::new(
            "data",
            Some(PaginationInfo {
                prev_page_info: None,
                next_page_info: Some("next".to_string()),
            }),
            None,
            None,
        );
        assert!(!response_without_prev.has_prev_page());
    }

    #[test]
    fn test_next_page_info_returns_option_str() {
        let response = ResourceResponse::new(
            "data",
            Some(PaginationInfo {
                prev_page_info: None,
                next_page_info: Some("eyJsYXN0X2lkIjo0fQ".to_string()),
            }),
            None,
            None,
        );

        assert_eq!(response.next_page_info(), Some("eyJsYXN0X2lkIjo0fQ"));
    }

    #[test]
    fn test_prev_page_info_returns_option_str() {
        let response = ResourceResponse::new(
            "data",
            Some(PaginationInfo {
                prev_page_info: Some("eyJsYXN0X2lkIjoxfQ".to_string()),
                next_page_info: None,
            }),
            None,
            None,
        );

        assert_eq!(response.prev_page_info(), Some("eyJsYXN0X2lkIjoxfQ"));
    }

    #[test]
    fn test_resource_response_vec_allows_iteration_via_deref() {
        let products = vec![
            TestProduct {
                id: 1,
                title: "Product 1".to_string(),
            },
            TestProduct {
                id: 2,
                title: "Product 2".to_string(),
            },
        ];

        let response = ResourceResponse::new(products, None, None, None);

        // Iterate via Deref to Vec
        let titles: Vec<&str> = response.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(titles, vec!["Product 1", "Product 2"]);
    }

    #[test]
    fn test_resource_response_single_allows_field_access_via_deref() {
        let product = TestProduct {
            id: 123,
            title: "Test Product".to_string(),
        };

        let response = ResourceResponse::new(product, None, None, None);

        // Access fields via Deref
        assert_eq!(response.id, 123);
        assert_eq!(response.title, "Test Product");
    }

    #[test]
    fn test_rate_limit_returns_api_call_limit() {
        let rate_limit = ApiCallLimit {
            request_count: 10,
            bucket_size: 80,
        };

        let response = ResourceResponse::new("data", None, Some(rate_limit), None);

        let limit = response.rate_limit().unwrap();
        assert_eq!(limit.request_count, 10);
        assert_eq!(limit.bucket_size, 80);
    }

    #[test]
    fn test_request_id_returns_option_str() {
        let response = ResourceResponse::new("data", None, None, Some("abc-123-xyz".to_string()));

        assert_eq!(response.request_id(), Some("abc-123-xyz"));

        let response_no_id: ResourceResponse<&str> =
            ResourceResponse::new("data", None, None, None);
        assert_eq!(response_no_id.request_id(), None);
    }

    #[test]
    fn test_from_http_response_deserializes_data() {
        let mut headers = HashMap::new();
        headers.insert("x-request-id".to_string(), vec!["req-456".to_string()]);
        headers.insert(
            "x-shopify-shop-api-call-limit".to_string(),
            vec!["5/40".to_string()],
        );

        let body = json!({
            "product": {
                "id": 123,
                "title": "Test Product"
            }
        });

        let http_response = HttpResponse::new(200, headers, body);

        let response: ResourceResponse<TestProduct> =
            ResourceResponse::from_http_response(http_response, "product").unwrap();

        assert_eq!(response.id, 123);
        assert_eq!(response.title, "Test Product");
        assert_eq!(response.request_id(), Some("req-456"));
        assert!(response.rate_limit().is_some());
    }

    #[test]
    fn test_from_http_response_preserves_pagination() {
        let mut headers = HashMap::new();
        headers.insert(
            "link".to_string(),
            vec![
                r#"<https://shop.myshopify.com/admin/api/2024-10/products.json?page_info=next123>; rel="next""#
                    .to_string(),
            ],
        );

        let body = json!({
            "products": [
                {"id": 1, "title": "Product 1"},
                {"id": 2, "title": "Product 2"}
            ]
        });

        let http_response = HttpResponse::new(200, headers, body);

        let response: ResourceResponse<Vec<TestProduct>> =
            ResourceResponse::from_http_response(http_response, "products").unwrap();

        assert!(response.has_next_page());
        assert_eq!(response.next_page_info(), Some("next123"));
    }

    #[test]
    fn test_map_transforms_data_preserving_metadata() {
        let response = ResourceResponse::new(
            vec![1, 2, 3],
            Some(PaginationInfo {
                prev_page_info: None,
                next_page_info: Some("next".to_string()),
            }),
            Some(ApiCallLimit {
                request_count: 1,
                bucket_size: 40,
            }),
            Some("req-123".to_string()),
        );

        let mapped: ResourceResponse<Vec<String>> =
            response.map(|v| v.iter().map(|n| n.to_string()).collect());

        assert_eq!(*mapped, vec!["1", "2", "3"]);
        assert!(mapped.has_next_page());
        assert!(mapped.rate_limit().is_some());
        assert_eq!(mapped.request_id(), Some("req-123"));
    }
}

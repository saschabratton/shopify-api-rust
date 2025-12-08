//! Path building infrastructure for REST resources.
//!
//! This module provides the path resolution system that enables resources to
//! support multiple access patterns (e.g., standalone and nested paths).
//!
//! # Path Resolution
//!
//! Resources can be accessed through multiple paths. For example, a `Variant`
//! resource might be accessible via:
//! - `/products/{product_id}/variants/{id}` (nested under product)
//! - `/variants/{id}` (standalone)
//!
//! The path resolution system selects the most specific path that matches
//! the available IDs. If both `product_id` and `id` are available, the
//! nested path is preferred.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::rest::{ResourcePath, ResourceOperation, get_path, build_path};
//! use shopify_api::HttpMethod;
//! use std::collections::HashMap;
//!
//! // Define paths for a resource
//! const PATHS: &[ResourcePath] = &[
//!     ResourcePath::new(
//!         HttpMethod::Get,
//!         ResourceOperation::Find,
//!         &["product_id", "id"],
//!         "products/{product_id}/variants/{id}",
//!     ),
//!     ResourcePath::new(
//!         HttpMethod::Get,
//!         ResourceOperation::Find,
//!         &["id"],
//!         "variants/{id}",
//!     ),
//! ];
//!
//! // Find the best matching path
//! let available_ids = vec!["product_id", "id"];
//! let path = get_path(PATHS, ResourceOperation::Find, &available_ids);
//! assert!(path.is_some());
//!
//! // Build the actual URL
//! let mut ids = HashMap::new();
//! ids.insert("product_id", "123");
//! ids.insert("id", "456");
//! let url = build_path(path.unwrap().template, &ids);
//! assert_eq!(url, "products/123/variants/456");
//! ```

use crate::clients::HttpMethod;
use std::collections::HashMap;
use std::fmt::Display;

/// Operations that can be performed on a REST resource.
///
/// Each operation corresponds to a specific HTTP method and URL pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceOperation {
    /// Find a single resource by ID (GET /resources/{id}).
    Find,
    /// List all resources (GET /resources).
    All,
    /// Create a new resource (POST /resources).
    Create,
    /// Update an existing resource (PUT /resources/{id}).
    Update,
    /// Delete a resource (DELETE /resources/{id}).
    Delete,
    /// Count resources (GET /resources/count).
    Count,
}

impl ResourceOperation {
    /// Returns the default HTTP method for this operation.
    #[must_use]
    pub const fn default_http_method(&self) -> HttpMethod {
        match self {
            Self::Find | Self::All | Self::Count => HttpMethod::Get,
            Self::Create => HttpMethod::Post,
            Self::Update => HttpMethod::Put,
            Self::Delete => HttpMethod::Delete,
        }
    }

    /// Returns the operation name as a string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Find => "find",
            Self::All => "all",
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Count => "count",
        }
    }
}

/// A path configuration for a REST resource operation.
///
/// Each `ResourcePath` defines how to access a resource for a specific
/// operation, including the HTTP method, required IDs, and URL template.
///
/// # Path Templates
///
/// Templates use `{id_name}` placeholders for ID interpolation:
/// - `products/{id}` - Single ID
/// - `products/{product_id}/variants/{id}` - Multiple IDs
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::{ResourcePath, ResourceOperation};
/// use shopify_api::HttpMethod;
///
/// const PRODUCT_FIND: ResourcePath = ResourcePath::new(
///     HttpMethod::Get,
///     ResourceOperation::Find,
///     &["id"],
///     "products/{id}",
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourcePath {
    /// The HTTP method for this path.
    pub http_method: HttpMethod,
    /// The operation this path is used for.
    pub operation: ResourceOperation,
    /// Required ID parameters in order (e.g., `["product_id", "id"]`).
    pub ids: &'static [&'static str],
    /// The URL template with `{id}` placeholders.
    pub template: &'static str,
}

impl ResourcePath {
    /// Creates a new `ResourcePath`.
    ///
    /// This is a `const fn` to allow paths to be defined as constants.
    ///
    /// # Arguments
    ///
    /// * `http_method` - The HTTP method for this path
    /// * `operation` - The operation this path handles
    /// * `ids` - Required ID parameter names in order
    /// * `template` - The URL template with `{id}` placeholders
    #[must_use]
    pub const fn new(
        http_method: HttpMethod,
        operation: ResourceOperation,
        ids: &'static [&'static str],
        template: &'static str,
    ) -> Self {
        Self {
            http_method,
            operation,
            ids,
            template,
        }
    }

    /// Returns the number of required IDs for this path.
    #[must_use]
    pub const fn id_count(&self) -> usize {
        self.ids.len()
    }

    /// Checks if all required IDs are available.
    ///
    /// # Arguments
    ///
    /// * `available_ids` - Slice of available ID parameter names
    #[must_use]
    pub fn matches_ids(&self, available_ids: &[&str]) -> bool {
        self.ids.iter().all(|id| available_ids.contains(id))
    }
}

/// Selects the best matching path for an operation.
///
/// The function filters paths by operation type and then selects the
/// path that:
/// 1. Has all required IDs available
/// 2. Has the most required IDs (most specific)
///
/// # Arguments
///
/// * `paths` - Available paths for the resource
/// * `operation` - The operation to perform
/// * `available_ids` - IDs that are available for path building
///
/// # Returns
///
/// The most specific matching path, or `None` if no path matches.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::{ResourcePath, ResourceOperation, get_path};
/// use shopify_api::HttpMethod;
///
/// const PATHS: &[ResourcePath] = &[
///     ResourcePath::new(HttpMethod::Get, ResourceOperation::Find, &["product_id", "id"], "products/{product_id}/variants/{id}"),
///     ResourcePath::new(HttpMethod::Get, ResourceOperation::Find, &["id"], "variants/{id}"),
/// ];
///
/// // With both IDs, prefers the nested path
/// let path = get_path(PATHS, ResourceOperation::Find, &["product_id", "id"]);
/// assert_eq!(path.unwrap().template, "products/{product_id}/variants/{id}");
///
/// // With only id, uses the standalone path
/// let path = get_path(PATHS, ResourceOperation::Find, &["id"]);
/// assert_eq!(path.unwrap().template, "variants/{id}");
/// ```
#[must_use]
pub fn get_path<'a>(
    paths: &'a [ResourcePath],
    operation: ResourceOperation,
    available_ids: &[&str],
) -> Option<&'a ResourcePath> {
    paths
        .iter()
        // Filter by operation
        .filter(|p| p.operation == operation)
        // Filter by available IDs
        .filter(|p| p.matches_ids(available_ids))
        // Select the most specific (most IDs)
        .max_by_key(|p| p.id_count())
}

/// Builds a URL from a template by interpolating IDs.
///
/// Replaces `{id_name}` placeholders in the template with values from
/// the provided map.
///
/// # Arguments
///
/// * `template` - The URL template with placeholders
/// * `ids` - A map of ID names to values
///
/// # Returns
///
/// The interpolated URL string.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::build_path;
/// use std::collections::HashMap;
///
/// let mut ids = HashMap::new();
/// ids.insert("product_id", "123");
/// ids.insert("id", "456");
///
/// let url = build_path("products/{product_id}/variants/{id}", &ids);
/// assert_eq!(url, "products/123/variants/456");
/// ```
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn build_path<V: Display>(template: &str, ids: &HashMap<&str, V>) -> String {
    let mut result = template.to_string();

    for (key, value) in ids {
        let placeholder = format!("{{{key}}}");
        result = result.replace(&placeholder, &value.to_string());
    }

    result
}

/// Builds a URL from a `ResourcePath` by interpolating IDs.
///
/// Convenience function that combines path selection with URL building.
///
/// # Arguments
///
/// * `path` - The resource path configuration
/// * `ids` - A map of ID names to values
///
/// # Returns
///
/// The interpolated URL string.
#[must_use]
#[allow(dead_code, clippy::implicit_hasher)]
pub fn build_path_from_resource_path<V: Display>(
    path: &ResourcePath,
    ids: &HashMap<&str, V>,
) -> String {
    build_path(path.template, ids)
}

// Verify types are Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ResourceOperation>();
    assert_send_sync::<ResourcePath>();
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_path_stores_fields_correctly() {
        let path = ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["product_id", "id"],
            "products/{product_id}/variants/{id}",
        );

        assert_eq!(path.http_method, HttpMethod::Get);
        assert_eq!(path.operation, ResourceOperation::Find);
        assert_eq!(path.ids, &["product_id", "id"]);
        assert_eq!(path.template, "products/{product_id}/variants/{id}");
    }

    #[test]
    fn test_path_template_interpolation_single_id() {
        let mut ids = HashMap::new();
        ids.insert("id", "123");

        let result = build_path("products/{id}", &ids);
        assert_eq!(result, "products/123");
    }

    #[test]
    fn test_path_template_interpolation_multiple_ids() {
        let mut ids = HashMap::new();
        ids.insert("product_id", "123");
        ids.insert("id", "456");

        let result = build_path("products/{product_id}/variants/{id}", &ids);
        assert_eq!(result, "products/123/variants/456");
    }

    #[test]
    fn test_get_path_selects_most_specific_path() {
        const PATHS: &[ResourcePath] = &[
            ResourcePath::new(
                HttpMethod::Get,
                ResourceOperation::Find,
                &["product_id", "id"],
                "products/{product_id}/variants/{id}",
            ),
            ResourcePath::new(
                HttpMethod::Get,
                ResourceOperation::Find,
                &["id"],
                "variants/{id}",
            ),
        ];

        // With both IDs available, should select the nested path
        let path = get_path(PATHS, ResourceOperation::Find, &["product_id", "id"]);
        assert!(path.is_some());
        assert_eq!(
            path.unwrap().template,
            "products/{product_id}/variants/{id}"
        );
    }

    #[test]
    fn test_get_path_falls_back_to_less_specific() {
        const PATHS: &[ResourcePath] = &[
            ResourcePath::new(
                HttpMethod::Get,
                ResourceOperation::Find,
                &["product_id", "id"],
                "products/{product_id}/variants/{id}",
            ),
            ResourcePath::new(
                HttpMethod::Get,
                ResourceOperation::Find,
                &["id"],
                "variants/{id}",
            ),
        ];

        // With only id available, should select standalone path
        let path = get_path(PATHS, ResourceOperation::Find, &["id"]);
        assert!(path.is_some());
        assert_eq!(path.unwrap().template, "variants/{id}");
    }

    #[test]
    fn test_get_path_returns_none_when_no_match() {
        const PATHS: &[ResourcePath] = &[ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "products/{id}",
        )];

        // Wrong operation
        let path = get_path(PATHS, ResourceOperation::Delete, &["id"]);
        assert!(path.is_none());

        // Missing required ID
        let path = get_path(PATHS, ResourceOperation::Find, &[]);
        assert!(path.is_none());
    }

    #[test]
    fn test_get_path_filters_by_operation() {
        const PATHS: &[ResourcePath] = &[
            ResourcePath::new(
                HttpMethod::Get,
                ResourceOperation::Find,
                &["id"],
                "products/{id}",
            ),
            ResourcePath::new(
                HttpMethod::Delete,
                ResourceOperation::Delete,
                &["id"],
                "products/{id}",
            ),
            ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "products"),
        ];

        let find_path = get_path(PATHS, ResourceOperation::Find, &["id"]);
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);
        assert_eq!(find_path.unwrap().operation, ResourceOperation::Find);

        let delete_path = get_path(PATHS, ResourceOperation::Delete, &["id"]);
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);
        assert_eq!(delete_path.unwrap().operation, ResourceOperation::Delete);

        let all_path = get_path(PATHS, ResourceOperation::All, &[]);
        assert_eq!(all_path.unwrap().template, "products");
    }

    #[test]
    fn test_path_building_with_optional_prefix() {
        // Simulate a resource with a prefix
        let prefix = "admin/api/2024-10";
        let template = "products/{id}";

        let mut ids = HashMap::new();
        ids.insert("id", "123");

        let path = build_path(template, &ids);
        let full_path = format!("{prefix}/{path}");

        assert_eq!(full_path, "admin/api/2024-10/products/123");
    }

    #[test]
    fn test_resource_operation_default_http_method() {
        assert_eq!(
            ResourceOperation::Find.default_http_method(),
            HttpMethod::Get
        );
        assert_eq!(
            ResourceOperation::All.default_http_method(),
            HttpMethod::Get
        );
        assert_eq!(
            ResourceOperation::Create.default_http_method(),
            HttpMethod::Post
        );
        assert_eq!(
            ResourceOperation::Update.default_http_method(),
            HttpMethod::Put
        );
        assert_eq!(
            ResourceOperation::Delete.default_http_method(),
            HttpMethod::Delete
        );
        assert_eq!(
            ResourceOperation::Count.default_http_method(),
            HttpMethod::Get
        );
    }

    #[test]
    fn test_resource_path_matches_ids() {
        let path = ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["product_id", "id"],
            "products/{product_id}/variants/{id}",
        );

        assert!(path.matches_ids(&["product_id", "id"]));
        assert!(path.matches_ids(&["product_id", "id", "extra"]));
        assert!(!path.matches_ids(&["id"]));
        assert!(!path.matches_ids(&["product_id"]));
        assert!(!path.matches_ids(&[]));
    }

    #[test]
    fn test_resource_path_id_count() {
        let path_two = ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["product_id", "id"],
            "products/{product_id}/variants/{id}",
        );
        assert_eq!(path_two.id_count(), 2);

        let path_one = ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "variants/{id}",
        );
        assert_eq!(path_one.id_count(), 1);

        let path_zero = ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "products");
        assert_eq!(path_zero.id_count(), 0);
    }

    #[test]
    fn test_build_path_handles_numeric_ids() {
        let mut ids: HashMap<&str, u64> = HashMap::new();
        ids.insert("id", 123u64);

        let result = build_path("products/{id}", &ids);
        assert_eq!(result, "products/123");
    }

    #[test]
    fn test_build_path_handles_missing_ids() {
        let ids: HashMap<&str, &str> = HashMap::new();

        // Placeholders that aren't in the map remain unchanged
        let result = build_path("products/{id}", &ids);
        assert_eq!(result, "products/{id}");
    }
}

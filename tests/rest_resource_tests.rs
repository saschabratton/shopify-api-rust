//! Integration tests for REST Resource infrastructure.
//!
//! These tests verify the REST resource trait implementation, dirty tracking,
//! path building, response wrapping, and error handling.

use serde::{Deserialize, Serialize};
use serde_json::json;
use shopify_api::clients::{ApiCallLimit, HttpResponse, PaginationInfo};
use shopify_api::rest::{
    build_path, get_path, ResourceError, ResourceOperation, ResourcePath, ResourceResponse,
    RestResource, TrackedResource,
};
use shopify_api::HttpMethod;
use std::collections::HashMap;

// ============================================================================
// Mock Resource for Testing
// ============================================================================

/// A test product resource for integration testing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
}

impl TestProduct {
    fn new(title: &str) -> Self {
        Self {
            id: None,
            title: title.to_string(),
            vendor: None,
            price: None,
        }
    }

    fn with_id(id: u64, title: &str) -> Self {
        Self {
            id: Some(id),
            title: title.to_string(),
            vendor: None,
            price: None,
        }
    }
}

/// Parameters for listing products
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

impl RestResource for TestProduct {
    type Id = u64;
    type FindParams = ();
    type AllParams = ProductListParams;
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

/// A test variant resource for nested resource testing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestVariant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
}

impl RestResource for TestVariant {
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
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["product_id"],
            "products/{product_id}/variants/count",
        ),
        // Standalone paths (less specific, fallback)
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

// ============================================================================
// Full Workflow Integration Tests
// ============================================================================

#[test]
fn test_full_workflow_find_resource_verifies_response() {
    // Define a mock resource
    let product = TestProduct::with_id(123, "Test Product");

    // Wrap in ResourceResponse (simulating API response)
    let response = ResourceResponse::new(
        product.clone(),
        None,
        Some(ApiCallLimit {
            request_count: 1,
            bucket_size: 40,
        }),
        Some("req-12345".to_string()),
    );

    // Verify response data via Deref
    assert_eq!(response.id, Some(123));
    assert_eq!(response.title, "Test Product");

    // Verify metadata
    assert_eq!(response.request_id(), Some("req-12345"));
    assert!(response.rate_limit().is_some());
}

#[test]
fn test_full_workflow_list_resources_with_pagination() {
    // Create paginated response
    let products = vec![
        TestProduct::with_id(1, "Product 1"),
        TestProduct::with_id(2, "Product 2"),
        TestProduct::with_id(3, "Product 3"),
    ];

    let response = ResourceResponse::new(
        products,
        Some(PaginationInfo {
            prev_page_info: None,
            next_page_info: Some("eyJsYXN0X2lkIjozfQ".to_string()),
        }),
        Some(ApiCallLimit {
            request_count: 2,
            bucket_size: 40,
        }),
        Some("req-paginated".to_string()),
    );

    // Iterate via Deref to Vec
    let titles: Vec<&str> = response.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, vec!["Product 1", "Product 2", "Product 3"]);

    // Check pagination
    assert!(response.has_next_page());
    assert!(!response.has_prev_page());
    assert_eq!(response.next_page_info(), Some("eyJsYXN0X2lkIjozfQ"));
}

#[test]
fn test_full_workflow_create_new_resource_with_tracking() {
    // Create new resource (no ID)
    let product = TestProduct::new("New Product");

    // Wrap in TrackedResource
    let tracked = TrackedResource::new(product);

    // New resources are always dirty
    assert!(tracked.is_dirty());
    assert!(tracked.is_new());

    // changed_fields returns serialized fields for POST
    // Note: id field is skipped when None due to skip_serializing_if
    let changes = tracked.changed_fields();
    assert!(changes.get("title").is_some());
    assert_eq!(changes.get("title").unwrap(), "New Product");

    // id is NOT included because Option::is_none skips serialization
    assert!(changes.get("id").is_none());
}

#[test]
fn test_full_workflow_load_modify_save_partial_update() {
    // Simulate loading an existing resource from API
    let product = TestProduct {
        id: Some(456),
        title: "Original Title".to_string(),
        vendor: Some("Original Vendor".to_string()),
        price: Some("100.00".to_string()),
    };

    // Create tracked resource from existing data
    let mut tracked = TrackedResource::from_existing(product);

    // Initially not dirty
    assert!(!tracked.is_dirty());
    assert!(!tracked.is_new());

    // Modify via DerefMut
    tracked.title = "Updated Title".to_string();

    // Now dirty
    assert!(tracked.is_dirty());

    // Get changed fields for partial update
    let changes = tracked.changed_fields();

    // Only title should be in changes
    assert!(changes.get("title").is_some());
    assert_eq!(changes.get("title").unwrap(), "Updated Title");

    // Unchanged fields should not be present
    assert!(changes.get("vendor").is_none());
    assert!(changes.get("price").is_none());
    assert!(changes.get("id").is_none());

    // Simulate successful save by marking clean
    tracked.mark_clean();
    assert!(!tracked.is_dirty());
}

#[test]
fn test_full_workflow_delete_resource_path_building() {
    // Build delete path for a resource
    let mut ids: HashMap<&str, u64> = HashMap::new();
    ids.insert("id", 789);

    let path = get_path(TestProduct::PATHS, ResourceOperation::Delete, &["id"]);
    assert!(path.is_some());
    assert_eq!(path.unwrap().template, "products/{id}");
    assert_eq!(path.unwrap().http_method, HttpMethod::Delete);

    let url = build_path(path.unwrap().template, &ids);
    assert_eq!(url, "products/789");
}

#[test]
fn test_full_workflow_count_resources() {
    // Verify count path exists
    let path = get_path(TestProduct::PATHS, ResourceOperation::Count, &[]);
    assert!(path.is_some());
    assert_eq!(path.unwrap().template, "products/count");
    assert_eq!(path.unwrap().http_method, HttpMethod::Get);
}

// ============================================================================
// Nested Resource Path Selection Tests
// ============================================================================

#[test]
fn test_nested_resource_path_selection_prefers_most_specific() {
    // With both product_id and id available, should select nested path
    let path = get_path(
        TestVariant::PATHS,
        ResourceOperation::Find,
        &["product_id", "id"],
    );
    assert!(path.is_some());
    assert_eq!(
        path.unwrap().template,
        "products/{product_id}/variants/{id}"
    );

    // Build the URL
    let mut ids: HashMap<&str, u64> = HashMap::new();
    ids.insert("product_id", 123);
    ids.insert("id", 456);

    let url = build_path(path.unwrap().template, &ids);
    assert_eq!(url, "products/123/variants/456");
}

#[test]
fn test_nested_resource_falls_back_to_standalone_path() {
    // With only id available, should select standalone path
    let path = get_path(TestVariant::PATHS, ResourceOperation::Find, &["id"]);
    assert!(path.is_some());
    assert_eq!(path.unwrap().template, "variants/{id}");
}

#[test]
fn test_nested_resource_all_with_parent_id() {
    // List all variants under a product
    let path = get_path(TestVariant::PATHS, ResourceOperation::All, &["product_id"]);
    assert!(path.is_some());
    assert_eq!(path.unwrap().template, "products/{product_id}/variants");

    let mut ids: HashMap<&str, u64> = HashMap::new();
    ids.insert("product_id", 999);

    let url = build_path(path.unwrap().template, &ids);
    assert_eq!(url, "products/999/variants");
}

#[test]
fn test_nested_resource_count_with_parent_id() {
    let path = get_path(
        TestVariant::PATHS,
        ResourceOperation::Count,
        &["product_id"],
    );
    assert!(path.is_some());
    assert_eq!(
        path.unwrap().template,
        "products/{product_id}/variants/count"
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_error_handling_404_maps_to_not_found() {
    let body = json!({"error": "Product not found"});

    let error = ResourceError::from_http_response(404, &body, "Product", Some("12345"), None);

    // Check the error message first (before pattern matching which moves values)
    let message = error.to_string();
    assert!(message.contains("Product"));
    assert!(message.contains("12345"));
    assert!(message.contains("not found"));

    // Now pattern match to verify the variant and values
    match error {
        ResourceError::NotFound { resource, ref id } => {
            assert_eq!(resource, "Product");
            assert_eq!(id, "12345");
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_error_handling_422_maps_to_validation_failed() {
    let body = json!({
        "errors": {
            "title": ["can't be blank", "is too short (minimum is 1 character)"],
            "price": ["must be a positive number"]
        }
    });

    let error =
        ResourceError::from_http_response(422, &body, "Product", None, Some("req-validation"));

    match error {
        ResourceError::ValidationFailed { errors, request_id } => {
            assert!(errors.contains_key("title"));
            assert!(errors.contains_key("price"));

            let title_errors = errors.get("title").unwrap();
            assert_eq!(title_errors.len(), 2);
            assert!(title_errors.contains(&"can't be blank".to_string()));

            assert_eq!(request_id, Some("req-validation".to_string()));
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_error_handling_422_with_array_format() {
    // Shopify sometimes returns errors as an array
    let body = json!({
        "errors": ["Title can't be blank", "Price must be positive"]
    });

    let error = ResourceError::from_http_response(422, &body, "Product", None, None);

    match error {
        ResourceError::ValidationFailed { errors, .. } => {
            // Array errors are stored under "base" key
            assert!(errors.contains_key("base"));
            let base_errors = errors.get("base").unwrap();
            assert_eq!(base_errors.len(), 2);
        }
        _ => panic!("Expected ValidationFailed error"),
    }
}

#[test]
fn test_error_handling_other_status_codes_map_to_http() {
    let body = json!({"error": "Internal Server Error"});

    let error = ResourceError::from_http_response(500, &body, "Product", None, Some("req-500"));

    match error {
        ResourceError::Http(_) => {
            // Correct variant
            assert_eq!(error.request_id(), Some("req-500"));
        }
        _ => panic!("Expected Http error"),
    }
}

#[test]
fn test_error_path_resolution_failed() {
    let error = ResourceError::PathResolutionFailed {
        resource: "Variant",
        operation: "all",
    };

    let message = error.to_string();
    assert!(message.contains("Variant"));
    assert!(message.contains("all"));
    assert!(message.contains("path"));
}

// ============================================================================
// Pagination Flow Tests
// ============================================================================

#[test]
fn test_pagination_next_page_info_flow() {
    // First page
    let first_page = ResourceResponse::new(
        vec![TestProduct::with_id(1, "P1"), TestProduct::with_id(2, "P2")],
        Some(PaginationInfo {
            prev_page_info: None,
            next_page_info: Some("page2token".to_string()),
        }),
        None,
        None,
    );

    assert!(!first_page.has_prev_page());
    assert!(first_page.has_next_page());
    let next_token = first_page.next_page_info().unwrap();
    assert_eq!(next_token, "page2token");

    // Second (middle) page
    let second_page = ResourceResponse::new(
        vec![TestProduct::with_id(3, "P3"), TestProduct::with_id(4, "P4")],
        Some(PaginationInfo {
            prev_page_info: Some("page1token".to_string()),
            next_page_info: Some("page3token".to_string()),
        }),
        None,
        None,
    );

    assert!(second_page.has_prev_page());
    assert!(second_page.has_next_page());

    // Last page
    let last_page = ResourceResponse::new(
        vec![TestProduct::with_id(5, "P5")],
        Some(PaginationInfo {
            prev_page_info: Some("page2token".to_string()),
            next_page_info: None,
        }),
        None,
        None,
    );

    assert!(last_page.has_prev_page());
    assert!(!last_page.has_next_page());
}

// ============================================================================
// ResourceResponse Transformation Tests
// ============================================================================

#[test]
fn test_resource_response_map_transforms_data() {
    let response = ResourceResponse::new(
        vec![
            TestProduct::with_id(1, "A"),
            TestProduct::with_id(2, "B"),
            TestProduct::with_id(3, "C"),
        ],
        Some(PaginationInfo {
            prev_page_info: None,
            next_page_info: Some("next".to_string()),
        }),
        Some(ApiCallLimit {
            request_count: 5,
            bucket_size: 40,
        }),
        Some("req-map".to_string()),
    );

    // Transform data while preserving metadata
    let titles: ResourceResponse<Vec<String>> =
        response.map(|products| products.iter().map(|p| p.title.clone()).collect());

    assert_eq!(
        *titles,
        vec!["A".to_string(), "B".to_string(), "C".to_string()]
    );

    // Metadata preserved
    assert!(titles.has_next_page());
    assert!(titles.rate_limit().is_some());
    assert_eq!(titles.request_id(), Some("req-map"));
}

#[test]
fn test_resource_response_into_inner_transfers_ownership() {
    let product = TestProduct::with_id(123, "Test");
    let response = ResourceResponse::new(product.clone(), None, None, None);

    let inner: TestProduct = response.into_inner();
    assert_eq!(inner, product);
}

// ============================================================================
// TrackedResource Edge Cases
// ============================================================================

#[test]
fn test_tracked_resource_multiple_modifications() {
    let product = TestProduct {
        id: Some(100),
        title: "Original".to_string(),
        vendor: Some("Vendor A".to_string()),
        price: Some("50.00".to_string()),
    };

    let mut tracked = TrackedResource::from_existing(product);

    // First modification
    tracked.title = "Changed Once".to_string();
    assert!(tracked.is_dirty());

    // Second modification (different field)
    tracked.vendor = Some("Vendor B".to_string());
    assert!(tracked.is_dirty());

    let changes = tracked.changed_fields();
    assert!(changes.get("title").is_some());
    assert!(changes.get("vendor").is_some());
    assert!(changes.get("price").is_none()); // Unchanged
}

#[test]
fn test_tracked_resource_mark_clean_then_modify_again() {
    let product = TestProduct::with_id(200, "Initial");
    let mut tracked = TrackedResource::from_existing(product);

    // Modify and mark clean
    tracked.title = "First Update".to_string();
    tracked.mark_clean();
    assert!(!tracked.is_dirty());

    // Modify again
    tracked.title = "Second Update".to_string();
    assert!(tracked.is_dirty());

    let changes = tracked.changed_fields();
    assert_eq!(changes.get("title").unwrap(), "Second Update");
}

// ============================================================================
// Type Export and Thread Safety Tests
// ============================================================================

#[test]
fn test_types_are_thread_safe() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<ResourceError>();
    assert_send_sync::<ResourcePath>();
    assert_send_sync::<ResourceOperation>();
    assert_send_sync::<ResourceResponse<TestProduct>>();
    assert_send_sync::<ResourceResponse<Vec<TestProduct>>>();
    assert_send_sync::<TrackedResource<TestProduct>>();
}

#[test]
fn test_types_exported_at_crate_root() {
    // Verify key types are accessible from crate root
    let _: fn(shopify_api::ResourceError) = |_| {};
    let _: fn(shopify_api::ResourceResponse<TestProduct>) = |_| {};
    let _: fn(shopify_api::TrackedResource<TestProduct>) = |_| {};
}

#[test]
fn test_types_exported_from_rest_module() {
    // Verify types accessible from rest module
    let _: fn(shopify_api::rest::ResourceError) = |_| {};
    let _: fn(shopify_api::rest::ResourceResponse<TestProduct>) = |_| {};
    let _: fn(shopify_api::rest::TrackedResource<TestProduct>) = |_| {};
    let _: fn(shopify_api::rest::ResourcePath) = |_| {};
    let _: fn(shopify_api::rest::ResourceOperation) = |_| {};
}

// ============================================================================
// HttpResponse Deserialization Tests
// ============================================================================

#[test]
fn test_from_http_response_deserializes_single_resource() {
    let body = json!({
        "product": {
            "id": 999,
            "title": "From API",
            "vendor": "Test Vendor"
        }
    });

    let mut headers = HashMap::new();
    headers.insert("x-request-id".to_string(), vec!["req-deser".to_string()]);
    headers.insert(
        "x-shopify-shop-api-call-limit".to_string(),
        vec!["3/40".to_string()],
    );

    let http_response = HttpResponse::new(200, headers, body);

    let response: ResourceResponse<TestProduct> =
        ResourceResponse::from_http_response(http_response, "product").unwrap();

    assert_eq!(response.id, Some(999));
    assert_eq!(response.title, "From API");
    assert_eq!(response.vendor, Some("Test Vendor".to_string()));
    assert_eq!(response.request_id(), Some("req-deser"));
    assert!(response.rate_limit().is_some());
}

#[test]
fn test_from_http_response_deserializes_collection() {
    let body = json!({
        "products": [
            {"id": 1, "title": "Product 1"},
            {"id": 2, "title": "Product 2"}
        ]
    });

    let mut headers = HashMap::new();
    headers.insert(
        "link".to_string(),
        vec![
            r#"<https://shop.myshopify.com/admin/api/2024-10/products.json?page_info=nexttoken>; rel="next""#
                .to_string(),
        ],
    );

    let http_response = HttpResponse::new(200, headers, body);

    let response: ResourceResponse<Vec<TestProduct>> =
        ResourceResponse::from_http_response(http_response, "products").unwrap();

    assert_eq!(response.len(), 2);
    assert_eq!(response[0].title, "Product 1");
    assert_eq!(response[1].title, "Product 2");

    assert!(response.has_next_page());
    assert_eq!(response.next_page_info(), Some("nexttoken"));
}

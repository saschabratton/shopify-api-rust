//! Integration tests for the HTTP client functionality.
//!
//! These tests verify the client configuration, request building,
//! response parsing, and error handling behavior.

use shopify_api::clients::{DataType, HttpClient, HttpMethod, HttpRequest};
use shopify_api::{AuthScopes, Session, ShopDomain};
use std::collections::HashMap;

/// Creates a test session with the given shop domain.
fn create_test_session(shop: &str, access_token: &str) -> Session {
    Session::new(
        "test-session".to_string(),
        ShopDomain::new(shop).unwrap(),
        access_token.to_string(),
        AuthScopes::new(),
        false,
        None,
    )
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_workflow_session_to_client_to_request() {
    // Create session
    let session = create_test_session("test-shop", "test-token");

    // Create HTTP client
    let client = HttpClient::new("/admin/api/2024-10", &session, None);

    // Verify client configuration
    assert_eq!(client.base_uri(), "https://test-shop.myshopify.com");
    assert_eq!(client.base_path(), "/admin/api/2024-10");
    assert!(client
        .default_headers()
        .contains_key("X-Shopify-Access-Token"));

    // Build request
    let request = HttpRequest::builder(HttpMethod::Get, "products.json")
        .query_param("limit", "50")
        .build()
        .unwrap();

    assert_eq!(request.http_method, HttpMethod::Get);
    assert_eq!(request.path, "products.json");
    assert!(request.query.is_some());
}

#[tokio::test]
async fn test_invalid_request_produces_correct_error() {
    // POST without body should fail
    let result = HttpRequest::builder(HttpMethod::Post, "products.json").build();

    assert!(matches!(
        result,
        Err(shopify_api::InvalidHttpRequestError::MissingBody { .. })
    ));

    // Body without body_type should fail when we manually construct
    let request = HttpRequest {
        http_method: HttpMethod::Get,
        path: "test".to_string(),
        body: Some(serde_json::json!({"key": "value"})),
        body_type: None,
        query: None,
        extra_headers: None,
        tries: 1,
    };

    let verify_result = request.verify();
    assert!(matches!(
        verify_result,
        Err(shopify_api::InvalidHttpRequestError::MissingBodyType)
    ));
}

#[tokio::test]
async fn test_multi_tenant_multiple_clients_with_different_sessions() {
    // Create multiple sessions for different shops
    let session1 = create_test_session("shop-one", "token-1");
    let session2 = create_test_session("shop-two", "token-2");
    let session3 = create_test_session("shop-three", "token-3");

    // Create clients for each session
    let client1 = HttpClient::new("/admin/api/2024-10", &session1, None);
    let client2 = HttpClient::new("/admin/api/2024-10", &session2, None);
    let client3 = HttpClient::new("/admin/api/2024-10", &session3, None);

    // Verify each client has independent configuration
    assert_eq!(client1.base_uri(), "https://shop-one.myshopify.com");
    assert_eq!(client2.base_uri(), "https://shop-two.myshopify.com");
    assert_eq!(client3.base_uri(), "https://shop-three.myshopify.com");

    assert_eq!(
        client1.default_headers().get("X-Shopify-Access-Token"),
        Some(&"token-1".to_string())
    );
    assert_eq!(
        client2.default_headers().get("X-Shopify-Access-Token"),
        Some(&"token-2".to_string())
    );
    assert_eq!(
        client3.default_headers().get("X-Shopify-Access-Token"),
        Some(&"token-3".to_string())
    );
}

#[tokio::test]
async fn test_request_with_all_options() {
    let session = create_test_session("test-shop", "test-token");
    let _client = HttpClient::new("/admin/api/2024-10", &session, None);

    // Build a request with all options
    let mut extra_headers = HashMap::new();
    extra_headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

    let request = HttpRequest::builder(HttpMethod::Post, "products.json")
        .body(serde_json::json!({
            "product": {
                "title": "New Product",
                "body_html": "<p>Description</p>",
                "vendor": "Test Vendor"
            }
        }))
        .body_type(DataType::Json)
        .query_param("published", "true")
        .extra_headers(extra_headers)
        .tries(3)
        .build()
        .unwrap();

    assert_eq!(request.http_method, HttpMethod::Post);
    assert_eq!(request.path, "products.json");
    assert!(request.body.is_some());
    assert_eq!(request.body_type, Some(DataType::Json));
    assert!(request.query.as_ref().unwrap().contains_key("published"));
    assert!(request
        .extra_headers
        .as_ref()
        .unwrap()
        .contains_key("X-Custom-Header"));
    assert_eq!(request.tries, 3);
}

#[tokio::test]
async fn test_response_parsing_all_header_fields() {
    use shopify_api::clients::{ApiCallLimit, HttpResponse, PaginationInfo};

    // Test ApiCallLimit parsing
    let limit = ApiCallLimit::parse("40/80").unwrap();
    assert_eq!(limit.request_count, 40);
    assert_eq!(limit.bucket_size, 80);

    // Test PaginationInfo parsing
    let link_header = r#"<https://shop.myshopify.com/admin/api/2024-10/products.json?page_info=next123>; rel="next", <https://shop.myshopify.com/admin/api/2024-10/products.json?page_info=prev456>; rel="previous""#;
    let pagination = PaginationInfo::parse_link_header(link_header);
    assert_eq!(pagination.next_page_info, Some("next123".to_string()));
    assert_eq!(pagination.prev_page_info, Some("prev456".to_string()));

    // Test HttpResponse with all parsed headers
    let mut headers = std::collections::HashMap::new();
    headers.insert(
        "x-shopify-shop-api-call-limit".to_string(),
        vec!["40/80".to_string()],
    );
    headers.insert("x-request-id".to_string(), vec!["req-12345".to_string()]);
    headers.insert("retry-after".to_string(), vec!["2.5".to_string()]);
    headers.insert("link".to_string(), vec![link_header.to_string()]);
    headers.insert(
        "x-shopify-api-deprecated-reason".to_string(),
        vec!["This endpoint is deprecated".to_string()],
    );

    let response = HttpResponse::new(200, headers, serde_json::json!({"products": []}));

    assert!(response.is_ok());
    assert_eq!(response.request_id(), Some("req-12345"));
    assert_eq!(
        response.deprecation_reason(),
        Some("This endpoint is deprecated")
    );
    assert_eq!(response.api_call_limit.unwrap().request_count, 40);
    assert!((response.retry_request_after.unwrap() - 2.5).abs() < f64::EPSILON);
    assert_eq!(response.next_page_info, Some("next123".to_string()));
    assert_eq!(response.prev_page_info, Some("prev456".to_string()));
}

#[tokio::test]
async fn test_error_types_provide_debugging_info() {
    use shopify_api::clients::{HttpResponseError, MaxHttpRetriesExceededError};

    // HttpResponseError includes status code and request ID
    let error = HttpResponseError {
        code: 422,
        message: r#"{"errors":{"title":["can't be blank"]},"error_reference":"If you report this error, please include this id: abc-123."}"#.to_string(),
        error_reference: Some("abc-123".to_string()),
    };

    let error_string = error.to_string();
    assert!(error_string.contains("title"));
    assert!(error_string.contains("abc-123"));

    // MaxHttpRetriesExceededError includes try count
    let retry_error = MaxHttpRetriesExceededError {
        code: 429,
        tries: 5,
        message: r#"{"error":"Rate limited"}"#.to_string(),
        error_reference: Some("xyz-789".to_string()),
    };

    let retry_error_string = retry_error.to_string();
    assert!(retry_error_string.contains("5"));
    assert!(retry_error_string.contains("Exceeded maximum retry count"));
}

#[tokio::test]
async fn test_data_type_content_types() {
    assert_eq!(DataType::Json.as_content_type(), "application/json");
    assert_eq!(DataType::GraphQL.as_content_type(), "application/graphql");
}

#[tokio::test]
async fn test_http_method_display() {
    assert_eq!(HttpMethod::Get.to_string(), "get");
    assert_eq!(HttpMethod::Post.to_string(), "post");
    assert_eq!(HttpMethod::Put.to_string(), "put");
    assert_eq!(HttpMethod::Delete.to_string(), "delete");
}

#[tokio::test]
async fn test_client_default_headers() {
    let session = create_test_session("my-shop", "my-token");
    let client = HttpClient::new("/admin/api/2024-10", &session, None);

    let headers = client.default_headers();

    // Should have User-Agent
    assert!(headers.contains_key("User-Agent"));
    let user_agent = headers.get("User-Agent").unwrap();
    assert!(user_agent.contains("Shopify API Library"));
    assert!(user_agent.contains("Rust"));

    // Should have Accept: application/json
    assert_eq!(headers.get("Accept"), Some(&"application/json".to_string()));

    // Should have X-Shopify-Access-Token
    assert_eq!(
        headers.get("X-Shopify-Access-Token"),
        Some(&"my-token".to_string())
    );
}

#[tokio::test]
async fn test_client_without_access_token() {
    let session = Session::new(
        "session-id".to_string(),
        ShopDomain::new("my-shop").unwrap(),
        String::new(), // Empty access token
        AuthScopes::new(),
        false,
        None,
    );
    let client = HttpClient::new("/admin/api/2024-10", &session, None);

    // Should NOT have X-Shopify-Access-Token when token is empty
    assert!(!client
        .default_headers()
        .contains_key("X-Shopify-Access-Token"));
}

#[tokio::test]
async fn test_request_builder_chaining() {
    let request = HttpRequest::builder(HttpMethod::Get, "products.json")
        .query_param("limit", "50")
        .query_param("fields", "id,title")
        .header("X-Custom", "value")
        .tries(2)
        .build()
        .unwrap();

    let query = request.query.unwrap();
    assert_eq!(query.len(), 2);
    assert_eq!(query.get("limit"), Some(&"50".to_string()));
    assert_eq!(query.get("fields"), Some(&"id,title".to_string()));

    let headers = request.extra_headers.unwrap();
    assert_eq!(headers.get("X-Custom"), Some(&"value".to_string()));

    assert_eq!(request.tries, 2);
}

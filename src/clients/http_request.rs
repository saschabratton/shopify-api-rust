//! HTTP request types for the Shopify API SDK.
//!
//! This module provides the [`HttpRequest`] type and its builder for
//! constructing requests to the Shopify API.

use std::collections::HashMap;
use std::fmt;

use crate::clients::errors::InvalidHttpRequestError;

/// HTTP methods supported by the Shopify API.
///
/// The SDK supports the four standard HTTP methods used by REST APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    /// HTTP GET method for retrieving resources.
    Get,
    /// HTTP POST method for creating resources.
    Post,
    /// HTTP PUT method for updating resources.
    Put,
    /// HTTP DELETE method for removing resources.
    Delete,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Get => write!(f, "get"),
            Self::Post => write!(f, "post"),
            Self::Put => write!(f, "put"),
            Self::Delete => write!(f, "delete"),
        }
    }
}

/// Content type for HTTP request bodies.
///
/// Specifies the format of the request body and sets the appropriate
/// `Content-Type` header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    /// JSON content type (`application/json`).
    Json,
    /// GraphQL content type (`application/graphql`).
    GraphQL,
}

impl DataType {
    /// Returns the MIME type string for this data type.
    #[must_use]
    pub const fn as_content_type(&self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::GraphQL => "application/graphql",
        }
    }
}

/// An HTTP request to be sent to the Shopify API.
///
/// Use [`HttpRequest::builder`] to construct requests with the builder pattern.
///
/// # Example
///
/// ```rust
/// use shopify_api::clients::{HttpRequest, HttpMethod, DataType};
/// use serde_json::json;
///
/// // GET request
/// let get_request = HttpRequest::builder(HttpMethod::Get, "products.json")
///     .build()
///     .unwrap();
///
/// // POST request with JSON body
/// let post_request = HttpRequest::builder(HttpMethod::Post, "products.json")
///     .body(json!({"product": {"title": "New Product"}}))
///     .body_type(DataType::Json)
///     .build()
///     .unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct HttpRequest {
    /// The HTTP method for this request.
    pub http_method: HttpMethod,
    /// The path (relative to base path) for this request.
    pub path: String,
    /// The request body, if any.
    pub body: Option<serde_json::Value>,
    /// The content type of the body.
    pub body_type: Option<DataType>,
    /// Query parameters to append to the URL.
    pub query: Option<HashMap<String, String>>,
    /// Additional headers to include in the request.
    pub extra_headers: Option<HashMap<String, String>>,
    /// Number of times to attempt the request (default: 1).
    pub tries: u32,
}

impl HttpRequest {
    /// Creates a new builder for constructing an `HttpRequest`.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method for the request
    /// * `path` - The path (relative to base path) for the request
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::clients::{HttpRequest, HttpMethod};
    ///
    /// let request = HttpRequest::builder(HttpMethod::Get, "products.json")
    ///     .tries(3)
    ///     .build()
    ///     .unwrap();
    /// ```
    #[must_use]
    pub fn builder(method: HttpMethod, path: impl Into<String>) -> HttpRequestBuilder {
        HttpRequestBuilder::new(method, path)
    }

    /// Validates the request, ensuring it meets all requirements.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidHttpRequestError`] if:
    /// - `body` is `Some` but `body_type` is `None`
    /// - `http_method` is `Post` or `Put` but `body` is `None`
    pub fn verify(&self) -> Result<(), InvalidHttpRequestError> {
        // Validate body_type is set when body is present
        if self.body.is_some() && self.body_type.is_none() {
            return Err(InvalidHttpRequestError::MissingBodyType);
        }

        // Validate body is present for POST/PUT methods
        if matches!(self.http_method, HttpMethod::Post | HttpMethod::Put) && self.body.is_none() {
            return Err(InvalidHttpRequestError::MissingBody {
                method: self.http_method.to_string(),
            });
        }

        Ok(())
    }
}

/// Builder for constructing [`HttpRequest`] instances.
///
/// Provides a fluent API for building requests with optional parameters.
#[derive(Debug)]
pub struct HttpRequestBuilder {
    http_method: HttpMethod,
    path: String,
    body: Option<serde_json::Value>,
    body_type: Option<DataType>,
    query: Option<HashMap<String, String>>,
    extra_headers: Option<HashMap<String, String>>,
    tries: u32,
}

impl HttpRequestBuilder {
    /// Creates a new builder with the required method and path.
    fn new(method: HttpMethod, path: impl Into<String>) -> Self {
        Self {
            http_method: method,
            path: path.into(),
            body: None,
            body_type: None,
            query: None,
            extra_headers: None,
            tries: 1,
        }
    }

    /// Sets the request body.
    ///
    /// When setting a body, you must also set the body type via [`body_type`](Self::body_type).
    #[must_use]
    pub fn body(mut self, body: impl Into<serde_json::Value>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Sets the content type of the request body.
    #[must_use]
    pub const fn body_type(mut self, body_type: DataType) -> Self {
        self.body_type = Some(body_type);
        self
    }

    /// Sets all query parameters at once.
    #[must_use]
    pub fn query(mut self, query: HashMap<String, String>) -> Self {
        self.query = Some(query);
        self
    }

    /// Adds a single query parameter.
    #[must_use]
    pub fn query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Sets all extra headers at once.
    #[must_use]
    pub fn extra_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.extra_headers = Some(headers);
        self
    }

    /// Adds a single extra header.
    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_headers
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Sets the number of times to attempt the request.
    ///
    /// Default is 1 (no retries). Set to a higher value to enable
    /// automatic retries for 429 and 500 responses.
    #[must_use]
    pub const fn tries(mut self, tries: u32) -> Self {
        self.tries = tries;
        self
    }

    /// Builds the [`HttpRequest`], validating it in the process.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidHttpRequestError`] if the request fails validation.
    pub fn build(self) -> Result<HttpRequest, InvalidHttpRequestError> {
        let request = HttpRequest {
            http_method: self.http_method,
            path: self.path,
            body: self.body,
            body_type: self.body_type,
            query: self.query,
            extra_headers: self.extra_headers,
            tries: self.tries,
        };
        request.verify()?;
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "get");
        assert_eq!(HttpMethod::Post.to_string(), "post");
        assert_eq!(HttpMethod::Put.to_string(), "put");
        assert_eq!(HttpMethod::Delete.to_string(), "delete");
    }

    #[test]
    fn test_data_type_content_type() {
        assert_eq!(DataType::Json.as_content_type(), "application/json");
        assert_eq!(DataType::GraphQL.as_content_type(), "application/graphql");
    }

    #[test]
    fn test_builder_creates_valid_get_request() {
        let request = HttpRequest::builder(HttpMethod::Get, "products.json")
            .build()
            .unwrap();

        assert_eq!(request.http_method, HttpMethod::Get);
        assert_eq!(request.path, "products.json");
        assert!(request.body.is_none());
        assert!(request.body_type.is_none());
        assert_eq!(request.tries, 1);
    }

    #[test]
    fn test_builder_creates_valid_post_request() {
        let request = HttpRequest::builder(HttpMethod::Post, "products.json")
            .body(json!({"product": {"title": "Test"}}))
            .body_type(DataType::Json)
            .build()
            .unwrap();

        assert_eq!(request.http_method, HttpMethod::Post);
        assert!(request.body.is_some());
        assert_eq!(request.body_type, Some(DataType::Json));
    }

    #[test]
    fn test_verify_requires_body_for_post() {
        let result = HttpRequest::builder(HttpMethod::Post, "products.json").build();

        assert!(matches!(
            result,
            Err(InvalidHttpRequestError::MissingBody { method }) if method == "post"
        ));
    }

    #[test]
    fn test_verify_requires_body_for_put() {
        let result = HttpRequest::builder(HttpMethod::Put, "products/123.json").build();

        assert!(matches!(
            result,
            Err(InvalidHttpRequestError::MissingBody { method }) if method == "put"
        ));
    }

    #[test]
    fn test_verify_requires_body_type_when_body_present() {
        let request = HttpRequest {
            http_method: HttpMethod::Get,
            path: "test".to_string(),
            body: Some(json!({"key": "value"})),
            body_type: None,
            query: None,
            extra_headers: None,
            tries: 1,
        };

        assert!(matches!(
            request.verify(),
            Err(InvalidHttpRequestError::MissingBodyType)
        ));
    }

    #[test]
    fn test_builder_with_query_params() {
        let request = HttpRequest::builder(HttpMethod::Get, "products.json")
            .query_param("limit", "50")
            .query_param("page_info", "abc123")
            .build()
            .unwrap();

        let query = request.query.unwrap();
        assert_eq!(query.get("limit"), Some(&"50".to_string()));
        assert_eq!(query.get("page_info"), Some(&"abc123".to_string()));
    }

    #[test]
    fn test_builder_with_extra_headers() {
        let request = HttpRequest::builder(HttpMethod::Get, "products.json")
            .header("X-Custom-Header", "custom-value")
            .build()
            .unwrap();

        let headers = request.extra_headers.unwrap();
        assert_eq!(
            headers.get("X-Custom-Header"),
            Some(&"custom-value".to_string())
        );
    }

    #[test]
    fn test_default_tries_is_one() {
        let request = HttpRequest::builder(HttpMethod::Get, "test")
            .build()
            .unwrap();
        assert_eq!(request.tries, 1);
    }
}

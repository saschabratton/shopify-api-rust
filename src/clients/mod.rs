//! HTTP client types for Shopify API communication.
//!
//! This module provides the foundational HTTP client layer for making
//! authenticated requests to the Shopify API. It handles request/response
//! processing, retry logic, and Shopify-specific header parsing.
//!
//! # Overview
//!
//! The main types in this module are:
//!
//! - [`HttpClient`]: The async HTTP client for API communication
//! - [`HttpRequest`]: A request to be sent to the API
//! - [`HttpResponse`]: A parsed response from the API
//! - [`HttpMethod`]: Supported HTTP methods (GET, POST, PUT, DELETE)
//! - [`DataType`]: Content types for request bodies
//! - [`rest::RestClient`]: Higher-level REST API client
//! - [`rest::RestError`]: REST-specific error types
//! - [`graphql::GraphqlClient`]: Higher-level GraphQL API client (Admin API)
//! - [`graphql::GraphqlError`]: GraphQL-specific error types
//! - [`storefront::StorefrontClient`]: GraphQL client for Storefront API
//! - [`storefront::StorefrontToken`]: Token type for Storefront API authentication
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::{HttpClient, HttpRequest, HttpMethod, DataType, Session, ShopDomain};
//!
//! // Create a session
//! let session = Session::new(
//!     "session-id".to_string(),
//!     ShopDomain::new("my-store").unwrap(),
//!     "access-token".to_string(),
//!     "read_products".parse().unwrap(),
//!     false,
//!     None,
//! );
//!
//! // Create an HTTP client
//! let client = HttpClient::new("/admin/api/2024-10", &session, None);
//!
//! // Build and send a request
//! let request = HttpRequest::builder(HttpMethod::Get, "products.json")
//!     .build()
//!     .unwrap();
//!
//! let response = client.request(request).await?;
//! ```
//!
//! # Retry Behavior
//!
//! The client implements automatic retry logic for transient failures:
//!
//! - **429 (Rate Limited)**: Retries using `Retry-After` header value, or 1 second if not present
//! - **500 (Server Error)**: Retries with fixed 1-second delay
//! - **Other errors (4xx)**: Returns immediately without retry
//!
//! The default `tries` is 1, meaning no automatic retries. Configure via
//! [`HttpRequest::builder`] with `.tries(n)` to enable retries.

mod errors;
pub mod graphql;
mod http_client;
mod http_request;
mod http_response;
pub mod rest;
pub mod storefront;

pub use errors::{
    HttpError, HttpResponseError, InvalidHttpRequestError, MaxHttpRetriesExceededError,
};
pub use http_client::{HttpClient, SDK_VERSION};
pub use http_request::{DataType, HttpMethod, HttpRequest, HttpRequestBuilder};
pub use http_response::{ApiCallLimit, ApiDeprecationInfo, HttpResponse, PaginationInfo};

// Re-export REST client types at the clients module level
pub use rest::{RestClient, RestError};

// Re-export GraphQL client types at the clients module level
pub use graphql::{GraphqlClient, GraphqlError};

// Re-export Storefront client types at the clients module level
pub use storefront::{StorefrontClient, StorefrontToken};

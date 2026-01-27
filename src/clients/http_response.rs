//! HTTP response types for the Shopify API SDK.
//!
//! This module provides the [`HttpResponse`] type and related types for
//! parsing and accessing API response data.

use std::collections::HashMap;

/// Information about a deprecated API endpoint or feature.
///
/// When Shopify deprecates an API endpoint, they include the
/// `X-Shopify-API-Deprecated-Reason` header in responses. This struct
/// provides structured access to that deprecation information.
///
/// # Example
///
/// ```rust
/// use shopify_api::ApiDeprecationInfo;
///
/// let info = ApiDeprecationInfo {
///     reason: "This endpoint will be removed in 2025-07".to_string(),
///     path: Some("/admin/api/2024-01/products.json".to_string()),
/// };
///
/// println!("Deprecation: {} at {:?}", info.reason, info.path);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiDeprecationInfo {
    /// The reason for deprecation from the `X-Shopify-API-Deprecated-Reason` header.
    pub reason: String,
    /// The request path that triggered the deprecation notice, if available.
    pub path: Option<String>,
}

/// Rate limit information parsed from the `X-Shopify-Shop-Api-Call-Limit` header.
///
/// The header format is "X/Y" where X is the current request count and Y is
/// the bucket size.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::clients::ApiCallLimit;
///
/// let limit = ApiCallLimit::parse("40/80").unwrap();
/// assert_eq!(limit.request_count, 40);
/// assert_eq!(limit.bucket_size, 80);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApiCallLimit {
    /// The current number of requests made in this bucket.
    pub request_count: u32,
    /// The maximum number of requests allowed in this bucket.
    pub bucket_size: u32,
}

impl ApiCallLimit {
    /// Parses the rate limit header value.
    ///
    /// # Arguments
    ///
    /// * `header_value` - The header value in "X/Y" format
    ///
    /// # Returns
    ///
    /// `Some(ApiCallLimit)` if parsing succeeds, `None` otherwise.
    #[must_use]
    pub fn parse(header_value: &str) -> Option<Self> {
        let parts: Vec<&str> = header_value.split('/').collect();
        if parts.len() != 2 {
            return None;
        }

        let request_count = parts[0].parse().ok()?;
        let bucket_size = parts[1].parse().ok()?;

        Some(Self {
            request_count,
            bucket_size,
        })
    }
}

/// Pagination information parsed from the `Link` header.
///
/// Shopify uses cursor-based pagination with `page_info` parameters in
/// the Link header URLs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaginationInfo {
    /// The `page_info` value for the previous page, if available.
    pub prev_page_info: Option<String>,
    /// The `page_info` value for the next page, if available.
    pub next_page_info: Option<String>,
}

impl PaginationInfo {
    /// Parses pagination info from a Link header value.
    ///
    /// The Link header format is:
    /// `<url>; rel="next", <url>; rel="previous"`
    ///
    /// # Arguments
    ///
    /// * `header_value` - The Link header value
    #[must_use]
    pub fn parse_link_header(header_value: &str) -> Self {
        let mut result = Self::default();

        for link in header_value.split(',') {
            let link = link.trim();

            // Extract rel type
            let rel = link.split(';').find_map(|part| {
                let part = part.trim();
                if part.starts_with("rel=") {
                    // Remove rel=" and trailing "
                    Some(part.trim_start_matches("rel=").trim_matches('"'))
                } else {
                    None
                }
            });

            // Extract URL
            let url = link
                .split(';')
                .next()
                .map(|s| s.trim().trim_start_matches('<').trim_end_matches('>'));

            if let (Some(rel), Some(url)) = (rel, url) {
                // Extract page_info from URL query params
                if let Some(page_info) = Self::extract_page_info(url) {
                    match rel {
                        "previous" => result.prev_page_info = Some(page_info),
                        "next" => result.next_page_info = Some(page_info),
                        _ => {}
                    }
                }
            }
        }

        result
    }

    /// Extracts the `page_info` parameter from a URL.
    fn extract_page_info(url: &str) -> Option<String> {
        // Find the query string
        let query_start = url.find('?')?;
        let query = &url[query_start + 1..];

        // Parse query parameters
        for param in query.split('&') {
            let mut parts = param.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                if key == "page_info" {
                    return Some(value.to_string());
                }
            }
        }

        None
    }
}

/// An HTTP response from the Shopify API.
///
/// Contains the response status code, headers, body, and parsed
/// Shopify-specific header values like rate limits and pagination.
#[derive(Clone, Debug)]
pub struct HttpResponse {
    /// The HTTP status code.
    pub code: u16,
    /// Response headers (headers may have multiple values).
    pub headers: HashMap<String, Vec<String>>,
    /// The parsed response body.
    pub body: serde_json::Value,
    /// Page info for the previous page (from Link header).
    pub prev_page_info: Option<String>,
    /// Page info for the next page (from Link header).
    pub next_page_info: Option<String>,
    /// Rate limit information (from `X-Shopify-Shop-Api-Call-Limit` header).
    pub api_call_limit: Option<ApiCallLimit>,
    /// Seconds to wait before retrying (from `Retry-After` header).
    pub retry_request_after: Option<f64>,
}

impl HttpResponse {
    /// Creates a new `HttpResponse` with automatic header parsing.
    ///
    /// This constructor parses Shopify-specific headers automatically:
    /// - `X-Shopify-Shop-Api-Call-Limit` -> `api_call_limit`
    /// - `Link` -> `prev_page_info`, `next_page_info`
    /// - `Retry-After` -> `retry_request_after`
    #[must_use]
    pub fn new(code: u16, headers: HashMap<String, Vec<String>>, body: serde_json::Value) -> Self {
        // Parse Link header for pagination
        let (prev_page_info, next_page_info) = headers
            .get("link")
            .and_then(|values| values.first())
            .map_or((None, None), |link| {
                let info = PaginationInfo::parse_link_header(link);
                (info.prev_page_info, info.next_page_info)
            });

        // Parse API call limit
        let api_call_limit = headers
            .get("x-shopify-shop-api-call-limit")
            .and_then(|values| values.first())
            .and_then(|value| ApiCallLimit::parse(value));

        // Parse Retry-After
        let retry_request_after = headers
            .get("retry-after")
            .and_then(|values| values.first())
            .and_then(|value| value.parse::<f64>().ok());

        Self {
            code,
            headers,
            body,
            prev_page_info,
            next_page_info,
            api_call_limit,
            retry_request_after,
        }
    }

    /// Returns `true` if the response status code is in the 2xx range.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        self.code >= 200 && self.code <= 299
    }

    /// Returns the `X-Request-Id` header value, if present.
    ///
    /// This ID is useful for debugging and should be included in error reports.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        self.headers
            .get("x-request-id")
            .and_then(|values| values.first())
            .map(String::as_str)
    }

    /// Returns the `X-Shopify-API-Deprecated-Reason` header value, if present.
    ///
    /// When present, this indicates the API endpoint is deprecated and
    /// should be updated.
    #[must_use]
    pub fn deprecation_reason(&self) -> Option<&str> {
        self.headers
            .get("x-shopify-api-deprecated-reason")
            .and_then(|values| values.first())
            .map(String::as_str)
    }

    /// Returns structured deprecation information if the response indicates deprecation.
    ///
    /// This method parses the `X-Shopify-API-Deprecated-Reason` header and returns
    /// an [`ApiDeprecationInfo`] struct with the deprecation details.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::HttpResponse;
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "x-shopify-api-deprecated-reason".to_string(),
    ///     vec!["This endpoint is deprecated".to_string()],
    /// );
    ///
    /// let response = HttpResponse::new(200, headers, json!({}));
    ///
    /// if let Some(info) = response.deprecation_info() {
    ///     println!("Warning: {}", info.reason);
    /// }
    /// ```
    #[must_use]
    pub fn deprecation_info(&self) -> Option<ApiDeprecationInfo> {
        self.deprecation_reason().map(|reason| ApiDeprecationInfo {
            reason: reason.to_string(),
            path: None, // Path is set by the caller who knows the request path
        })
    }

    /// Returns `true` if the response indicates a deprecated API endpoint.
    ///
    /// This checks for the presence of the `X-Shopify-API-Deprecated-Reason` header.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::HttpResponse;
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// let mut headers = HashMap::new();
    /// headers.insert(
    ///     "x-shopify-api-deprecated-reason".to_string(),
    ///     vec!["This endpoint is deprecated".to_string()],
    /// );
    ///
    /// let response = HttpResponse::new(200, headers, json!({}));
    /// assert!(response.is_deprecated());
    /// ```
    #[must_use]
    pub fn is_deprecated(&self) -> bool {
        self.deprecation_reason().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_ok_returns_true_for_2xx() {
        for code in 200..=299 {
            let response = HttpResponse::new(code, HashMap::new(), json!({}));
            assert!(
                response.is_ok(),
                "Expected is_ok() to be true for code {code}"
            );
        }
    }

    #[test]
    fn test_is_ok_returns_false_for_4xx_and_5xx() {
        let response_400 = HttpResponse::new(400, HashMap::new(), json!({}));
        assert!(!response_400.is_ok());

        let response_404 = HttpResponse::new(404, HashMap::new(), json!({}));
        assert!(!response_404.is_ok());

        let response_429 = HttpResponse::new(429, HashMap::new(), json!({}));
        assert!(!response_429.is_ok());

        let response_500 = HttpResponse::new(500, HashMap::new(), json!({}));
        assert!(!response_500.is_ok());
    }

    #[test]
    fn test_api_call_limit_parsing() {
        let limit = ApiCallLimit::parse("40/80").unwrap();
        assert_eq!(limit.request_count, 40);
        assert_eq!(limit.bucket_size, 80);

        let limit = ApiCallLimit::parse("1/40").unwrap();
        assert_eq!(limit.request_count, 1);
        assert_eq!(limit.bucket_size, 40);

        // Invalid formats
        assert!(ApiCallLimit::parse("invalid").is_none());
        assert!(ApiCallLimit::parse("40").is_none());
        assert!(ApiCallLimit::parse("40/").is_none());
        assert!(ApiCallLimit::parse("/80").is_none());
        assert!(ApiCallLimit::parse("abc/def").is_none());
    }

    #[test]
    fn test_link_header_parsing() {
        // Both prev and next
        let link = r#"<https://shop.myshopify.com/admin/api/2024-10/products.json?page_info=abc123>; rel="next", <https://shop.myshopify.com/admin/api/2024-10/products.json?page_info=xyz789>; rel="previous""#;
        let info = PaginationInfo::parse_link_header(link);
        assert_eq!(info.next_page_info, Some("abc123".to_string()));
        assert_eq!(info.prev_page_info, Some("xyz789".to_string()));

        // Only next
        let link = r#"<https://shop.myshopify.com/admin/api/2024-10/products.json?page_info=abc123>; rel="next""#;
        let info = PaginationInfo::parse_link_header(link);
        assert_eq!(info.next_page_info, Some("abc123".to_string()));
        assert!(info.prev_page_info.is_none());

        // Only prev
        let link = r#"<https://shop.myshopify.com/admin/api/2024-10/products.json?page_info=xyz789>; rel="previous""#;
        let info = PaginationInfo::parse_link_header(link);
        assert!(info.next_page_info.is_none());
        assert_eq!(info.prev_page_info, Some("xyz789".to_string()));
    }

    #[test]
    fn test_retry_after_parsing() {
        let mut headers = HashMap::new();
        headers.insert("retry-after".to_string(), vec!["2.5".to_string()]);

        let response = HttpResponse::new(429, headers, json!({}));
        assert!((response.retry_request_after.unwrap() - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_body_returns_empty_json() {
        let response = HttpResponse::new(200, HashMap::new(), json!({}));
        assert_eq!(response.body, json!({}));
    }

    #[test]
    fn test_request_id_extraction() {
        let mut headers = HashMap::new();
        headers.insert("x-request-id".to_string(), vec!["abc-123-xyz".to_string()]);

        let response = HttpResponse::new(200, headers, json!({}));
        assert_eq!(response.request_id(), Some("abc-123-xyz"));
    }

    #[test]
    fn test_deprecation_reason_extraction() {
        let mut headers = HashMap::new();
        headers.insert(
            "x-shopify-api-deprecated-reason".to_string(),
            vec!["This endpoint is deprecated".to_string()],
        );

        let response = HttpResponse::new(200, headers, json!({}));
        assert_eq!(
            response.deprecation_reason(),
            Some("This endpoint is deprecated")
        );
    }

    #[test]
    fn test_deprecation_info_parses_header() {
        let mut headers = HashMap::new();
        headers.insert(
            "x-shopify-api-deprecated-reason".to_string(),
            vec!["This endpoint will be removed in 2025-07".to_string()],
        );

        let response = HttpResponse::new(200, headers, json!({}));
        let info = response.deprecation_info().unwrap();

        assert_eq!(info.reason, "This endpoint will be removed in 2025-07");
        assert!(info.path.is_none()); // Path is set by caller
    }

    #[test]
    fn test_deprecation_info_returns_none_when_not_deprecated() {
        let response = HttpResponse::new(200, HashMap::new(), json!({}));
        assert!(response.deprecation_info().is_none());
    }

    #[test]
    fn test_is_deprecated_true_when_header_present() {
        let mut headers = HashMap::new();
        headers.insert(
            "x-shopify-api-deprecated-reason".to_string(),
            vec!["Deprecated".to_string()],
        );

        let response = HttpResponse::new(200, headers, json!({}));
        assert!(response.is_deprecated());
    }

    #[test]
    fn test_is_deprecated_false_when_no_header() {
        let response = HttpResponse::new(200, HashMap::new(), json!({}));
        assert!(!response.is_deprecated());
    }
}

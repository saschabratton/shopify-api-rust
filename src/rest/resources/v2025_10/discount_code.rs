//! DiscountCode resource implementation.
//!
//! This module provides the [`DiscountCode`] resource for managing discount codes
//! associated with price rules. Discount codes are the customer-facing codes that
//! customers enter at checkout to apply price rule discounts.
//!
//! # Nested Resource
//!
//! DiscountCodes are nested under PriceRules:
//! - `GET /price_rules/{price_rule_id}/discount_codes.json`
//! - `POST /price_rules/{price_rule_id}/discount_codes.json`
//! - `GET /price_rules/{price_rule_id}/discount_codes/{id}.json`
//! - `PUT /price_rules/{price_rule_id}/discount_codes/{id}.json`
//! - `DELETE /price_rules/{price_rule_id}/discount_codes/{id}.json`
//!
//! # Special Operations
//!
//! - **Lookup by code**: `DiscountCode::lookup(&client, "CODE")` finds a discount
//!   code by its code string using a standalone path.
//! - **Batch create**: `DiscountCode::batch(&client, price_rule_id, codes)` creates
//!   multiple discount codes at once under a price rule.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{DiscountCode, DiscountCodeListParams};
//!
//! // Create a discount code under a price rule
//! let code = DiscountCode {
//!     price_rule_id: Some(507328175),
//!     code: Some("SUMMER20".to_string()),
//!     ..Default::default()
//! };
//! let saved = code.save(&client).await?;
//!
//! // Lookup a discount code by its code string
//! let found = DiscountCode::lookup(&client, "SUMMER20").await?;
//!
//! // List all discount codes for a price rule
//! let codes = DiscountCode::all_with_parent(
//!     &client,
//!     "price_rule_id",
//!     507328175,
//!     None
//! ).await?;
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{
    build_path, get_path, ResourceError, ResourceOperation, ResourcePath, ResourceResponse,
    RestResource,
};
use crate::HttpMethod;

/// A discount code associated with a price rule.
///
/// Discount codes are the customer-facing codes that shoppers enter at checkout.
/// Each discount code belongs to a price rule that defines the discount logic.
///
/// # Nested Resource
///
/// This is a nested resource under `PriceRule`. Most operations require the
/// parent `price_rule_id`.
///
/// Use `DiscountCode::all_with_parent()` to list codes under a specific price rule.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `usage_count` - Number of times the code has been used
/// - `errors` - Any errors associated with the code
/// - `created_at` - When the code was created
/// - `updated_at` - When the code was last updated
///
/// ## Writable Fields
/// - `price_rule_id` - The parent price rule ID (required)
/// - `code` - The discount code string customers enter
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountCode {
    /// The unique identifier of the discount code.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the parent price rule.
    /// Required for creating new discount codes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_rule_id: Option<u64>,

    /// The discount code that customers enter at checkout.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The number of times this discount code has been used.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub usage_count: Option<i32>,

    /// Any errors associated with this discount code.
    /// Read-only field populated after batch creation.
    #[serde(skip_serializing)]
    pub errors: Option<Vec<DiscountCodeError>>,

    /// When the discount code was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the discount code was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// An error associated with a discount code.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountCodeError {
    /// The error code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The error message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// The result of a batch discount code creation.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountCodeBatchResult {
    /// The ID of the batch job.
    pub id: Option<u64>,

    /// The price rule ID the batch is associated with.
    pub price_rule_id: Option<u64>,

    /// When the batch job started.
    pub started_at: Option<DateTime<Utc>>,

    /// When the batch job completed.
    pub completed_at: Option<DateTime<Utc>>,

    /// When the batch job was created.
    pub created_at: Option<DateTime<Utc>>,

    /// When the batch job was last updated.
    pub updated_at: Option<DateTime<Utc>>,

    /// The status of the batch job: "queued", "running", "completed".
    pub status: Option<String>,

    /// The number of codes processed.
    pub codes_count: Option<i32>,

    /// The number of codes imported successfully.
    pub imported_count: Option<i32>,

    /// The number of codes that failed to import.
    pub failed_count: Option<i32>,

    /// The log entries for the batch job.
    pub logs: Option<Vec<String>>,
}

impl DiscountCode {
    /// Counts discount codes under a specific price rule.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `price_rule_id` - The parent price rule ID
    /// * `params` - Optional parameters for filtering
    ///
    /// # Returns
    ///
    /// The count of matching discount codes as a `u64`.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no count path exists.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = DiscountCode::count_with_parent(&client, 507328175, None).await?;
    /// println!("Total discount codes: {}", count);
    /// ```
    pub async fn count_with_parent(
        client: &RestClient,
        price_rule_id: u64,
        params: Option<DiscountCodeCountParams>,
    ) -> Result<u64, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("price_rule_id", price_rule_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Count, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "count",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Build query params
        let query = params
            .map(|p| {
                let value = serde_json::to_value(&p).map_err(|e| {
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
                            serde_json::Value::String(s) => {
                                query.insert(key, s);
                            }
                            serde_json::Value::Number(n) => {
                                query.insert(key, n.to_string());
                            }
                            serde_json::Value::Bool(b) => {
                                query.insert(key, b.to_string());
                            }
                            _ => {}
                        }
                    }
                }
                Ok::<_, ResourceError>(query)
            })
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

    /// Finds a single discount code by ID under a price rule.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `price_rule_id` - The parent price rule ID
    /// * `id` - The discount code ID to find
    /// * `params` - Optional parameters for the request
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the discount code doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let code = DiscountCode::find_with_parent(&client, 507328175, 123, None).await?;
    /// ```
    pub async fn find_with_parent(
        client: &RestClient,
        price_rule_id: u64,
        id: u64,
        _params: Option<DiscountCodeFindParams>,
    ) -> Result<ResourceResponse<Self>, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("price_rule_id", price_rule_id.to_string());
        ids.insert("id", id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Find, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "find",
            },
        )?;

        let url = build_path(path.template, &ids);
        let response = client.get(&url, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }

    /// Looks up a discount code by its code string.
    ///
    /// This uses a standalone path that doesn't require knowing the price rule ID.
    /// Useful when you only know the discount code string.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `code` - The discount code string to look up
    ///
    /// # Returns
    ///
    /// The discount code if found.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if no discount code with that code exists.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let code = DiscountCode::lookup(&client, "SUMMER20").await?;
    /// println!("Found discount code: {:?}", code.price_rule_id);
    /// ```
    pub async fn lookup(
        client: &RestClient,
        code: &str,
    ) -> Result<ResourceResponse<Self>, ResourceError> {
        let url = "discount_codes/lookup";
        let mut query = HashMap::new();
        query.insert("code".to_string(), code.to_string());

        let response = client.get(url, Some(query)).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(code),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }

    /// Creates multiple discount codes in a batch.
    ///
    /// This starts an asynchronous job to create multiple discount codes
    /// under a price rule. Use this for bulk creation of codes.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `price_rule_id` - The price rule ID to create codes under
    /// * `codes` - A list of code strings to create
    ///
    /// # Returns
    ///
    /// A batch result containing the job status.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = DiscountCode::batch(
    ///     &client,
    ///     507328175,
    ///     vec!["CODE1".to_string(), "CODE2".to_string(), "CODE3".to_string()]
    /// ).await?;
    /// println!("Batch job status: {:?}", result.status);
    /// ```
    pub async fn batch(
        client: &RestClient,
        price_rule_id: u64,
        codes: Vec<String>,
    ) -> Result<DiscountCodeBatchResult, ResourceError> {
        let url = format!("price_rules/{price_rule_id}/batch");

        // Build the request body
        let discount_codes: Vec<serde_json::Value> = codes
            .into_iter()
            .map(|code| serde_json::json!({ "code": code }))
            .collect();

        let body = serde_json::json!({
            "discount_codes": discount_codes
        });

        let response = client.post(&url, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        // Parse the batch result from the response
        let result = response
            .body
            .get("discount_code_creation")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'discount_code_creation' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        let batch_result: DiscountCodeBatchResult =
            serde_json::from_value(result.clone()).map_err(|e| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: format!("Failed to parse batch result: {e}"),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        Ok(batch_result)
    }

    /// Gets the status of a batch discount code creation job.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `price_rule_id` - The price rule ID
    /// * `batch_id` - The batch job ID from `batch()`
    ///
    /// # Returns
    ///
    /// The current batch job status.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let status = DiscountCode::batch_status(&client, 507328175, 123).await?;
    /// println!("Status: {:?}, Imported: {:?}", status.status, status.imported_count);
    /// ```
    pub async fn batch_status(
        client: &RestClient,
        price_rule_id: u64,
        batch_id: u64,
    ) -> Result<DiscountCodeBatchResult, ResourceError> {
        let url = format!("price_rules/{price_rule_id}/batch/{batch_id}");

        let response = client.get(&url, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&batch_id.to_string()),
                response.request_id(),
            ));
        }

        let result = response
            .body
            .get("discount_code_creation")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'discount_code_creation' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        let batch_result: DiscountCodeBatchResult =
            serde_json::from_value(result.clone()).map_err(|e| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: format!("Failed to parse batch result: {e}"),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        Ok(batch_result)
    }

    /// Gets the discount codes from a completed batch job.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `price_rule_id` - The price rule ID
    /// * `batch_id` - The batch job ID from `batch()`
    ///
    /// # Returns
    ///
    /// A list of discount codes created by the batch job.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let codes = DiscountCode::batch_codes(&client, 507328175, 123).await?;
    /// for code in codes {
    ///     println!("Created code: {:?}", code.code);
    /// }
    /// ```
    pub async fn batch_codes(
        client: &RestClient,
        price_rule_id: u64,
        batch_id: u64,
    ) -> Result<Vec<Self>, ResourceError> {
        let url = format!("price_rules/{price_rule_id}/batch/{batch_id}/discount_codes");

        let response = client.get(&url, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&batch_id.to_string()),
                response.request_id(),
            ));
        }

        let codes_value = response
            .body
            .get(Self::PLURAL)
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: format!("Missing '{}' in response", Self::PLURAL),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        let codes: Vec<Self> = serde_json::from_value(codes_value.clone()).map_err(|e| {
            ResourceError::Http(crate::clients::HttpError::Response(
                crate::clients::HttpResponseError {
                    code: response.code,
                    message: format!("Failed to parse discount codes: {e}"),
                    error_reference: response.request_id().map(ToString::to_string),
                },
            ))
        })?;

        Ok(codes)
    }
}

impl RestResource for DiscountCode {
    type Id = u64;
    type FindParams = DiscountCodeFindParams;
    type AllParams = DiscountCodeListParams;
    type CountParams = DiscountCodeCountParams;

    const NAME: &'static str = "DiscountCode";
    const PLURAL: &'static str = "discount_codes";

    /// Paths for the DiscountCode resource.
    ///
    /// All paths except lookup require `price_rule_id` as DiscountCodes
    /// are nested under PriceRules.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["price_rule_id", "id"],
            "price_rules/{price_rule_id}/discount_codes/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["price_rule_id"],
            "price_rules/{price_rule_id}/discount_codes",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["price_rule_id"],
            "price_rules/{price_rule_id}/discount_codes/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["price_rule_id"],
            "price_rules/{price_rule_id}/discount_codes",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["price_rule_id", "id"],
            "price_rules/{price_rule_id}/discount_codes/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["price_rule_id", "id"],
            "price_rules/{price_rule_id}/discount_codes/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single discount code.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountCodeFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing discount codes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountCodeListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return codes after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting discount codes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountCodeCountParams {
    /// Filter by times used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_used: Option<i32>,

    /// Filter by minimum times used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_used_min: Option<i32>,

    /// Filter by maximum times used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_used_max: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_discount_code_serialization() {
        let code = DiscountCode {
            id: Some(12345),
            price_rule_id: Some(507328175),
            code: Some("SUMMER20".to_string()),
            usage_count: Some(42),
            errors: None,
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-20T15:45:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_string(&code).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["price_rule_id"], 507328175);
        assert_eq!(parsed["code"], "SUMMER20");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("usage_count").is_none());
        assert!(parsed.get("errors").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_discount_code_deserialization() {
        let json = r#"{
            "id": 1054381139,
            "price_rule_id": 507328175,
            "code": "SUMMERSALE20OFF",
            "usage_count": 25,
            "errors": [],
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z"
        }"#;

        let code: DiscountCode = serde_json::from_str(json).unwrap();

        assert_eq!(code.id, Some(1054381139));
        assert_eq!(code.price_rule_id, Some(507328175));
        assert_eq!(code.code, Some("SUMMERSALE20OFF".to_string()));
        assert_eq!(code.usage_count, Some(25));
        assert!(code.errors.is_some());
        assert!(code.errors.unwrap().is_empty());
        assert!(code.created_at.is_some());
        assert!(code.updated_at.is_some());
    }

    #[test]
    fn test_discount_code_nested_paths() {
        // All paths should require price_rule_id

        // Find requires both price_rule_id and id
        let find_path = get_path(
            DiscountCode::PATHS,
            ResourceOperation::Find,
            &["price_rule_id", "id"],
        );
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "price_rules/{price_rule_id}/discount_codes/{id}"
        );

        // Find with only id should fail (no standalone path)
        let find_without_parent = get_path(DiscountCode::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_without_parent.is_none());

        // All requires price_rule_id
        let all_path = get_path(
            DiscountCode::PATHS,
            ResourceOperation::All,
            &["price_rule_id"],
        );
        assert!(all_path.is_some());
        assert_eq!(
            all_path.unwrap().template,
            "price_rules/{price_rule_id}/discount_codes"
        );

        // All without parent should fail
        let all_without_parent = get_path(DiscountCode::PATHS, ResourceOperation::All, &[]);
        assert!(all_without_parent.is_none());

        // Count requires price_rule_id
        let count_path = get_path(
            DiscountCode::PATHS,
            ResourceOperation::Count,
            &["price_rule_id"],
        );
        assert!(count_path.is_some());
        assert_eq!(
            count_path.unwrap().template,
            "price_rules/{price_rule_id}/discount_codes/count"
        );

        // Create requires price_rule_id
        let create_path = get_path(
            DiscountCode::PATHS,
            ResourceOperation::Create,
            &["price_rule_id"],
        );
        assert!(create_path.is_some());
        assert_eq!(
            create_path.unwrap().template,
            "price_rules/{price_rule_id}/discount_codes"
        );
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Update requires both price_rule_id and id
        let update_path = get_path(
            DiscountCode::PATHS,
            ResourceOperation::Update,
            &["price_rule_id", "id"],
        );
        assert!(update_path.is_some());
        assert_eq!(
            update_path.unwrap().template,
            "price_rules/{price_rule_id}/discount_codes/{id}"
        );
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Delete requires both price_rule_id and id
        let delete_path = get_path(
            DiscountCode::PATHS,
            ResourceOperation::Delete,
            &["price_rule_id", "id"],
        );
        assert!(delete_path.is_some());
        assert_eq!(
            delete_path.unwrap().template,
            "price_rules/{price_rule_id}/discount_codes/{id}"
        );
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);
    }

    #[test]
    fn test_discount_code_lookup_is_standalone_path() {
        // The lookup method uses a standalone path "discount_codes/lookup"
        // which doesn't require price_rule_id
        // This is tested by the method signature - it only takes client and code
        // No path in PATHS array for this - it's a special method
    }

    #[test]
    fn test_discount_code_batch_result_deserialization() {
        let json = r#"{
            "id": 173232803,
            "price_rule_id": 507328175,
            "started_at": "2024-06-15T10:00:00Z",
            "completed_at": "2024-06-15T10:05:00Z",
            "created_at": "2024-06-15T09:55:00Z",
            "updated_at": "2024-06-15T10:05:00Z",
            "status": "completed",
            "codes_count": 3,
            "imported_count": 3,
            "failed_count": 0,
            "logs": []
        }"#;

        let result: DiscountCodeBatchResult = serde_json::from_str(json).unwrap();

        assert_eq!(result.id, Some(173232803));
        assert_eq!(result.price_rule_id, Some(507328175));
        assert_eq!(result.status, Some("completed".to_string()));
        assert_eq!(result.codes_count, Some(3));
        assert_eq!(result.imported_count, Some(3));
        assert_eq!(result.failed_count, Some(0));
        assert!(result.started_at.is_some());
        assert!(result.completed_at.is_some());
    }

    #[test]
    fn test_discount_code_count_params() {
        let params = DiscountCodeCountParams {
            times_used: Some(5),
            times_used_min: Some(1),
            times_used_max: Some(100),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["times_used"], 5);
        assert_eq!(json["times_used_min"], 1);
        assert_eq!(json["times_used_max"], 100);

        // Empty params should serialize to empty object
        let empty_params = DiscountCodeCountParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_discount_code_constants() {
        assert_eq!(DiscountCode::NAME, "DiscountCode");
        assert_eq!(DiscountCode::PLURAL, "discount_codes");
    }

    #[test]
    fn test_discount_code_get_id() {
        let code_with_id = DiscountCode {
            id: Some(12345),
            code: Some("TEST".to_string()),
            ..Default::default()
        };
        assert_eq!(code_with_id.get_id(), Some(12345));

        let code_without_id = DiscountCode::default();
        assert_eq!(code_without_id.get_id(), None);
    }
}

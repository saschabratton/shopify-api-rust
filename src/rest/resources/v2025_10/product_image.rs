//! ProductImageResource resource implementation.
//!
//! This module provides the [`ProductImageResource`] resource for managing product
//! images. Product images are nested under products.
//!
//! # Difference from `common::ProductImage`
//!
//! This `ProductImageResource` is a REST resource for direct CRUD operations on product
//! images. The `common::ProductImage` type is an embedded struct returned in Product
//! responses. Use this resource when you need to:
//! - Create new product images
//! - Update existing product images
//! - Delete product images
//! - List or count product images directly
//!
//! # Nested Resource
//!
//! ProductImages are nested under Products:
//! - `GET /products/{product_id}/images.json`
//! - `POST /products/{product_id}/images.json`
//! - `GET /products/{product_id}/images/{id}.json`
//! - `PUT /products/{product_id}/images/{id}.json`
//! - `DELETE /products/{product_id}/images/{id}.json`
//! - `GET /products/{product_id}/images/count.json`
//!
//! # Binary Upload Support
//!
//! Use the `attachment` field to upload images as base64-encoded data.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{ProductImageResource, ProductImageListParams};
//!
//! // List images for a product
//! let images = ProductImageResource::all_with_parent(&client, "product_id", 632910392, None).await?;
//!
//! // Upload a new image via URL
//! let image = ProductImageResource {
//!     product_id: Some(632910392),
//!     src: Some("https://example.com/image.png".to_string()),
//!     alt: Some("Product image".to_string()),
//!     ..Default::default()
//! };
//! let saved = image.save(&client).await?;
//!
//! // Upload via base64
//! let image_data = std::fs::read("image.png").unwrap();
//! let base64_image = base64::encode(&image_data);
//! let image = ProductImageResource {
//!     product_id: Some(632910392),
//!     attachment: Some(base64_image),
//!     ..Default::default()
//! };
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

/// A product image REST resource.
///
/// This is the REST resource for direct CRUD operations on product images.
/// See also `common::ProductImage` for the embedded struct in Product responses.
///
/// Product images can be uploaded via URL (`src`) or base64-encoded data
/// (`attachment`). Each image can be associated with specific product
/// variants via `variant_ids`.
///
/// # Nested Resource
///
/// This is a nested resource under `Product`. All operations require
/// the parent `product_id`.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `width` - Image width in pixels
/// - `height` - Image height in pixels
/// - `created_at` - When the image was created
/// - `updated_at` - When the image was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `product_id` - The parent product ID (required)
/// - `position` - The display position (1-indexed)
/// - `src` - The source URL for the image
/// - `attachment` - Base64-encoded image data
/// - `variant_ids` - IDs of variants this image is associated with
/// - `alt` - Alternative text for accessibility
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductImageResource {
    /// The unique identifier of the image.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the parent product.
    /// Required for creating new images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// The display position of the image (1-indexed).
    /// Position 1 is the main product image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,

    /// The source URL of the image.
    /// Can be used for uploading via URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// Base64-encoded image data for uploading.
    /// Use this for binary uploads instead of src.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<String>,

    /// The width of the image in pixels.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub width: Option<i32>,

    /// The height of the image in pixels.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub height: Option<i32>,

    /// IDs of variants this image is associated with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_ids: Option<Vec<u64>>,

    /// Alternative text for the image (accessibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,

    /// When the image was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the image was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this image.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl ProductImageResource {
    /// Counts images under a specific product.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client
    /// * `product_id` - The parent product ID
    /// * `params` - Optional count parameters
    pub async fn count_with_parent(
        client: &RestClient,
        product_id: u64,
        params: Option<ProductImageCountParams>,
    ) -> Result<u64, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("product_id", product_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Count, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "count",
            },
        )?;

        let url = build_path(path.template, &ids);

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

    /// Finds a single image by ID under a product.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client
    /// * `product_id` - The parent product ID
    /// * `id` - The image ID to find
    /// * `params` - Optional parameters
    pub async fn find_with_parent(
        client: &RestClient,
        product_id: u64,
        id: u64,
        _params: Option<ProductImageFindParams>,
    ) -> Result<ResourceResponse<Self>, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("product_id", product_id.to_string());
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
}

impl RestResource for ProductImageResource {
    type Id = u64;
    type FindParams = ProductImageFindParams;
    type AllParams = ProductImageListParams;
    type CountParams = ProductImageCountParams;

    const NAME: &'static str = "ProductImageResource";
    const PLURAL: &'static str = "images";

    /// Paths for the ProductImageResource.
    ///
    /// All paths require product_id - images are nested under products.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["product_id", "id"],
            "products/{product_id}/images/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["product_id"],
            "products/{product_id}/images",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["product_id"],
            "products/{product_id}/images/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["product_id"],
            "products/{product_id}/images",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["product_id", "id"],
            "products/{product_id}/images/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["product_id", "id"],
            "products/{product_id}/images/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single image.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductImageFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing images.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductImageListParams {
    /// Return images after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting images.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductImageCountParams {
    /// Return images after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_product_image_resource_serialization() {
        let image = ProductImageResource {
            id: Some(850703190),
            product_id: Some(632910392),
            position: Some(1),
            src: Some("https://cdn.shopify.com/product.jpg".to_string()),
            alt: Some("Product main image".to_string()),
            variant_ids: Some(vec![808950810, 49148385]),
            width: Some(1200),
            height: Some(800),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: None,
            attachment: None,
            admin_graphql_api_id: Some("gid://shopify/ProductImage/850703190".to_string()),
        };

        let json = serde_json::to_string(&image).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["product_id"], 632910392);
        assert_eq!(parsed["position"], 1);
        assert_eq!(parsed["src"], "https://cdn.shopify.com/product.jpg");
        assert_eq!(parsed["alt"], "Product main image");
        assert_eq!(parsed["variant_ids"], serde_json::json!([808950810, 49148385]));

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("width").is_none());
        assert!(parsed.get("height").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_product_image_resource_deserialization() {
        let json = r#"{
            "id": 850703190,
            "product_id": 632910392,
            "position": 1,
            "src": "https://cdn.shopify.com/s/files/1/0/product.jpg",
            "width": 1200,
            "height": 800,
            "variant_ids": [808950810],
            "alt": "Main product image",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/ProductImage/850703190"
        }"#;

        let image: ProductImageResource = serde_json::from_str(json).unwrap();

        assert_eq!(image.id, Some(850703190));
        assert_eq!(image.product_id, Some(632910392));
        assert_eq!(image.position, Some(1));
        assert!(image.src.is_some());
        assert_eq!(image.width, Some(1200));
        assert_eq!(image.height, Some(800));
        assert_eq!(image.variant_ids, Some(vec![808950810]));
        assert_eq!(image.alt, Some("Main product image".to_string()));
        assert!(image.created_at.is_some());
        assert!(image.updated_at.is_some());
    }

    #[test]
    fn test_product_image_resource_nested_paths() {
        // All paths require product_id

        // Find requires both product_id and id
        let find_path = get_path(
            ProductImageResource::PATHS,
            ResourceOperation::Find,
            &["product_id", "id"],
        );
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "products/{product_id}/images/{id}"
        );

        // Find with only id should fail
        let find_without_parent = get_path(ProductImageResource::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_without_parent.is_none());

        // All requires product_id
        let all_path = get_path(ProductImageResource::PATHS, ResourceOperation::All, &["product_id"]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "products/{product_id}/images");

        // Count requires product_id
        let count_path = get_path(ProductImageResource::PATHS, ResourceOperation::Count, &["product_id"]);
        assert!(count_path.is_some());
        assert_eq!(
            count_path.unwrap().template,
            "products/{product_id}/images/count"
        );

        // Create requires product_id
        let create_path = get_path(
            ProductImageResource::PATHS,
            ResourceOperation::Create,
            &["product_id"],
        );
        assert!(create_path.is_some());
        assert_eq!(
            create_path.unwrap().template,
            "products/{product_id}/images"
        );

        // Update requires both product_id and id
        let update_path = get_path(
            ProductImageResource::PATHS,
            ResourceOperation::Update,
            &["product_id", "id"],
        );
        assert!(update_path.is_some());

        // Delete requires both product_id and id
        let delete_path = get_path(
            ProductImageResource::PATHS,
            ResourceOperation::Delete,
            &["product_id", "id"],
        );
        assert!(delete_path.is_some());
    }

    #[test]
    fn test_product_image_resource_base64_attachment() {
        let image = ProductImageResource {
            product_id: Some(632910392),
            attachment: Some("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&image).unwrap();
        assert!(json.get("attachment").is_some());
        // When using attachment, src is typically not needed
    }

    #[test]
    fn test_product_image_resource_constants() {
        assert_eq!(ProductImageResource::NAME, "ProductImageResource");
        assert_eq!(ProductImageResource::PLURAL, "images");
    }

    #[test]
    fn test_product_image_resource_get_id() {
        let image_with_id = ProductImageResource {
            id: Some(850703190),
            ..Default::default()
        };
        assert_eq!(image_with_id.get_id(), Some(850703190));

        let image_without_id = ProductImageResource::default();
        assert_eq!(image_without_id.get_id(), None);
    }
}

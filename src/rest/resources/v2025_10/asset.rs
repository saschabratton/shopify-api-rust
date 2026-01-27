//! Asset resource implementation.
//!
//! This module provides the Asset resource, which represents a theme asset in a Shopify store.
//! Assets are files (templates, CSS, JavaScript, images) that make up a theme.
//!
//! # Key-Based Identification
//!
//! Unlike most resources that use numeric IDs, Assets use a string `key` as their identifier.
//! The key is the path to the asset within the theme (e.g., `templates/index.liquid`).
//!
//! # Nested Resource
//!
//! Assets are nested under themes:
//! - List: `GET /themes/{theme_id}/assets`
//! - Find: `GET /themes/{theme_id}/assets?asset[key]={key}`
//! - Create/Update: `PUT /themes/{theme_id}/assets` (uses PUT for both)
//! - Delete: `DELETE /themes/{theme_id}/assets?asset[key]={key}`
//!
//! # Binary Support
//!
//! Assets support both text and binary content:
//! - Text files use the `value` field (Liquid templates, CSS, JS)
//! - Binary files use the `attachment` field (base64-encoded images, fonts)
//!
//! Use `Asset::upload_from_bytes()` for creating binary assets and
//! `Asset::download_content()` for retrieving content regardless of type.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::Asset;
//!
//! // List all assets in a theme
//! let assets = Asset::all_with_parent(&client, "theme_id", 123, None).await?;
//! for asset in assets.iter() {
//!     println!("Asset: {}", asset.key);
//! }
//!
//! // Find a specific asset by key
//! let asset = Asset::find_by_key(&client, 123, "templates/index.liquid").await?;
//! println!("Content: {}", asset.value.as_deref().unwrap_or(""));
//!
//! // Create a text asset
//! let mut asset = Asset {
//!     key: "snippets/custom.liquid".to_string(),
//!     value: Some("<div>Custom content</div>".to_string()),
//!     ..Default::default()
//! };
//! let saved = Asset::save_to_theme(&client, 123, &asset).await?;
//!
//! // Upload a binary asset
//! let image_bytes = std::fs::read("logo.png")?;
//! let asset = Asset::upload_from_bytes("assets/logo.png", &image_bytes);
//! let saved = Asset::save_to_theme(&client, 123, &asset).await?;
//!
//! // Download asset content (handles both text and binary)
//! let content = asset.download_content()?;
//! ```

use base64::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::clients::RestClient;
use crate::rest::{build_path, get_path, ResourceError, ResourceOperation, ResourcePath};
use crate::HttpMethod;

/// A theme asset in a Shopify store.
///
/// Assets are files that make up a theme, including Liquid templates,
/// CSS stylesheets, JavaScript files, and images.
///
/// # Key-Based Identification
///
/// Assets use a string `key` instead of a numeric ID. The key is the
/// path to the asset within the theme (e.g., `templates/index.liquid`).
///
/// # Content Types
///
/// - **Text assets**: Use the `value` field (templates, CSS, JS)
/// - **Binary assets**: Use the `attachment` field (base64-encoded)
///
/// # Fields
///
/// ## Key Field (Required)
/// - `key` - The path to the asset (e.g., `templates/index.liquid`)
///
/// ## Content Fields (Mutually Exclusive)
/// - `value` - Text content for text-based assets
/// - `attachment` - Base64-encoded content for binary assets
///
/// ## Metadata (Read-Only)
/// - `public_url` - The public CDN URL for the asset
/// - `content_type` - The MIME type of the asset
/// - `size` - The size of the asset in bytes
/// - `checksum` - The MD5 checksum of the asset
/// - `created_at` - When the asset was created
/// - `updated_at` - When the asset was last updated
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Asset {
    /// The path to the asset within the theme.
    /// This serves as the asset's identifier.
    /// Examples: `templates/index.liquid`, `assets/style.css`, `config/settings_data.json`
    pub key: String,

    /// The public CDN URL for the asset.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub public_url: Option<String>,

    /// The text content of the asset.
    /// Used for text-based assets like Liquid templates, CSS, and JavaScript.
    /// Mutually exclusive with `attachment`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The base64-encoded binary content of the asset.
    /// Used for binary assets like images and fonts.
    /// Mutually exclusive with `value`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<String>,

    /// The MIME type of the asset.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub content_type: Option<String>,

    /// The size of the asset in bytes.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub size: Option<i64>,

    /// The MD5 checksum of the asset content.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub checksum: Option<String>,

    /// The ID of the theme this asset belongs to.
    /// Set when retrieved via the API.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub theme_id: Option<u64>,

    /// When the asset was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the asset was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
}

impl Asset {
    /// Available paths for the Asset resource.
    ///
    /// Assets are nested under themes and use a special query parameter
    /// pattern for accessing individual assets by key.
    const PATHS: &'static [ResourcePath] = &[
        // List all assets in a theme
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["theme_id"],
            "themes/{theme_id}/assets",
        ),
        // Create or update an asset (PUT for both operations)
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Create,
            &["theme_id"],
            "themes/{theme_id}/assets",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["theme_id"],
            "themes/{theme_id}/assets",
        ),
        // Find and Delete use query parameter for key
        // These paths are handled specially by find_by_key() and delete_from_theme()
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["theme_id"],
            "themes/{theme_id}/assets",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["theme_id"],
            "themes/{theme_id}/assets",
        ),
    ];

    /// Creates an asset ready for upload from binary content.
    ///
    /// This method base64-encodes the provided bytes and sets up the asset
    /// for saving to a theme.
    ///
    /// # Arguments
    ///
    /// * `key` - The path for the asset (e.g., `assets/logo.png`)
    /// * `bytes` - The binary content to upload
    ///
    /// # Returns
    ///
    /// An `Asset` with the `key` and `attachment` fields set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::rest::resources::v2025_10::Asset;
    ///
    /// let image_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header bytes
    /// let asset = Asset::upload_from_bytes("assets/logo.png", &image_data);
    ///
    /// assert_eq!(asset.key, "assets/logo.png");
    /// assert!(asset.attachment.is_some());
    /// assert!(asset.value.is_none());
    /// ```
    #[must_use]
    pub fn upload_from_bytes(key: &str, bytes: &[u8]) -> Self {
        Self {
            key: key.to_string(),
            attachment: Some(BASE64_STANDARD.encode(bytes)),
            value: None,
            ..Default::default()
        }
    }

    /// Downloads the asset content as bytes.
    ///
    /// This method handles both text and binary assets:
    /// - For text assets (with `value`), returns UTF-8 encoded bytes
    /// - For binary assets (with `attachment`), decodes base64 and returns bytes
    ///
    /// # Returns
    ///
    /// The asset content as a `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if neither `value` nor
    /// `attachment` is present.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::rest::resources::v2025_10::Asset;
    ///
    /// // Text asset
    /// let asset = Asset {
    ///     key: "templates/index.liquid".to_string(),
    ///     value: Some("<div>Hello</div>".to_string()),
    ///     ..Default::default()
    /// };
    /// let content = asset.download_content().unwrap();
    /// assert_eq!(content, b"<div>Hello</div>");
    ///
    /// // Binary asset (base64 encoded)
    /// let asset = Asset {
    ///     key: "assets/logo.png".to_string(),
    ///     attachment: Some("SGVsbG8=".to_string()), // "Hello" in base64
    ///     ..Default::default()
    /// };
    /// let content = asset.download_content().unwrap();
    /// assert_eq!(content, b"Hello");
    /// ```
    pub fn download_content(&self) -> Result<Vec<u8>, ResourceError> {
        if let Some(value) = &self.value {
            return Ok(value.as_bytes().to_vec());
        }

        if let Some(attachment) = &self.attachment {
            return BASE64_STANDARD.decode(attachment).map_err(|e| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: 400,
                        message: format!("Failed to decode base64 attachment: {e}"),
                        error_reference: None,
                    },
                ))
            });
        }

        Err(ResourceError::PathResolutionFailed {
            resource: "Asset",
            operation: "download_content (no value or attachment)",
        })
    }

    /// Returns whether this asset is binary.
    ///
    /// Binary assets have content in the `attachment` field (base64-encoded),
    /// while text assets have content in the `value` field.
    ///
    /// # Returns
    ///
    /// `true` if the asset has an `attachment` field set, `false` if it has
    /// a `value` field set or neither.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::rest::resources::v2025_10::Asset;
    ///
    /// let binary_asset = Asset {
    ///     key: "assets/logo.png".to_string(),
    ///     attachment: Some("SGVsbG8=".to_string()),
    ///     ..Default::default()
    /// };
    /// assert!(binary_asset.is_binary());
    ///
    /// let text_asset = Asset {
    ///     key: "templates/index.liquid".to_string(),
    ///     value: Some("<div>Hello</div>".to_string()),
    ///     ..Default::default()
    /// };
    /// assert!(!text_asset.is_binary());
    /// ```
    #[must_use]
    pub const fn is_binary(&self) -> bool {
        self.attachment.is_some()
    }

    /// Lists all assets in a theme.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    /// * `theme_id` - The ID of the theme
    /// * `params` - Optional parameters for the request
    ///
    /// # Returns
    ///
    /// A list of assets in the theme.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let assets = Asset::all_for_theme(&client, 123, None).await?;
    /// for asset in &assets {
    ///     println!("Asset: {}", asset.key);
    /// }
    /// ```
    pub async fn all_for_theme(
        client: &RestClient,
        theme_id: u64,
        params: Option<AssetListParams>,
    ) -> Result<Vec<Self>, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("theme_id", theme_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::All, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: "Asset",
                operation: "all",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Build query params
        let query = params
            .map(|p| {
                let mut query = HashMap::new();
                if let Some(fields) = p.fields {
                    query.insert("fields".to_string(), fields);
                }
                query
            })
            .filter(|q| !q.is_empty());

        let response = client.get(&url, query).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "Asset",
                None,
                response.request_id(),
            ));
        }

        // Parse the response
        let assets: Vec<Self> = response
            .body
            .get("assets")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'assets' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        Ok(assets)
    }

    /// Finds a specific asset by its key.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    /// * `theme_id` - The ID of the theme
    /// * `key` - The asset key (path)
    ///
    /// # Returns
    ///
    /// The asset if found.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the asset doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let asset = Asset::find_by_key(&client, 123, "templates/index.liquid").await?;
    /// println!("Content: {}", asset.value.as_deref().unwrap_or(""));
    /// ```
    pub async fn find_by_key(
        client: &RestClient,
        theme_id: u64,
        key: &str,
    ) -> Result<Self, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("theme_id", theme_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Find, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: "Asset",
                operation: "find",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Add the asset[key] query parameter
        let mut query = HashMap::new();
        query.insert("asset[key]".to_string(), key.to_string());

        let response = client.get(&url, Some(query)).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "Asset",
                Some(key),
                response.request_id(),
            ));
        }

        // Parse the response
        let asset: Self = response
            .body
            .get("asset")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'asset' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        Ok(asset)
    }

    /// Saves an asset to a theme (creates or updates).
    ///
    /// The Asset API uses PUT for both create and update operations.
    /// If an asset with the same key exists, it will be updated.
    /// Otherwise, a new asset will be created.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    /// * `theme_id` - The ID of the theme
    /// * `asset` - The asset to save
    ///
    /// # Returns
    ///
    /// The saved asset.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Create a text asset
    /// let asset = Asset {
    ///     key: "snippets/custom.liquid".to_string(),
    ///     value: Some("<div>Custom content</div>".to_string()),
    ///     ..Default::default()
    /// };
    /// let saved = Asset::save_to_theme(&client, 123, &asset).await?;
    ///
    /// // Update an existing asset
    /// let mut asset = Asset::find_by_key(&client, 123, "snippets/custom.liquid").await?;
    /// asset.value = Some("<div>Updated content</div>".to_string());
    /// let saved = Asset::save_to_theme(&client, 123, &asset).await?;
    /// ```
    pub async fn save_to_theme(
        client: &RestClient,
        theme_id: u64,
        asset: &Self,
    ) -> Result<Self, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("theme_id", theme_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Update, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: "Asset",
                operation: "save",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Wrap asset in the asset key
        let mut body_map = serde_json::Map::new();
        body_map.insert(
            "asset".to_string(),
            serde_json::to_value(asset).map_err(|e| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: 400,
                        message: format!("Failed to serialize asset: {e}"),
                        error_reference: None,
                    },
                ))
            })?,
        );
        let body = serde_json::Value::Object(body_map);

        let response = client.put(&url, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "Asset",
                Some(&asset.key),
                response.request_id(),
            ));
        }

        // Parse the response
        let saved_asset: Self = response
            .body
            .get("asset")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'asset' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        Ok(saved_asset)
    }

    /// Deletes an asset from a theme.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use
    /// * `theme_id` - The ID of the theme
    /// * `key` - The asset key to delete
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the asset doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Asset::delete_from_theme(&client, 123, "snippets/custom.liquid").await?;
    /// ```
    pub async fn delete_from_theme(
        client: &RestClient,
        theme_id: u64,
        key: &str,
    ) -> Result<(), ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("theme_id", theme_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Delete, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: "Asset",
                operation: "delete",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Add the asset[key] query parameter
        let mut query = HashMap::new();
        query.insert("asset[key]".to_string(), key.to_string());

        let response = client.delete(&url, Some(query)).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                "Asset",
                Some(key),
                response.request_id(),
            ));
        }

        Ok(())
    }
}

/// Parameters for listing assets.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AssetListParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_struct_serialization() {
        let asset = Asset {
            key: "templates/index.liquid".to_string(),
            public_url: Some("https://cdn.shopify.com/asset.liquid".to_string()),
            value: Some("<div>Hello World</div>".to_string()),
            attachment: None,
            content_type: Some("text/x-liquid".to_string()),
            size: Some(1234),
            checksum: Some("abc123".to_string()),
            theme_id: Some(67890),
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

        let json = serde_json::to_string(&asset).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["key"], "templates/index.liquid");
        assert_eq!(parsed["value"], "<div>Hello World</div>");

        // Read-only fields should be omitted
        assert!(parsed.get("public_url").is_none());
        assert!(parsed.get("content_type").is_none());
        assert!(parsed.get("size").is_none());
        assert!(parsed.get("checksum").is_none());
        assert!(parsed.get("theme_id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_asset_deserialization_from_api_response() {
        let json = r#"{
            "key": "templates/index.liquid",
            "public_url": "https://cdn.shopify.com/s/files/1/0001/asset.liquid",
            "value": "<div>Hello World</div>",
            "content_type": "text/x-liquid",
            "size": 1234,
            "checksum": "abc123def456",
            "theme_id": 828155753,
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z"
        }"#;

        let asset: Asset = serde_json::from_str(json).unwrap();

        assert_eq!(asset.key, "templates/index.liquid");
        assert_eq!(
            asset.public_url,
            Some("https://cdn.shopify.com/s/files/1/0001/asset.liquid".to_string())
        );
        assert_eq!(asset.value, Some("<div>Hello World</div>".to_string()));
        assert!(asset.attachment.is_none());
        assert_eq!(asset.content_type, Some("text/x-liquid".to_string()));
        assert_eq!(asset.size, Some(1234));
        assert_eq!(asset.checksum, Some("abc123def456".to_string()));
        assert_eq!(asset.theme_id, Some(828155753));
        assert!(asset.created_at.is_some());
        assert!(asset.updated_at.is_some());
    }

    #[test]
    fn test_asset_with_binary_attachment() {
        let json = r#"{
            "key": "assets/logo.png",
            "public_url": "https://cdn.shopify.com/s/files/1/0001/logo.png",
            "attachment": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
            "content_type": "image/png",
            "size": 85,
            "checksum": "png123checksum"
        }"#;

        let asset: Asset = serde_json::from_str(json).unwrap();

        assert_eq!(asset.key, "assets/logo.png");
        assert!(asset.attachment.is_some());
        assert!(asset.value.is_none());
        assert!(asset.is_binary());
    }

    #[test]
    fn test_asset_upload_from_bytes() {
        let data = b"Hello, World!";
        let asset = Asset::upload_from_bytes("assets/test.txt", data);

        assert_eq!(asset.key, "assets/test.txt");
        assert!(asset.attachment.is_some());
        assert!(asset.value.is_none());
        assert!(asset.is_binary());

        // Verify the base64 encoding
        let expected_base64 = BASE64_STANDARD.encode(data);
        assert_eq!(asset.attachment, Some(expected_base64));
    }

    #[test]
    fn test_asset_download_content_text() {
        let asset = Asset {
            key: "templates/index.liquid".to_string(),
            value: Some("<div>Hello World</div>".to_string()),
            ..Default::default()
        };

        let content = asset.download_content().unwrap();
        assert_eq!(content, b"<div>Hello World</div>");
    }

    #[test]
    fn test_asset_download_content_binary() {
        // "Hello" in base64
        let asset = Asset {
            key: "assets/test.bin".to_string(),
            attachment: Some("SGVsbG8=".to_string()),
            ..Default::default()
        };

        let content = asset.download_content().unwrap();
        assert_eq!(content, b"Hello");
    }

    #[test]
    fn test_asset_download_content_no_content() {
        let asset = Asset {
            key: "assets/empty.txt".to_string(),
            value: None,
            attachment: None,
            ..Default::default()
        };

        let result = asset.download_content();
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_is_binary() {
        // Binary asset (has attachment)
        let binary_asset = Asset {
            key: "assets/logo.png".to_string(),
            attachment: Some("SGVsbG8=".to_string()),
            ..Default::default()
        };
        assert!(binary_asset.is_binary());

        // Text asset (has value)
        let text_asset = Asset {
            key: "templates/index.liquid".to_string(),
            value: Some("<div>Hello</div>".to_string()),
            ..Default::default()
        };
        assert!(!text_asset.is_binary());

        // Empty asset (neither)
        let empty_asset = Asset {
            key: "assets/empty".to_string(),
            ..Default::default()
        };
        assert!(!empty_asset.is_binary());
    }

    #[test]
    fn test_asset_upload_and_download_roundtrip() {
        let original_data = b"Binary data with special chars: \x00\x01\x02\xFF";

        // Upload
        let asset = Asset::upload_from_bytes("assets/test.bin", original_data);

        // Download
        let downloaded = asset.download_content().unwrap();

        assert_eq!(downloaded, original_data);
    }

    #[test]
    fn test_asset_paths_are_nested_under_theme() {
        // All path should require theme_id
        let all_path = get_path(Asset::PATHS, ResourceOperation::All, &["theme_id"]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "themes/{theme_id}/assets");

        // Create path (uses PUT)
        let create_path = get_path(Asset::PATHS, ResourceOperation::Create, &["theme_id"]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "themes/{theme_id}/assets");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Put);

        // Update path (uses PUT)
        let update_path = get_path(Asset::PATHS, ResourceOperation::Update, &["theme_id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "themes/{theme_id}/assets");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Find path
        let find_path = get_path(Asset::PATHS, ResourceOperation::Find, &["theme_id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "themes/{theme_id}/assets");

        // Delete path
        let delete_path = get_path(Asset::PATHS, ResourceOperation::Delete, &["theme_id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "themes/{theme_id}/assets");

        // No standalone paths (without theme_id)
        let standalone = get_path(Asset::PATHS, ResourceOperation::All, &[]);
        assert!(standalone.is_none());
    }

    #[test]
    fn test_asset_list_params_serialization() {
        let params = AssetListParams {
            fields: Some("key,content_type,size".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["fields"], "key,content_type,size");

        // Test empty params
        let empty_params = AssetListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_asset_key_based_identification() {
        // Assets use string keys, not numeric IDs
        let asset = Asset {
            key: "templates/product.liquid".to_string(),
            value: Some("{{ product.title }}".to_string()),
            ..Default::default()
        };

        // Key should be serialized
        let json = serde_json::to_value(&asset).unwrap();
        assert_eq!(json["key"], "templates/product.liquid");
    }
}

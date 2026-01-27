//! StorefrontAccessToken resource implementation.
//!
//! This module provides the [`StorefrontAccessToken`] resource for managing
//! Storefront API access tokens.
//!
//! # Limited Operations
//!
//! StorefrontAccessToken has limited operations:
//! - **Create**: Generate a new storefront access token
//! - **List**: Get all storefront access tokens
//! - **Delete**: Revoke a storefront access token
//!
//! **No Update operation** - tokens cannot be modified, only created or deleted.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::StorefrontAccessToken;
//!
//! // Create a new storefront access token
//! let token = StorefrontAccessToken {
//!     title: Some("My Custom Storefront".to_string()),
//!     ..Default::default()
//! };
//! let saved = token.save(&client).await?;
//! println!("Token: {}", saved.access_token.as_deref().unwrap_or(""));
//!
//! // List all storefront access tokens
//! let tokens = StorefrontAccessToken::all(&client, None).await?;
//! for token in tokens.iter() {
//!     println!("{}: {}", token.title.as_deref().unwrap_or(""), token.access_token.as_deref().unwrap_or(""));
//! }
//!
//! // Delete a token
//! StorefrontAccessToken::delete(&client, token_id).await?;
//! ```

use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A Storefront API access token.
///
/// Storefront access tokens allow unauthenticated access to specific
/// storefront data through the Storefront API.
///
/// # Limited Operations
///
/// - **Create**: Yes - generate new tokens
/// - **List**: Yes - view existing tokens
/// - **Find**: No - must list all tokens
/// - **Update**: No - tokens cannot be modified
/// - **Delete**: Yes - revoke tokens
/// - **Count**: No - no count endpoint
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `access_token` - The actual token value (only shown on create)
/// - `created_at` - When the token was created
/// - `access_scope` - The access scope (always "unauthenticated_read_*")
///
/// ## Writable Fields
/// - `title` - A descriptive title for the token
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StorefrontAccessToken {
    /// The unique identifier of the token.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The actual Storefront API access token.
    /// Only returned when creating a new token.
    #[serde(skip_serializing)]
    pub access_token: Option<String>,

    /// A descriptive title for the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// When the token was created.
    #[serde(skip_serializing)]
    pub created_at: Option<String>,

    /// The access scope for this token.
    /// Storefront tokens always have unauthenticated access.
    #[serde(skip_serializing)]
    pub access_scope: Option<String>,

    /// The admin GraphQL API ID for this token.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for StorefrontAccessToken {
    type Id = u64;
    type FindParams = ();
    type AllParams = ();
    type CountParams = ();

    const NAME: &'static str = "StorefrontAccessToken";
    const PLURAL: &'static str = "storefront_access_tokens";

    /// Paths for the StorefrontAccessToken resource.
    ///
    /// Limited operations: Create, List, Delete only.
    /// No Find, Update, or Count operations.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "storefront_access_tokens",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "storefront_access_tokens",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "storefront_access_tokens/{id}",
        ),
        // Note: No Find - must list all tokens
        // Note: No Update - tokens cannot be modified
        // Note: No Count endpoint
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation, RestResource};

    #[test]
    fn test_storefront_access_token_serialization() {
        let token = StorefrontAccessToken {
            id: Some(755357713),
            access_token: Some("abc123def456".to_string()),
            title: Some("My Storefront".to_string()),
            created_at: Some("2024-01-15T10:30:00-05:00".to_string()),
            access_scope: Some("unauthenticated_read_product_listings".to_string()),
            admin_graphql_api_id: Some("gid://shopify/StorefrontAccessToken/755357713".to_string()),
        };

        let json = serde_json::to_string(&token).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Only title should be serialized
        assert_eq!(parsed["title"], "My Storefront");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("access_token").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("access_scope").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_storefront_access_token_deserialization() {
        let json = r#"{
            "id": 755357713,
            "access_token": "abc123def456ghi789",
            "title": "My Custom Storefront",
            "created_at": "2024-01-15T10:30:00-05:00",
            "access_scope": "unauthenticated_read_product_listings",
            "admin_graphql_api_id": "gid://shopify/StorefrontAccessToken/755357713"
        }"#;

        let token: StorefrontAccessToken = serde_json::from_str(json).unwrap();

        assert_eq!(token.id, Some(755357713));
        assert_eq!(token.access_token, Some("abc123def456ghi789".to_string()));
        assert_eq!(token.title, Some("My Custom Storefront".to_string()));
        assert_eq!(
            token.created_at,
            Some("2024-01-15T10:30:00-05:00".to_string())
        );
        assert_eq!(
            token.access_scope,
            Some("unauthenticated_read_product_listings".to_string())
        );
    }

    #[test]
    fn test_storefront_access_token_limited_paths() {
        // All (list)
        let all_path = get_path(StorefrontAccessToken::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "storefront_access_tokens");

        // Create
        let create_path = get_path(StorefrontAccessToken::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "storefront_access_tokens");

        // Delete
        let delete_path =
            get_path(StorefrontAccessToken::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(
            delete_path.unwrap().template,
            "storefront_access_tokens/{id}"
        );

        // No Find
        let find_path = get_path(StorefrontAccessToken::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_none());

        // No Update
        let update_path =
            get_path(StorefrontAccessToken::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());

        // No Count
        let count_path = get_path(StorefrontAccessToken::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());
    }

    #[test]
    fn test_storefront_access_token_constants() {
        assert_eq!(StorefrontAccessToken::NAME, "StorefrontAccessToken");
        assert_eq!(StorefrontAccessToken::PLURAL, "storefront_access_tokens");
    }

    #[test]
    fn test_storefront_access_token_get_id() {
        let token_with_id = StorefrontAccessToken {
            id: Some(755357713),
            title: Some("Test".to_string()),
            ..Default::default()
        };
        assert_eq!(token_with_id.get_id(), Some(755357713));

        let token_without_id = StorefrontAccessToken::default();
        assert_eq!(token_without_id.get_id(), None);
    }
}

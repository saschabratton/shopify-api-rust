//! Redirect resource implementation.
//!
//! This module provides the [`Redirect`] resource for managing URL redirects
//! in a Shopify store. Redirects allow merchants to set up URL redirections
//! from old URLs to new URLs.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{Redirect, RedirectListParams};
//!
//! // Find a single redirect
//! let redirect = Redirect::find(&client, 123, None).await?;
//! println!("Redirect: {} -> {}", redirect.path.as_deref().unwrap_or(""), redirect.target.as_deref().unwrap_or(""));
//!
//! // List all redirects
//! let redirects = Redirect::all(&client, None).await?;
//! for redirect in redirects.iter() {
//!     println!("Redirect: {} -> {}", redirect.path.as_deref().unwrap_or(""), redirect.target.as_deref().unwrap_or(""));
//! }
//!
//! // Create a new redirect
//! let mut redirect = Redirect {
//!     path: Some("/old-page".to_string()),
//!     target: Some("/new-page".to_string()),
//!     ..Default::default()
//! };
//! let saved = redirect.save(&client).await?;
//!
//! // Count redirects
//! let count = Redirect::count(&client, None).await?;
//! println!("Total redirects: {}", count);
//! ```

use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A URL redirect in a Shopify store.
///
/// Redirects allow merchants to set up automatic URL redirections from old
/// URLs to new URLs. This is useful for maintaining SEO when pages are moved
/// or renamed.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the redirect
///
/// ## Writable Fields
/// - `path` - The old path to redirect from (e.g., "/old-page")
/// - `target` - The new URL to redirect to (e.g., "/new-page" or full URL)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Redirect {
    /// The unique identifier of the redirect.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The old path to redirect from.
    /// Must start with "/" (e.g., "/old-page").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// The target URL to redirect to.
    /// Can be a path (e.g., "/new-page") or a full URL (e.g., "https://example.com/page").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

impl RestResource for Redirect {
    type Id = u64;
    type FindParams = RedirectFindParams;
    type AllParams = RedirectListParams;
    type CountParams = RedirectCountParams;

    const NAME: &'static str = "Redirect";
    const PLURAL: &'static str = "redirects";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "redirects/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "redirects"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "redirects/count",
        ),
        ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "redirects"),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "redirects/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "redirects/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single redirect.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RedirectFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing redirects.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RedirectListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return only redirects after the specified ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Filter redirects by path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Filter redirects by target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting redirects.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RedirectCountParams {
    /// Filter redirects by path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Filter redirects by target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_redirect_struct_serialization() {
        let redirect = Redirect {
            id: Some(12345),
            path: Some("/old-page".to_string()),
            target: Some("/new-page".to_string()),
        };

        let json = serde_json::to_string(&redirect).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["path"], "/old-page");
        assert_eq!(parsed["target"], "/new-page");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
    }

    #[test]
    fn test_redirect_deserialization() {
        let json = r#"{
            "id": 668809255,
            "path": "/products/ipod",
            "target": "/products/iphone"
        }"#;

        let redirect: Redirect = serde_json::from_str(json).unwrap();

        assert_eq!(redirect.id, Some(668809255));
        assert_eq!(redirect.path, Some("/products/ipod".to_string()));
        assert_eq!(redirect.target, Some("/products/iphone".to_string()));
    }

    #[test]
    fn test_redirect_full_crud_paths() {
        // Test Find path
        let find_path = get_path(Redirect::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "redirects/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(Redirect::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "redirects");
        assert_eq!(all_path.unwrap().http_method, HttpMethod::Get);

        // Test Count path
        let count_path = get_path(Redirect::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "redirects/count");

        // Test Create path
        let create_path = get_path(Redirect::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(Redirect::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test Delete path
        let delete_path = get_path(Redirect::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);
    }

    #[test]
    fn test_redirect_list_params_serialization() {
        let params = RedirectListParams {
            limit: Some(50),
            since_id: Some(100),
            path: Some("/old".to_string()),
            target: Some("/new".to_string()),
            fields: Some("id,path,target".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);
        assert_eq!(json["path"], "/old");
        assert_eq!(json["target"], "/new");
        assert_eq!(json["fields"], "id,path,target");

        // Test empty params
        let empty_params = RedirectListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_redirect_get_id_returns_correct_value() {
        let redirect_with_id = Redirect {
            id: Some(668809255),
            path: Some("/old".to_string()),
            target: Some("/new".to_string()),
        };
        assert_eq!(redirect_with_id.get_id(), Some(668809255));

        let redirect_without_id = Redirect {
            id: None,
            path: Some("/old".to_string()),
            target: Some("/new".to_string()),
        };
        assert_eq!(redirect_without_id.get_id(), None);
    }

    #[test]
    fn test_redirect_constants() {
        assert_eq!(Redirect::NAME, "Redirect");
        assert_eq!(Redirect::PLURAL, "redirects");
    }
}

//! AccessScope resource implementation.
//!
//! This module provides the [`AccessScope`] resource for retrieving the access
//! scopes associated with the current access token.
//!
//! # Read-Only Resource
//!
//! AccessScopes implement [`ReadOnlyResource`](crate::rest::ReadOnlyResource) - they
//! can only be listed, not created, updated, or deleted through the API.
//! Access scopes are determined during OAuth authorization.
//!
//! # Special OAuth Endpoint
//!
//! This resource uses the `/admin/oauth/access_scopes.json` endpoint, which is
//! different from the standard `/admin/api/{version}/` prefix used by other resources.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::v2025_10::AccessScope;
//!
//! // List all access scopes for the current token
//! let scopes = AccessScope::all(&client).await?;
//! for scope in scopes.iter() {
//!     println!("Scope: {}", scope.handle.as_deref().unwrap_or(""));
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ReadOnlyResource, ResourceError, ResourceResponse, RestResource, ResourceOperation, ResourcePath};
use crate::HttpMethod;

/// An OAuth access scope associated with an access token.
///
/// Access scopes define what permissions the current access token has.
/// They are read-only and determined during the OAuth authorization process.
///
/// # Read-Only Resource
///
/// This resource implements [`ReadOnlyResource`] - only GET operations are
/// available. Access scopes cannot be modified through the API.
///
/// # Special Endpoint
///
/// This resource uses `oauth/access_scopes` instead of the standard
/// API-versioned endpoint.
///
/// # Fields
///
/// - `handle` - The scope identifier (e.g., "read_products", "write_orders")
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AccessScope {
    /// The scope identifier (e.g., "read_products", "write_orders").
    #[serde(skip_serializing)]
    pub handle: Option<String>,
}

impl AccessScope {
    /// Lists all access scopes for the current access token.
    ///
    /// This uses the special `/admin/oauth/access_scopes.json` endpoint.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let scopes = AccessScope::all(&client).await?;
    /// for scope in &scopes {
    ///     println!("Has scope: {}", scope.handle.as_deref().unwrap_or(""));
    /// }
    /// ```
    pub async fn all(client: &RestClient) -> Result<ResourceResponse<Vec<Self>>, ResourceError> {
        // AccessScopes uses a special OAuth endpoint, not the standard API-versioned path
        let url = "oauth/access_scopes";
        let response = client.get(url, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        let key = Self::PLURAL;
        ResourceResponse::from_http_response(response, key)
    }
}

impl RestResource for AccessScope {
    type Id = String;
    type FindParams = ();
    type AllParams = ();
    type CountParams = ();

    const NAME: &'static str = "AccessScope";
    const PLURAL: &'static str = "access_scopes";

    /// Paths for the AccessScope resource.
    ///
    /// Note: The actual endpoint is `oauth/access_scopes`, not the standard
    /// API-versioned path. Use the `all()` method instead of the trait method.
    const PATHS: &'static [ResourcePath] = &[
        // Special path - uses oauth prefix instead of api version
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "oauth/access_scopes",
        ),
        // Note: No Find, Count, Create, Update, or Delete - list only
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.handle.clone()
    }
}

impl ReadOnlyResource for AccessScope {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ReadOnlyResource, ResourceOperation, RestResource};

    #[test]
    fn test_access_scope_implements_read_only_resource() {
        fn assert_read_only<T: ReadOnlyResource>() {}
        assert_read_only::<AccessScope>();
    }

    #[test]
    fn test_access_scope_deserialization() {
        let json = r#"{
            "handle": "read_products"
        }"#;

        let scope: AccessScope = serde_json::from_str(json).unwrap();

        assert_eq!(scope.handle, Some("read_products".to_string()));
    }

    #[test]
    fn test_access_scope_list_deserialization() {
        let json = r#"[
            {"handle": "read_products"},
            {"handle": "write_products"},
            {"handle": "read_orders"}
        ]"#;

        let scopes: Vec<AccessScope> = serde_json::from_str(json).unwrap();

        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].handle, Some("read_products".to_string()));
        assert_eq!(scopes[1].handle, Some("write_products".to_string()));
        assert_eq!(scopes[2].handle, Some("read_orders".to_string()));
    }

    #[test]
    fn test_access_scope_special_oauth_path() {
        // All path uses oauth prefix
        let all_path = get_path(AccessScope::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "oauth/access_scopes");
    }

    #[test]
    fn test_access_scope_list_only_no_other_operations() {
        // No Find path
        let find_path = get_path(AccessScope::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_none());

        // No Count path
        let count_path = get_path(AccessScope::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());

        // No Create path
        let create_path = get_path(AccessScope::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_none());

        // No Update path
        let update_path = get_path(AccessScope::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());

        // No Delete path
        let delete_path = get_path(AccessScope::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_access_scope_constants() {
        assert_eq!(AccessScope::NAME, "AccessScope");
        assert_eq!(AccessScope::PLURAL, "access_scopes");
    }

    #[test]
    fn test_access_scope_get_id_returns_handle() {
        let scope_with_handle = AccessScope {
            handle: Some("read_products".to_string()),
        };
        assert_eq!(scope_with_handle.get_id(), Some("read_products".to_string()));

        let scope_without_handle = AccessScope::default();
        assert_eq!(scope_without_handle.get_id(), None);
    }

    #[test]
    fn test_access_scope_all_fields_are_read_only() {
        // All fields should be skipped during serialization
        let scope = AccessScope {
            handle: Some("write_orders".to_string()),
        };

        let json = serde_json::to_value(&scope).unwrap();
        // All fields should be omitted (empty object)
        assert_eq!(json, serde_json::json!({}));
    }
}

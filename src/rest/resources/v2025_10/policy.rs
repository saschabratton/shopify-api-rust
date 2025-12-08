//! Policy resource implementation.
//!
//! This module provides the [`Policy`] resource for retrieving store policies
//! like refund policy, privacy policy, terms of service, and shipping policy.
//!
//! # Read-Only Resource
//!
//! Policies implement [`ReadOnlyResource`](crate::rest::ReadOnlyResource) - they
//! can only be retrieved, not created, updated, or deleted through the API.
//! Policies are managed through the Shopify admin.
//!
//! # Special Characteristics
//!
//! - **No ID field**: Policies are identified by their `handle`, not a numeric ID
//! - **List only**: No Find by ID or Count endpoints
//! - **Read-only**: Policies cannot be modified through the API
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::Policy;
//!
//! // List all store policies
//! let policies = Policy::all(&client, None).await?;
//! for policy in policies.iter() {
//!     println!("{}: {}", policy.title.as_deref().unwrap_or(""),
//!         policy.handle.as_deref().unwrap_or(""));
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ReadOnlyResource, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A store policy (refund, privacy, terms of service, shipping).
///
/// Policies are read-only records that merchants configure through the
/// Shopify admin. They cannot be created, updated, or deleted through
/// the API.
///
/// # Read-Only Resource
///
/// This resource implements [`ReadOnlyResource`] - only GET operations are
/// available. Policies are managed through the Shopify admin.
///
/// # No ID Field
///
/// Unlike most resources, policies don't have a numeric ID. They are
/// identified by their `handle` (e.g., "refund-policy", "privacy-policy").
///
/// # Fields
///
/// All fields are read-only:
/// - `title` - The title of the policy (e.g., "Refund Policy")
/// - `body` - The HTML content of the policy
/// - `handle` - The URL-friendly identifier
/// - `url` - The public URL of the policy
/// - `created_at` - When the policy was created
/// - `updated_at` - When the policy was last updated
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Policy {
    /// The title of the policy.
    #[serde(skip_serializing)]
    pub title: Option<String>,

    /// The HTML content of the policy.
    #[serde(skip_serializing)]
    pub body: Option<String>,

    /// The URL-friendly identifier of the policy.
    /// Used as the identifier since policies don't have numeric IDs.
    #[serde(skip_serializing)]
    pub handle: Option<String>,

    /// The public URL where the policy can be viewed.
    #[serde(skip_serializing)]
    pub url: Option<String>,

    /// When the policy was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the policy was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
}

impl RestResource for Policy {
    /// Policies don't have a numeric ID - they use handles.
    /// We use String as the ID type but `get_id()` always returns None.
    type Id = String;
    type FindParams = ();
    type AllParams = ();
    type CountParams = ();

    const NAME: &'static str = "Policy";
    const PLURAL: &'static str = "policies";

    /// Paths for the Policy resource.
    ///
    /// Only list operation is available - no Find by ID or Count.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "policies"),
        // Note: No Find path - policies don't have numeric IDs
        // Note: No Count path - not supported by the API
        // Note: No Create, Update, or Delete paths - read-only resource
    ];

    /// Policies don't have a standard ID - they use handles.
    /// Returns None for compatibility with the RestResource trait.
    fn get_id(&self) -> Option<Self::Id> {
        // Return the handle as a fallback identifier
        self.handle.clone()
    }
}

impl ReadOnlyResource for Policy {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ReadOnlyResource, ResourceOperation, RestResource};

    #[test]
    fn test_policy_implements_read_only_resource() {
        // This test verifies that Policy implements ReadOnlyResource
        fn assert_read_only<T: ReadOnlyResource>() {}
        assert_read_only::<Policy>();
    }

    #[test]
    fn test_policy_deserialization() {
        let json = r#"{
            "title": "Refund Policy",
            "body": "<p>We offer a 30-day return policy...</p>",
            "handle": "refund-policy",
            "url": "https://example.myshopify.com/policies/refund-policy",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z"
        }"#;

        let policy: Policy = serde_json::from_str(json).unwrap();

        assert_eq!(policy.title, Some("Refund Policy".to_string()));
        assert_eq!(
            policy.body,
            Some("<p>We offer a 30-day return policy...</p>".to_string())
        );
        assert_eq!(policy.handle, Some("refund-policy".to_string()));
        assert_eq!(
            policy.url,
            Some("https://example.myshopify.com/policies/refund-policy".to_string())
        );
        assert!(policy.created_at.is_some());
        assert!(policy.updated_at.is_some());
    }

    #[test]
    fn test_policy_read_only_paths() {
        // Only list path available
        let all_path = get_path(Policy::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "policies");

        // No find path (policies identified by handle, not ID)
        let find_path = get_path(Policy::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_none());

        // No count path
        let count_path = get_path(Policy::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());

        // No create, update, or delete paths
        let create_path = get_path(Policy::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_none());

        let update_path = get_path(Policy::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());

        let delete_path = get_path(Policy::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_policy_has_no_standard_id() {
        // Policy uses handle instead of numeric ID
        let policy = Policy {
            title: Some("Privacy Policy".to_string()),
            handle: Some("privacy-policy".to_string()),
            ..Default::default()
        };

        // get_id returns the handle as a fallback
        assert_eq!(policy.get_id(), Some("privacy-policy".to_string()));

        // Policy without handle returns None
        let policy_without_handle = Policy {
            title: Some("Some Policy".to_string()),
            handle: None,
            ..Default::default()
        };
        assert_eq!(policy_without_handle.get_id(), None);
    }

    #[test]
    fn test_policy_constants() {
        assert_eq!(Policy::NAME, "Policy");
        assert_eq!(Policy::PLURAL, "policies");
    }

    #[test]
    fn test_policy_all_fields_are_read_only() {
        // All fields should be skipped during serialization
        let policy = Policy {
            title: Some("Test Policy".to_string()),
            body: Some("<p>Content</p>".to_string()),
            handle: Some("test-policy".to_string()),
            url: Some("https://example.com/policies/test".to_string()),
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

        let json = serde_json::to_value(&policy).unwrap();
        // All fields should be omitted (empty object)
        assert_eq!(json, serde_json::json!({}));
    }
}

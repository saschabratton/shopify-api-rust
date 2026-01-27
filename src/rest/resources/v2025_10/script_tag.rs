//! ScriptTag resource implementation.
//!
//! This module provides the [`ScriptTag`] resource for managing script tags
//! that inject JavaScript into storefronts.
//!
//! # Deprecation Notice
//!
//! **⚠️ DEPRECATED**: Script tags are deprecated in favor of App Blocks and
//! theme app extensions. Consider migrating to the newer approaches for
//! injecting functionality into storefronts. Script tags may be removed in
//! a future API version.
//!
//! # Display Scopes
//!
//! Script tags can be displayed in different scopes:
//! - `online_store` - Only the online store
//! - `order_status` - Only the order status page
//! - `all` - Both online store and order status page
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{
//!     ScriptTag, ScriptTagListParams, ScriptTagDisplayScope, ScriptTagEvent
//! };
//!
//! // Create a script tag (deprecated - use App Blocks instead)
//! let script_tag = ScriptTag {
//!     event: Some(ScriptTagEvent::Onload),
//!     src: Some("https://myapp.com/script.js".to_string()),
//!     display_scope: Some(ScriptTagDisplayScope::All),
//!     ..Default::default()
//! };
//! let saved = script_tag.save(&client).await?;
//!
//! // List all script tags
//! let tags = ScriptTag::all(&client, None).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// The event that triggers the script tag to load.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScriptTagEvent {
    /// The script loads when the DOM is ready.
    Onload,
}

/// Where the script tag is displayed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScriptTagDisplayScope {
    /// The script is displayed only on the online store.
    OnlineStore,
    /// The script is displayed only on the order status page.
    OrderStatus,
    /// The script is displayed on both the online store and order status page.
    All,
}

/// A script tag that injects JavaScript into storefronts.
///
/// **⚠️ DEPRECATED**: Use App Blocks and theme app extensions instead.
///
/// Script tags allow apps to inject JavaScript into a store's storefront
/// and order status page. However, this approach is deprecated in favor
/// of the more performant and flexible App Blocks.
///
/// # Migration Guide
///
/// Instead of script tags, consider:
/// 1. **App Blocks** - For theme-integrated functionality
/// 2. **Theme App Extensions** - For deeper theme integration
/// 3. **Checkout Extensions** - For checkout-specific functionality
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `created_at` - When the script tag was created
/// - `updated_at` - When the script tag was last updated
///
/// ## Writable Fields
/// - `event` - The event that triggers loading (currently only "onload")
/// - `src` - The URL of the JavaScript file
/// - `display_scope` - Where the script is displayed
/// - `cache` - Whether to cache the script
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ScriptTag {
    /// The unique identifier of the script tag.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The event that triggers the script to load.
    /// Currently only "onload" is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<ScriptTagEvent>,

    /// The URL of the JavaScript file to load.
    /// Must be an HTTPS URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// Where the script tag is displayed.
    /// Default is "all" if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_scope: Option<ScriptTagDisplayScope>,

    /// Whether to enable caching of the script.
    /// When true, the script is cached for a short period.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<bool>,

    /// When the script tag was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the script tag was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
}

impl RestResource for ScriptTag {
    type Id = u64;
    type FindParams = ScriptTagFindParams;
    type AllParams = ScriptTagListParams;
    type CountParams = ScriptTagCountParams;

    const NAME: &'static str = "ScriptTag";
    const PLURAL: &'static str = "script_tags";

    /// Paths for the ScriptTag resource.
    ///
    /// Full CRUD operations are supported.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "script_tags/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "script_tags",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "script_tags/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "script_tags",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "script_tags/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "script_tags/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single script tag.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ScriptTagFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing script tags.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ScriptTagListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return script tags after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Show script tags created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show script tags created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show script tags updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show script tags updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Filter script tags by source URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting script tags.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ScriptTagCountParams {
    /// Filter script tags by source URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_script_tag_serialization() {
        let script_tag = ScriptTag {
            id: Some(596726825),
            event: Some(ScriptTagEvent::Onload),
            src: Some("https://myapp.com/script.js".to_string()),
            display_scope: Some(ScriptTagDisplayScope::All),
            cache: Some(true),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-15T10:35:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_string(&script_tag).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["event"], "onload");
        assert_eq!(parsed["src"], "https://myapp.com/script.js");
        assert_eq!(parsed["display_scope"], "all");
        assert_eq!(parsed["cache"], true);

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_script_tag_deserialization() {
        let json = r#"{
            "id": 596726825,
            "event": "onload",
            "src": "https://myapp.com/script.js",
            "display_scope": "online_store",
            "cache": false,
            "created_at": "2024-06-15T10:30:00Z",
            "updated_at": "2024-06-15T10:35:00Z"
        }"#;

        let script_tag: ScriptTag = serde_json::from_str(json).unwrap();

        assert_eq!(script_tag.id, Some(596726825));
        assert_eq!(script_tag.event, Some(ScriptTagEvent::Onload));
        assert_eq!(script_tag.src, Some("https://myapp.com/script.js".to_string()));
        assert_eq!(script_tag.display_scope, Some(ScriptTagDisplayScope::OnlineStore));
        assert_eq!(script_tag.cache, Some(false));
        assert!(script_tag.created_at.is_some());
        assert!(script_tag.updated_at.is_some());
    }

    #[test]
    fn test_script_tag_display_scope_serialization() {
        // Test that display_scope enum serializes to snake_case
        let online_store = ScriptTagDisplayScope::OnlineStore;
        let json = serde_json::to_value(&online_store).unwrap();
        assert_eq!(json, "online_store");

        let order_status = ScriptTagDisplayScope::OrderStatus;
        let json = serde_json::to_value(&order_status).unwrap();
        assert_eq!(json, "order_status");

        let all = ScriptTagDisplayScope::All;
        let json = serde_json::to_value(&all).unwrap();
        assert_eq!(json, "all");

        // Deserialize back
        let parsed: ScriptTagDisplayScope = serde_json::from_str("\"online_store\"").unwrap();
        assert_eq!(parsed, ScriptTagDisplayScope::OnlineStore);
    }

    #[test]
    fn test_script_tag_full_crud_paths() {
        // Find by ID
        let find_path = get_path(ScriptTag::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "script_tags/{id}");

        // List all
        let all_path = get_path(ScriptTag::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "script_tags");

        // Count
        let count_path = get_path(ScriptTag::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "script_tags/count");

        // Create
        let create_path = get_path(ScriptTag::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "script_tags");

        // Update
        let update_path = get_path(ScriptTag::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "script_tags/{id}");

        // Delete
        let delete_path = get_path(ScriptTag::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "script_tags/{id}");
    }

    #[test]
    fn test_script_tag_constants() {
        assert_eq!(ScriptTag::NAME, "ScriptTag");
        assert_eq!(ScriptTag::PLURAL, "script_tags");
    }

    #[test]
    fn test_script_tag_get_id() {
        let tag_with_id = ScriptTag {
            id: Some(596726825),
            src: Some("https://example.com/script.js".to_string()),
            ..Default::default()
        };
        assert_eq!(tag_with_id.get_id(), Some(596726825));

        let tag_without_id = ScriptTag::default();
        assert_eq!(tag_without_id.get_id(), None);
    }
}

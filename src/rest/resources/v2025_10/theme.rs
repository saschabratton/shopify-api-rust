//! Theme resource implementation.
//!
//! This module provides the Theme resource, which represents a theme in a Shopify store.
//! Themes define the look and feel of an online store.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{Theme, ThemeListParams, ThemeRole};
//!
//! // List all themes
//! let themes = Theme::all(&client, None).await?;
//! for theme in themes.iter() {
//!     println!("Theme: {} ({})", theme.name.as_deref().unwrap_or(""),
//!         theme.role.map(|r| format!("{:?}", r)).unwrap_or_default());
//! }
//!
//! // Find a specific theme
//! let theme = Theme::find(&client, 123, None).await?;
//! println!("Theme: {}", theme.name.as_deref().unwrap_or(""));
//!
//! // Create a new theme
//! let mut theme = Theme {
//!     name: Some("My Custom Theme".to_string()),
//!     role: Some(ThemeRole::Unpublished),
//!     ..Default::default()
//! };
//! let saved = theme.save(&client).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

// Re-export ThemeRole from common
pub use super::common::ThemeRole;

/// A theme in a Shopify store.
///
/// Themes define the look and feel of an online store, including the
/// layout, colors, and typography. A store can have multiple themes,
/// but only one can be published (main) at a time.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the theme
/// - `created_at` - When the theme was created
/// - `updated_at` - When the theme was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `name` - The name of the theme
/// - `role` - The role of the theme (Main, Unpublished, Demo, Development)
///
/// ## Status Fields (Read-Only)
/// - `previewable` - Whether the theme can be previewed
/// - `processing` - Whether the theme is being processed
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Theme {
    /// The unique identifier of the theme.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The name of the theme.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The role of the theme in the store.
    /// Determines whether the theme is published, in development, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ThemeRole>,

    /// Whether the theme can be previewed.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub previewable: Option<bool>,

    /// Whether the theme is currently being processed.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub processing: Option<bool>,

    /// When the theme was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the theme was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this theme.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for Theme {
    type Id = u64;
    type FindParams = ThemeFindParams;
    type AllParams = ThemeListParams;
    type CountParams = ();

    const NAME: &'static str = "Theme";
    const PLURAL: &'static str = "themes";

    /// Paths for the Theme resource.
    ///
    /// The Theme resource supports standard CRUD operations.
    /// Note: There is no count endpoint for themes.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "themes/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "themes"),
        ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "themes"),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "themes/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "themes/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single theme.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ThemeFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing themes.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ThemeListParams {
    /// Filter by theme role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ThemeRole>,

    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return themes after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_theme_struct_serialization() {
        let theme = Theme {
            id: Some(12345),
            name: Some("My Custom Theme".to_string()),
            role: Some(ThemeRole::Unpublished),
            previewable: Some(true),
            processing: Some(false),
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
            admin_graphql_api_id: Some("gid://shopify/Theme/12345".to_string()),
        };

        let json = serde_json::to_string(&theme).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["name"], "My Custom Theme");
        assert_eq!(parsed["role"], "unpublished");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("previewable").is_none());
        assert!(parsed.get("processing").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_theme_deserialization_from_api_response() {
        let json = r#"{
            "id": 828155753,
            "name": "Dawn",
            "role": "main",
            "previewable": true,
            "processing": false,
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/Theme/828155753"
        }"#;

        let theme: Theme = serde_json::from_str(json).unwrap();

        assert_eq!(theme.id, Some(828155753));
        assert_eq!(theme.name, Some("Dawn".to_string()));
        assert_eq!(theme.role, Some(ThemeRole::Main));
        assert_eq!(theme.previewable, Some(true));
        assert_eq!(theme.processing, Some(false));
        assert!(theme.created_at.is_some());
        assert!(theme.updated_at.is_some());
        assert_eq!(
            theme.admin_graphql_api_id,
            Some("gid://shopify/Theme/828155753".to_string())
        );
    }

    #[test]
    fn test_theme_role_enum_variants() {
        // Test all role variants deserialize correctly
        let main: ThemeRole = serde_json::from_str("\"main\"").unwrap();
        assert_eq!(main, ThemeRole::Main);

        let unpublished: ThemeRole = serde_json::from_str("\"unpublished\"").unwrap();
        assert_eq!(unpublished, ThemeRole::Unpublished);

        let demo: ThemeRole = serde_json::from_str("\"demo\"").unwrap();
        assert_eq!(demo, ThemeRole::Demo);

        let development: ThemeRole = serde_json::from_str("\"development\"").unwrap();
        assert_eq!(development, ThemeRole::Development);
    }

    #[test]
    fn test_theme_paths() {
        // Find path
        let find_path = get_path(Theme::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "themes/{id}");

        // All path
        let all_path = get_path(Theme::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "themes");

        // Create path
        let create_path = get_path(Theme::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "themes");

        // Update path
        let update_path = get_path(Theme::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "themes/{id}");

        // Delete path
        let delete_path = get_path(Theme::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "themes/{id}");

        // No count path for themes
        let count_path = get_path(Theme::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());
    }

    #[test]
    fn test_theme_list_params_serialization() {
        let params = ThemeListParams {
            role: Some(ThemeRole::Main),
            limit: Some(50),
            since_id: Some(12345),
            page_info: None,
            fields: Some("id,name,role".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["role"], "main");
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 12345);
        assert_eq!(json["fields"], "id,name,role");
        assert!(json.get("page_info").is_none());

        // Test empty params
        let empty_params = ThemeListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_theme_get_id_returns_correct_value() {
        // Theme with ID
        let theme_with_id = Theme {
            id: Some(123456789),
            name: Some("Test Theme".to_string()),
            ..Default::default()
        };
        assert_eq!(theme_with_id.get_id(), Some(123456789));

        // Theme without ID (new theme)
        let theme_without_id = Theme {
            id: None,
            name: Some("New Theme".to_string()),
            ..Default::default()
        };
        assert_eq!(theme_without_id.get_id(), None);
    }

    #[test]
    fn test_theme_constants() {
        assert_eq!(Theme::NAME, "Theme");
        assert_eq!(Theme::PLURAL, "themes");
    }

    #[test]
    fn test_theme_status_fields() {
        let json = r#"{
            "id": 123,
            "name": "Processing Theme",
            "role": "unpublished",
            "previewable": false,
            "processing": true
        }"#;

        let theme: Theme = serde_json::from_str(json).unwrap();

        assert_eq!(theme.previewable, Some(false));
        assert_eq!(theme.processing, Some(true));

        // When serialized, status fields should be omitted
        let serialized = serde_json::to_value(&theme).unwrap();
        assert!(serialized.get("previewable").is_none());
        assert!(serialized.get("processing").is_none());
    }
}

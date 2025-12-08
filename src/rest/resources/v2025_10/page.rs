//! Page resource implementation.
//!
//! This module provides the Page resource, which represents a static page
//! in a Shopify store. Pages are used for static content like "About Us",
//! "Contact", or "Privacy Policy" pages.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{Page, PageListParams};
//!
//! // Find a single page
//! let page = Page::find(&client, 123, None).await?;
//! println!("Page: {}", page.title.as_deref().unwrap_or(""));
//!
//! // List pages
//! let params = PageListParams {
//!     published_status: Some("published".to_string()),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let pages = Page::all(&client, Some(params)).await?;
//!
//! // Create a new page
//! let mut page = Page {
//!     title: Some("About Us".to_string()),
//!     body_html: Some("<p>Welcome to our store!</p>".to_string()),
//!     ..Default::default()
//! };
//! let saved = page.save(&client).await?;
//!
//! // Count pages
//! let count = Page::count(&client, None).await?;
//! println!("Total pages: {}", count);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A static page in a Shopify store.
///
/// Pages are used for static content that doesn't change frequently,
/// such as "About Us", "Contact", "Privacy Policy", or "Terms of Service" pages.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the page
/// - `shop_id` - The ID of the shop the page belongs to
/// - `created_at` - When the page was created
/// - `updated_at` - When the page was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `title` - The title of the page
/// - `handle` - The URL-friendly handle (auto-generated from title if not set)
/// - `body_html` - The HTML content of the page
/// - `author` - The author of the page
/// - `template_suffix` - The suffix of the Liquid template used for the page
/// - `published_at` - When the page was published (can be set to future for scheduling)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Page {
    /// The unique identifier of the page.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the shop the page belongs to.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub shop_id: Option<u64>,

    /// The title of the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The URL-friendly handle of the page.
    /// Auto-generated from the title if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// The HTML content of the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,

    /// The author of the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// The suffix of the Liquid template used for the page.
    /// For example, if the value is "contact", the page uses the
    /// `page.contact.liquid` template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_suffix: Option<String>,

    /// When the page was or will be published.
    /// Set to a future date to schedule publication.
    /// Set to `null` to unpublish.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,

    /// When the page was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the page was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this page.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for Page {
    type Id = u64;
    type FindParams = PageFindParams;
    type AllParams = PageListParams;
    type CountParams = PageCountParams;

    const NAME: &'static str = "Page";
    const PLURAL: &'static str = "pages";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "pages/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "pages"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "pages/count",
        ),
        ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "pages"),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "pages/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "pages/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single page.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PageFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing pages.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PageListParams {
    /// Filter by page title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Filter by page handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// Filter by published status.
    /// Valid values: `published`, `unpublished`, `any`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_status: Option<String>,

    /// Show pages created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show pages created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show pages updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show pages updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show pages published after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_min: Option<DateTime<Utc>>,

    /// Show pages published before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_max: Option<DateTime<Utc>>,

    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return pages after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting pages.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PageCountParams {
    /// Filter by page title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Filter by published status.
    /// Valid values: `published`, `unpublished`, `any`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_status: Option<String>,

    /// Show pages created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show pages created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show pages updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show pages updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show pages published after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_min: Option<DateTime<Utc>>,

    /// Show pages published before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_max: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_page_struct_serialization() {
        let page = Page {
            id: Some(12345),
            shop_id: Some(67890),
            title: Some("About Us".to_string()),
            handle: Some("about-us".to_string()),
            body_html: Some("<p>Welcome to our store!</p>".to_string()),
            author: Some("Store Admin".to_string()),
            template_suffix: Some("contact".to_string()),
            published_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-10T08:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-20T15:45:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            admin_graphql_api_id: Some("gid://shopify/OnlineStorePage/12345".to_string()),
        };

        let json = serde_json::to_string(&page).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["title"], "About Us");
        assert_eq!(parsed["handle"], "about-us");
        assert_eq!(parsed["body_html"], "<p>Welcome to our store!</p>");
        assert_eq!(parsed["author"], "Store Admin");
        assert_eq!(parsed["template_suffix"], "contact");
        assert!(parsed["published_at"].as_str().is_some());

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("shop_id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_page_deserialization_from_api_response() {
        let json = r#"{
            "id": 131092082,
            "shop_id": 548380009,
            "title": "About Us",
            "handle": "about-us",
            "body_html": "<p>Welcome to our store!</p>",
            "author": "Store Admin",
            "template_suffix": null,
            "published_at": "2024-01-15T10:30:00Z",
            "created_at": "2024-01-10T08:00:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/OnlineStorePage/131092082"
        }"#;

        let page: Page = serde_json::from_str(json).unwrap();

        assert_eq!(page.id, Some(131092082));
        assert_eq!(page.shop_id, Some(548380009));
        assert_eq!(page.title, Some("About Us".to_string()));
        assert_eq!(page.handle, Some("about-us".to_string()));
        assert_eq!(
            page.body_html,
            Some("<p>Welcome to our store!</p>".to_string())
        );
        assert_eq!(page.author, Some("Store Admin".to_string()));
        assert!(page.template_suffix.is_none());
        assert!(page.published_at.is_some());
        assert!(page.created_at.is_some());
        assert!(page.updated_at.is_some());
        assert_eq!(
            page.admin_graphql_api_id,
            Some("gid://shopify/OnlineStorePage/131092082".to_string())
        );
    }

    #[test]
    fn test_page_list_params_serialization() {
        let params = PageListParams {
            title: Some("Contact".to_string()),
            handle: Some("contact".to_string()),
            published_status: Some("published".to_string()),
            limit: Some(50),
            since_id: Some(100),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["title"], "Contact");
        assert_eq!(json["handle"], "contact");
        assert_eq!(json["published_status"], "published");
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);

        // Fields not set should be omitted
        assert!(json.get("created_at_min").is_none());
        assert!(json.get("page_info").is_none());
    }

    #[test]
    fn test_page_count_params_serialization() {
        let params = PageCountParams {
            title: Some("About".to_string()),
            published_status: Some("any".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["title"], "About");
        assert_eq!(json["published_status"], "any");

        // Test empty params
        let empty_params = PageCountParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_page_path_constants_are_correct() {
        // Test Find path
        let find_path = get_path(Page::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "pages/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(Page::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "pages");
        assert_eq!(all_path.unwrap().http_method, HttpMethod::Get);

        // Test Count path
        let count_path = get_path(Page::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "pages/count");
        assert_eq!(count_path.unwrap().http_method, HttpMethod::Get);

        // Test Create path
        let create_path = get_path(Page::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "pages");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(Page::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "pages/{id}");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test Delete path
        let delete_path = get_path(Page::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "pages/{id}");
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        // Verify constants
        assert_eq!(Page::NAME, "Page");
        assert_eq!(Page::PLURAL, "pages");
    }

    #[test]
    fn test_page_get_id_returns_correct_value() {
        // Page with ID
        let page_with_id = Page {
            id: Some(123456789),
            title: Some("Test Page".to_string()),
            ..Default::default()
        };
        assert_eq!(page_with_id.get_id(), Some(123456789));

        // Page without ID (new page)
        let page_without_id = Page {
            id: None,
            title: Some("New Page".to_string()),
            ..Default::default()
        };
        assert_eq!(page_without_id.get_id(), None);
    }

    #[test]
    fn test_page_published_at_for_scheduling() {
        // A page can be scheduled for future publication
        let future_date = DateTime::parse_from_rfc3339("2025-12-31T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);

        let page = Page {
            title: Some("Upcoming Sale".to_string()),
            body_html: Some("<p>Coming soon!</p>".to_string()),
            published_at: Some(future_date),
            ..Default::default()
        };

        let json = serde_json::to_value(&page).unwrap();
        assert!(json["published_at"].as_str().is_some());
        assert!(json["published_at"]
            .as_str()
            .unwrap()
            .contains("2025-12-31"));
    }
}

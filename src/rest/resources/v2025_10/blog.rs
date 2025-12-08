//! Blog resource implementation.
//!
//! This module provides the Blog resource, which represents a blog
//! in a Shopify store. Blogs are containers for articles.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{Blog, BlogListParams};
//! use shopify_api::rest::resources::v2025_10::common::BlogCommentable;
//!
//! // Find a single blog
//! let blog = Blog::find(&client, 123, None).await?;
//! println!("Blog: {}", blog.title.as_deref().unwrap_or(""));
//!
//! // List blogs
//! let params = BlogListParams {
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let blogs = Blog::all(&client, Some(params)).await?;
//!
//! // Create a new blog with comment moderation
//! let mut blog = Blog {
//!     title: Some("News".to_string()),
//!     commentable: Some(BlogCommentable::Moderate),
//!     ..Default::default()
//! };
//! let saved = blog.save(&client).await?;
//!
//! // Count blogs
//! let count = Blog::count(&client, None).await?;
//! println!("Total blogs: {}", count);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::BlogCommentable;

/// A blog in a Shopify store.
///
/// Blogs are containers for articles. A store can have multiple blogs,
/// each with its own set of articles. Blogs support comment settings
/// and can integrate with Feedburner.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the blog
/// - `created_at` - When the blog was created
/// - `updated_at` - When the blog was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `title` - The title of the blog
/// - `handle` - The URL-friendly handle (auto-generated from title if not set)
/// - `commentable` - Comment settings (no, moderate, yes)
/// - `template_suffix` - The suffix of the Liquid template used for the blog
/// - `feedburner` - The Feedburner URL (optional)
/// - `feedburner_location` - The Feedburner location (optional)
/// - `tags` - Comma-separated tags for the blog
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Blog {
    /// The unique identifier of the blog.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The title of the blog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The URL-friendly handle of the blog.
    /// Auto-generated from the title if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// The comment moderation setting for the blog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commentable: Option<BlogCommentable>,

    /// The suffix of the Liquid template used for the blog.
    /// For example, if the value is "custom", the blog uses the
    /// `blog.custom.liquid` template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_suffix: Option<String>,

    /// The Feedburner URL for the blog.
    /// Used to redirect RSS feeds through Feedburner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedburner: Option<String>,

    /// The Feedburner location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedburner_location: Option<String>,

    /// Comma-separated tags for the blog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// When the blog was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the blog was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this blog.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for Blog {
    type Id = u64;
    type FindParams = BlogFindParams;
    type AllParams = BlogListParams;
    type CountParams = BlogCountParams;

    const NAME: &'static str = "Blog";
    const PLURAL: &'static str = "blogs";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "blogs/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "blogs"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "blogs/count",
        ),
        ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "blogs"),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "blogs/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "blogs/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single blog.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BlogFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing blogs.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BlogListParams {
    /// Filter by blog handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// Show blogs created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show blogs created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show blogs updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show blogs updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return blogs after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting blogs.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BlogCountParams {
    // No specific count params for blogs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_blog_struct_serialization() {
        let blog = Blog {
            id: Some(12345),
            title: Some("Company News".to_string()),
            handle: Some("news".to_string()),
            commentable: Some(BlogCommentable::Moderate),
            template_suffix: Some("custom".to_string()),
            feedburner: Some("https://feeds.feedburner.com/example".to_string()),
            feedburner_location: Some("example".to_string()),
            tags: Some("news, updates".to_string()),
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
            admin_graphql_api_id: Some("gid://shopify/OnlineStoreBlog/12345".to_string()),
        };

        let json = serde_json::to_string(&blog).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["title"], "Company News");
        assert_eq!(parsed["handle"], "news");
        assert_eq!(parsed["commentable"], "moderate");
        assert_eq!(parsed["template_suffix"], "custom");
        assert_eq!(parsed["feedburner"], "https://feeds.feedburner.com/example");
        assert_eq!(parsed["feedburner_location"], "example");
        assert_eq!(parsed["tags"], "news, updates");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_blog_deserialization_from_api_response() {
        let json = r#"{
            "id": 241253187,
            "handle": "apple-blog",
            "title": "Apple Blog",
            "updated_at": "2024-06-20T15:45:00Z",
            "commentable": "no",
            "feedburner": null,
            "feedburner_location": null,
            "created_at": "2024-01-10T08:00:00Z",
            "template_suffix": null,
            "tags": "apple, tech",
            "admin_graphql_api_id": "gid://shopify/OnlineStoreBlog/241253187"
        }"#;

        let blog: Blog = serde_json::from_str(json).unwrap();

        assert_eq!(blog.id, Some(241253187));
        assert_eq!(blog.handle, Some("apple-blog".to_string()));
        assert_eq!(blog.title, Some("Apple Blog".to_string()));
        assert_eq!(blog.commentable, Some(BlogCommentable::No));
        assert!(blog.feedburner.is_none());
        assert!(blog.feedburner_location.is_none());
        assert!(blog.template_suffix.is_none());
        assert_eq!(blog.tags, Some("apple, tech".to_string()));
        assert!(blog.created_at.is_some());
        assert!(blog.updated_at.is_some());
        assert_eq!(
            blog.admin_graphql_api_id,
            Some("gid://shopify/OnlineStoreBlog/241253187".to_string())
        );
    }

    #[test]
    fn test_blog_commentable_enum_handling() {
        // Test all BlogCommentable variants in blog context
        let variants = [
            (BlogCommentable::No, "no"),
            (BlogCommentable::Moderate, "moderate"),
            (BlogCommentable::Yes, "yes"),
        ];

        for (commentable, expected_str) in variants {
            let blog = Blog {
                title: Some("Test Blog".to_string()),
                commentable: Some(commentable),
                ..Default::default()
            };

            let json = serde_json::to_value(&blog).unwrap();
            assert_eq!(json["commentable"], expected_str);
        }
    }

    #[test]
    fn test_blog_list_params_serialization() {
        let params = BlogListParams {
            handle: Some("news".to_string()),
            limit: Some(50),
            since_id: Some(100),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["handle"], "news");
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);

        // Fields not set should be omitted
        assert!(json.get("created_at_min").is_none());
        assert!(json.get("page_info").is_none());
    }

    #[test]
    fn test_blog_path_constants_are_correct() {
        // Test Find path
        let find_path = get_path(Blog::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "blogs/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(Blog::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "blogs");
        assert_eq!(all_path.unwrap().http_method, HttpMethod::Get);

        // Test Count path
        let count_path = get_path(Blog::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "blogs/count");
        assert_eq!(count_path.unwrap().http_method, HttpMethod::Get);

        // Test Create path
        let create_path = get_path(Blog::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "blogs");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(Blog::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "blogs/{id}");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test Delete path
        let delete_path = get_path(Blog::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "blogs/{id}");
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        // Verify constants
        assert_eq!(Blog::NAME, "Blog");
        assert_eq!(Blog::PLURAL, "blogs");
    }

    #[test]
    fn test_blog_get_id_returns_correct_value() {
        // Blog with ID
        let blog_with_id = Blog {
            id: Some(123456789),
            title: Some("Test Blog".to_string()),
            ..Default::default()
        };
        assert_eq!(blog_with_id.get_id(), Some(123456789));

        // Blog without ID (new blog)
        let blog_without_id = Blog {
            id: None,
            title: Some("New Blog".to_string()),
            ..Default::default()
        };
        assert_eq!(blog_without_id.get_id(), None);
    }

    #[test]
    fn test_blog_tags_field_handling() {
        // Tags as comma-separated string
        let blog = Blog {
            title: Some("Tech Blog".to_string()),
            tags: Some("tech, programming, rust, web".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&blog).unwrap();
        assert_eq!(json["tags"], "tech, programming, rust, web");

        // Deserialize tags back
        let deserialized: Blog = serde_json::from_value(json).unwrap();
        assert_eq!(
            deserialized.tags,
            Some("tech, programming, rust, web".to_string())
        );
    }
}

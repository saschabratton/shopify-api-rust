//! Article resource implementation.
//!
//! This module provides the Article resource, which represents a blog article
//! in a Shopify store. Articles are nested under blogs and follow the same
//! nested path pattern as Variants under Products.
//!
//! # Nested Path Pattern
//!
//! Articles are always accessed under a blog:
//! - List: `/blogs/{blog_id}/articles`
//! - Find: `/blogs/{blog_id}/articles/{id}`
//! - Create: `/blogs/{blog_id}/articles`
//! - Update: `/blogs/{blog_id}/articles/{id}`
//! - Delete: `/blogs/{blog_id}/articles/{id}`
//! - Count: `/blogs/{blog_id}/articles/count`
//!
//! Use `Article::all_with_parent()` to list articles under a specific blog.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{Article, ArticleListParams};
//!
//! // List articles under a specific blog
//! let articles = Article::all_with_parent(&client, "blog_id", 123, None).await?;
//! for article in articles.iter() {
//!     println!("Article: {}", article.title.as_deref().unwrap_or(""));
//! }
//!
//! // Create a new article under a blog
//! let mut article = Article {
//!     blog_id: Some(123),
//!     title: Some("New Post".to_string()),
//!     body_html: Some("<p>Article content</p>".to_string()),
//!     author: Some("Admin".to_string()),
//!     ..Default::default()
//! };
//! let saved = article.save(&client).await?;
//!
//! // Count articles in a blog
//! let count = Article::count_with_parent(&client, "blog_id", 123, None).await?;
//! println!("Total articles: {}", count);
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{
    build_path, get_path, ResourceError, ResourceOperation, ResourcePath, RestResource,
};
use crate::HttpMethod;

/// An image associated with an article.
///
/// Similar to `CollectionImage`, this represents the featured image
/// for a blog article.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ArticleImage {
    /// The source URL of the article image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,

    /// Alternative text for the image (for accessibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,

    /// The width of the image in pixels.
    #[serde(skip_serializing)]
    pub width: Option<i64>,

    /// The height of the image in pixels.
    #[serde(skip_serializing)]
    pub height: Option<i64>,

    /// When the image was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,
}

/// A blog article in a Shopify store.
///
/// Articles are blog posts that belong to a specific blog. They are
/// nested resources that require a `blog_id` for all operations.
///
/// # Nested Resource
///
/// Articles follow the same nested path pattern as Variants:
/// - All operations require `blog_id` context
/// - Use `all_with_parent()` to list articles under a blog
/// - The `blog_id` field is required for creating new articles
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the article
/// - `blog_id` - The ID of the blog this article belongs to (also required for creation)
/// - `user_id` - The ID of the user who authored the article
/// - `created_at` - When the article was created
/// - `updated_at` - When the article was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `title` - The title of the article
/// - `handle` - The URL-friendly handle (auto-generated from title if not set)
/// - `body_html` - The HTML content of the article
/// - `author` - The author name displayed on the article
/// - `summary_html` - The summary/excerpt of the article
/// - `template_suffix` - The suffix of the Liquid template used for the article
/// - `tags` - Comma-separated tags for the article
/// - `image` - The featured image for the article
/// - `published_at` - When the article was published (can be set to future for scheduling)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Article {
    /// The unique identifier of the article.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the blog this article belongs to.
    /// Required for creating new articles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blog_id: Option<u64>,

    /// The title of the article.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The URL-friendly handle of the article.
    /// Auto-generated from the title if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// The HTML content of the article.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,

    /// The author name displayed on the article.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// The summary/excerpt of the article in HTML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_html: Option<String>,

    /// The suffix of the Liquid template used for the article.
    /// For example, if the value is "custom", the article uses the
    /// `article.custom.liquid` template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_suffix: Option<String>,

    /// Comma-separated tags for the article.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// The featured image for the article.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ArticleImage>,

    /// When the article was or will be published.
    /// Set to a future date to schedule publication.
    /// Set to `null` to unpublish.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,

    /// The ID of the user who authored the article.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub user_id: Option<u64>,

    /// When the article was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the article was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this article.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl Article {
    /// Counts articles under a specific blog.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `parent_id_name` - The name of the parent ID parameter (should be `blog_id`)
    /// * `parent_id` - The blog ID
    /// * `params` - Optional parameters for filtering
    ///
    /// # Returns
    ///
    /// The count of matching articles as a `u64`.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::PathResolutionFailed`] if no count path exists.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let count = Article::count_with_parent(&client, "blog_id", 123, None).await?;
    /// println!("Articles in blog: {}", count);
    /// ```
    pub async fn count_with_parent<ParentId: std::fmt::Display + Send>(
        client: &RestClient,
        parent_id_name: &str,
        parent_id: ParentId,
        params: Option<ArticleCountParams>,
    ) -> Result<u64, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert(parent_id_name, parent_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Count, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "count",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Build query params
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
                            // Skip null, arrays, and objects
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

        // Extract count from response
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
}

impl RestResource for Article {
    type Id = u64;
    type FindParams = ArticleFindParams;
    type AllParams = ArticleListParams;
    type CountParams = ArticleCountParams;

    const NAME: &'static str = "Article";
    const PLURAL: &'static str = "articles";

    /// Paths for the Article resource.
    ///
    /// Articles are NESTED under blogs. All operations require `blog_id`.
    /// This follows the same pattern as Variants nested under Products.
    const PATHS: &'static [ResourcePath] = &[
        // All paths require blog_id
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["blog_id", "id"],
            "blogs/{blog_id}/articles/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["blog_id"],
            "blogs/{blog_id}/articles",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["blog_id"],
            "blogs/{blog_id}/articles/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &["blog_id"],
            "blogs/{blog_id}/articles",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["blog_id", "id"],
            "blogs/{blog_id}/articles/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["blog_id", "id"],
            "blogs/{blog_id}/articles/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single article.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ArticleFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing articles.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ArticleListParams {
    /// Filter by article author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Filter by article handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// Filter by tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Filter by published status.
    /// Valid values: `published`, `unpublished`, `any`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_status: Option<String>,

    /// Show articles created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show articles created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show articles updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show articles updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show articles published after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_min: Option<DateTime<Utc>>,

    /// Show articles published before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_max: Option<DateTime<Utc>>,

    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return articles after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting articles.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ArticleCountParams {
    /// Filter by article author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Filter by tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Filter by published status.
    /// Valid values: `published`, `unpublished`, `any`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_status: Option<String>,

    /// Show articles created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show articles created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show articles updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show articles updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show articles published after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_min: Option<DateTime<Utc>>,

    /// Show articles published before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_max: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_article_struct_serialization() {
        let article = Article {
            id: Some(12345),
            blog_id: Some(67890),
            title: Some("New Blog Post".to_string()),
            handle: Some("new-blog-post".to_string()),
            body_html: Some("<p>This is the article content.</p>".to_string()),
            author: Some("Jane Doe".to_string()),
            summary_html: Some("<p>Article summary.</p>".to_string()),
            template_suffix: Some("custom".to_string()),
            tags: Some("tech, rust, web".to_string()),
            image: Some(ArticleImage {
                src: Some("https://cdn.shopify.com/article.jpg".to_string()),
                alt: Some("Article image".to_string()),
                width: Some(1200),
                height: Some(800),
                created_at: None,
            }),
            published_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            user_id: Some(111222),
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
            admin_graphql_api_id: Some("gid://shopify/OnlineStoreArticle/12345".to_string()),
        };

        let json = serde_json::to_string(&article).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["blog_id"], 67890);
        assert_eq!(parsed["title"], "New Blog Post");
        assert_eq!(parsed["handle"], "new-blog-post");
        assert_eq!(parsed["body_html"], "<p>This is the article content.</p>");
        assert_eq!(parsed["author"], "Jane Doe");
        assert_eq!(parsed["summary_html"], "<p>Article summary.</p>");
        assert_eq!(parsed["template_suffix"], "custom");
        assert_eq!(parsed["tags"], "tech, rust, web");
        assert!(parsed["published_at"].as_str().is_some());
        assert_eq!(
            parsed["image"]["src"],
            "https://cdn.shopify.com/article.jpg"
        );
        assert_eq!(parsed["image"]["alt"], "Article image");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("user_id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());

        // Image read-only fields should be omitted
        assert!(parsed["image"].get("width").is_none());
        assert!(parsed["image"].get("height").is_none());
    }

    #[test]
    fn test_article_deserialization_from_api_response() {
        let json = r#"{
            "id": 134645308,
            "blog_id": 241253187,
            "title": "My new blog post",
            "handle": "my-new-blog-post",
            "body_html": "<p>This is the content of the article.</p>",
            "author": "John Smith",
            "summary_html": "<p>Summary here.</p>",
            "template_suffix": null,
            "tags": "tech, news",
            "image": {
                "src": "https://cdn.shopify.com/s/files/1/article.jpg",
                "alt": "Blog image",
                "width": 1200,
                "height": 800,
                "created_at": "2024-01-15T10:30:00Z"
            },
            "published_at": "2024-01-15T10:30:00Z",
            "user_id": 799407056,
            "created_at": "2024-01-10T08:00:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/OnlineStoreArticle/134645308"
        }"#;

        let article: Article = serde_json::from_str(json).unwrap();

        assert_eq!(article.id, Some(134645308));
        assert_eq!(article.blog_id, Some(241253187));
        assert_eq!(article.title, Some("My new blog post".to_string()));
        assert_eq!(article.handle, Some("my-new-blog-post".to_string()));
        assert_eq!(
            article.body_html,
            Some("<p>This is the content of the article.</p>".to_string())
        );
        assert_eq!(article.author, Some("John Smith".to_string()));
        assert_eq!(
            article.summary_html,
            Some("<p>Summary here.</p>".to_string())
        );
        assert!(article.template_suffix.is_none());
        assert_eq!(article.tags, Some("tech, news".to_string()));
        assert!(article.image.is_some());
        let image = article.image.unwrap();
        assert_eq!(
            image.src,
            Some("https://cdn.shopify.com/s/files/1/article.jpg".to_string())
        );
        assert_eq!(image.alt, Some("Blog image".to_string()));
        assert_eq!(image.width, Some(1200));
        assert_eq!(image.height, Some(800));
        assert!(image.created_at.is_some());
        assert!(article.published_at.is_some());
        assert_eq!(article.user_id, Some(799407056));
        assert!(article.created_at.is_some());
        assert!(article.updated_at.is_some());
        assert_eq!(
            article.admin_graphql_api_id,
            Some("gid://shopify/OnlineStoreArticle/134645308".to_string())
        );
    }

    #[test]
    fn test_article_nested_paths_require_blog_id() {
        // All paths should require blog_id (nested under blogs)

        // Find requires both blog_id and id
        let find_path = get_path(Article::PATHS, ResourceOperation::Find, &["blog_id", "id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "blogs/{blog_id}/articles/{id}");

        // Find with only id should fail (no standalone path)
        let find_without_blog = get_path(Article::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_without_blog.is_none());

        // All requires blog_id
        let all_path = get_path(Article::PATHS, ResourceOperation::All, &["blog_id"]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "blogs/{blog_id}/articles");

        // All without blog_id should fail
        let all_without_blog = get_path(Article::PATHS, ResourceOperation::All, &[]);
        assert!(all_without_blog.is_none());

        // Count requires blog_id
        let count_path = get_path(Article::PATHS, ResourceOperation::Count, &["blog_id"]);
        assert!(count_path.is_some());
        assert_eq!(
            count_path.unwrap().template,
            "blogs/{blog_id}/articles/count"
        );

        // Create requires blog_id
        let create_path = get_path(Article::PATHS, ResourceOperation::Create, &["blog_id"]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "blogs/{blog_id}/articles");

        // Update requires both blog_id and id
        let update_path = get_path(
            Article::PATHS,
            ResourceOperation::Update,
            &["blog_id", "id"],
        );
        assert!(update_path.is_some());
        assert_eq!(
            update_path.unwrap().template,
            "blogs/{blog_id}/articles/{id}"
        );

        // Delete requires both blog_id and id
        let delete_path = get_path(
            Article::PATHS,
            ResourceOperation::Delete,
            &["blog_id", "id"],
        );
        assert!(delete_path.is_some());
        assert_eq!(
            delete_path.unwrap().template,
            "blogs/{blog_id}/articles/{id}"
        );
    }

    #[test]
    fn test_article_list_params_serialization() {
        let params = ArticleListParams {
            author: Some("Jane Doe".to_string()),
            handle: Some("my-post".to_string()),
            tag: Some("tech".to_string()),
            published_status: Some("published".to_string()),
            limit: Some(50),
            since_id: Some(100),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["author"], "Jane Doe");
        assert_eq!(json["handle"], "my-post");
        assert_eq!(json["tag"], "tech");
        assert_eq!(json["published_status"], "published");
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);

        // Fields not set should be omitted
        assert!(json.get("created_at_min").is_none());
        assert!(json.get("page_info").is_none());
    }

    #[test]
    fn test_article_count_params_serialization() {
        let params = ArticleCountParams {
            author: Some("Jane Doe".to_string()),
            tag: Some("tech".to_string()),
            published_status: Some("any".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["author"], "Jane Doe");
        assert_eq!(json["tag"], "tech");
        assert_eq!(json["published_status"], "any");

        // Test empty params
        let empty_params = ArticleCountParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_article_tags_field_handling() {
        // Tags as comma-separated string
        let article = Article {
            title: Some("Tech Article".to_string()),
            tags: Some("rust, programming, web, api".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&article).unwrap();
        assert_eq!(json["tags"], "rust, programming, web, api");

        // Deserialize tags back
        let deserialized: Article = serde_json::from_value(json).unwrap();
        assert_eq!(
            deserialized.tags,
            Some("rust, programming, web, api".to_string())
        );
    }

    #[test]
    fn test_article_get_id_returns_correct_value() {
        // Article with ID
        let article_with_id = Article {
            id: Some(123456789),
            blog_id: Some(987654321),
            title: Some("Test Article".to_string()),
            ..Default::default()
        };
        assert_eq!(article_with_id.get_id(), Some(123456789));

        // Article without ID (new article)
        let article_without_id = Article {
            id: None,
            blog_id: Some(987654321),
            title: Some("New Article".to_string()),
            ..Default::default()
        };
        assert_eq!(article_without_id.get_id(), None);
    }

    #[test]
    fn test_article_image_struct() {
        let image = ArticleImage {
            src: Some("https://cdn.shopify.com/article-img.jpg".to_string()),
            alt: Some("Featured image".to_string()),
            width: Some(1920),
            height: Some(1080),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_value(&image).unwrap();

        // Writable fields should be present
        assert_eq!(json["src"], "https://cdn.shopify.com/article-img.jpg");
        assert_eq!(json["alt"], "Featured image");

        // Read-only fields should be omitted
        assert!(json.get("width").is_none());
        assert!(json.get("height").is_none());
        assert!(json.get("created_at").is_none());
    }

    #[test]
    fn test_article_constants() {
        assert_eq!(Article::NAME, "Article");
        assert_eq!(Article::PLURAL, "articles");
    }
}

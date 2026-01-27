//! Comment resource implementation.
//!
//! This module provides the [`Comment`] resource for managing blog article
//! comments in a Shopify store. Comments can be moderated, approved, marked
//! as spam, or removed.
//!
//! # Comment Moderation
//!
//! Comments have several moderation methods:
//! - `approve()` - Approve a pending comment for publication
//! - `spam()` - Mark a comment as spam
//! - `not_spam()` - Mark a comment as not spam
//! - `remove()` - Remove a comment from publication
//! - `restore()` - Restore a removed comment
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{Comment, CommentListParams};
//!
//! // List all comments
//! let comments = Comment::all(&client, None).await?;
//!
//! // Moderate a comment
//! let comment = Comment::find(&client, 653537639, None).await?.into_inner();
//! let approved = comment.approve(&client).await?;
//!
//! // Mark a comment as spam
//! let marked = comment.spam(&client).await?;
//!
//! // Filter comments by status
//! let params = CommentListParams {
//!     status: Some("pending".to_string()),
//!     ..Default::default()
//! };
//! let pending = Comment::all(&client, Some(params)).await?;
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{
    build_path, get_path, ResourceError, ResourceOperation, ResourcePath, ResourceResponse,
    RestResource,
};
use crate::HttpMethod;

/// A comment on a blog article.
///
/// Comments can be moderated through various methods. The status
/// determines the visibility and state of the comment.
///
/// # Moderation Methods
///
/// - `approve()` - Change status to "published"
/// - `spam()` - Mark as spam
/// - `not_spam()` - Remove spam flag
/// - `remove()` - Change status to "removed"
/// - `restore()` - Change status from "removed" to previous state
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `article_id` - The article the comment belongs to
/// - `blog_id` - The blog the article belongs to
/// - `status` - The comment status (pending, published, spam, removed)
/// - `ip` - The IP address of the commenter
/// - `user_agent` - The browser user agent of the commenter
/// - `published_at` - When the comment was published
/// - `created_at` - When the comment was created
/// - `updated_at` - When the comment was last updated
///
/// ## Writable Fields
/// - `author` - The name of the comment author
/// - `email` - The email of the comment author
/// - `body` - The comment text
/// - `body_html` - The comment text in HTML
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Comment {
    /// The unique identifier of the comment.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the article this comment belongs to.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub article_id: Option<u64>,

    /// The ID of the blog this comment belongs to.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub blog_id: Option<u64>,

    /// The name of the comment author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// The email address of the comment author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// The text of the comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// The text of the comment in HTML format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,

    /// The status of the comment: pending, published, spam, removed.
    /// Read-only field - use moderation methods to change.
    #[serde(skip_serializing)]
    pub status: Option<String>,

    /// The IP address of the commenter.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub ip: Option<String>,

    /// The browser user agent of the commenter.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub user_agent: Option<String>,

    /// When the comment was published.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub published_at: Option<DateTime<Utc>>,

    /// When the comment was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the comment was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
}

impl Comment {
    /// Approves a comment for publication.
    ///
    /// Changes the comment status to "published".
    ///
    /// # Errors
    ///
    /// Returns an error if the comment has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let comment = Comment::find(&client, 653537639, None).await?.into_inner();
    /// let approved = comment.approve(&client).await?;
    /// ```
    pub async fn approve(&self, client: &RestClient) -> Result<ResourceResponse<Self>, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "approve",
        })?;

        let url = format!("comments/{id}/approve");
        let response = client.post(&url, serde_json::json!({}), None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }

    /// Marks a comment as spam.
    ///
    /// # Errors
    ///
    /// Returns an error if the comment has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let comment = Comment::find(&client, 653537639, None).await?.into_inner();
    /// let marked = comment.spam(&client).await?;
    /// ```
    pub async fn spam(&self, client: &RestClient) -> Result<ResourceResponse<Self>, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "spam",
        })?;

        let url = format!("comments/{id}/spam");
        let response = client.post(&url, serde_json::json!({}), None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }

    /// Marks a comment as not spam.
    ///
    /// # Errors
    ///
    /// Returns an error if the comment has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let comment = Comment::find(&client, 653537639, None).await?.into_inner();
    /// let marked = comment.not_spam(&client).await?;
    /// ```
    pub async fn not_spam(&self, client: &RestClient) -> Result<ResourceResponse<Self>, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "not_spam",
        })?;

        let url = format!("comments/{id}/not_spam");
        let response = client.post(&url, serde_json::json!({}), None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }

    /// Removes a comment from publication.
    ///
    /// Changes the comment status to "removed".
    ///
    /// # Errors
    ///
    /// Returns an error if the comment has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let comment = Comment::find(&client, 653537639, None).await?.into_inner();
    /// let removed = comment.remove(&client).await?;
    /// ```
    pub async fn remove(&self, client: &RestClient) -> Result<ResourceResponse<Self>, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "remove",
        })?;

        let url = format!("comments/{id}/remove");
        let response = client.post(&url, serde_json::json!({}), None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }

    /// Restores a removed comment.
    ///
    /// Returns the comment to its previous state before removal.
    ///
    /// # Errors
    ///
    /// Returns an error if the comment has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let comment = Comment::find(&client, 653537639, None).await?.into_inner();
    /// let restored = comment.restore(&client).await?;
    /// ```
    pub async fn restore(&self, client: &RestClient) -> Result<ResourceResponse<Self>, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "restore",
        })?;

        let url = format!("comments/{id}/restore");
        let response = client.post(&url, serde_json::json!({}), None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }

    /// Counts comments under a specific article.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client
    /// * `article_id` - The article ID
    /// * `params` - Optional count parameters
    pub async fn count_for_article(
        client: &RestClient,
        article_id: u64,
        params: Option<CommentCountParams>,
    ) -> Result<u64, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert("article_id", article_id.to_string());

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

impl RestResource for Comment {
    type Id = u64;
    type FindParams = CommentFindParams;
    type AllParams = CommentListParams;
    type CountParams = CommentCountParams;

    const NAME: &'static str = "Comment";
    const PLURAL: &'static str = "comments";

    /// Paths for the Comment resource.
    ///
    /// Full CRUD operations plus article-specific paths.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "comments/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "comments"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "comments/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "comments",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "comments/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "comments/{id}",
        ),
        // Article-specific paths
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["article_id"],
            "articles/{article_id}/comments",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["article_id"],
            "articles/{article_id}/comments/count",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single comment.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CommentFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing comments.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CommentListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return comments after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Show comments created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show comments created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show comments updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show comments updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show comments published after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_min: Option<DateTime<Utc>>,

    /// Show comments published before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_max: Option<DateTime<Utc>>,

    /// Filter comments by status: pending, published, spam, removed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting comments.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CommentCountParams {
    /// Show comments created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show comments created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show comments updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show comments updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show comments published after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_min: Option<DateTime<Utc>>,

    /// Show comments published before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_max: Option<DateTime<Utc>>,

    /// Filter comments by status: pending, published, spam, removed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_comment_serialization() {
        let comment = Comment {
            id: Some(653537639),
            article_id: Some(134645308),
            blog_id: Some(241253187),
            author: Some("John Doe".to_string()),
            email: Some("john@example.com".to_string()),
            body: Some("Great article!".to_string()),
            body_html: Some("<p>Great article!</p>".to_string()),
            status: Some("published".to_string()),
            ip: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            published_at: Some(
                DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-06-15T10:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_string(&comment).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["author"], "John Doe");
        assert_eq!(parsed["email"], "john@example.com");
        assert_eq!(parsed["body"], "Great article!");
        assert_eq!(parsed["body_html"], "<p>Great article!</p>");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("article_id").is_none());
        assert!(parsed.get("blog_id").is_none());
        assert!(parsed.get("status").is_none());
        assert!(parsed.get("ip").is_none());
        assert!(parsed.get("user_agent").is_none());
        assert!(parsed.get("published_at").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_comment_deserialization() {
        let json = r#"{
            "id": 653537639,
            "article_id": 134645308,
            "blog_id": 241253187,
            "author": "John Doe",
            "email": "john@example.com",
            "body": "Great article!",
            "body_html": "<p>Great article!</p>",
            "status": "published",
            "ip": "192.168.1.1",
            "user_agent": "Mozilla/5.0",
            "published_at": "2024-06-15T10:30:00Z",
            "created_at": "2024-06-15T10:00:00Z",
            "updated_at": "2024-06-15T10:30:00Z"
        }"#;

        let comment: Comment = serde_json::from_str(json).unwrap();

        assert_eq!(comment.id, Some(653537639));
        assert_eq!(comment.article_id, Some(134645308));
        assert_eq!(comment.blog_id, Some(241253187));
        assert_eq!(comment.author, Some("John Doe".to_string()));
        assert_eq!(comment.email, Some("john@example.com".to_string()));
        assert_eq!(comment.body, Some("Great article!".to_string()));
        assert_eq!(comment.status, Some("published".to_string()));
        assert_eq!(comment.ip, Some("192.168.1.1".to_string()));
        assert!(comment.published_at.is_some());
        assert!(comment.created_at.is_some());
    }

    #[test]
    fn test_comment_moderation_methods_path_construction() {
        // The moderation methods use URLs like:
        // POST /comments/{id}/approve
        // POST /comments/{id}/spam
        // POST /comments/{id}/not_spam
        // POST /comments/{id}/remove
        // POST /comments/{id}/restore
        //
        // These are implemented as instance methods that construct URLs directly

        let comment = Comment {
            id: Some(653537639),
            ..Default::default()
        };

        // Verify that the ID is available for URL construction
        assert_eq!(comment.id, Some(653537639));
        // URLs would be: comments/653537639/approve, etc.
    }

    #[test]
    fn test_comment_full_crud_paths() {
        // Find by ID
        let find_path = get_path(Comment::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "comments/{id}");

        // List all
        let all_path = get_path(Comment::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "comments");

        // Count
        let count_path = get_path(Comment::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "comments/count");

        // Create
        let create_path = get_path(Comment::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "comments");

        // Update
        let update_path = get_path(Comment::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "comments/{id}");

        // Delete
        let delete_path = get_path(Comment::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "comments/{id}");
    }

    #[test]
    fn test_comment_article_specific_paths() {
        // Comments for an article
        let article_comments = get_path(Comment::PATHS, ResourceOperation::All, &["article_id"]);
        assert!(article_comments.is_some());
        assert_eq!(
            article_comments.unwrap().template,
            "articles/{article_id}/comments"
        );

        // Count comments for an article
        let article_count = get_path(Comment::PATHS, ResourceOperation::Count, &["article_id"]);
        assert!(article_count.is_some());
        assert_eq!(
            article_count.unwrap().template,
            "articles/{article_id}/comments/count"
        );
    }

    #[test]
    fn test_comment_list_params() {
        let params = CommentListParams {
            limit: Some(50),
            status: Some("pending".to_string()),
            since_id: Some(100),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["status"], "pending");
        assert_eq!(json["since_id"], 100);
    }

    #[test]
    fn test_comment_constants() {
        assert_eq!(Comment::NAME, "Comment");
        assert_eq!(Comment::PLURAL, "comments");
    }

    #[test]
    fn test_comment_get_id() {
        let comment_with_id = Comment {
            id: Some(653537639),
            ..Default::default()
        };
        assert_eq!(comment_with_id.get_id(), Some(653537639));

        let comment_without_id = Comment::default();
        assert_eq!(comment_without_id.get_id(), None);
    }
}

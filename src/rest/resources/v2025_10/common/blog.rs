//! Blog-related types for blog resources.
//!
//! This module provides types for blog comment settings.

use serde::{Deserialize, Serialize};

/// The comment moderation setting for a blog.
///
/// Controls whether comments are allowed on blog articles
/// and how they are moderated.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::BlogCommentable;
///
/// let setting = BlogCommentable::Moderate;
/// let json = serde_json::to_string(&setting).unwrap();
/// assert_eq!(json, "\"moderate\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BlogCommentable {
    /// Comments are not allowed.
    #[default]
    No,
    /// Comments require moderation before appearing.
    Moderate,
    /// Comments are allowed without moderation.
    Yes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blog_commentable_serialization() {
        let setting = BlogCommentable::No;
        let json = serde_json::to_string(&setting).unwrap();
        assert_eq!(json, "\"no\"");

        let setting = BlogCommentable::Moderate;
        let json = serde_json::to_string(&setting).unwrap();
        assert_eq!(json, "\"moderate\"");

        let setting = BlogCommentable::Yes;
        let json = serde_json::to_string(&setting).unwrap();
        assert_eq!(json, "\"yes\"");
    }

    #[test]
    fn test_blog_commentable_deserialization() {
        let setting: BlogCommentable = serde_json::from_str("\"no\"").unwrap();
        assert_eq!(setting, BlogCommentable::No);

        let setting: BlogCommentable = serde_json::from_str("\"moderate\"").unwrap();
        assert_eq!(setting, BlogCommentable::Moderate);

        let setting: BlogCommentable = serde_json::from_str("\"yes\"").unwrap();
        assert_eq!(setting, BlogCommentable::Yes);
    }

    #[test]
    fn test_blog_commentable_default() {
        let setting = BlogCommentable::default();
        assert_eq!(setting, BlogCommentable::No);
    }
}

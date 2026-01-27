//! Theme-related types for theme resources.
//!
//! This module provides types for theme roles used by the Theme resource.

use serde::{Deserialize, Serialize};

/// The role of a theme in the store.
///
/// Each store has one main (published) theme and can have multiple
/// unpublished themes for development or preview.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::rest::resources::v2025_10::common::ThemeRole;
///
/// let role = ThemeRole::Main;
/// let json = serde_json::to_string(&role).unwrap();
/// assert_eq!(json, "\"main\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemeRole {
    /// The main (published) theme visible to customers.
    Main,
    /// An unpublished theme.
    #[default]
    Unpublished,
    /// A demo theme from the theme store.
    Demo,
    /// A development theme for theme development.
    Development,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_role_serialization() {
        let role = ThemeRole::Main;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"main\"");

        let role = ThemeRole::Unpublished;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"unpublished\"");

        let role = ThemeRole::Demo;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"demo\"");

        let role = ThemeRole::Development;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"development\"");
    }

    #[test]
    fn test_theme_role_deserialization() {
        let role: ThemeRole = serde_json::from_str("\"main\"").unwrap();
        assert_eq!(role, ThemeRole::Main);

        let role: ThemeRole = serde_json::from_str("\"unpublished\"").unwrap();
        assert_eq!(role, ThemeRole::Unpublished);

        let role: ThemeRole = serde_json::from_str("\"demo\"").unwrap();
        assert_eq!(role, ThemeRole::Demo);

        let role: ThemeRole = serde_json::from_str("\"development\"").unwrap();
        assert_eq!(role, ThemeRole::Development);
    }

    #[test]
    fn test_theme_role_default() {
        let role = ThemeRole::default();
        assert_eq!(role, ThemeRole::Unpublished);
    }
}

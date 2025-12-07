//! Associated user types for Shopify API online sessions.
//!
//! This module provides the [`AssociatedUser`] type for storing user information
//! associated with online (user-specific) sessions.
//!
//! # Overview
//!
//! When a Shopify app uses online access tokens, the OAuth response includes
//! information about the user who authorized the app. This user information
//! is stored in the `AssociatedUser` struct.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::AssociatedUser;
//!
//! let user = AssociatedUser::new(
//!     12345,
//!     "Jane".to_string(),
//!     "Doe".to_string(),
//!     "jane@example.com".to_string(),
//!     true,   // email_verified
//!     true,   // account_owner
//!     "en".to_string(),
//!     false,  // collaborator
//! );
//!
//! assert_eq!(user.id, 12345);
//! assert_eq!(user.email, "jane@example.com");
//! ```

use serde::{Deserialize, Serialize};

/// Represents a Shopify user associated with an online session.
///
/// This struct holds information about the user who authorized an app
/// during the OAuth flow when using online access tokens.
///
/// # Thread Safety
///
/// `AssociatedUser` is `Send + Sync`, making it safe to share across threads.
///
/// # Serialization
///
/// The struct derives `Serialize` and `Deserialize` for easy storage and
/// transmission in JSON format.
///
/// # Example
///
/// ```rust
/// use shopify_api::AssociatedUser;
///
/// let user = AssociatedUser::new(
///     12345,
///     "Jane".to_string(),
///     "Doe".to_string(),
///     "jane@example.com".to_string(),
///     true,
///     false,
///     "en".to_string(),
///     false,
/// );
///
/// // Serialize to JSON
/// let json = serde_json::to_string(&user).unwrap();
/// assert!(json.contains("12345"));
///
/// // Deserialize from JSON
/// let restored: AssociatedUser = serde_json::from_str(&json).unwrap();
/// assert_eq!(user, restored);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssociatedUser {
    /// The Shopify user ID (numeric).
    pub id: u64,

    /// The user's first name.
    pub first_name: String,

    /// The user's last name.
    pub last_name: String,

    /// The user's email address.
    pub email: String,

    /// Whether the user's email has been verified.
    pub email_verified: bool,

    /// Whether the user is the account owner.
    pub account_owner: bool,

    /// The user's locale preference (e.g., "en", "fr").
    pub locale: String,

    /// Whether the user is a collaborator.
    pub collaborator: bool,
}

impl AssociatedUser {
    /// Creates a new `AssociatedUser` with all required fields.
    ///
    /// # Arguments
    ///
    /// * `id` - The Shopify user ID
    /// * `first_name` - The user's first name
    /// * `last_name` - The user's last name
    /// * `email` - The user's email address
    /// * `email_verified` - Whether the email has been verified
    /// * `account_owner` - Whether the user is the account owner
    /// * `locale` - The user's locale preference
    /// * `collaborator` - Whether the user is a collaborator
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::AssociatedUser;
    ///
    /// let user = AssociatedUser::new(
    ///     12345,
    ///     "Jane".to_string(),
    ///     "Doe".to_string(),
    ///     "jane@example.com".to_string(),
    ///     true,
    ///     true,
    ///     "en".to_string(),
    ///     false,
    /// );
    /// ```
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        id: u64,
        first_name: String,
        last_name: String,
        email: String,
        email_verified: bool,
        account_owner: bool,
        locale: String,
        collaborator: bool,
    ) -> Self {
        Self {
            id,
            first_name,
            last_name,
            email,
            email_verified,
            account_owner,
            locale,
            collaborator,
        }
    }
}

// Verify AssociatedUser is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<AssociatedUser>();
};

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_user() -> AssociatedUser {
        AssociatedUser::new(
            12345,
            "Jane".to_string(),
            "Doe".to_string(),
            "jane@example.com".to_string(),
            true,
            true,
            "en".to_string(),
            false,
        )
    }

    #[test]
    fn test_associated_user_creation_with_all_fields() {
        let user = sample_user();

        assert_eq!(user.id, 12345);
        assert_eq!(user.first_name, "Jane");
        assert_eq!(user.last_name, "Doe");
        assert_eq!(user.email, "jane@example.com");
        assert!(user.email_verified);
        assert!(user.account_owner);
        assert_eq!(user.locale, "en");
        assert!(!user.collaborator);
    }

    #[test]
    fn test_associated_user_serialization_to_json() {
        let user = sample_user();
        let json = serde_json::to_string(&user).unwrap();

        assert!(json.contains("12345"));
        assert!(json.contains("Jane"));
        assert!(json.contains("Doe"));
        assert!(json.contains("jane@example.com"));
        assert!(json.contains("email_verified"));
        assert!(json.contains("account_owner"));
        assert!(json.contains("en"));
        assert!(json.contains("collaborator"));
    }

    #[test]
    fn test_associated_user_deserialization_from_json() {
        let json = r#"{
            "id": 67890,
            "first_name": "John",
            "last_name": "Smith",
            "email": "john@example.com",
            "email_verified": false,
            "account_owner": false,
            "locale": "fr",
            "collaborator": true
        }"#;

        let user: AssociatedUser = serde_json::from_str(json).unwrap();

        assert_eq!(user.id, 67890);
        assert_eq!(user.first_name, "John");
        assert_eq!(user.last_name, "Smith");
        assert_eq!(user.email, "john@example.com");
        assert!(!user.email_verified);
        assert!(!user.account_owner);
        assert_eq!(user.locale, "fr");
        assert!(user.collaborator);
    }

    #[test]
    fn test_associated_user_equality_comparison() {
        let user1 = sample_user();
        let user2 = sample_user();

        assert_eq!(user1, user2);

        // Different user should not be equal
        let user3 = AssociatedUser::new(
            99999,
            "Jane".to_string(),
            "Doe".to_string(),
            "jane@example.com".to_string(),
            true,
            true,
            "en".to_string(),
            false,
        );

        assert_ne!(user1, user3);
    }

    #[test]
    fn test_associated_user_clone_preserves_all_fields() {
        let user = sample_user();
        let cloned = user.clone();

        assert_eq!(user.id, cloned.id);
        assert_eq!(user.first_name, cloned.first_name);
        assert_eq!(user.last_name, cloned.last_name);
        assert_eq!(user.email, cloned.email);
        assert_eq!(user.email_verified, cloned.email_verified);
        assert_eq!(user.account_owner, cloned.account_owner);
        assert_eq!(user.locale, cloned.locale);
        assert_eq!(user.collaborator, cloned.collaborator);
        assert_eq!(user, cloned);
    }

    #[test]
    fn test_associated_user_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AssociatedUser>();
    }
}

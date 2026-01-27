//! User resource implementation.
//!
//! This module provides the [`User`] resource for retrieving staff users
//! in a Shopify store.
//!
//! # Read-Only Resource
//!
//! Users implement [`ReadOnlyResource`](crate::rest::ReadOnlyResource) - they
//! can only be retrieved, not created, updated, or deleted through the API.
//! Staff accounts are managed through the Shopify admin.
//!
//! # Special Operations
//!
//! - `User::current()` - Get the currently authenticated user
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{User, UserListParams};
//!
//! // Get the current authenticated user
//! let current = User::current(&client).await?;
//! println!("Logged in as: {} {}",
//!     current.first_name.as_deref().unwrap_or(""),
//!     current.last_name.as_deref().unwrap_or(""));
//!
//! // List all staff users
//! let users = User::all(&client, None).await?;
//! for user in users.iter() {
//!     println!("{} {} <{}>",
//!         user.first_name.as_deref().unwrap_or(""),
//!         user.last_name.as_deref().unwrap_or(""),
//!         user.email.as_deref().unwrap_or(""));
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ReadOnlyResource, ResourceError, ResourceOperation, ResourcePath, ResourceResponse, RestResource};
use crate::HttpMethod;

/// A staff user in a Shopify store.
///
/// Users represent staff accounts that can access the Shopify admin.
/// They are read-only through the API.
///
/// # Read-Only Resource
///
/// This resource implements [`ReadOnlyResource`] - only GET operations are
/// available. User accounts are managed through the Shopify admin.
///
/// # Fields
///
/// All fields are read-only:
/// - `id` - The unique identifier
/// - `first_name` - User's first name
/// - `last_name` - User's last name
/// - `email` - User's email address
/// - `phone` - User's phone number
/// - `url` - User's Shopify admin URL
/// - `bio` - User's biography
/// - `im` - Instant messenger handle
/// - `screen_name` - Screen name
/// - `locale` - User's preferred locale
/// - `user_type` - Type of user account
/// - `account_owner` - Whether user owns the account
/// - `receive_announcements` - Whether user receives announcements
/// - `permissions` - Array of permission strings
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct User {
    /// The unique identifier of the user.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The user's first name.
    #[serde(skip_serializing)]
    pub first_name: Option<String>,

    /// The user's last name.
    #[serde(skip_serializing)]
    pub last_name: Option<String>,

    /// The user's email address.
    #[serde(skip_serializing)]
    pub email: Option<String>,

    /// The user's phone number.
    #[serde(skip_serializing)]
    pub phone: Option<String>,

    /// URL to the user's Shopify admin page.
    #[serde(skip_serializing)]
    pub url: Option<String>,

    /// The user's biography.
    #[serde(skip_serializing)]
    pub bio: Option<String>,

    /// The user's instant messenger handle.
    #[serde(skip_serializing)]
    pub im: Option<String>,

    /// The user's screen name.
    #[serde(skip_serializing)]
    pub screen_name: Option<String>,

    /// The user's preferred locale.
    #[serde(skip_serializing)]
    pub locale: Option<String>,

    /// The type of user: "regular", "restricted", "invited".
    #[serde(skip_serializing)]
    pub user_type: Option<String>,

    /// Whether this user owns the account.
    #[serde(skip_serializing)]
    pub account_owner: Option<bool>,

    /// Whether the user receives announcements.
    #[serde(skip_serializing)]
    pub receive_announcements: Option<i32>,

    /// The user's permissions.
    #[serde(skip_serializing)]
    pub permissions: Option<Vec<String>>,

    /// The admin GraphQL API ID for this user.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl User {
    /// Gets the currently authenticated user.
    ///
    /// This returns information about the user whose access token
    /// is being used for the request.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let current = User::current(&client).await?;
    /// println!("Current user: {} {}", current.first_name.unwrap_or(""), current.last_name.unwrap_or(""));
    /// ```
    pub async fn current(client: &RestClient) -> Result<ResourceResponse<Self>, ResourceError> {
        let url = "users/current";
        let response = client.get(url, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some("current"),
                response.request_id(),
            ));
        }

        let key = Self::resource_key();
        ResourceResponse::from_http_response(response, &key)
    }
}

impl RestResource for User {
    type Id = u64;
    type FindParams = UserFindParams;
    type AllParams = UserListParams;
    type CountParams = ();

    const NAME: &'static str = "User";
    const PLURAL: &'static str = "users";

    /// Paths for the User resource.
    ///
    /// Only GET operations - users are read-only.
    /// No Count endpoint.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "users/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "users"),
        // Note: No Count path
        // Note: No Create, Update, or Delete paths - read-only resource
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl ReadOnlyResource for User {}

/// Parameters for finding a single user.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct UserFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing users.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct UserListParams {
    /// Maximum number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ReadOnlyResource, ResourceOperation, RestResource};

    #[test]
    fn test_user_implements_read_only_resource() {
        fn assert_read_only<T: ReadOnlyResource>() {}
        assert_read_only::<User>();
    }

    #[test]
    fn test_user_deserialization() {
        let json = r#"{
            "id": 548380009,
            "first_name": "John",
            "last_name": "Smith",
            "email": "john@example.com",
            "phone": "+1-555-0100",
            "url": "https://store.myshopify.com/admin/users/548380009",
            "bio": "Store manager",
            "im": null,
            "screen_name": null,
            "locale": "en",
            "user_type": "regular",
            "account_owner": false,
            "receive_announcements": 1,
            "permissions": ["full"],
            "admin_graphql_api_id": "gid://shopify/StaffMember/548380009"
        }"#;

        let user: User = serde_json::from_str(json).unwrap();

        assert_eq!(user.id, Some(548380009));
        assert_eq!(user.first_name, Some("John".to_string()));
        assert_eq!(user.last_name, Some("Smith".to_string()));
        assert_eq!(user.email, Some("john@example.com".to_string()));
        assert_eq!(user.phone, Some("+1-555-0100".to_string()));
        assert_eq!(user.locale, Some("en".to_string()));
        assert_eq!(user.user_type, Some("regular".to_string()));
        assert_eq!(user.account_owner, Some(false));
        assert_eq!(user.receive_announcements, Some(1));
        assert_eq!(user.permissions, Some(vec!["full".to_string()]));
    }

    #[test]
    fn test_user_read_only_paths() {
        // Find
        let find_path = get_path(User::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "users/{id}");

        // All
        let all_path = get_path(User::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "users");

        // No count path
        let count_path = get_path(User::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());

        // No create, update, or delete paths
        let create_path = get_path(User::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_none());

        let update_path = get_path(User::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());

        let delete_path = get_path(User::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_user_current_method_exists() {
        // The current() method is a static method that returns the current user
        // We can't test the actual HTTP call, but we verify the method exists
        // by checking the struct has the expected signature
        let _: fn(&RestClient) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ResourceResponse<User>, ResourceError>> + Send + '_>> = |client| Box::pin(User::current(client));
    }

    #[test]
    fn test_user_constants() {
        assert_eq!(User::NAME, "User");
        assert_eq!(User::PLURAL, "users");
    }

    #[test]
    fn test_user_get_id() {
        let user_with_id = User {
            id: Some(548380009),
            first_name: Some("John".to_string()),
            ..Default::default()
        };
        assert_eq!(user_with_id.get_id(), Some(548380009));

        let user_without_id = User::default();
        assert_eq!(user_without_id.get_id(), None);
    }

    #[test]
    fn test_user_all_fields_are_read_only() {
        // All fields should be skipped during serialization
        let user = User {
            id: Some(548380009),
            first_name: Some("John".to_string()),
            last_name: Some("Smith".to_string()),
            email: Some("john@example.com".to_string()),
            phone: Some("+1-555-0100".to_string()),
            locale: Some("en".to_string()),
            user_type: Some("regular".to_string()),
            account_owner: Some(true),
            receive_announcements: Some(1),
            permissions: Some(vec!["full".to_string()]),
            ..Default::default()
        };

        let json = serde_json::to_value(&user).unwrap();
        // All fields should be omitted (empty object)
        assert_eq!(json, serde_json::json!({}));
    }
}

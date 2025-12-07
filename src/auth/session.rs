//! Session management for Shopify API authentication.
//!
//! This module provides the [`Session`] type for representing authenticated
//! sessions used in API calls, along with helper methods for session ID generation
//! and factory methods for creating sessions from OAuth responses.
//!
//! # Session Types
//!
//! Shopify supports two types of access tokens, each with its own session pattern:
//!
//! - **Offline sessions** (`is_online = false`):
//!   - App-level tokens that persist indefinitely (or until expiration for expiring tokens)
//!   - ID format: `"offline_{shop}"` (e.g., `"offline_my-store.myshopify.com"`)
//!   - May include refresh tokens for token refresh flow
//!   - Used for background tasks, webhooks, and server-side operations
//!
//! - **Online sessions** (`is_online = true`):
//!   - User-specific tokens that expire
//!   - ID format: `"{shop}_{user_id}"` (e.g., `"my-store.myshopify.com_12345"`)
//!   - Include expiration time and associated user information
//!   - Used for user-facing operations where user identity matters
//!
//! # Immutability
//!
//! Sessions are immutable after creation. To "update" a session, create a new
//! `Session` instance. This design ensures thread safety and prevents accidental
//! mutation of authentication state.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::{Session, ShopDomain, AuthScopes};
//!
//! // Create an offline session with generated ID
//! let shop = ShopDomain::new("my-store").unwrap();
//! let session = Session::new(
//!     Session::generate_offline_id(&shop),
//!     shop,
//!     "access-token".to_string(),
//!     "read_products".parse().unwrap(),
//!     false,
//!     None,
//! );
//!
//! assert_eq!(session.id, "offline_my-store.myshopify.com");
//! assert!(session.is_active());
//! ```

use crate::auth::associated_user::AssociatedUser;
use crate::auth::AuthScopes;
use crate::config::ShopDomain;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Buffer time (in seconds) before considering a refresh token expired.
/// Matches the Ruby SDK's behavior.
const REFRESH_TOKEN_EXPIRY_BUFFER_SECONDS: i64 = 60;

/// Represents an authenticated session for Shopify API calls.
///
/// Sessions hold the authentication state needed to make API requests on behalf
/// of a shop. They can be either online (user-specific) or offline (app-level).
///
/// # Thread Safety
///
/// `Session` is `Send + Sync`, making it safe to share across threads.
///
/// # Serialization
///
/// Sessions can be serialized to JSON for storage and deserialized when needed:
///
/// ```rust
/// use shopify_api::{Session, ShopDomain, AuthScopes};
///
/// let session = Session::new(
///     "session-id".to_string(),
///     ShopDomain::new("my-store").unwrap(),
///     "access-token".to_string(),
///     "read_products".parse().unwrap(),
///     false,
///     None,
/// );
///
/// // Serialize to JSON
/// let json = serde_json::to_string(&session).unwrap();
///
/// // Deserialize from JSON
/// let restored: Session = serde_json::from_str(&json).unwrap();
/// assert_eq!(session, restored);
/// ```
///
/// # Example
///
/// ```rust
/// use shopify_api::{Session, ShopDomain, AuthScopes};
///
/// let session = Session::new(
///     "session-id".to_string(),
///     ShopDomain::new("my-store").unwrap(),
///     "access-token".to_string(),
///     "read_products".parse().unwrap(),
///     false, // offline session
///     None,  // no expiration
/// );
///
/// assert!(session.is_active());
/// assert!(!session.expired());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for this session.
    ///
    /// For offline sessions: `"offline_{shop}"` (e.g., `"offline_my-store.myshopify.com"`)
    /// For online sessions: `"{shop}_{user_id}"` (e.g., `"my-store.myshopify.com_12345"`)
    pub id: String,

    /// The shop this session is for.
    pub shop: ShopDomain,

    /// The access token for API authentication.
    pub access_token: String,

    /// The OAuth scopes granted to this session.
    pub scopes: AuthScopes,

    /// Whether this is an online (user-specific) session.
    pub is_online: bool,

    /// When this session expires, if applicable.
    ///
    /// Offline sessions have `None` (they don't expire) unless using expiring tokens.
    /// Online sessions have a specific expiration time.
    pub expires: Option<DateTime<Utc>>,

    /// OAuth state parameter, if applicable.
    ///
    /// Used during the OAuth flow for CSRF protection.
    pub state: Option<String>,

    /// Shopify-provided session ID, if applicable.
    pub shopify_session_id: Option<String>,

    /// User information for online sessions.
    ///
    /// Only present for online sessions (when `is_online` is `true`).
    pub associated_user: Option<AssociatedUser>,

    /// User-specific scopes for online sessions.
    ///
    /// These may be different from the app's granted scopes, representing
    /// what the specific user is allowed to do.
    pub associated_user_scopes: Option<AuthScopes>,

    /// The refresh token for obtaining new access tokens.
    ///
    /// Only present for expiring offline tokens. Use with [`refresh_access_token`]
    /// to obtain a new access token before the current one expires.
    ///
    /// [`refresh_access_token`]: crate::auth::oauth::refresh_access_token
    #[serde(default)]
    pub refresh_token: Option<String>,

    /// When the refresh token expires, if applicable.
    ///
    /// `None` indicates the refresh token does not expire or is not present.
    /// Use [`refresh_token_expired`](Session::refresh_token_expired) to check
    /// if the refresh token needs to be renewed.
    #[serde(default)]
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
}

impl Session {
    /// Creates a new session with the specified parameters.
    ///
    /// This constructor maintains backward compatibility with existing code.
    /// New fields (`associated_user`, `associated_user_scopes`, `refresh_token`,
    /// and `refresh_token_expires_at`) default to `None`.
    ///
    /// For online sessions with user information, use [`Session::with_user`] instead.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique session identifier
    /// * `shop` - The shop domain
    /// * `access_token` - The access token for API calls
    /// * `scopes` - OAuth scopes granted to this session
    /// * `is_online` - Whether this is a user-specific session
    /// * `expires` - When this session expires (None for offline sessions)
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{Session, ShopDomain, AuthScopes};
    ///
    /// let session = Session::new(
    ///     "offline_my-store.myshopify.com".to_string(),
    ///     ShopDomain::new("my-store").unwrap(),
    ///     "access-token".to_string(),
    ///     "read_products".parse().unwrap(),
    ///     false,
    ///     None,
    /// );
    /// ```
    #[must_use]
    pub const fn new(
        id: String,
        shop: ShopDomain,
        access_token: String,
        scopes: AuthScopes,
        is_online: bool,
        expires: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            shop,
            access_token,
            scopes,
            is_online,
            expires,
            state: None,
            shopify_session_id: None,
            associated_user: None,
            associated_user_scopes: None,
            refresh_token: None,
            refresh_token_expires_at: None,
        }
    }

    /// Creates a new online session with associated user information.
    ///
    /// This is a convenience constructor for online sessions that includes
    /// user details and user-specific scopes.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique session identifier (typically `"{shop}_{user_id}"`)
    /// * `shop` - The shop domain
    /// * `access_token` - The access token for API calls
    /// * `scopes` - OAuth scopes granted to this session
    /// * `expires` - When this session expires
    /// * `associated_user` - The user who authorized this session
    /// * `associated_user_scopes` - User-specific scopes (if different from app scopes)
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{Session, ShopDomain, AuthScopes, AssociatedUser};
    /// use chrono::{Utc, Duration};
    ///
    /// let shop = ShopDomain::new("my-store").unwrap();
    /// let user = AssociatedUser::new(
    ///     12345,
    ///     "Jane".to_string(),
    ///     "Doe".to_string(),
    ///     "jane@example.com".to_string(),
    ///     true, true, "en".to_string(), false,
    /// );
    ///
    /// let session = Session::with_user(
    ///     Session::generate_online_id(&shop, 12345),
    ///     shop,
    ///     "access-token".to_string(),
    ///     "read_products".parse().unwrap(),
    ///     Some(Utc::now() + Duration::hours(1)),
    ///     user,
    ///     Some("read_products".parse().unwrap()),
    /// );
    ///
    /// assert!(session.is_online);
    /// assert!(session.associated_user.is_some());
    /// ```
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn with_user(
        id: String,
        shop: ShopDomain,
        access_token: String,
        scopes: AuthScopes,
        expires: Option<DateTime<Utc>>,
        associated_user: AssociatedUser,
        associated_user_scopes: Option<AuthScopes>,
    ) -> Self {
        Self {
            id,
            shop,
            access_token,
            scopes,
            is_online: true,
            expires,
            state: None,
            shopify_session_id: None,
            associated_user: Some(associated_user),
            associated_user_scopes,
            refresh_token: None,
            refresh_token_expires_at: None,
        }
    }

    /// Generates a session ID for an offline session.
    ///
    /// The ID format is `"offline_{shop}"` where `{shop}` is the full shop domain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{Session, ShopDomain};
    ///
    /// let shop = ShopDomain::new("my-store").unwrap();
    /// let id = Session::generate_offline_id(&shop);
    /// assert_eq!(id, "offline_my-store.myshopify.com");
    /// ```
    #[must_use]
    pub fn generate_offline_id(shop: &ShopDomain) -> String {
        format!("offline_{}", shop.as_ref())
    }

    /// Generates a session ID for an online session.
    ///
    /// The ID format is `"{shop}_{user_id}"` where `{shop}` is the full shop domain
    /// and `{user_id}` is the Shopify user ID.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{Session, ShopDomain};
    ///
    /// let shop = ShopDomain::new("my-store").unwrap();
    /// let id = Session::generate_online_id(&shop, 12345);
    /// assert_eq!(id, "my-store.myshopify.com_12345");
    /// ```
    #[must_use]
    pub fn generate_online_id(shop: &ShopDomain, user_id: u64) -> String {
        format!("{}_{}", shop.as_ref(), user_id)
    }

    /// Creates a session from an OAuth access token response.
    ///
    /// This factory method automatically:
    /// - Generates the appropriate session ID based on session type
    /// - Parses scopes from the response
    /// - Calculates expiration time from `expires_in` seconds
    /// - Sets `is_online` based on presence of associated user
    /// - Populates refresh token fields if present in the response
    ///
    /// # Arguments
    ///
    /// * `shop` - The shop domain
    /// * `response` - The OAuth access token response from Shopify
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{Session, ShopDomain};
    /// use shopify_api::auth::session::AccessTokenResponse;
    ///
    /// let shop = ShopDomain::new("my-store").unwrap();
    /// let response = AccessTokenResponse {
    ///     access_token: "access-token".to_string(),
    ///     scope: "read_products,write_orders".to_string(),
    ///     expires_in: None,
    ///     associated_user_scope: None,
    ///     associated_user: None,
    ///     session: None,
    ///     refresh_token: None,
    ///     refresh_token_expires_in: None,
    /// };
    ///
    /// let session = Session::from_access_token_response(shop, &response);
    /// assert!(!session.is_online);
    /// assert_eq!(session.id, "offline_my-store.myshopify.com");
    /// ```
    #[must_use]
    pub fn from_access_token_response(shop: ShopDomain, response: &AccessTokenResponse) -> Self {
        let is_online = response.associated_user.is_some();

        let id = response.associated_user.as_ref().map_or_else(
            || Self::generate_offline_id(&shop),
            |user| Self::generate_online_id(&shop, user.id),
        );

        let scopes: AuthScopes = response.scope.parse().unwrap_or_default();

        let expires = response
            .expires_in
            .map(|secs| Utc::now() + Duration::seconds(i64::from(secs)));

        let associated_user_scopes = response
            .associated_user_scope
            .as_ref()
            .and_then(|s| s.parse().ok());

        let associated_user = response.associated_user.as_ref().map(|u| AssociatedUser {
            id: u.id,
            first_name: u.first_name.clone(),
            last_name: u.last_name.clone(),
            email: u.email.clone(),
            email_verified: u.email_verified,
            account_owner: u.account_owner,
            locale: u.locale.clone(),
            collaborator: u.collaborator,
        });

        let refresh_token = response.refresh_token.clone();

        let refresh_token_expires_at = response
            .refresh_token_expires_in
            .map(|secs| Utc::now() + Duration::seconds(i64::from(secs)));

        Self {
            id,
            shop,
            access_token: response.access_token.clone(),
            scopes,
            is_online,
            expires,
            state: None,
            shopify_session_id: response.session.clone(),
            associated_user,
            associated_user_scopes,
            refresh_token,
            refresh_token_expires_at,
        }
    }

    /// Returns `true` if this session has expired.
    ///
    /// Sessions without an expiration time are considered never expired.
    #[must_use]
    pub fn expired(&self) -> bool {
        self.expires.is_some_and(|expires| Utc::now() > expires)
    }

    /// Returns `true` if this session is active (not expired and has access token).
    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.access_token.is_empty() && !self.expired()
    }

    /// Returns `true` if the refresh token has expired or will expire within 60 seconds.
    ///
    /// This method uses a 60-second buffer (matching the Ruby SDK) to ensure
    /// you have time to refresh the token before it actually expires.
    ///
    /// Returns `false` if:
    /// - No `refresh_token_expires_at` is set (token doesn't expire)
    /// - The refresh token has more than 60 seconds before expiration
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::{Session, ShopDomain, AuthScopes};
    /// use chrono::{Utc, Duration};
    ///
    /// // Session without refresh token expiration
    /// let session = Session::new(
    ///     "session-id".to_string(),
    ///     ShopDomain::new("my-store").unwrap(),
    ///     "access-token".to_string(),
    ///     AuthScopes::new(),
    ///     false,
    ///     None,
    /// );
    /// assert!(!session.refresh_token_expired());
    /// ```
    #[must_use]
    pub fn refresh_token_expired(&self) -> bool {
        self.refresh_token_expires_at.is_some_and(|expires_at| {
            let buffer = Duration::seconds(REFRESH_TOKEN_EXPIRY_BUFFER_SECONDS);
            Utc::now() + buffer > expires_at
        })
    }
}

/// OAuth access token response from Shopify.
///
/// This struct represents the response from Shopify's OAuth token endpoint.
/// It is used with [`Session::from_access_token_response`] to create sessions.
///
/// Note: This struct is defined here temporarily and may be moved to an
/// OAuth module in a future release.
#[derive(Clone, Debug, Deserialize)]
pub struct AccessTokenResponse {
    /// The access token for API calls.
    pub access_token: String,

    /// Comma-separated list of granted scopes.
    pub scope: String,

    /// Number of seconds until the token expires (online tokens only).
    pub expires_in: Option<u32>,

    /// Comma-separated list of user-specific scopes (online tokens only).
    pub associated_user_scope: Option<String>,

    /// Associated user information (online tokens only).
    pub associated_user: Option<AssociatedUserResponse>,

    /// Shopify-provided session ID.
    #[serde(rename = "session")]
    pub session: Option<String>,

    /// The refresh token for obtaining new access tokens.
    ///
    /// Only present for expiring offline tokens.
    pub refresh_token: Option<String>,

    /// Number of seconds until the refresh token expires.
    ///
    /// Only present for expiring offline tokens.
    pub refresh_token_expires_in: Option<u32>,
}

/// User information from an OAuth access token response.
///
/// This struct matches the format of user data in Shopify's OAuth response.
#[derive(Clone, Debug, Deserialize)]
pub struct AssociatedUserResponse {
    /// The Shopify user ID.
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

    /// The user's locale preference.
    pub locale: String,

    /// Whether the user is a collaborator.
    pub collaborator: bool,
}

// Verify Session is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Session>();
};

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_shop() -> ShopDomain {
        ShopDomain::new("my-store").unwrap()
    }

    fn sample_scopes() -> AuthScopes {
        "read_products,write_orders".parse().unwrap()
    }

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

    // === Existing Session tests ===

    #[test]
    fn test_session_expired() {
        // Expired session
        let expired = Session::new(
            "id".to_string(),
            ShopDomain::new("shop").unwrap(),
            "token".to_string(),
            AuthScopes::new(),
            false,
            Some(Utc::now() - Duration::hours(1)),
        );
        assert!(expired.expired());

        // Not expired session
        let valid = Session::new(
            "id".to_string(),
            ShopDomain::new("shop").unwrap(),
            "token".to_string(),
            AuthScopes::new(),
            false,
            Some(Utc::now() + Duration::hours(1)),
        );
        assert!(!valid.expired());

        // No expiration
        let no_expiry = Session::new(
            "id".to_string(),
            ShopDomain::new("shop").unwrap(),
            "token".to_string(),
            AuthScopes::new(),
            false,
            None,
        );
        assert!(!no_expiry.expired());
    }

    #[test]
    fn test_session_is_active() {
        // Active session
        let active = Session::new(
            "id".to_string(),
            ShopDomain::new("shop").unwrap(),
            "token".to_string(),
            AuthScopes::new(),
            false,
            None,
        );
        assert!(active.is_active());

        // Inactive due to empty token
        let no_token = Session::new(
            "id".to_string(),
            ShopDomain::new("shop").unwrap(),
            String::new(),
            AuthScopes::new(),
            false,
            None,
        );
        assert!(!no_token.is_active());

        // Inactive due to expiration
        let expired = Session::new(
            "id".to_string(),
            ShopDomain::new("shop").unwrap(),
            "token".to_string(),
            AuthScopes::new(),
            false,
            Some(Utc::now() - Duration::hours(1)),
        );
        assert!(!expired.is_active());
    }

    #[test]
    fn test_session_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Session>();
    }

    // === Task Group 3: Extended Session tests ===

    #[test]
    fn test_session_with_associated_user_field() {
        let user = sample_user();
        let session = Session::with_user(
            "test-session".to_string(),
            sample_shop(),
            "access-token".to_string(),
            sample_scopes(),
            Some(Utc::now() + Duration::hours(1)),
            user.clone(),
            None,
        );

        assert!(session.associated_user.is_some());
        let stored_user = session.associated_user.unwrap();
        assert_eq!(stored_user.id, 12345);
        assert_eq!(stored_user.first_name, "Jane");
        assert_eq!(stored_user.email, "jane@example.com");
    }

    #[test]
    fn test_session_with_associated_user_scopes_field() {
        let user = sample_user();
        let user_scopes: AuthScopes = "read_products".parse().unwrap();

        let session = Session::with_user(
            "test-session".to_string(),
            sample_shop(),
            "access-token".to_string(),
            sample_scopes(),
            None,
            user,
            Some(user_scopes.clone()),
        );

        assert!(session.associated_user_scopes.is_some());
        let stored_scopes = session.associated_user_scopes.unwrap();
        assert!(stored_scopes.iter().any(|s| s == "read_products"));
    }

    #[test]
    fn test_session_serialization_to_json() {
        let session = Session::new(
            "offline_my-store.myshopify.com".to_string(),
            sample_shop(),
            "access-token".to_string(),
            sample_scopes(),
            false,
            None,
        );

        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("offline_my-store.myshopify.com"));
        assert!(json.contains("access-token"));
        assert!(json.contains("my-store.myshopify.com"));
    }

    #[test]
    fn test_session_deserialization_from_json() {
        let json = r#"{
            "id": "test-session",
            "shop": "test-shop.myshopify.com",
            "access_token": "token123",
            "scopes": "read_products",
            "is_online": false,
            "expires": null,
            "state": null,
            "shopify_session_id": null,
            "associated_user": null,
            "associated_user_scopes": null
        }"#;

        let session: Session = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, "test-session");
        assert_eq!(session.access_token, "token123");
        assert!(!session.is_online);
        assert!(session.associated_user.is_none());
        // Verify refresh token fields default to None
        assert!(session.refresh_token.is_none());
        assert!(session.refresh_token_expires_at.is_none());
    }

    #[test]
    fn test_session_equality_comparison() {
        let session1 = Session::new(
            "id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );

        let session2 = Session::new(
            "id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );

        assert_eq!(session1, session2);

        // Different ID should not be equal
        let session3 = Session::new(
            "different-id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );

        assert_ne!(session1, session3);
    }

    #[test]
    fn test_session_clone_preserves_all_fields() {
        let user = sample_user();
        let session = Session::with_user(
            "test-id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            Some(Utc::now() + Duration::hours(1)),
            user,
            Some("read_products".parse().unwrap()),
        );

        let cloned = session.clone();

        assert_eq!(session.id, cloned.id);
        assert_eq!(session.shop, cloned.shop);
        assert_eq!(session.access_token, cloned.access_token);
        assert_eq!(session.scopes, cloned.scopes);
        assert_eq!(session.is_online, cloned.is_online);
        assert_eq!(session.expires, cloned.expires);
        assert_eq!(session.associated_user, cloned.associated_user);
        assert_eq!(
            session.associated_user_scopes,
            cloned.associated_user_scopes
        );
    }

    // === Task Group 4: ID Generation and Factory Method tests ===

    #[test]
    fn test_generate_offline_id_produces_correct_format() {
        let shop = ShopDomain::new("my-store").unwrap();
        let id = Session::generate_offline_id(&shop);
        assert_eq!(id, "offline_my-store.myshopify.com");
    }

    #[test]
    fn test_generate_online_id_produces_correct_format() {
        let shop = ShopDomain::new("my-store").unwrap();
        let id = Session::generate_online_id(&shop, 12345);
        assert_eq!(id, "my-store.myshopify.com_12345");
    }

    #[test]
    fn test_from_access_token_response_with_offline_response() {
        let shop = ShopDomain::new("my-store").unwrap();
        let response = AccessTokenResponse {
            access_token: "offline-token".to_string(),
            scope: "read_products,write_orders".to_string(),
            expires_in: None,
            associated_user_scope: None,
            associated_user: None,
            session: None,
            refresh_token: None,
            refresh_token_expires_in: None,
        };

        let session = Session::from_access_token_response(shop, &response);

        assert!(!session.is_online);
        assert_eq!(session.id, "offline_my-store.myshopify.com");
        assert_eq!(session.access_token, "offline-token");
        assert!(session.associated_user.is_none());
        assert!(session.expires.is_none());
    }

    #[test]
    fn test_from_access_token_response_with_online_response() {
        let shop = ShopDomain::new("my-store").unwrap();
        let response = AccessTokenResponse {
            access_token: "online-token".to_string(),
            scope: "read_products".to_string(),
            expires_in: Some(3600),
            associated_user_scope: Some("read_products".to_string()),
            associated_user: Some(AssociatedUserResponse {
                id: 12345,
                first_name: "Jane".to_string(),
                last_name: "Doe".to_string(),
                email: "jane@example.com".to_string(),
                email_verified: true,
                account_owner: true,
                locale: "en".to_string(),
                collaborator: false,
            }),
            session: Some("shopify-session-id".to_string()),
            refresh_token: None,
            refresh_token_expires_in: None,
        };

        let session = Session::from_access_token_response(shop, &response);

        assert!(session.is_online);
        assert_eq!(session.id, "my-store.myshopify.com_12345");
        assert_eq!(session.access_token, "online-token");
        assert!(session.associated_user.is_some());
        assert!(session.expires.is_some());
        assert_eq!(
            session.shopify_session_id,
            Some("shopify-session-id".to_string())
        );

        let user = session.associated_user.unwrap();
        assert_eq!(user.id, 12345);
        assert_eq!(user.email, "jane@example.com");
    }

    #[test]
    fn test_from_access_token_response_calculates_expires() {
        let shop = ShopDomain::new("my-store").unwrap();
        let response = AccessTokenResponse {
            access_token: "token".to_string(),
            scope: "read_products".to_string(),
            expires_in: Some(3600), // 1 hour
            associated_user_scope: None,
            associated_user: Some(AssociatedUserResponse {
                id: 1,
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                email: "test@example.com".to_string(),
                email_verified: true,
                account_owner: false,
                locale: "en".to_string(),
                collaborator: false,
            }),
            session: None,
            refresh_token: None,
            refresh_token_expires_in: None,
        };

        let before = Utc::now();
        let session = Session::from_access_token_response(shop, &response);
        let after = Utc::now();

        assert!(session.expires.is_some());
        let expires = session.expires.unwrap();

        // Expires should be roughly 1 hour from now
        let expected_min = before + Duration::seconds(3600);
        let expected_max = after + Duration::seconds(3600);

        assert!(expires >= expected_min && expires <= expected_max);
    }

    #[test]
    fn test_from_access_token_response_parses_scopes() {
        let shop = ShopDomain::new("my-store").unwrap();
        let response = AccessTokenResponse {
            access_token: "token".to_string(),
            scope: "read_products,write_orders".to_string(),
            expires_in: None,
            associated_user_scope: None,
            associated_user: None,
            session: None,
            refresh_token: None,
            refresh_token_expires_in: None,
        };

        let session = Session::from_access_token_response(shop, &response);

        assert!(session.scopes.iter().any(|s| s == "read_products"));
        assert!(session.scopes.iter().any(|s| s == "write_orders"));
        // write_orders implies read_orders
        assert!(session.scopes.iter().any(|s| s == "read_orders"));
    }

    #[test]
    fn test_from_access_token_response_sets_is_online_correctly() {
        let shop = ShopDomain::new("my-store").unwrap();

        // Offline response
        let offline_response = AccessTokenResponse {
            access_token: "token".to_string(),
            scope: "read_products".to_string(),
            expires_in: None,
            associated_user_scope: None,
            associated_user: None,
            session: None,
            refresh_token: None,
            refresh_token_expires_in: None,
        };
        let offline_session = Session::from_access_token_response(shop.clone(), &offline_response);
        assert!(!offline_session.is_online);

        // Online response
        let online_response = AccessTokenResponse {
            access_token: "token".to_string(),
            scope: "read_products".to_string(),
            expires_in: Some(3600),
            associated_user_scope: None,
            associated_user: Some(AssociatedUserResponse {
                id: 1,
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                email: "test@example.com".to_string(),
                email_verified: true,
                account_owner: false,
                locale: "en".to_string(),
                collaborator: false,
            }),
            session: None,
            refresh_token: None,
            refresh_token_expires_in: None,
        };
        let online_session = Session::from_access_token_response(shop, &online_response);
        assert!(online_session.is_online);
    }

    // === Refresh token tests ===

    #[test]
    fn test_session_serialization_includes_refresh_token_field() {
        let mut session = Session::new(
            "offline_my-store.myshopify.com".to_string(),
            sample_shop(),
            "access-token".to_string(),
            sample_scopes(),
            false,
            None,
        );
        session.refresh_token = Some("refresh-token-123".to_string());

        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("refresh_token"));
        assert!(json.contains("refresh-token-123"));
    }

    #[test]
    fn test_session_serialization_includes_refresh_token_expires_at_field() {
        let mut session = Session::new(
            "offline_my-store.myshopify.com".to_string(),
            sample_shop(),
            "access-token".to_string(),
            sample_scopes(),
            false,
            None,
        );
        session.refresh_token_expires_at = Some(Utc::now() + Duration::days(30));

        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("refresh_token_expires_at"));
    }

    #[test]
    fn test_session_deserialization_handles_missing_refresh_token_fields_backward_compat() {
        // Old format without refresh token fields
        let json = r#"{
            "id": "test-session",
            "shop": "test-shop.myshopify.com",
            "access_token": "token123",
            "scopes": "read_products",
            "is_online": false,
            "expires": null,
            "state": null,
            "shopify_session_id": null,
            "associated_user": null,
            "associated_user_scopes": null
        }"#;

        let session: Session = serde_json::from_str(json).unwrap();
        assert!(session.refresh_token.is_none());
        assert!(session.refresh_token_expires_at.is_none());
    }

    #[test]
    fn test_refresh_token_expired_returns_false_when_expires_at_is_none() {
        let session = Session::new(
            "id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );
        assert!(!session.refresh_token_expired());
    }

    #[test]
    fn test_refresh_token_expired_returns_false_when_expires_at_is_in_future_more_than_60s() {
        let mut session = Session::new(
            "id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );
        // Set refresh token expires at 2 hours from now (well past 60 second buffer)
        session.refresh_token_expires_at = Some(Utc::now() + Duration::hours(2));

        assert!(!session.refresh_token_expired());
    }

    #[test]
    fn test_refresh_token_expired_returns_true_when_expires_at_is_within_60_seconds() {
        let mut session = Session::new(
            "id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );
        // Set refresh token expires at 30 seconds from now (within 60 second buffer)
        session.refresh_token_expires_at = Some(Utc::now() + Duration::seconds(30));

        assert!(session.refresh_token_expired());
    }

    #[test]
    fn test_refresh_token_expired_returns_true_when_already_expired() {
        let mut session = Session::new(
            "id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );
        // Set refresh token expires at 1 hour ago
        session.refresh_token_expires_at = Some(Utc::now() - Duration::hours(1));

        assert!(session.refresh_token_expired());
    }

    #[test]
    fn test_from_access_token_response_populates_refresh_token_fields() {
        let shop = ShopDomain::new("my-store").unwrap();
        let response = AccessTokenResponse {
            access_token: "access-token".to_string(),
            scope: "read_products".to_string(),
            expires_in: Some(86400), // 24 hours
            associated_user_scope: None,
            associated_user: None,
            session: None,
            refresh_token: Some("refresh-token-xyz".to_string()),
            refresh_token_expires_in: Some(2592000), // 30 days
        };

        let before = Utc::now();
        let session = Session::from_access_token_response(shop, &response);
        let after = Utc::now();

        assert_eq!(session.refresh_token, Some("refresh-token-xyz".to_string()));
        assert!(session.refresh_token_expires_at.is_some());

        let expires_at = session.refresh_token_expires_at.unwrap();
        let expected_min = before + Duration::seconds(2592000);
        let expected_max = after + Duration::seconds(2592000);

        assert!(expires_at >= expected_min && expires_at <= expected_max);
    }

    #[test]
    fn test_access_token_response_deserializes_refresh_token_field() {
        let json = r#"{
            "access_token": "test-token",
            "scope": "read_products",
            "refresh_token": "refresh-abc"
        }"#;

        let response: AccessTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.refresh_token, Some("refresh-abc".to_string()));
    }

    #[test]
    fn test_access_token_response_deserializes_refresh_token_expires_in_field() {
        let json = r#"{
            "access_token": "test-token",
            "scope": "read_products",
            "refresh_token_expires_in": 2592000
        }"#;

        let response: AccessTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.refresh_token_expires_in, Some(2592000));
    }

    #[test]
    fn test_access_token_response_handles_missing_optional_refresh_token_fields() {
        let json = r#"{
            "access_token": "test-token",
            "scope": "read_products"
        }"#;

        let response: AccessTokenResponse = serde_json::from_str(json).unwrap();
        assert!(response.refresh_token.is_none());
        assert!(response.refresh_token_expires_in.is_none());
    }

    #[test]
    fn test_refresh_token_expired_at_boundary_61_seconds_is_false() {
        // Use 61 seconds to avoid timing issues (1 second buffer)
        let mut session = Session::new(
            "id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );
        // Set refresh token expires at 61 seconds from now (just past buffer)
        session.refresh_token_expires_at = Some(Utc::now() + Duration::seconds(61));

        // Should NOT be expired (61 > 60)
        assert!(!session.refresh_token_expired());
    }

    #[test]
    fn test_refresh_token_expired_at_58_seconds_is_true() {
        // Use 58 seconds to avoid timing issues (within buffer)
        let mut session = Session::new(
            "id".to_string(),
            sample_shop(),
            "token".to_string(),
            sample_scopes(),
            false,
            None,
        );
        // Set refresh token expires at 58 seconds from now (within buffer)
        session.refresh_token_expires_at = Some(Utc::now() + Duration::seconds(58));

        // Should be expired (58 < 60)
        assert!(session.refresh_token_expired());
    }

    #[test]
    fn test_session_roundtrip_serialization_with_refresh_token() {
        let mut original = Session::new(
            "offline_test-shop.myshopify.com".to_string(),
            sample_shop(),
            "access-token-123".to_string(),
            sample_scopes(),
            false,
            None,
        );
        original.refresh_token = Some("refresh-token-xyz".to_string());
        original.refresh_token_expires_at = Some(Utc::now() + Duration::days(30));

        let json = serde_json::to_string(&original).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(original.refresh_token, restored.refresh_token);
        assert_eq!(
            original.refresh_token_expires_at,
            restored.refresh_token_expires_at
        );
    }
}

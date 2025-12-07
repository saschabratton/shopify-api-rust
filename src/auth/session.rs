//! Session management for Shopify API authentication.
//!
//! This module provides the [`Session`] type for representing authenticated
//! sessions used in API calls.

use crate::auth::AuthScopes;
use crate::config::ShopDomain;
use chrono::{DateTime, Utc};

/// Represents an authenticated session for Shopify API calls.
///
/// Sessions hold the authentication state needed to make API requests on behalf
/// of a shop. They can be either online (user-specific) or offline (app-level).
///
/// # Thread Safety
///
/// `Session` is `Send + Sync`, making it safe to share across threads.
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
#[derive(Clone, Debug)]
pub struct Session {
    /// Unique identifier for this session.
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
    pub expires: Option<DateTime<Utc>>,

    /// OAuth state parameter, if applicable.
    pub state: Option<String>,

    /// Shopify-provided session ID, if applicable.
    pub shopify_session_id: Option<String>,
}

impl Session {
    /// Creates a new session with the specified parameters.
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
}

// Verify Session is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Session>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

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
            "".to_string(),
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
}

//! OAuth scope handling for Shopify API.
//!
//! This module provides the [`AuthScopes`] type for managing OAuth scopes,
//! including parsing and implied scope handling.

use crate::error::ConfigError;
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

/// A set of OAuth scopes for Shopify API access.
///
/// This type handles parsing, deduplication, and implied scope logic.
/// For example, `write_products` implies `read_products`.
///
/// # Implied Scopes
///
/// Shopify's scope system includes implied scopes:
/// - `write_products` implies `read_products`
/// - `unauthenticated_write_products` implies `unauthenticated_read_products`
///
/// This type automatically expands implied scopes when parsing.
///
/// # Example
///
/// ```rust
/// use shopify_api::AuthScopes;
///
/// let scopes: AuthScopes = "read_products, write_orders".parse().unwrap();
/// assert!(!scopes.is_empty());
///
/// // Check if scopes cover another set
/// let required: AuthScopes = "read_products".parse().unwrap();
/// assert!(scopes.covers(&required));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AuthScopes {
    scopes: HashSet<String>,
}

impl AuthScopes {
    /// Creates an empty scope set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the scope set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    /// Returns `true` if this scope set covers all scopes in `other`.
    ///
    /// A scope set "covers" another if it contains all the scopes
    /// from the other set (considering implied scopes).
    #[must_use]
    pub fn covers(&self, other: &Self) -> bool {
        other.scopes.iter().all(|s| self.scopes.contains(s))
    }

    /// Returns an iterator over the scopes.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.scopes.iter().map(String::as_str)
    }

    /// Adds implied scopes for write permissions.
    ///
    /// - `write_foo` implies `read_foo`
    /// - `unauthenticated_write_foo` implies `unauthenticated_read_foo`
    fn add_implied_scopes(&mut self) {
        let implied: Vec<String> = self
            .scopes
            .iter()
            .filter_map(|scope| Self::get_implied_scope(scope))
            .collect();

        for scope in implied {
            self.scopes.insert(scope);
        }
    }

    fn get_implied_scope(scope: &str) -> Option<String> {
        scope
            .strip_prefix("unauthenticated_write_")
            .map(|rest| format!("unauthenticated_read_{rest}"))
            .or_else(|| {
                scope
                    .strip_prefix("write_")
                    .map(|rest| format!("read_{rest}"))
            })
    }
}

impl FromStr for AuthScopes {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut scopes = HashSet::new();

        for scope in s.split(',') {
            let scope = scope.trim();
            if scope.is_empty() {
                continue;
            }

            // Validate scope format (alphanumeric and underscores)
            if !scope.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(ConfigError::InvalidScopes {
                    reason: format!("Invalid characters in scope: '{scope}'"),
                });
            }

            scopes.insert(scope.to_string());
        }

        let mut auth_scopes = Self { scopes };
        auth_scopes.add_implied_scopes();

        Ok(auth_scopes)
    }
}

impl From<Vec<String>> for AuthScopes {
    fn from(scopes: Vec<String>) -> Self {
        let scopes: HashSet<String> = scopes
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut auth_scopes = Self { scopes };
        auth_scopes.add_implied_scopes();

        auth_scopes
    }
}

impl fmt::Display for AuthScopes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut scopes: Vec<&str> = self.scopes.iter().map(String::as_str).collect();
        scopes.sort_unstable();
        write!(f, "{}", scopes.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_scopes_parses_comma_separated() {
        let scopes: AuthScopes = "read_products, write_orders".parse().unwrap();
        assert!(scopes.iter().any(|s| s == "read_products"));
        assert!(scopes.iter().any(|s| s == "write_orders"));
    }

    #[test]
    fn test_auth_scopes_handles_implied_scopes() {
        let scopes: AuthScopes = "write_products".parse().unwrap();

        // write_products implies read_products
        assert!(scopes.iter().any(|s| s == "write_products"));
        assert!(scopes.iter().any(|s| s == "read_products"));
    }

    #[test]
    fn test_auth_scopes_handles_unauthenticated_implied() {
        let scopes: AuthScopes = "unauthenticated_write_products".parse().unwrap();

        // unauthenticated_write_products implies unauthenticated_read_products
        assert!(scopes.iter().any(|s| s == "unauthenticated_write_products"));
        assert!(scopes.iter().any(|s| s == "unauthenticated_read_products"));
    }

    #[test]
    fn test_auth_scopes_covers() {
        let scopes: AuthScopes = "read_products, write_orders".parse().unwrap();
        let required: AuthScopes = "read_products".parse().unwrap();

        assert!(scopes.covers(&required));

        let more_required: AuthScopes = "read_products, read_customers".parse().unwrap();
        assert!(!scopes.covers(&more_required));
    }

    #[test]
    fn test_auth_scopes_is_empty() {
        let empty = AuthScopes::new();
        assert!(empty.is_empty());

        let scopes: AuthScopes = "read_products".parse().unwrap();
        assert!(!scopes.is_empty());
    }

    #[test]
    fn test_auth_scopes_from_vec() {
        let scopes = AuthScopes::from(vec![
            "read_products".to_string(),
            "write_orders".to_string(),
        ]);
        assert!(scopes.iter().any(|s| s == "read_products"));
        assert!(scopes.iter().any(|s| s == "write_orders"));
        // write_orders implies read_orders
        assert!(scopes.iter().any(|s| s == "read_orders"));
    }
}

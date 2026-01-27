//! Metafield owner types for polymorphic metafield paths.
//!
//! This module provides the `MetafieldOwner` enum used to specify
//! which resource type owns a metafield.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the owner type of a metafield.
///
/// Metafields can be attached to various resource types in Shopify.
/// This enum is used to generate the correct API path for metafield operations.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::rest::resources::v2025_10::common::MetafieldOwner;
///
/// let owner = MetafieldOwner::Product;
/// assert_eq!(owner.to_path_segment(), "products");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetafieldOwner {
    /// Metafield belongs to a product.
    Product,
    /// Metafield belongs to a product variant.
    Variant,
    /// Metafield belongs to a customer.
    Customer,
    /// Metafield belongs to an order.
    Order,
    /// Metafield belongs to the shop (global metafields).
    Shop,
    /// Metafield belongs to a collection (custom or smart).
    Collection,
    /// Metafield belongs to a page.
    Page,
    /// Metafield belongs to a blog.
    Blog,
    /// Metafield belongs to an article.
    Article,
}

impl MetafieldOwner {
    /// Returns the path segment for this owner type.
    ///
    /// This is used to construct the API path for metafield operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::rest::resources::v2025_10::common::MetafieldOwner;
    ///
    /// assert_eq!(MetafieldOwner::Product.to_path_segment(), "products");
    /// assert_eq!(MetafieldOwner::Customer.to_path_segment(), "customers");
    /// assert_eq!(MetafieldOwner::Shop.to_path_segment(), "");
    /// ```
    #[must_use]
    pub const fn to_path_segment(&self) -> &'static str {
        match self {
            Self::Product => "products",
            Self::Variant => "variants",
            Self::Customer => "customers",
            Self::Order => "orders",
            Self::Shop => "", // Shop metafields have no parent path segment
            Self::Collection => "collections",
            Self::Page => "pages",
            Self::Blog => "blogs",
            Self::Article => "articles",
        }
    }
}

impl fmt::Display for MetafieldOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Product => "product",
            Self::Variant => "variant",
            Self::Customer => "customer",
            Self::Order => "order",
            Self::Shop => "shop",
            Self::Collection => "collection",
            Self::Page => "page",
            Self::Blog => "blog",
            Self::Article => "article",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metafield_owner_serialization() {
        let owner = MetafieldOwner::Product;
        let json = serde_json::to_string(&owner).unwrap();
        assert_eq!(json, "\"product\"");

        let owner = MetafieldOwner::Customer;
        let json = serde_json::to_string(&owner).unwrap();
        assert_eq!(json, "\"customer\"");

        let owner = MetafieldOwner::Shop;
        let json = serde_json::to_string(&owner).unwrap();
        assert_eq!(json, "\"shop\"");
    }

    #[test]
    fn test_metafield_owner_deserialization() {
        let owner: MetafieldOwner = serde_json::from_str("\"product\"").unwrap();
        assert_eq!(owner, MetafieldOwner::Product);

        let owner: MetafieldOwner = serde_json::from_str("\"variant\"").unwrap();
        assert_eq!(owner, MetafieldOwner::Variant);

        let owner: MetafieldOwner = serde_json::from_str("\"order\"").unwrap();
        assert_eq!(owner, MetafieldOwner::Order);
    }

    #[test]
    fn test_metafield_owner_to_path_segment() {
        assert_eq!(MetafieldOwner::Product.to_path_segment(), "products");
        assert_eq!(MetafieldOwner::Variant.to_path_segment(), "variants");
        assert_eq!(MetafieldOwner::Customer.to_path_segment(), "customers");
        assert_eq!(MetafieldOwner::Order.to_path_segment(), "orders");
        assert_eq!(MetafieldOwner::Shop.to_path_segment(), "");
        assert_eq!(MetafieldOwner::Collection.to_path_segment(), "collections");
        assert_eq!(MetafieldOwner::Page.to_path_segment(), "pages");
        assert_eq!(MetafieldOwner::Blog.to_path_segment(), "blogs");
        assert_eq!(MetafieldOwner::Article.to_path_segment(), "articles");
    }

    #[test]
    fn test_metafield_owner_display() {
        assert_eq!(format!("{}", MetafieldOwner::Product), "product");
        assert_eq!(format!("{}", MetafieldOwner::Customer), "customer");
        assert_eq!(format!("{}", MetafieldOwner::Shop), "shop");
        assert_eq!(format!("{}", MetafieldOwner::Collection), "collection");
    }
}

//! Common types and embedded structs used across multiple REST resources.
//!
//! This module provides shared types that are embedded within other resources,
//! such as addresses, line items, tax lines, and discount applications.
//!
//! These types are not full REST resources themselves (they don't implement
//! `RestResource`), but are used as nested data within resources like `Order`,
//! `Product`, and `Customer`.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::resources::v2025_10::common::{Address, LineItem, TaxLine};
//!
//! // Address is embedded in orders and customers
//! let address = Address {
//!     first_name: Some("John".to_string()),
//!     last_name: Some("Doe".to_string()),
//!     address1: Some("123 Main St".to_string()),
//!     city: Some("New York".to_string()),
//!     province: Some("New York".to_string()),
//!     country: Some("United States".to_string()),
//!     zip: Some("10001".to_string()),
//!     ..Default::default()
//! };
//! ```

mod address;
mod billing;
mod blog;
mod collection;
mod line_item;
mod metafield;
mod money;
mod product;
mod theme;
mod webhook;

pub use address::{Address, CustomerAddress};
pub use billing::{ChargeCurrency, ChargeStatus};
pub use blog::BlogCommentable;
pub use collection::{CollectionImage, SmartCollectionRule};
pub use line_item::{
    DiscountAllocation, DiscountApplication, LineItem, LineItemProperty, NoteAttribute,
    ShippingLine, TaxLine,
};
pub use metafield::MetafieldOwner;
pub use money::{Money, MoneySet};
pub use product::{ProductImage, ProductOption};
pub use theme::ThemeRole;
pub use webhook::{WebhookFormat, WebhookTopic};

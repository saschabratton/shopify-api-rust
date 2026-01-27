//! Version-specific REST resource implementations.
//!
//! This module contains REST resource implementations organized by API version.
//! Each version module contains the resource structs for that API version.
//!
//! # Version Structure
//!
//! Resources are organized by API version to allow for version-specific
//! differences in resource structure or behavior:
//!
//! ```text
//! resources/
//!   mod.rs           <- This file (re-exports latest version)
//!   v2025_10/
//!     mod.rs         <- Version-specific resources
//! ```
//!
//! # Using Resources
//!
//! The latest stable version is re-exported at this module level for convenience:
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::Product;  // Uses latest version
//!
//! // Or explicitly specify a version:
//! use shopify_sdk::rest::resources::v2025_10::Product;
//! ```
//!
//! # Available Resources
//!
//! ## Product Resource
//!
//! Products are the goods or services that merchants sell.
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::{Product, ProductListParams, ProductStatus};
//! use shopify_sdk::rest::RestResource;
//!
//! // Find a single product
//! let product = Product::find(&client, 123, None).await?;
//!
//! // List active products
//! let params = ProductListParams {
//!     status: Some(ProductStatus::Active),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let products = Product::all(&client, Some(params)).await?;
//! ```
//!
//! ## Variant Resource
//!
//! Variants represent different versions of a product (size, color, etc).
//! Supports dual path patterns for nested and standalone access.
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::{Variant, VariantListParams};
//! use shopify_sdk::rest::RestResource;
//!
//! // Find a variant by ID (standalone path)
//! let variant = Variant::find(&client, 456, None).await?;
//!
//! // List variants under a product (nested path)
//! let variants = Variant::all_with_parent(&client, "product_id", 123, None).await?;
//! ```
//!
//! ## Customer Resource
//!
//! Customers represent people who have created accounts with the store.
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::{Customer, CustomerListParams};
//! use shopify_sdk::rest::RestResource;
//!
//! // Find a customer
//! let customer = Customer::find(&client, 789, None).await?;
//! ```
//!
//! ## Order Resource
//!
//! Orders represent completed checkout transactions.
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::{Order, OrderListParams, FinancialStatus};
//! use shopify_sdk::rest::RestResource;
//!
//! // Find an order
//! let order = Order::find(&client, 123, None).await?;
//!
//! // List paid orders
//! let params = OrderListParams {
//!     financial_status: Some(FinancialStatus::Paid),
//!     ..Default::default()
//! };
//! let orders = Order::all(&client, Some(params)).await?;
//!
//! // Cancel an order
//! let cancelled = order.cancel(&client).await?;
//! ```
//!
//! ## Fulfillment Resource
//!
//! Fulfillments represent shipments of order line items.
//! Nested under orders: `/orders/{order_id}/fulfillments/{id}`
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::{Fulfillment, TrackingInfo};
//! use shopify_sdk::rest::RestResource;
//!
//! // List fulfillments for an order
//! let fulfillments = Fulfillment::all_with_parent(&client, "order_id", 123, None).await?;
//!
//! // Update tracking
//! let tracking = TrackingInfo {
//!     tracking_number: Some("1Z999AA10123456784".to_string()),
//!     tracking_company: Some("UPS".to_string()),
//!     ..Default::default()
//! };
//! let updated = fulfillment.update_tracking(&client, tracking).await?;
//! ```
//!
//! ## `InventoryItem` Resource
//!
//! Inventory items are linked to product variants via `inventory_item_id`.
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::{InventoryItem, InventoryItemListParams};
//! use shopify_sdk::rest::RestResource;
//!
//! // List inventory items by IDs (required parameter)
//! let params = InventoryItemListParams {
//!     ids: Some(vec![808950810, 808950811]),
//!     ..Default::default()
//! };
//! let items = InventoryItem::all(&client, Some(params)).await?;
//! ```
//!
//! # Version Support
//!
//! Currently supported API versions:
//! - `v2025_10` (2025-10) - Latest stable
//!
//! Future versions will be added as needed without breaking existing code.
//!
//! ## API Version Lifecycle
//!
//! Shopify API versions follow a quarterly release schedule and have an
//! approximately 12-month support window:
//!
//! - **Supported**: Versions within the support window receive full support
//! - **Deprecated**: Older versions may stop working at any time
//!
//! Use [`ApiVersion::is_supported()`](crate::ApiVersion::is_supported) and
//! [`ApiVersion::is_deprecated()`](crate::ApiVersion::is_deprecated) to check
//! version status at runtime.
//!
//! ## Multi-Version Resources
//!
//! Resources are organized by API version to support version-specific differences.
//! The latest stable version is re-exported at the module root for convenience:
//!
//! ```rust,ignore
//! // Recommended: Use the default (latest) version
//! use shopify_api::rest::resources::Product;
//!
//! // Explicit version selection (for version-specific behavior)
//! use shopify_api::rest::resources::v2025_10::Product;
//! ```
//!
//! When Shopify introduces breaking changes in a new API version, a new
//! version-specific module will be added without breaking existing code.

pub mod v2025_10;

// Re-export types from the latest version for convenience
pub use v2025_10::*;

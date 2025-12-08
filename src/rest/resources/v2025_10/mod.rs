//! REST resources for API version 2025-10.
//!
//! This module contains resource implementations for the 2025-10 API version.
//!
//! # Available Resources
//!
//! ## Product Resource
//!
//! - [`Product`] - A product in a Shopify store
//! - [`ProductStatus`] - The status of a product (active, archived, draft)
//! - [`ProductVariant`] - A variant embedded within a Product response
//! - [`ProductListParams`] - Parameters for listing products
//! - [`ProductFindParams`] - Parameters for finding a single product
//! - [`ProductCountParams`] - Parameters for counting products
//!
//! ## Variant Resource
//!
//! - [`Variant`] - A product variant with full CRUD operations
//! - [`WeightUnit`] - Unit of measurement for variant weight (kg, g, lb, oz)
//! - [`VariantListParams`] - Parameters for listing variants
//! - [`VariantFindParams`] - Parameters for finding a single variant
//! - [`VariantCountParams`] - Parameters for counting variants
//!
//! The Variant resource supports dual path patterns:
//! - Nested: `/products/{product_id}/variants/{id}` (preferred when `product_id` available)
//! - Standalone: `/variants/{id}` (fallback)
//!
//! ## Customer Resource
//!
//! - [`Customer`] - A customer in a Shopify store
//! - [`CustomerState`] - The state of a customer account (disabled, invited, enabled, declined)
//! - [`CustomerListParams`] - Parameters for listing customers
//! - [`CustomerFindParams`] - Parameters for finding a single customer
//! - [`CustomerCountParams`] - Parameters for counting customers
//! - [`EmailMarketingConsent`] - Email marketing consent information
//! - [`SmsMarketingConsent`] - SMS marketing consent information
//!
//! ## Order Resource
//!
//! - [`Order`] - An order in a Shopify store
//! - [`FinancialStatus`] - The financial status of an order (pending, paid, refunded, etc.)
//! - [`FulfillmentStatus`] - The fulfillment status of an order
//! - [`CancelReason`] - The reason for canceling an order
//! - [`OrderListParams`] - Parameters for listing orders
//! - [`OrderFindParams`] - Parameters for finding a single order
//! - [`OrderCountParams`] - Parameters for counting orders
//! - [`DiscountCode`] - A discount code applied to an order
//! - [`Refund`] - A refund associated with an order
//! - [`OrderFulfillment`] - Fulfillment data embedded in order responses
//!
//! The Order resource also provides resource-specific operations:
//! - `Order::cancel()` - Cancel an order
//! - `Order::close()` - Close an order
//! - `Order::open()` - Re-open a closed order
//!
//! ## Fulfillment Resource
//!
//! - [`Fulfillment`] - A fulfillment for shipping order items
//! - [`FulfillmentStatus`](fulfillment::FulfillmentStatus) - The status of a fulfillment
//! - [`ShipmentStatus`] - The shipment/delivery status
//! - [`FulfillmentListParams`] - Parameters for listing fulfillments
//! - [`FulfillmentFindParams`] - Parameters for finding a single fulfillment
//! - [`FulfillmentCountParams`] - Parameters for counting fulfillments
//! - [`FulfillmentLineItem`] - A line item in a fulfillment
//! - [`TrackingInfo`] - Tracking information for `update_tracking` operation
//!
//! Fulfillments are nested under orders: `/orders/{order_id}/fulfillments/{id}`
//!
//! The Fulfillment resource provides resource-specific operations:
//! - `Fulfillment::cancel()` - Cancel a fulfillment
//! - `Fulfillment::update_tracking()` - Update tracking information
//!
//! ## `InventoryItem` Resource
//!
//! - [`InventoryItem`] - An inventory item linked to a product variant
//! - [`InventoryItemListParams`] - Parameters for listing inventory items
//! - [`InventoryItemFindParams`] - Parameters for finding a single inventory item
//! - [`CountryHarmonizedSystemCode`] - Country-specific customs code
//!
//! `InventoryItem` uses standalone paths only (`/inventory_items/{id}`).
//! The list operation requires the `ids` parameter (comma-separated).
//! `InventoryItem` is linked to `Variant` via `Variant.inventory_item_id`.
//!
//! ## Common Types (Embedded Structs)
//!
//! The `common` module provides shared types used across multiple resources:
//!
//! - [`common::Address`] - Physical address for billing/shipping
//! - [`common::CustomerAddress`] - Customer address with ID and default flag
//! - [`common::LineItem`] - Order line item with product/variant info
//! - [`common::TaxLine`] - Tax information on orders/line items
//! - [`common::DiscountApplication`] - Discount details on orders
//! - [`common::ProductImage`] - Product image with dimensions
//! - [`common::ProductOption`] - Product option (e.g., Size, Color)
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{Product, ProductListParams, ProductStatus};
//! use shopify_api::rest::resources::v2025_10::{Variant, VariantListParams, WeightUnit};
//! use shopify_api::rest::resources::v2025_10::{Customer, CustomerListParams, CustomerState};
//! use shopify_api::rest::resources::v2025_10::{Order, OrderListParams, FinancialStatus};
//! use shopify_api::rest::resources::v2025_10::{Fulfillment, FulfillmentListParams, TrackingInfo};
//! use shopify_api::rest::resources::v2025_10::{InventoryItem, InventoryItemListParams};
//!
//! // Find a single product
//! let product = Product::find(&client, 123, None).await?;
//! println!("Product: {}", product.title.as_deref().unwrap_or(""));
//!
//! // List products with filters
//! let params = ProductListParams {
//!     status: Some(ProductStatus::Active),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let products = Product::all(&client, Some(params)).await?;
//!
//! // Find a variant by ID (standalone path)
//! let variant = Variant::find(&client, 456, None).await?;
//!
//! // List variants under a product (nested path)
//! let variants = Variant::all_with_parent(&client, "product_id", 123, None).await?;
//!
//! // Find a customer
//! let customer = Customer::find(&client, 789, None).await?;
//! println!("Customer: {} {}",
//!     customer.first_name.as_deref().unwrap_or(""),
//!     customer.last_name.as_deref().unwrap_or("")
//! );
//!
//! // Find an order
//! let order = Order::find(&client, 456, None).await?;
//! println!("Order: {}", order.name.as_deref().unwrap_or(""));
//!
//! // List paid orders
//! let params = OrderListParams {
//!     financial_status: Some(FinancialStatus::Paid),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let orders = Order::all(&client, Some(params)).await?;
//!
//! // Cancel an order
//! let cancelled = order.cancel(&client).await?;
//!
//! // List fulfillments for an order
//! let fulfillments = Fulfillment::all_with_parent(&client, "order_id", 123, None).await?;
//!
//! // Update tracking on a fulfillment
//! let tracking = TrackingInfo {
//!     tracking_number: Some("1Z999AA10123456784".to_string()),
//!     tracking_company: Some("UPS".to_string()),
//!     ..Default::default()
//! };
//! let updated = fulfillment.update_tracking(&client, tracking).await?;
//!
//! // Find an inventory item (linked to variant via inventory_item_id)
//! let inventory_item = InventoryItem::find(&client, 808950810, None).await?;
//! println!("SKU: {}", inventory_item.sku.as_deref().unwrap_or(""));
//!
//! // List inventory items by IDs (ids parameter required)
//! let params = InventoryItemListParams {
//!     ids: Some(vec![808950810, 808950811]),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let inventory_items = InventoryItem::all(&client, Some(params)).await?;
//!
//! // Create a new product
//! let mut product = Product {
//!     title: Some("My New Product".to_string()),
//!     vendor: Some("My Store".to_string()),
//!     product_type: Some("T-Shirts".to_string()),
//!     ..Default::default()
//! };
//! let saved = product.save(&client).await?;
//! ```

pub mod common;
mod customer;
mod fulfillment;
mod inventory_item;
mod order;
mod product;
mod variant;

// Re-export common types for convenience
pub use common::{
    Address, CustomerAddress, DiscountAllocation, DiscountApplication, LineItem, LineItemProperty,
    Money, MoneySet, NoteAttribute, ProductImage, ProductOption, ShippingLine, TaxLine,
};

// Re-export Product resource types
pub use product::{
    Product, ProductCountParams, ProductFindParams, ProductListParams, ProductStatus,
    ProductVariant,
};

// Re-export Variant resource types
pub use variant::{Variant, VariantCountParams, VariantFindParams, VariantListParams, WeightUnit};

// Re-export Customer resource types
pub use customer::{
    Customer, CustomerCountParams, CustomerFindParams, CustomerListParams, CustomerState,
    EmailMarketingConsent, SmsMarketingConsent,
};

// Re-export Order resource types
pub use order::{
    CancelReason, DiscountCode, FinancialStatus, FulfillmentStatus, Order, OrderCountParams,
    OrderFindParams, OrderFulfillment, OrderListParams, Refund,
};

// Re-export Fulfillment resource types
pub use fulfillment::{
    Fulfillment, FulfillmentCountParams, FulfillmentFindParams, FulfillmentLineItem,
    FulfillmentListParams, FulfillmentStatus as FulfillmentResourceStatus, ShipmentStatus,
    TrackingInfo,
};

// Re-export InventoryItem resource types
pub use inventory_item::{
    CountryHarmonizedSystemCode, InventoryItem, InventoryItemFindParams, InventoryItemListParams,
};

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
//! - [`Refund`] - A refund associated with an order (embedded)
//! - [`OrderFulfillment`] - Fulfillment data embedded in order responses
//!
//! The Order resource also provides resource-specific operations:
//! - `Order::cancel()` - Cancel an order
//! - `Order::close()` - Close an order
//! - `Order::open()` - Re-open a closed order
//!
//! ## DraftOrder Resource (B2B/Wholesale)
//!
//! - [`DraftOrder`] - A draft order for B2B/wholesale workflows
//! - [`DraftOrderStatus`] - The status of a draft order (Open, InvoiceSent, Completed)
//! - [`DraftOrderLineItem`] - A line item in a draft order
//! - [`AppliedDiscount`] - A discount applied to a draft order or line item
//! - [`DraftOrderInvoice`] - Invoice details for sending to customers
//! - [`DraftOrderCompleteParams`] - Parameters for completing a draft order
//! - [`DraftOrderListParams`] - Parameters for listing draft orders
//! - [`DraftOrderFindParams`] - Parameters for finding a single draft order
//! - [`DraftOrderCountParams`] - Parameters for counting draft orders
//!
//! The DraftOrder resource provides resource-specific operations:
//! - `DraftOrder::complete()` - Convert draft to actual order (PUT method)
//! - `DraftOrder::send_invoice()` - Send invoice email to customer
//!
//! ## FulfillmentOrder Resource (Modern Fulfillment Workflows)
//!
//! - [`FulfillmentOrder`] - A fulfillment order (auto-created by Shopify)
//! - [`FulfillmentOrderStatus`] - The status of a fulfillment order
//! - [`FulfillmentOrderRequestStatus`] - The request status of a fulfillment order
//! - [`HoldReason`] - Reasons for placing a fulfillment order on hold
//! - [`FulfillmentOrderHoldParams`] - Parameters for hold operation
//! - [`FulfillmentOrderMoveParams`] - Parameters for move operation
//! - [`FulfillmentOrderRescheduleParams`] - Parameters for reschedule operation
//! - [`FulfillmentOrderLineItem`] - A line item in a fulfillment order
//! - [`FulfillmentOrderLineItemInput`] - Input for fulfillment order line items
//! - [`FulfillmentOrderListParams`] - Parameters for listing fulfillment orders
//! - [`FulfillmentOrderFindParams`] - Parameters for finding a single fulfillment order
//!
//! FulfillmentOrder is primarily read-only with special operations:
//! - `cancel()`, `close()`, `hold()`, `move_location()`, `open()`, `release_hold()`, `reschedule()`
//!
//! Related structs for fulfillment service integration:
//! - [`FulfillmentRequest`] - Request fulfillment from a service (create, accept, reject)
//! - [`CancellationRequest`] - Request cancellation of fulfillment (create, accept, reject)
//!
//! ## GiftCard Resource (Shopify Plus)
//!
//! - [`GiftCard`] - A gift card in a Shopify store
//! - [`GiftCardListParams`] - Parameters for listing gift cards
//! - [`GiftCardFindParams`] - Parameters for finding a single gift card
//! - [`GiftCardCountParams`] - Parameters for counting gift cards
//!
//! **Note**: The `read_gift_cards` and `write_gift_cards` scopes require
//! approval from Shopify Support.
//!
//! The GiftCard resource provides resource-specific operations:
//! - `GiftCard::disable()` - Disable a gift card (cannot be re-enabled)
//! - `GiftCard::search()` - Search for gift cards by query
//!
//! Key constraints:
//! - `code` is write-only (only `last_characters` readable after creation)
//! - No Delete operation - use `disable()` instead
//! - Only `expires_on`, `note`, `template_suffix` are updatable
//!
//! ## Transaction Resource (Nested under Order)
//!
//! - [`Transaction`] - A payment transaction nested under an order
//! - [`TransactionKind`] - The type of transaction (authorization, capture, sale, void, refund)
//! - [`TransactionStatus`] - The status of a transaction (pending, failure, success, error)
//! - [`TransactionListParams`] - Parameters for listing transactions
//! - [`TransactionFindParams`] - Parameters for finding a single transaction
//! - [`TransactionCountParams`] - Parameters for counting transactions
//! - [`PaymentDetails`] - Payment details for a transaction
//! - [`CurrencyExchangeAdjustment`] - Currency exchange adjustment
//!
//! Transactions are nested under orders: `/orders/{order_id}/transactions/{id}`
//! Note: Transactions cannot be updated or deleted - they are immutable records.
//!
//! Use `Transaction::all_with_parent()` to list transactions under an order.
//! Use `Transaction::count_with_parent()` to count transactions under an order.
//!
//! ## `RefundResource` (Nested under Order)
//!
//! - [`RefundResource`] - A refund resource for direct operations (separate from embedded Refund)
//! - [`RefundListParams`] - Parameters for listing refunds
//! - [`RefundFindParams`] - Parameters for finding a single refund
//! - [`RefundCalculateParams`] - Parameters for calculating a refund
//! - [`RefundLineItem`] - A line item in a refund
//! - [`RefundLineItemInput`] - Input for refund line items
//! - [`RefundShipping`] - Shipping refund information
//! - [`RefundShippingLine`] - A refund shipping line
//! - [`OrderAdjustment`] - An order adjustment from a refund
//!
//! Refunds are nested under orders: `/orders/{order_id}/refunds/{id}`
//! Note: Refunds cannot be updated or deleted after creation.
//!
//! Special operation:
//! - `RefundResource::calculate()` - Calculate refund amounts without creating a refund
//!
//! Use `RefundResource::all_with_parent()` to list refunds under an order.
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
//! ## `InventoryLevel` Resource
//!
//! - [`InventoryLevel`] - Inventory quantity at a location
//! - [`InventoryLevelListParams`] - Parameters for listing inventory levels
//!
//! `InventoryLevel` uses a composite key (`inventory_item_id` + `location_id`) instead of a single ID.
//! Special operations are implemented as associated functions:
//! - `InventoryLevel::adjust()` - Adjust inventory by a relative amount
//! - `InventoryLevel::connect()` - Connect an inventory item to a location
//! - `InventoryLevel::set()` - Set inventory to an absolute value
//! - `InventoryLevel::delete_at_location()` - Delete inventory level at a location
//!
//! ## Shop Resource (Singleton)
//!
//! - [`Shop`] - The current shop's information
//!
//! The Shop resource is a read-only singleton. Use `Shop::current()` to retrieve it.
//! Shop does not support standard CRUD operations (no Create, Update, Delete).
//!
//! ## Location Resource (Read-Only)
//!
//! - [`Location`] - A store location
//! - [`LocationListParams`] - Parameters for listing locations
//! - [`LocationFindParams`] - Parameters for finding a single location
//! - [`LocationCountParams`] - Parameters for counting locations
//! - [`LocationInventoryLevelsParams`] - Parameters for getting inventory levels at a location
//!
//! Location implements the [`ReadOnlyResource`](crate::rest::ReadOnlyResource) marker trait.
//! Only GET operations are available (find, all, count).
//! Use `Location::inventory_levels()` to get inventory levels at a location.
//!
//! ## Redirect Resource
//!
//! - [`Redirect`] - A URL redirect
//! - [`RedirectListParams`] - Parameters for listing redirects
//! - [`RedirectFindParams`] - Parameters for finding a single redirect
//! - [`RedirectCountParams`] - Parameters for counting redirects
//!
//! Redirects allow merchants to set up automatic URL redirections.
//! Full CRUD operations are available.
//!
//! ## Metafield Resource (Polymorphic Nested)
//!
//! - [`Metafield`] - Custom metadata attached to various resources
//! - [`MetafieldListParams`] - Parameters for listing metafields
//! - [`MetafieldFindParams`] - Parameters for finding a single metafield
//! - [`MetafieldCountParams`] - Parameters for counting metafields
//!
//! Metafields support polymorphic paths based on the owner resource:
//! - Products: `/products/{product_id}/metafields/{id}`
//! - Variants: `/variants/{variant_id}/metafields/{id}`
//! - Customers: `/customers/{customer_id}/metafields/{id}`
//! - Orders: `/orders/{order_id}/metafields/{id}`
//! - Collections: `/collections/{collection_id}/metafields/{id}`
//! - Pages: `/pages/{page_id}/metafields/{id}`
//! - Blogs: `/blogs/{blog_id}/metafields/{id}`
//! - Articles: `/articles/{article_id}/metafields/{id}`
//! - Shop (global): `/metafields/{id}`
//!
//! Use `Metafield::all_for_owner()` for a convenient way to fetch metafields by owner type.
//!
//! ## `CustomCollection` Resource
//!
//! - [`CustomCollection`] - A manually curated collection of products
//! - [`CustomCollectionListParams`] - Parameters for listing custom collections
//! - [`CustomCollectionFindParams`] - Parameters for finding a single custom collection
//! - [`CustomCollectionCountParams`] - Parameters for counting custom collections
//!
//! Custom collections allow merchants to manually select products for the collection.
//!
//! ## `SmartCollection` Resource
//!
//! - [`SmartCollection`] - A rule-based collection of products
//! - [`SmartCollectionListParams`] - Parameters for listing smart collections
//! - [`SmartCollectionFindParams`] - Parameters for finding a single smart collection
//! - [`SmartCollectionCountParams`] - Parameters for counting smart collections
//!
//! Smart collections automatically include products matching specified rules.
//! The `SmartCollection::order()` method allows manual product ordering when `sort_order` is "manual".
//!
//! ## Collection Trait
//!
//! - [`Collection`] - Trait for shared collection behavior
//!
//! Both `CustomCollection` and `SmartCollection` implement the `Collection` trait,
//! providing polymorphic access to:
//! - `products()` - Fetch products in the collection
//! - `product_count()` - Count products in the collection
//!
//! ## Webhook Resource
//!
//! - [`Webhook`] - A webhook subscription for event notifications
//! - [`WebhookListParams`] - Parameters for listing webhooks
//! - [`WebhookFindParams`] - Parameters for finding a single webhook
//! - [`WebhookCountParams`] - Parameters for counting webhooks
//!
//! Webhooks allow apps to receive HTTP notifications when events occur in the store.
//! Use [`common::WebhookTopic`] for available event topics.
//!
//! ## Page Resource
//!
//! - [`Page`] - A static page in a Shopify store
//! - [`PageListParams`] - Parameters for listing pages
//! - [`PageFindParams`] - Parameters for finding a single page
//! - [`PageCountParams`] - Parameters for counting pages
//!
//! Pages are used for static content like "About Us", "Contact", or "Privacy Policy" pages.
//!
//! ## Blog Resource
//!
//! - [`Blog`] - A blog in a Shopify store
//! - [`BlogListParams`] - Parameters for listing blogs
//! - [`BlogFindParams`] - Parameters for finding a single blog
//! - [`BlogCountParams`] - Parameters for counting blogs
//!
//! Blogs are containers for articles. Use [`common::BlogCommentable`] for comment settings.
//!
//! ## Article Resource (Nested under Blog)
//!
//! - [`Article`] - A blog article nested under a blog
//! - [`ArticleListParams`] - Parameters for listing articles
//! - [`ArticleFindParams`] - Parameters for finding a single article
//! - [`ArticleCountParams`] - Parameters for counting articles
//! - [`ArticleImage`] - An image associated with an article
//!
//! Articles are nested under blogs and follow the same pattern as Variants under Products.
//! All operations require `blog_id`:
//! - List: `/blogs/{blog_id}/articles`
//! - Find: `/blogs/{blog_id}/articles/{id}`
//! - Count: `/blogs/{blog_id}/articles/count`
//!
//! Use `Article::all_with_parent()` to list articles under a specific blog.
//! Use `Article::count_with_parent()` to count articles under a specific blog.
//!
//! ## Theme Resource
//!
//! - [`Theme`] - A theme in a Shopify store
//! - [`ThemeListParams`] - Parameters for listing themes
//! - [`ThemeFindParams`] - Parameters for finding a single theme
//!
//! Themes define the look and feel of an online store.
//! Use [`common::ThemeRole`] for theme role types (Main, Unpublished, Demo, Development).
//! Note: There is no count endpoint for themes.
//!
//! ## Asset Resource (Nested under Theme)
//!
//! - [`Asset`] - A theme asset nested under a theme
//! - [`AssetListParams`] - Parameters for listing assets
//!
//! Assets are files that make up a theme (templates, CSS, JS, images).
//! Assets use a string `key` as their identifier (not numeric ID).
//!
//! Key features:
//! - **Key-based identification**: Assets use `key` (path) instead of numeric ID
//! - **Binary support**: Use `Asset::upload_from_bytes()` for binary files
//! - **Content download**: Use `Asset::download_content()` for both text and binary
//! - **PUT for create/update**: The API uses PUT for both operations
//!
//! Asset operations:
//! - `Asset::all_for_theme()` - List all assets in a theme
//! - `Asset::find_by_key()` - Find a specific asset by key
//! - `Asset::save_to_theme()` - Create or update an asset
//! - `Asset::delete_from_theme()` - Delete an asset
//!
//! ## Billing Resources
//!
//! ### `ApplicationCharge` Resource
//!
//! - [`ApplicationCharge`] - A one-time application charge
//! - [`ApplicationChargeFindParams`] - Parameters for finding a single charge
//! - [`ApplicationChargeListParams`] - Parameters for listing charges
//!
//! Application charges allow apps to bill merchants for one-time purchases.
//! Note: No Update or Delete operations - charges cannot be modified after creation.
//!
//! Convenience methods:
//! - `is_active()` - Check if charge is active
//! - `is_pending()` - Check if charge is pending approval
//! - `is_test()` - Check if charge is a test charge
//!
//! ### `RecurringApplicationCharge` Resource
//!
//! - [`RecurringApplicationCharge`] - A recurring subscription charge
//! - [`RecurringApplicationChargeFindParams`] - Parameters for finding a single charge
//! - [`RecurringApplicationChargeListParams`] - Parameters for listing charges
//!
//! Recurring charges allow apps to bill merchants on a subscription basis.
//!
//! Special operations:
//! - `customize()` - Update the capped_amount for usage-based billing
//! - `current()` - Get the currently active recurring charge
//!
//! Convenience methods:
//! - `is_active()` - Check if charge is active
//! - `is_pending()` - Check if charge is pending approval
//! - `is_cancelled()` - Check if charge has been cancelled
//! - `is_test()` - Check if charge is a test charge
//! - `is_in_trial()` - Check if subscription is in trial period
//!
//! ### `UsageCharge` Resource (Nested under `RecurringApplicationCharge`)
//!
//! - [`UsageCharge`] - A usage-based charge under a recurring charge
//! - [`UsageChargeFindParams`] - Parameters for finding a single charge
//! - [`UsageChargeListParams`] - Parameters for listing charges
//!
//! Usage charges are nested under `RecurringApplicationCharge`:
//! - List: `/recurring_application_charges/{charge_id}/usage_charges`
//! - Find: `/recurring_application_charges/{charge_id}/usage_charges/{id}`
//!
//! Use `UsageCharge::all_with_parent()` to list usage charges.
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
//! - [`common::MetafieldOwner`] - Owner types for metafields
//! - [`common::WebhookTopic`] - Webhook event topics
//! - [`common::WebhookFormat`] - Webhook payload formats
//! - [`common::CollectionImage`] - Collection image data
//! - [`common::SmartCollectionRule`] - Rules for smart collections
//! - [`common::ThemeRole`] - Theme role types
//! - [`common::BlogCommentable`] - Blog comment settings
//! - [`common::ChargeStatus`] - Billing charge status
//! - [`common::ChargeCurrency`] - Billing charge currency
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
//! use shopify_api::rest::resources::v2025_10::{InventoryLevel, InventoryLevelListParams};
//! use shopify_api::rest::resources::v2025_10::{Location, LocationListParams};
//! use shopify_api::rest::resources::v2025_10::{Redirect, RedirectListParams};
//! use shopify_api::rest::resources::v2025_10::{Metafield, MetafieldListParams};
//! use shopify_api::rest::resources::v2025_10::{CustomCollection, SmartCollection, Collection};
//! use shopify_api::rest::resources::v2025_10::{Webhook, WebhookListParams};
//! use shopify_api::rest::resources::v2025_10::{Page, PageListParams};
//! use shopify_api::rest::resources::v2025_10::{Blog, BlogListParams};
//! use shopify_api::rest::resources::v2025_10::{Article, ArticleListParams};
//! use shopify_api::rest::resources::v2025_10::{Theme, ThemeListParams, Asset};
//! use shopify_api::rest::resources::v2025_10::{
//!     ApplicationCharge, RecurringApplicationCharge, UsageCharge
//! };
//! use shopify_api::rest::resources::v2025_10::{
//!     Transaction, TransactionKind, TransactionListParams
//! };
//! use shopify_api::rest::resources::v2025_10::{
//!     RefundResource, RefundListParams, RefundCalculateParams
//! };
//! use shopify_api::rest::resources::v2025_10::{
//!     DraftOrder, DraftOrderListParams, DraftOrderStatus, DraftOrderInvoice
//! };
//! use shopify_api::rest::resources::v2025_10::{
//!     FulfillmentOrder, FulfillmentOrderListParams, FulfillmentRequest, CancellationRequest
//! };
//! use shopify_api::rest::resources::v2025_10::{GiftCard, GiftCardListParams};
//! use shopify_api::rest::resources::v2025_10::common::{
//!     MetafieldOwner, WebhookTopic, WebhookFormat, BlogCommentable, ThemeRole,
//!     ChargeStatus, ChargeCurrency
//! };
//! use shopify_api::rest::resources::v2025_10::Shop;
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
//! // Work with draft orders (B2B/wholesale)
//! let mut draft = DraftOrder {
//!     line_items: Some(vec![DraftOrderLineItem {
//!         variant_id: Some(123456),
//!         quantity: Some(2),
//!         ..Default::default()
//!     }]),
//!     ..Default::default()
//! };
//! let saved = draft.save(&client).await?;
//!
//! // Send invoice and complete draft order
//! let invoice = DraftOrderInvoice {
//!     to: Some("customer@example.com".to_string()),
//!     ..Default::default()
//! };
//! let invoiced = saved.send_invoice(&client, invoice).await?;
//! let completed = invoiced.complete(&client, None).await?;
//!
//! // Work with fulfillment orders (modern fulfillment)
//! let fulfillment_orders = FulfillmentOrder::all_with_parent(&client, "order_id", 123, None).await?;
//! for fo in fulfillment_orders.iter() {
//!     // Place on hold if needed
//!     let held = fo.hold(&client, FulfillmentOrderHoldParams {
//!         reason: HoldReason::AwaitingPayment,
//!         ..Default::default()
//!     }).await?;
//! }
//!
//! // Submit a fulfillment request
//! let fo = FulfillmentRequest::create(&client, 123, Some("Please fulfill"), None).await?;
//!
//! // Work with gift cards (Shopify Plus)
//! let mut gift_card = GiftCard {
//!     initial_value: Some("100.00".to_string()),
//!     note: Some("Employee reward".to_string()),
//!     ..Default::default()
//! };
//! let saved = gift_card.save(&client).await?;
//! let results = GiftCard::search(&client, "employee").await?;
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
//! // List transactions for an order
//! let transactions = Transaction::all_with_parent(&client, "order_id", 450789469, None).await?;
//! for txn in transactions.iter() {
//!     println!("Transaction: {:?} - {}", txn.kind, txn.amount.as_deref().unwrap_or("0"));
//! }
//!
//! // Calculate a refund without creating it
//! let calc_params = RefundCalculateParams {
//!     shipping: Some(RefundShipping { full_refund: Some(true), ..Default::default() }),
//!     refund_line_items: Some(vec![
//!         RefundLineItemInput { line_item_id: 669751112, quantity: 1, restock_type: None },
//!     ]),
//!     ..Default::default()
//! };
//! let calculation = RefundResource::calculate(&client, 450789469, calc_params).await?;
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
//! // Work with inventory levels
//! let params = InventoryLevelListParams {
//!     inventory_item_ids: Some("808950810".to_string()),
//!     location_ids: Some("655441491".to_string()),
//!     ..Default::default()
//! };
//! let levels = InventoryLevel::all(&client, Some(params)).await?;
//!
//! // Adjust inventory
//! let level = InventoryLevel::adjust(&client, 808950810, 655441491, -5).await?;
//!
//! // Set inventory to absolute value
//! let level = InventoryLevel::set(&client, 808950810, 655441491, 100, None).await?;
//!
//! // Work with locations (read-only)
//! let locations = Location::all(&client, None).await?;
//! for location in locations.iter() {
//!     println!("Location: {} in {}",
//!         location.name.as_deref().unwrap_or(""),
//!         location.city.as_deref().unwrap_or("")
//!     );
//! }
//!
//! // Get inventory levels at a location
//! let location = Location::find(&client, 655441491, None).await?.into_inner();
//! let levels = location.inventory_levels(&client, None).await?;
//!
//! // Work with redirects
//! let redirect = Redirect {
//!     path: Some("/old-page".to_string()),
//!     target: Some("/new-page".to_string()),
//!     ..Default::default()
//! };
//! let saved = redirect.save(&client).await?;
//!
//! // Get current shop information (singleton resource)
//! let shop = Shop::current(&client).await?;
//! println!("Shop: {}", shop.name.as_deref().unwrap_or(""));
//! println!("Domain: {}", shop.myshopify_domain.as_deref().unwrap_or(""));
//!
//! // List metafields for a product
//! let metafields = Metafield::all_for_owner(
//!     &client,
//!     MetafieldOwner::Product,
//!     123,
//!     None
//! ).await?;
//! for mf in metafields.iter() {
//!     println!("{}.{} = {}",
//!         mf.namespace.as_deref().unwrap_or(""),
//!         mf.key.as_deref().unwrap_or(""),
//!         mf.value.as_deref().unwrap_or("")
//!     );
//! }
//!
//! // Work with collections
//! let custom_collection = CustomCollection::find(&client, 123, None).await?.into_inner();
//! let smart_collection = SmartCollection::find(&client, 456, None).await?.into_inner();
//!
//! // Use Collection trait for polymorphic access
//! let products = custom_collection.products(&client, None).await?;
//! let count = smart_collection.product_count(&client).await?;
//!
//! // Manually reorder products in a smart collection (when sort_order is "manual")
//! smart_collection.order(&client, vec![111, 222, 333]).await?;
//!
//! // Work with webhooks
//! let webhook = Webhook {
//!     topic: Some(WebhookTopic::OrdersCreate),
//!     address: Some("https://example.com/webhooks".to_string()),
//!     format: Some(WebhookFormat::Json),
//!     ..Default::default()
//! };
//! let saved = webhook.save(&client).await?;
//!
//! // List webhooks filtered by topic
//! let params = WebhookListParams {
//!     topic: Some("orders/create".to_string()),
//!     ..Default::default()
//! };
//! let webhooks = Webhook::all(&client, Some(params)).await?;
//!
//! // Work with pages
//! let page = Page::find(&client, 123, None).await?.into_inner();
//! println!("Page: {}", page.title.as_deref().unwrap_or(""));
//!
//! // Create a new page
//! let mut page = Page {
//!     title: Some("About Us".to_string()),
//!     body_html: Some("<p>Welcome!</p>".to_string()),
//!     ..Default::default()
//! };
//! let saved = page.save(&client).await?;
//!
//! // Work with blogs
//! let blog = Blog::find(&client, 123, None).await?.into_inner();
//! println!("Blog: {}", blog.title.as_deref().unwrap_or(""));
//!
//! // Create a blog with comment moderation
//! let mut blog = Blog {
//!     title: Some("News".to_string()),
//!     commentable: Some(BlogCommentable::Moderate),
//!     ..Default::default()
//! };
//! let saved = blog.save(&client).await?;
//!
//! // Work with articles (nested under blogs)
//! let articles = Article::all_with_parent(&client, "blog_id", 123, None).await?;
//! for article in articles.iter() {
//!     println!("Article: {}", article.title.as_deref().unwrap_or(""));
//! }
//!
//! // Count articles in a blog
//! let count = Article::count_with_parent(&client, "blog_id", 123, None).await?;
//! println!("Total articles: {}", count);
//!
//! // Create a new article under a blog
//! let mut article = Article {
//!     blog_id: Some(123),
//!     title: Some("New Post".to_string()),
//!     body_html: Some("<p>Article content</p>".to_string()),
//!     author: Some("Admin".to_string()),
//!     ..Default::default()
//! };
//! let saved = article.save(&client).await?;
//!
//! // Work with themes
//! let themes = Theme::all(&client, None).await?;
//! for theme in themes.iter() {
//!     println!("Theme: {} ({:?})", theme.name.as_deref().unwrap_or(""), theme.role);
//! }
//!
//! // Find the main theme
//! let params = ThemeListParams {
//!     role: Some(ThemeRole::Main),
//!     ..Default::default()
//! };
//! let main_themes = Theme::all(&client, Some(params)).await?;
//!
//! // Work with assets (nested under themes)
//! let assets = Asset::all_for_theme(&client, 123, None).await?;
//! for asset in &assets {
//!     println!("Asset: {}", asset.key);
//! }
//!
//! // Find a specific asset
//! let asset = Asset::find_by_key(&client, 123, "templates/index.liquid").await?;
//! println!("Content: {}", asset.value.as_deref().unwrap_or("(binary)"));
//!
//! // Upload a binary asset
//! let image_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG header
//! let asset = Asset::upload_from_bytes("assets/logo.png", &image_data);
//! let saved = Asset::save_to_theme(&client, 123, &asset).await?;
//!
//! // Download asset content (handles both text and binary)
//! let content = saved.download_content()?;
//!
//! // Create a new product
//! let mut product = Product {
//!     title: Some("My New Product".to_string()),
//!     vendor: Some("My Store".to_string()),
//!     product_type: Some("T-Shirts".to_string()),
//!     ..Default::default()
//! };
//! let saved = product.save(&client).await?;
//!
//! // Work with billing - Create a one-time charge
//! let charge = ApplicationCharge {
//!     name: Some("Pro Widget".to_string()),
//!     price: Some("9.99".to_string()),
//!     return_url: Some("https://myapp.com/callback".to_string()),
//!     test: Some(true),
//!     ..Default::default()
//! };
//! let saved = charge.save(&client).await?;
//! if saved.is_pending() {
//!     println!("Redirect to: {:?}", saved.confirmation_url);
//! }
//!
//! // Create a recurring subscription charge
//! let subscription = RecurringApplicationCharge {
//!     name: Some("Pro Plan".to_string()),
//!     price: Some("29.99".to_string()),
//!     return_url: Some("https://myapp.com/callback".to_string()),
//!     trial_days: Some(14),
//!     capped_amount: Some("100.00".to_string()),
//!     terms: Some("$29.99/month plus usage".to_string()),
//!     ..Default::default()
//! };
//! let saved = subscription.save(&client).await?;
//!
//! // Get the currently active recurring charge
//! if let Some(current) = RecurringApplicationCharge::current(&client).await? {
//!     println!("Active plan: {} at ${}/month",
//!         current.name.as_deref().unwrap_or(""),
//!         current.price.as_deref().unwrap_or("0")
//!     );
//!
//!     // Update capped amount for usage billing
//!     let updated = current.customize(&client, "200.00").await?;
//! }
//!
//! // Create usage charges under a recurring charge
//! let usage = UsageCharge {
//!     recurring_application_charge_id: Some(455696195),
//!     description: Some("100 emails sent".to_string()),
//!     price: Some("1.00".to_string()),
//!     ..Default::default()
//! };
//! // Note: save() requires the parent ID to be set
//!
//! // List usage charges for a recurring charge
//! let usages = UsageCharge::all_with_parent(
//!     &client,
//!     "recurring_application_charge_id",
//!     455696195,
//!     None
//! ).await?;
//! ```

mod access_scope;
mod application_charge;
mod article;
mod asset;
mod blog;
mod collect;
mod collection_trait;
mod comment;
pub mod common;
mod country;
mod currency;
mod custom_collection;
mod customer;
mod discount_code;
mod draft_order;
mod event;
mod fulfillment;
mod fulfillment_order;
mod fulfillment_service;
mod gift_card;
mod inventory_item;
mod inventory_level;
mod location;
mod metafield;
mod order;
mod page;
mod policy;
mod price_rule;
mod product;
mod product_image;
mod province;
mod recurring_application_charge;
mod redirect;
mod refund;
mod script_tag;
mod shop;
mod smart_collection;
mod storefront_access_token;
mod theme;
mod transaction;
mod usage_charge;
mod user;
mod variant;
mod webhook;

// Re-export common types for convenience
pub use common::{
    Address, BlogCommentable, ChargeCurrency, ChargeStatus, CollectionImage, CustomerAddress,
    DiscountAllocation, DiscountApplication, LineItem, LineItemProperty, MetafieldOwner, Money,
    MoneySet, NoteAttribute, ProductImage, ProductOption, ShippingLine, SmartCollectionRule,
    TaxLine, ThemeRole, WebhookFormat, WebhookTopic,
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

// Re-export DraftOrder resource types
pub use draft_order::{
    AppliedDiscount, DraftOrder, DraftOrderCompleteParams, DraftOrderCountParams,
    DraftOrderFindParams, DraftOrderInvoice, DraftOrderLineItem, DraftOrderListParams,
    DraftOrderStatus,
};

// Re-export FulfillmentOrder resource types
pub use fulfillment_order::{
    CancellationRequest, DeliveryMethod, FulfillmentHold, FulfillmentOrder,
    FulfillmentOrderCountParams, FulfillmentOrderDestination, FulfillmentOrderFindParams,
    FulfillmentOrderHoldParams, FulfillmentOrderLineItem, FulfillmentOrderLineItemInput,
    FulfillmentOrderListParams, FulfillmentOrderMoveParams, FulfillmentOrderRescheduleParams,
    FulfillmentOrderRequestStatus, FulfillmentOrderStatus, FulfillmentRequest, HoldReason,
};

// Re-export GiftCard resource types
pub use gift_card::{GiftCard, GiftCardCountParams, GiftCardFindParams, GiftCardListParams};

// Re-export Transaction resource types
pub use transaction::{
    CurrencyExchangeAdjustment, PaymentDetails, Transaction, TransactionCountParams,
    TransactionFindParams, TransactionKind, TransactionListParams, TransactionStatus,
};

// Re-export RefundResource types
pub use refund::{
    OrderAdjustment, RefundCalculateParams, RefundCountParams, RefundFindParams,
    RefundLineItem, RefundLineItemInput, RefundListParams, RefundResource, RefundShipping,
    RefundShippingLine,
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

// Re-export InventoryLevel resource types
pub use inventory_level::{InventoryLevel, InventoryLevelListParams};

// Re-export Location resource types
pub use location::{
    Location, LocationCountParams, LocationFindParams, LocationInventoryLevelsParams,
    LocationListParams,
};

// Re-export Redirect resource types
pub use redirect::{Redirect, RedirectCountParams, RedirectFindParams, RedirectListParams};

// Re-export Shop resource types
pub use shop::Shop;

// Re-export Metafield resource types
pub use metafield::{Metafield, MetafieldCountParams, MetafieldFindParams, MetafieldListParams};

// Re-export Collection trait and collection resource types
pub use collection_trait::Collection;
pub use custom_collection::{
    CustomCollection, CustomCollectionCountParams, CustomCollectionFindParams,
    CustomCollectionListParams,
};
pub use smart_collection::{
    SmartCollection, SmartCollectionCountParams, SmartCollectionFindParams,
    SmartCollectionListParams,
};

// Re-export Webhook resource types
pub use webhook::{Webhook, WebhookCountParams, WebhookFindParams, WebhookListParams};

// Re-export Page resource types
pub use page::{Page, PageCountParams, PageFindParams, PageListParams};

// Re-export Blog resource types
pub use blog::{Blog, BlogCountParams, BlogFindParams, BlogListParams};

// Re-export Article resource types
pub use article::{
    Article, ArticleCountParams, ArticleFindParams, ArticleImage, ArticleListParams,
};

// Re-export Theme resource types
pub use theme::{Theme, ThemeFindParams, ThemeListParams};

// Re-export Asset resource types
pub use asset::{Asset, AssetListParams};

// Re-export Billing resource types
pub use application_charge::{
    ApplicationCharge, ApplicationChargeFindParams, ApplicationChargeListParams,
};
pub use recurring_application_charge::{
    RecurringApplicationCharge, RecurringApplicationChargeFindParams,
    RecurringApplicationChargeListParams,
};
pub use usage_charge::{UsageCharge, UsageChargeFindParams, UsageChargeListParams};

// Re-export PriceRule resource types (deprecated - use GraphQL Discount APIs)
pub use price_rule::{
    BxgyRatio, PrerequisiteRange, PrerequisiteToEntitlement, PriceRule, PriceRuleAllocationMethod,
    PriceRuleCountParams, PriceRuleCustomerSelection, PriceRuleFindParams, PriceRuleListParams,
    PriceRuleTargetSelection, PriceRuleTargetType, PriceRuleValueType,
};

// Re-export DiscountCode resource types (nested under PriceRule)
pub use discount_code::{
    DiscountCode as DiscountCodeResource, DiscountCodeBatchResult, DiscountCodeCountParams,
    DiscountCodeError, DiscountCodeFindParams, DiscountCodeListParams,
};

// Re-export Event resource types (read-only)
pub use event::{Event, EventCountParams, EventFindParams, EventListParams};

// Re-export Comment resource types
pub use comment::{Comment, CommentCountParams, CommentFindParams, CommentListParams};

// Re-export ScriptTag resource types (deprecated - use App Blocks)
pub use script_tag::{
    ScriptTag, ScriptTagCountParams, ScriptTagDisplayScope, ScriptTagEvent, ScriptTagFindParams,
    ScriptTagListParams,
};

// Re-export Policy resource types (read-only, list only)
pub use policy::Policy;

// Re-export FulfillmentService resource types
pub use fulfillment_service::{
    FulfillmentService, FulfillmentServiceFindParams, FulfillmentServiceListParams,
};

// Re-export Country resource types
pub use country::{Country, CountryCountParams, CountryFindParams, CountryListParams};

// Re-export Province resource types (nested under Country)
pub use province::{
    Province, ProvinceCountParams, ProvinceFindParams, ProvinceListParams,
};

// Re-export ProductImageResource types (nested under Product)
// Note: This is distinct from common::ProductImage which is an embedded struct
pub use product_image::{
    ProductImageCountParams, ProductImageFindParams, ProductImageListParams, ProductImageResource,
};

// Re-export User resource types (read-only)
pub use user::{User, UserFindParams, UserListParams};

// Re-export Currency resource types (read-only, list only)
pub use currency::Currency;

// Re-export AccessScope resource types (read-only, special OAuth path)
pub use access_scope::AccessScope;

// Re-export StorefrontAccessToken resource types (limited CRUD)
pub use storefront_access_token::StorefrontAccessToken;

// Re-export Collect resource types (no Update operation)
pub use collect::{Collect, CollectCountParams, CollectFindParams, CollectListParams};

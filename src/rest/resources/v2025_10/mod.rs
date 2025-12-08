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
//! ## Shop Resource (Singleton)
//!
//! - [`Shop`] - The current shop's information
//!
//! The Shop resource is a read-only singleton. Use `Shop::current()` to retrieve it.
//! Shop does not support standard CRUD operations (no Create, Update, Delete).
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
//! use shopify_api::rest::resources::v2025_10::{Metafield, MetafieldListParams};
//! use shopify_api::rest::resources::v2025_10::{CustomCollection, SmartCollection, Collection};
//! use shopify_api::rest::resources::v2025_10::{Webhook, WebhookListParams};
//! use shopify_api::rest::resources::v2025_10::{Page, PageListParams};
//! use shopify_api::rest::resources::v2025_10::{Blog, BlogListParams};
//! use shopify_api::rest::resources::v2025_10::{Article, ArticleListParams};
//! use shopify_api::rest::resources::v2025_10::{Theme, ThemeListParams, Asset};
//! use shopify_api::rest::resources::v2025_10::common::{MetafieldOwner, WebhookTopic, WebhookFormat, BlogCommentable, ThemeRole};
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
//! ```

mod article;
mod asset;
mod blog;
mod collection_trait;
pub mod common;
mod custom_collection;
mod customer;
mod fulfillment;
mod inventory_item;
mod metafield;
mod order;
mod page;
mod product;
mod shop;
mod smart_collection;
mod theme;
mod variant;
mod webhook;

// Re-export common types for convenience
pub use common::{
    Address, BlogCommentable, CollectionImage, CustomerAddress, DiscountAllocation,
    DiscountApplication, LineItem, LineItemProperty, MetafieldOwner, Money, MoneySet,
    NoteAttribute, ProductImage, ProductOption, ShippingLine, SmartCollectionRule, TaxLine,
    ThemeRole, WebhookFormat, WebhookTopic,
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

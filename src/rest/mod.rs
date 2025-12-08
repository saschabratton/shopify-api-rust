//! REST Resource infrastructure for Shopify API.
//!
//! This module provides the foundational infrastructure for REST resources with:
//!
//! - **[`RestResource`] trait**: A standardized interface for CRUD operations
//! - **[`ReadOnlyResource`] marker trait**: Indicates resources that only support read operations
//! - **[`ResourceResponse<T>`]**: A Deref-based wrapper for ergonomic response handling
//! - **[`TrackedResource<T>`]**: Dirty tracking for efficient partial updates
//! - **Path building**: Multiple path support for nested resources
//! - **[`ResourceError`]**: Semantic error types for resource operations
//!
//! # Overview
//!
//! This module is the foundation for REST resource implementations. Individual
//! resources (Product, Order, etc.) are implemented in the `resources` submodule.
//!
//! # Example: Using a Resource
//!
//! ```rust,ignore
//! use shopify_api::{RestClient, Session, ShopDomain, AuthScopes};
//! use shopify_api::rest::{RestResource, ResourceResponse, TrackedResource};
//!
//! // Create a client
//! let session = Session::new(/* ... */);
//! let client = RestClient::new(&session, None)?;
//!
//! // Find a single product
//! let response: ResourceResponse<Product> = Product::find(&client, 123, None).await?;
//! println!("Product: {}", response.title);  // Deref to Product
//!
//! // List products with pagination
//! let response: ResourceResponse<Vec<Product>> = Product::all(&client, None).await?;
//! for product in response.iter() {  // Deref to Vec<Product>
//!     println!("- {}", product.title);
//! }
//!
//! // Check for next page
//! if response.has_next_page() {
//!     let page_info = response.next_page_info().unwrap();
//!     // Fetch next page...
//! }
//!
//! // Create a new product with tracking
//! let product = Product { id: None, title: "New Product".to_string(), vendor: None };
//! let mut tracked = TrackedResource::new(product);
//! let saved = tracked.save(&client).await?;  // POST
//!
//! // Update existing product with partial update
//! let response = Product::find(&client, 123, None).await?;
//! let mut tracked = TrackedResource::from_existing(response.into_inner());
//! tracked.title = "Updated Title".to_string();
//!
//! if tracked.is_dirty() {
//!     let changes = tracked.changed_fields();  // Only "title" changed
//!     let saved = tracked.save_partial(&client, changes).await?;  // PUT with partial body
//!     tracked.mark_clean();
//! }
//!
//! // Delete product
//! let product = Product::find(&client, 123, None).await?.into_inner();
//! product.delete(&client).await?;
//!
//! // Count products
//! let count = Product::count(&client, None).await?;
//! println!("Total products: {}", count);
//! ```
//!
//! # Key Types
//!
//! - [`ResourceError`]: Error types for resource operations
//! - [`ResourcePath`] and [`ResourceOperation`]: Path building infrastructure
//! - [`ResourceResponse`]: Response wrapper with Deref for transparent data access
//! - [`TrackedResource`]: Dirty tracking wrapper for partial updates
//! - [`RestResource`]: Trait defining CRUD operations for resources
//! - [`ReadOnlyResource`]: Marker trait for read-only resources
//! - [`resources`]: Version-specific resource implementations (e.g., Product, Order)

mod errors;
mod path;
mod resource;
mod response;
mod tracking;

pub mod resources;

// Public exports
pub use errors::ResourceError;
pub use path::{build_path, get_path, ResourceOperation, ResourcePath};
pub use resource::{ReadOnlyResource, RestResource};
pub use response::ResourceResponse;
pub use tracking::TrackedResource;

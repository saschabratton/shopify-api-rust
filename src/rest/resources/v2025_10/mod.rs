//! REST resources for API version 2025-10.
//!
//! This module will contain resource implementations for the 2025-10 API version.
//!
//! # Resource Implementations
//!
//! Resource implementations (Product, Order, Customer, etc.) are planned for
//! Items 12 and 13 in the roadmap. This module currently provides the
//! infrastructure for those implementations.
//!
//! # Implementing a Resource
//!
//! Resources are implemented by:
//!
//! 1. Defining a struct with serde derives
//! 2. Implementing the `RestResource` trait
//! 3. Optionally defining parameter structs for filtering
//!
//! Example structure (to be implemented in Items 12/13):
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourcePath, ResourceOperation};
//! use shopify_api::HttpMethod;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! pub struct Product {
//!     pub id: Option<u64>,
//!     pub title: String,
//!     #[serde(skip_serializing_if = "Option::is_none")]
//!     pub body_html: Option<String>,
//!     #[serde(skip_serializing_if = "Option::is_none")]
//!     pub vendor: Option<String>,
//!     // ... more fields
//! }
//!
//! impl RestResource for Product {
//!     type Id = u64;
//!     type FindParams = ProductFindParams;
//!     type AllParams = ProductListParams;
//!     type CountParams = ProductCountParams;
//!
//!     const NAME: &'static str = "Product";
//!     const PLURAL: &'static str = "products";
//!     const PATHS: &'static [ResourcePath] = &[
//!         ResourcePath::new(HttpMethod::Get, ResourceOperation::Find, &["id"], "products/{id}"),
//!         ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "products"),
//!         // ... more paths
//!     ];
//!
//!     fn get_id(&self) -> Option<Self::Id> {
//!         self.id
//!     }
//! }
//! ```

// Resource implementations will be added here in Items 12 and 13.
// Example modules (not yet implemented):
// mod product;
// mod order;
// mod customer;

// Re-exports will be added as resources are implemented:
// pub use product::Product;
// pub use order::Order;
// pub use customer::Customer;

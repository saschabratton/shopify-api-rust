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
//! use shopify_api::rest::resources::Product;  // Uses latest version
//!
//! // Or explicitly specify a version:
//! use shopify_api::rest::resources::v2025_10::Product;
//! ```
//!
//! # Version Support
//!
//! Currently supported API versions:
//! - `v2025_10` (2025-10) - Latest stable
//!
//! Future versions will be added as needed without breaking existing code.

pub mod v2025_10;

// Re-export types from the latest version for convenience
// pub use v2025_10::*;
//
// Note: Currently v2025_10 is empty as actual resource implementations
// (Product, Order, etc.) are Items 12 and 13 in the roadmap.

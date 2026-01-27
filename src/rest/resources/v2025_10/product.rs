//! Product resource implementation.
//!
//! This module provides the Product resource, which represents a product in a Shopify store.
//! Products can have variants, options, and images associated with them.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{Product, ProductListParams, ProductStatus};
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
//! // Create a new product
//! let mut product = Product {
//!     title: Some("My New Product".to_string()),
//!     vendor: Some("My Store".to_string()),
//!     product_type: Some("T-Shirts".to_string()),
//!     ..Default::default()
//! };
//! let saved = product.save(&client).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::{ProductImage, ProductOption};

/// The status of a product.
///
/// Determines whether a product is visible to customers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProductStatus {
    /// The product is active and visible to customers.
    #[default]
    Active,
    /// The product is archived and not visible to customers.
    Archived,
    /// The product is a draft and not visible to customers.
    Draft,
}

/// A variant embedded within a Product response.
///
/// This is a subset of the full Variant resource, containing only the fields
/// that are included when variants are embedded in a Product response.
///
/// For full variant operations, use the `Variant` resource directly.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductVariant {
    /// The unique identifier of the variant.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the product this variant belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// The title of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The price of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The original price of the variant for comparison.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compare_at_price: Option<String>,

    /// The stock keeping unit (SKU) of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,

    /// The position of the variant in the product's variant list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,

    /// The inventory quantity of the variant.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub inventory_quantity: Option<i64>,

    /// The value of the first option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option1: Option<String>,

    /// The value of the second option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option2: Option<String>,

    /// The value of the third option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option3: Option<String>,

    /// The ID of the image associated with this variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<u64>,

    /// When the variant was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the variant was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// A product in a Shopify store.
///
/// Products are the goods or services that merchants sell. A product can have
/// multiple variants (e.g., different sizes or colors), options, and images.
///
/// # Fields
///
/// ## Writable Fields
/// - `title` - The name of the product
/// - `body_html` - The description of the product in HTML format
/// - `vendor` - The name of the product's vendor
/// - `product_type` - A categorization for the product
/// - `published_at` - When the product was published
/// - `published_scope` - Where the product is published (e.g., "web", "global")
/// - `status` - Whether the product is active, archived, or draft
/// - `tags` - A comma-separated list of tags
/// - `template_suffix` - The suffix of the template used for this product
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `handle` - The URL-friendly name
/// - `created_at` - When the product was created
/// - `updated_at` - When the product was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Nested Resources
/// - `variants` - The product's variants
/// - `options` - The product's options
/// - `images` - The product's images
/// - `image` - The main/featured image
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Product {
    /// The unique identifier of the product.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The name of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The description of the product in HTML format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,

    /// The name of the product's vendor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    /// A categorization for the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,

    /// The URL-friendly name of the product.
    /// Read-only field - generated from the title.
    #[serde(skip_serializing)]
    pub handle: Option<String>,

    /// When the product was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the product was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// When the product was published.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,

    /// Where the product is published.
    /// Valid values: "web", "global".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_scope: Option<String>,

    /// The status of the product: active, archived, or draft.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ProductStatus>,

    /// A comma-separated list of tags for the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,

    /// The suffix of the Liquid template used for the product page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_suffix: Option<String>,

    /// The admin GraphQL API ID for this product.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,

    /// The variants of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<ProductVariant>>,

    /// The options of the product (e.g., Size, Color).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ProductOption>>,

    /// All images associated with the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ProductImage>>,

    /// The main/featured image of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ProductImage>,
}

impl RestResource for Product {
    type Id = u64;
    type FindParams = ProductFindParams;
    type AllParams = ProductListParams;
    type CountParams = ProductCountParams;

    const NAME: &'static str = "Product";
    const PLURAL: &'static str = "products";

    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "products/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "products"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "products/count",
        ),
        ResourcePath::new(HttpMethod::Post, ResourceOperation::Create, &[], "products"),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "products/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "products/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single product.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing products.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductListParams {
    /// Return only products with the given IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<u64>>,

    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return products after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Filter by product title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Filter by product vendor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    /// Filter by product handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// Filter by product type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,

    /// Filter by collection ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<u64>,

    /// Show products created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show products created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show products updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show products updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show products published after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_min: Option<DateTime<Utc>>,

    /// Show products published before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_max: Option<DateTime<Utc>>,

    /// Filter by published status.
    /// Valid values: "published", "unpublished", "any".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_status: Option<String>,

    /// Filter by product status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ProductStatus>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

/// Parameters for counting products.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProductCountParams {
    /// Filter by product vendor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    /// Filter by product type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,

    /// Filter by collection ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<u64>,

    /// Show products created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show products created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show products updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show products updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show products published after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_min: Option<DateTime<Utc>>,

    /// Show products published before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at_max: Option<DateTime<Utc>>,

    /// Filter by published status.
    /// Valid values: "published", "unpublished", "any".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_product_serialization_with_all_fields() {
        let product = Product {
            id: Some(12345), // Read-only, should be skipped in serialization
            title: Some("Test Product".to_string()),
            body_html: Some("<p>Description</p>".to_string()),
            vendor: Some("Test Vendor".to_string()),
            product_type: Some("T-Shirts".to_string()),
            handle: Some("test-product".to_string()), // Read-only
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ), // Read-only
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-20T15:45:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ), // Read-only
            published_at: Some(
                DateTime::parse_from_rfc3339("2024-01-20T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            published_scope: Some("global".to_string()),
            status: Some(ProductStatus::Active),
            tags: Some("summer, sale, featured".to_string()),
            template_suffix: Some("custom".to_string()),
            admin_graphql_api_id: Some("gid://shopify/Product/12345".to_string()), // Read-only
            variants: Some(vec![ProductVariant {
                id: Some(111),
                title: Some("Default".to_string()),
                price: Some("29.99".to_string()),
                ..Default::default()
            }]),
            options: Some(vec![ProductOption {
                name: Some("Size".to_string()),
                values: Some(vec!["Small".to_string(), "Medium".to_string()]),
                ..Default::default()
            }]),
            images: Some(vec![]),
            image: None,
        };

        let json = serde_json::to_string(&product).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["title"], "Test Product");
        assert_eq!(parsed["body_html"], "<p>Description</p>");
        assert_eq!(parsed["vendor"], "Test Vendor");
        assert_eq!(parsed["product_type"], "T-Shirts");
        assert_eq!(parsed["published_scope"], "global");
        assert_eq!(parsed["status"], "active");
        assert_eq!(parsed["tags"], "summer, sale, featured");
        assert_eq!(parsed["template_suffix"], "custom");

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("handle").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_product_deserialization_from_api_response() {
        let json = r#"{
            "id": 788032119674292922,
            "title": "Example T-Shirt",
            "body_html": "<strong>Good cotton T-shirt</strong>",
            "vendor": "Acme",
            "product_type": "Shirts",
            "handle": "example-t-shirt",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "published_at": "2024-01-20T12:00:00Z",
            "published_scope": "global",
            "status": "active",
            "tags": "cotton, summer",
            "template_suffix": null,
            "admin_graphql_api_id": "gid://shopify/Product/788032119674292922",
            "variants": [
                {
                    "id": 39072856,
                    "product_id": 788032119674292922,
                    "title": "Small",
                    "price": "19.99",
                    "compare_at_price": "24.99",
                    "sku": "SHIRT-SM",
                    "position": 1,
                    "inventory_quantity": 100,
                    "option1": "Small",
                    "option2": null,
                    "option3": null,
                    "image_id": null,
                    "created_at": "2024-01-15T10:30:00Z",
                    "updated_at": "2024-06-20T15:45:00Z"
                }
            ],
            "options": [
                {
                    "id": 594680422,
                    "product_id": 788032119674292922,
                    "name": "Size",
                    "position": 1,
                    "values": ["Small", "Medium", "Large"]
                }
            ],
            "images": [],
            "image": null
        }"#;

        let product: Product = serde_json::from_str(json).unwrap();

        // Verify all fields are deserialized
        assert_eq!(product.id, Some(788032119674292922));
        assert_eq!(product.title, Some("Example T-Shirt".to_string()));
        assert_eq!(
            product.body_html,
            Some("<strong>Good cotton T-shirt</strong>".to_string())
        );
        assert_eq!(product.vendor, Some("Acme".to_string()));
        assert_eq!(product.product_type, Some("Shirts".to_string()));
        assert_eq!(product.handle, Some("example-t-shirt".to_string()));
        assert!(product.created_at.is_some());
        assert!(product.updated_at.is_some());
        assert!(product.published_at.is_some());
        assert_eq!(product.published_scope, Some("global".to_string()));
        assert_eq!(product.status, Some(ProductStatus::Active));
        assert_eq!(product.tags, Some("cotton, summer".to_string()));
        assert_eq!(product.template_suffix, None);
        assert_eq!(
            product.admin_graphql_api_id,
            Some("gid://shopify/Product/788032119674292922".to_string())
        );

        // Verify nested variants
        let variants = product.variants.unwrap();
        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].id, Some(39072856));
        assert_eq!(variants[0].title, Some("Small".to_string()));
        assert_eq!(variants[0].price, Some("19.99".to_string()));
        assert_eq!(variants[0].compare_at_price, Some("24.99".to_string()));
        assert_eq!(variants[0].sku, Some("SHIRT-SM".to_string()));
        assert_eq!(variants[0].inventory_quantity, Some(100));

        // Verify nested options
        let options = product.options.unwrap();
        assert_eq!(options.len(), 1);
        assert_eq!(options[0].name, Some("Size".to_string()));
        assert_eq!(
            options[0].values,
            Some(vec![
                "Small".to_string(),
                "Medium".to_string(),
                "Large".to_string()
            ])
        );
    }

    #[test]
    fn test_product_list_params_serialization() {
        let params = ProductListParams {
            ids: Some(vec![123, 456, 789]),
            limit: Some(50),
            vendor: Some("Acme".to_string()),
            status: Some(ProductStatus::Active),
            published_status: Some("published".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["ids"], serde_json::json!([123, 456, 789]));
        assert_eq!(json["limit"], 50);
        assert_eq!(json["vendor"], "Acme");
        assert_eq!(json["status"], "active");
        assert_eq!(json["published_status"], "published");

        // Fields not set should be omitted
        assert!(json.get("title").is_none());
        assert!(json.get("handle").is_none());
        assert!(json.get("created_at_min").is_none());
    }

    #[test]
    fn test_product_status_enum_serialization() {
        // Test serialization to lowercase
        let active = ProductStatus::Active;
        let archived = ProductStatus::Archived;
        let draft = ProductStatus::Draft;

        assert_eq!(serde_json::to_string(&active).unwrap(), "\"active\"");
        assert_eq!(serde_json::to_string(&archived).unwrap(), "\"archived\"");
        assert_eq!(serde_json::to_string(&draft).unwrap(), "\"draft\"");

        // Test deserialization from lowercase
        let active: ProductStatus = serde_json::from_str("\"active\"").unwrap();
        let archived: ProductStatus = serde_json::from_str("\"archived\"").unwrap();
        let draft: ProductStatus = serde_json::from_str("\"draft\"").unwrap();

        assert_eq!(active, ProductStatus::Active);
        assert_eq!(archived, ProductStatus::Archived);
        assert_eq!(draft, ProductStatus::Draft);
    }

    #[test]
    fn test_product_get_id_returns_correct_value() {
        // Product with ID
        let product_with_id = Product {
            id: Some(123456789),
            title: Some("Test".to_string()),
            ..Default::default()
        };
        assert_eq!(product_with_id.get_id(), Some(123456789));

        // Product without ID (new product)
        let product_without_id = Product {
            id: None,
            title: Some("New Product".to_string()),
            ..Default::default()
        };
        assert_eq!(product_without_id.get_id(), None);
    }

    #[test]
    fn test_product_path_constants_are_correct() {
        // Test Find path
        let find_path = get_path(Product::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "products/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(Product::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "products");
        assert_eq!(all_path.unwrap().http_method, HttpMethod::Get);

        // Test Count path
        let count_path = get_path(Product::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "products/count");
        assert_eq!(count_path.unwrap().http_method, HttpMethod::Get);

        // Test Create path
        let create_path = get_path(Product::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "products");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(Product::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "products/{id}");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test Delete path
        let delete_path = get_path(Product::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "products/{id}");
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        // Verify constants
        assert_eq!(Product::NAME, "Product");
        assert_eq!(Product::PLURAL, "products");
    }

    #[test]
    fn test_product_variant_embedded_struct() {
        let variant = ProductVariant {
            id: Some(111222333),
            product_id: Some(444555666),
            title: Some("Large / Blue".to_string()),
            price: Some("39.99".to_string()),
            compare_at_price: Some("49.99".to_string()),
            sku: Some("PROD-LG-BL".to_string()),
            position: Some(2),
            inventory_quantity: Some(50),
            option1: Some("Large".to_string()),
            option2: Some("Blue".to_string()),
            option3: None,
            image_id: Some(999888777),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-20T15:45:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        // Test serialization - read-only fields should be skipped
        let json = serde_json::to_string(&variant).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["product_id"], 444555666);
        assert_eq!(parsed["title"], "Large / Blue");
        assert_eq!(parsed["price"], "39.99");
        assert_eq!(parsed["compare_at_price"], "49.99");
        assert_eq!(parsed["sku"], "PROD-LG-BL");
        assert_eq!(parsed["position"], 2);
        assert_eq!(parsed["option1"], "Large");
        assert_eq!(parsed["option2"], "Blue");
        assert_eq!(parsed["image_id"], 999888777);

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("inventory_quantity").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_product_count_params_serialization() {
        let params = ProductCountParams {
            vendor: Some("Acme".to_string()),
            product_type: Some("Shirts".to_string()),
            collection_id: Some(123456),
            published_status: Some("published".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["vendor"], "Acme");
        assert_eq!(json["product_type"], "Shirts");
        assert_eq!(json["collection_id"], 123456);
        assert_eq!(json["published_status"], "published");
    }
}

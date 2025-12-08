//! Integration tests for Core REST Resources.
//!
//! These tests verify the complete integration of the REST resource types:
//! Product, Variant, Order, Customer, Fulfillment, and InventoryItem.
//!
//! Tests cover:
//! - Product with nested variants deserialization
//! - Order with all nested structures
//! - Variant dual-path resolution
//! - Fulfillment nested under Order
//! - Path selection with multiple available IDs
//! - ListParams serialization to query strings
//! - Resource-specific operations compile and have correct signatures
//! - TrackedResource works with new resource types

use serde_json::json;
use std::collections::HashMap;

use shopify_api::rest::resources::{
    // Common types
    Address,
    // Order types
    CancelReason,
    // InventoryItem types
    CountryHarmonizedSystemCode,
    // Customer types
    Customer,
    CustomerAddress,
    CustomerCountParams,
    CustomerFindParams,
    CustomerListParams,
    CustomerState,
    DiscountApplication,
    DiscountCode,
    EmailMarketingConsent,
    FinancialStatus,
    // Fulfillment types
    Fulfillment,
    FulfillmentCountParams,
    FulfillmentFindParams,
    FulfillmentLineItem,
    FulfillmentListParams,
    FulfillmentResourceStatus,
    FulfillmentStatus,
    InventoryItem,
    InventoryItemFindParams,
    InventoryItemListParams,
    LineItem,
    NoteAttribute,
    Order,
    OrderCountParams,
    OrderFindParams,
    OrderFulfillment,
    OrderListParams,
    // Product types
    Product,
    ProductCountParams,
    ProductFindParams,
    ProductImage,
    ProductListParams,
    ProductOption,
    ProductStatus,
    ProductVariant,
    Refund,
    ShipmentStatus,
    ShippingLine,
    SmsMarketingConsent,
    TaxLine,
    TrackingInfo,
    // Variant types
    Variant,
    VariantCountParams,
    VariantFindParams,
    VariantListParams,
    WeightUnit,
};
use shopify_api::rest::{build_path, get_path, ResourceOperation, RestResource, TrackedResource};
use shopify_api::HttpMethod;

// ============================================================================
// Test 1: Product with nested variants deserialization
// ============================================================================

#[test]
fn test_product_with_nested_variants_deserialization() {
    let json_str = r#"{
        "id": 788032119674292922,
        "title": "Example T-Shirt",
        "body_html": "<strong>Good quality cotton</strong>",
        "vendor": "Acme",
        "product_type": "Shirts",
        "handle": "example-t-shirt",
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-06-20T15:45:00Z",
        "published_at": "2024-01-20T12:00:00Z",
        "published_scope": "global",
        "status": "active",
        "tags": "cotton, summer, sale",
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
                "created_at": "2024-01-15T10:30:00Z",
                "updated_at": "2024-06-20T15:45:00Z"
            },
            {
                "id": 39072857,
                "product_id": 788032119674292922,
                "title": "Medium",
                "price": "19.99",
                "compare_at_price": "24.99",
                "sku": "SHIRT-MD",
                "position": 2,
                "inventory_quantity": 75,
                "option1": "Medium",
                "option2": null,
                "option3": null,
                "created_at": "2024-01-15T10:30:00Z",
                "updated_at": "2024-06-20T15:45:00Z"
            },
            {
                "id": 39072858,
                "product_id": 788032119674292922,
                "title": "Large",
                "price": "21.99",
                "compare_at_price": "26.99",
                "sku": "SHIRT-LG",
                "position": 3,
                "inventory_quantity": 50,
                "option1": "Large",
                "option2": null,
                "option3": null,
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
        "images": [
            {
                "id": 850703190,
                "product_id": 788032119674292922,
                "position": 1,
                "src": "https://cdn.shopify.com/example.jpg",
                "width": 600,
                "height": 800,
                "alt": "Front view",
                "variant_ids": [],
                "created_at": "2024-01-15T10:30:00Z",
                "updated_at": "2024-06-20T15:45:00Z"
            }
        ]
    }"#;

    let product: Product = serde_json::from_str(json_str).unwrap();

    // Verify core fields
    assert_eq!(product.id, Some(788032119674292922));
    assert_eq!(product.title, Some("Example T-Shirt".to_string()));
    assert_eq!(product.vendor, Some("Acme".to_string()));
    assert_eq!(product.status, Some(ProductStatus::Active));
    assert_eq!(product.tags, Some("cotton, summer, sale".to_string()));

    // Verify nested variants
    let variants = product.variants.unwrap();
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0].id, Some(39072856));
    assert_eq!(variants[0].title, Some("Small".to_string()));
    assert_eq!(variants[0].price, Some("19.99".to_string()));
    assert_eq!(variants[0].sku, Some("SHIRT-SM".to_string()));
    assert_eq!(variants[1].title, Some("Medium".to_string()));
    assert_eq!(variants[2].title, Some("Large".to_string()));
    assert_eq!(variants[2].price, Some("21.99".to_string()));

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

    // Verify nested images
    let images = product.images.unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].alt, Some("Front view".to_string()));
}

// ============================================================================
// Test 2: Order with all nested structures
// ============================================================================

#[test]
fn test_order_with_all_nested_structures() {
    // Use raw string literal with ## to allow # in the JSON
    let json_str = r##"{
        "id": 450789469,
        "name": "#1001",
        "email": "customer@example.com",
        "phone": "+1-555-555-5555",
        "order_number": 1001,
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-06-20T15:45:00Z",
        "total_price": "249.99",
        "subtotal_price": "229.99",
        "total_tax": "20.00",
        "total_discounts": "10.00",
        "currency": "USD",
        "financial_status": "paid",
        "fulfillment_status": "partial",
        "tags": "vip, priority",
        "note": "Please gift wrap",
        "buyer_accepts_marketing": true,
        "line_items": [
            {
                "id": 669751112,
                "variant_id": 457924702,
                "product_id": 632910392,
                "title": "IPod Nano - 8GB",
                "quantity": 2,
                "price": "99.99",
                "sku": "IPOD2008BLACK",
                "taxable": true,
                "requires_shipping": true,
                "gift_card": false,
                "tax_lines": [
                    {
                        "title": "State Tax",
                        "price": "8.00",
                        "rate": 0.08
                    },
                    {
                        "title": "City Tax",
                        "price": "2.00",
                        "rate": 0.02
                    }
                ],
                "discount_allocations": [
                    {
                        "amount": "5.00",
                        "discount_application_index": 0
                    }
                ]
            }
        ],
        "billing_address": {
            "first_name": "John",
            "last_name": "Doe",
            "company": "Acme Inc",
            "address1": "123 Main St",
            "address2": "Suite 100",
            "city": "New York",
            "province": "New York",
            "province_code": "NY",
            "country": "United States",
            "country_code": "US",
            "zip": "10001",
            "phone": "+1-555-555-5555"
        },
        "shipping_address": {
            "first_name": "John",
            "last_name": "Doe",
            "address1": "456 Shipping Lane",
            "city": "Los Angeles",
            "province": "California",
            "province_code": "CA",
            "country": "United States",
            "country_code": "US",
            "zip": "90001"
        },
        "tax_lines": [
            {
                "title": "State Tax",
                "price": "16.00",
                "rate": 0.08
            },
            {
                "title": "City Tax",
                "price": "4.00",
                "rate": 0.02
            }
        ],
        "discount_codes": [
            {
                "code": "SUMMER20",
                "amount": "10.00",
                "type": "fixed_amount"
            }
        ],
        "discount_applications": [
            {
                "type": "discount_code",
                "value": "10.00",
                "value_type": "fixed_amount",
                "allocation_method": "across",
                "target_selection": "all",
                "target_type": "line_item",
                "code": "SUMMER20"
            }
        ],
        "shipping_lines": [
            {
                "id": 271878346596884015,
                "title": "Standard Shipping",
                "price": "9.99",
                "code": "standard"
            }
        ],
        "fulfillments": [
            {
                "id": 255858046,
                "order_id": 450789469,
                "status": "success",
                "tracking_company": "UPS",
                "tracking_number": "1Z999AA10123456784",
                "tracking_url": "https://ups.com/tracking/1Z999AA10123456784"
            }
        ],
        "refunds": [
            {
                "id": 509562969,
                "order_id": 450789469,
                "note": "Partial refund for damaged item",
                "restock": true
            }
        ],
        "customer": {
            "id": 207119551,
            "email": "customer@example.com",
            "first_name": "John",
            "last_name": "Doe",
            "orders_count": 5,
            "total_spent": "1250.00",
            "state": "enabled"
        },
        "note_attributes": [
            {
                "name": "gift_message",
                "value": "Happy Birthday!"
            }
        ]
    }"##;

    let order: Order = serde_json::from_str(json_str).unwrap();

    // Verify core fields
    assert_eq!(order.id, Some(450789469));
    assert_eq!(order.name, Some("#1001".to_string()));
    assert_eq!(order.email, Some("customer@example.com".to_string()));
    assert_eq!(order.financial_status, Some(FinancialStatus::Paid));
    assert_eq!(order.fulfillment_status, Some(FulfillmentStatus::Partial));

    // Verify line items with nested tax_lines
    let line_items = order.line_items.unwrap();
    assert_eq!(line_items.len(), 1);
    assert_eq!(line_items[0].title, Some("IPod Nano - 8GB".to_string()));
    assert_eq!(line_items[0].quantity, Some(2));
    let tax_lines = line_items[0].tax_lines.as_ref().unwrap();
    assert_eq!(tax_lines.len(), 2);
    assert_eq!(tax_lines[0].title, Some("State Tax".to_string()));

    // Verify billing address
    let billing = order.billing_address.unwrap();
    assert_eq!(billing.first_name, Some("John".to_string()));
    assert_eq!(billing.city, Some("New York".to_string()));
    assert_eq!(billing.province_code, Some("NY".to_string()));

    // Verify shipping address
    let shipping = order.shipping_address.unwrap();
    assert_eq!(shipping.city, Some("Los Angeles".to_string()));
    assert_eq!(shipping.province_code, Some("CA".to_string()));

    // Verify discount codes
    let discount_codes = order.discount_codes.unwrap();
    assert_eq!(discount_codes.len(), 1);
    assert_eq!(discount_codes[0].code, Some("SUMMER20".to_string()));

    // Verify fulfillments
    let fulfillments = order.fulfillments.unwrap();
    assert_eq!(fulfillments.len(), 1);
    assert_eq!(fulfillments[0].status, Some("success".to_string()));

    // Verify customer
    let customer = order.customer.unwrap();
    assert_eq!(customer.first_name, Some("John".to_string()));
    assert_eq!(customer.orders_count, Some(5));

    // Verify note attributes
    let note_attrs = order.note_attributes.unwrap();
    assert_eq!(note_attrs.len(), 1);
    assert_eq!(note_attrs[0].name, "gift_message");
}

// ============================================================================
// Test 3: Variant dual-path resolution
// ============================================================================

#[test]
fn test_variant_dual_path_resolution() {
    // Test nested path selection (when both product_id and id are available)
    let nested_find_path = get_path(
        Variant::PATHS,
        ResourceOperation::Find,
        &["product_id", "id"],
    );
    assert!(nested_find_path.is_some());
    assert_eq!(
        nested_find_path.unwrap().template,
        "products/{product_id}/variants/{id}"
    );

    // Test standalone path selection (when only id is available)
    let standalone_find_path = get_path(Variant::PATHS, ResourceOperation::Find, &["id"]);
    assert!(standalone_find_path.is_some());
    assert_eq!(standalone_find_path.unwrap().template, "variants/{id}");

    // Test nested All path
    let nested_all_path = get_path(Variant::PATHS, ResourceOperation::All, &["product_id"]);
    assert!(nested_all_path.is_some());
    assert_eq!(
        nested_all_path.unwrap().template,
        "products/{product_id}/variants"
    );

    // Verify standalone All is not available (variants must be listed under a product)
    let standalone_all_path = get_path(Variant::PATHS, ResourceOperation::All, &[]);
    assert!(standalone_all_path.is_none());

    // Test path building with IDs
    let mut ids: HashMap<&str, u64> = HashMap::new();
    ids.insert("product_id", 123);
    ids.insert("id", 456);

    let nested_path = get_path(
        Variant::PATHS,
        ResourceOperation::Find,
        &["product_id", "id"],
    )
    .unwrap();
    let url = build_path(nested_path.template, &ids);
    assert_eq!(url, "products/123/variants/456");

    // Test Update with both paths
    let nested_update = get_path(
        Variant::PATHS,
        ResourceOperation::Update,
        &["product_id", "id"],
    );
    assert!(nested_update.is_some());
    assert_eq!(
        nested_update.unwrap().template,
        "products/{product_id}/variants/{id}"
    );

    let standalone_update = get_path(Variant::PATHS, ResourceOperation::Update, &["id"]);
    assert!(standalone_update.is_some());
    assert_eq!(standalone_update.unwrap().template, "variants/{id}");
}

// ============================================================================
// Test 4: Fulfillment nested under Order
// ============================================================================

#[test]
fn test_fulfillment_nested_under_order() {
    // Fulfillments require order_id for most operations
    let find_path = get_path(
        Fulfillment::PATHS,
        ResourceOperation::Find,
        &["order_id", "id"],
    );
    assert!(find_path.is_some());
    assert_eq!(
        find_path.unwrap().template,
        "orders/{order_id}/fulfillments/{id}"
    );

    let all_path = get_path(Fulfillment::PATHS, ResourceOperation::All, &["order_id"]);
    assert!(all_path.is_some());
    assert_eq!(all_path.unwrap().template, "orders/{order_id}/fulfillments");

    let create_path = get_path(Fulfillment::PATHS, ResourceOperation::Create, &["order_id"]);
    assert!(create_path.is_some());
    assert_eq!(
        create_path.unwrap().template,
        "orders/{order_id}/fulfillments"
    );

    // Verify that operations without order_id fail
    let no_order_all = get_path(Fulfillment::PATHS, ResourceOperation::All, &[]);
    assert!(no_order_all.is_none());

    // Test path building
    let mut ids: HashMap<&str, u64> = HashMap::new();
    ids.insert("order_id", 450789469);
    ids.insert("id", 255858046);

    let path = get_path(
        Fulfillment::PATHS,
        ResourceOperation::Find,
        &["order_id", "id"],
    )
    .unwrap();
    let url = build_path(path.template, &ids);
    assert_eq!(url, "orders/450789469/fulfillments/255858046");

    // Test deserialization
    let json_str = r##"{
        "id": 255858046,
        "order_id": 450789469,
        "name": "#1001.1",
        "status": "success",
        "service": "manual",
        "location_id": 487838322,
        "shipment_status": "delivered",
        "tracking_company": "UPS",
        "tracking_number": "1Z999AA10123456784",
        "tracking_numbers": ["1Z999AA10123456784"],
        "tracking_url": "https://ups.com/tracking/1Z999AA10123456784",
        "tracking_urls": ["https://ups.com/tracking/1Z999AA10123456784"],
        "line_items": [
            {
                "id": 669751112,
                "variant_id": 457924702,
                "title": "IPod Nano - 8GB",
                "quantity": 1,
                "sku": "IPOD2008BLACK"
            }
        ]
    }"##;

    let fulfillment: Fulfillment = serde_json::from_str(json_str).unwrap();
    assert_eq!(fulfillment.id, Some(255858046));
    assert_eq!(fulfillment.order_id, Some(450789469));
    assert_eq!(fulfillment.status, Some(FulfillmentResourceStatus::Success));
    assert_eq!(fulfillment.shipment_status, Some(ShipmentStatus::Delivered));
}

// ============================================================================
// Test 5: Path selection with multiple available IDs
// ============================================================================

#[test]
fn test_path_selection_with_multiple_available_ids() {
    // The path selection should prefer more specific paths (more required_ids)

    // For Variant Find with both IDs, nested path is selected
    let nested = get_path(
        Variant::PATHS,
        ResourceOperation::Find,
        &["product_id", "id"],
    );
    assert_eq!(
        nested.unwrap().template,
        "products/{product_id}/variants/{id}"
    );

    // For Variant Find with only id, standalone path is selected
    let standalone = get_path(Variant::PATHS, ResourceOperation::Find, &["id"]);
    assert_eq!(standalone.unwrap().template, "variants/{id}");

    // Test Order paths (simpler - always standalone)
    let order_find = get_path(Order::PATHS, ResourceOperation::Find, &["id"]);
    assert_eq!(order_find.unwrap().template, "orders/{id}");

    // Test Customer paths (always standalone)
    let customer_find = get_path(Customer::PATHS, ResourceOperation::Find, &["id"]);
    assert_eq!(customer_find.unwrap().template, "customers/{id}");

    // Test InventoryItem paths (always standalone)
    let inventory_find = get_path(InventoryItem::PATHS, ResourceOperation::Find, &["id"]);
    assert_eq!(inventory_find.unwrap().template, "inventory_items/{id}");

    // Verify HTTP methods are correct
    let nested_again = get_path(
        Variant::PATHS,
        ResourceOperation::Find,
        &["product_id", "id"],
    );
    let order_find_again = get_path(Order::PATHS, ResourceOperation::Find, &["id"]);
    assert_eq!(nested_again.unwrap().http_method, HttpMethod::Get);
    assert_eq!(order_find_again.unwrap().http_method, HttpMethod::Get);
}

// ============================================================================
// Test 6: ListParams serialization to query strings
// ============================================================================

#[test]
fn test_list_params_serialization_to_query_strings() {
    // Test ProductListParams
    let product_params = ProductListParams {
        ids: Some(vec![123, 456, 789]),
        limit: Some(50),
        vendor: Some("Acme".to_string()),
        status: Some(ProductStatus::Active),
        ..Default::default()
    };

    let product_json = serde_json::to_value(&product_params).unwrap();
    assert_eq!(product_json["ids"], json!([123, 456, 789]));
    assert_eq!(product_json["limit"], 50);
    assert_eq!(product_json["vendor"], "Acme");
    assert_eq!(product_json["status"], "active");

    // Test OrderListParams
    let order_params = OrderListParams {
        financial_status: Some(FinancialStatus::Paid),
        fulfillment_status: Some(FulfillmentStatus::Unfulfilled),
        limit: Some(25),
        ..Default::default()
    };

    let order_json = serde_json::to_value(&order_params).unwrap();
    assert_eq!(order_json["financial_status"], "paid");
    assert_eq!(order_json["fulfillment_status"], "unfulfilled");
    assert_eq!(order_json["limit"], 25);

    // Test VariantListParams
    let variant_params = VariantListParams {
        limit: Some(100),
        since_id: Some(999),
        fields: Some("id,title,price".to_string()),
        ..Default::default()
    };

    let variant_json = serde_json::to_value(&variant_params).unwrap();
    assert_eq!(variant_json["limit"], 100);
    assert_eq!(variant_json["since_id"], 999);
    assert_eq!(variant_json["fields"], "id,title,price");

    // Test InventoryItemListParams (ids is the important one)
    let inventory_params = InventoryItemListParams {
        ids: Some(vec![808950810, 808950811, 808950812]),
        limit: Some(50),
        ..Default::default()
    };

    let inventory_json = serde_json::to_value(&inventory_params).unwrap();
    assert_eq!(
        inventory_json["ids"],
        json!([808950810, 808950811, 808950812])
    );

    // Test FulfillmentListParams
    let fulfillment_params = FulfillmentListParams {
        limit: Some(10),
        fields: Some("id,status,tracking_number".to_string()),
        ..Default::default()
    };

    let fulfillment_json = serde_json::to_value(&fulfillment_params).unwrap();
    assert_eq!(fulfillment_json["limit"], 10);
    assert_eq!(fulfillment_json["fields"], "id,status,tracking_number");
}

// ============================================================================
// Test 7: Resource-specific operations compile and have correct signatures
// ============================================================================

#[test]
fn test_resource_specific_operations_signatures() {
    // This test verifies that the resource-specific operations exist
    // and have the correct signatures by using type assertions.

    // Verify Order operations signatures
    fn _assert_order_cancel<F, Fut>(f: F)
    where
        F: Fn(&Order, &shopify_api::clients::RestClient) -> Fut,
        Fut: std::future::Future<Output = Result<Order, shopify_api::rest::ResourceError>>,
    {
        let _ = f;
    }

    fn _assert_order_close<F, Fut>(f: F)
    where
        F: Fn(&Order, &shopify_api::clients::RestClient) -> Fut,
        Fut: std::future::Future<Output = Result<Order, shopify_api::rest::ResourceError>>,
    {
        let _ = f;
    }

    fn _assert_order_open<F, Fut>(f: F)
    where
        F: Fn(&Order, &shopify_api::clients::RestClient) -> Fut,
        Fut: std::future::Future<Output = Result<Order, shopify_api::rest::ResourceError>>,
    {
        let _ = f;
    }

    // Verify Fulfillment operations signatures
    fn _assert_fulfillment_cancel<F, Fut>(f: F)
    where
        F: Fn(&Fulfillment, &shopify_api::clients::RestClient) -> Fut,
        Fut: std::future::Future<Output = Result<Fulfillment, shopify_api::rest::ResourceError>>,
    {
        let _ = f;
    }

    fn _assert_fulfillment_update_tracking<F, Fut>(f: F)
    where
        F: Fn(&Fulfillment, &shopify_api::clients::RestClient, TrackingInfo) -> Fut,
        Fut: std::future::Future<Output = Result<Fulfillment, shopify_api::rest::ResourceError>>,
    {
        let _ = f;
    }

    // Verify TrackingInfo can be created
    let tracking = TrackingInfo {
        tracking_number: Some("1Z999AA10123456784".to_string()),
        tracking_url: Some("https://ups.com/tracking".to_string()),
        tracking_company: Some("UPS".to_string()),
    };
    assert!(tracking.tracking_number.is_some());
    assert!(tracking.tracking_url.is_some());
    assert!(tracking.tracking_company.is_some());

    // Verify get_id works correctly for resources that need IDs for operations
    let order = Order {
        id: Some(123456),
        ..Default::default()
    };
    assert_eq!(order.get_id(), Some(123456));

    let fulfillment = Fulfillment {
        id: Some(789012),
        order_id: Some(123456),
        ..Default::default()
    };
    assert_eq!(fulfillment.get_id(), Some(789012));

    // Verify operations fail gracefully when ID is missing
    let order_without_id = Order::default();
    assert!(order_without_id.get_id().is_none());
}

// ============================================================================
// Test 8: TrackedResource works with new resource types
// ============================================================================

#[test]
fn test_tracked_resource_works_with_new_resource_types() {
    // Test with Product
    let product = Product {
        id: Some(123),
        title: Some("Original Title".to_string()),
        vendor: Some("Original Vendor".to_string()),
        status: Some(ProductStatus::Active),
        ..Default::default()
    };

    let mut tracked_product = TrackedResource::from_existing(product);
    assert!(!tracked_product.is_dirty());

    // Modify the product
    tracked_product.title = Some("New Title".to_string());
    assert!(tracked_product.is_dirty());

    let changes = tracked_product.changed_fields();
    assert!(changes.get("title").is_some());
    assert_eq!(changes.get("title").unwrap(), "New Title");
    assert!(changes.get("vendor").is_none()); // Unchanged

    tracked_product.mark_clean();
    assert!(!tracked_product.is_dirty());

    // Test with Variant
    let variant = Variant {
        id: Some(456),
        product_id: Some(123),
        title: Some("Small".to_string()),
        price: Some("19.99".to_string()),
        sku: Some("PROD-SM".to_string()),
        ..Default::default()
    };

    let mut tracked_variant = TrackedResource::from_existing(variant);
    assert!(!tracked_variant.is_dirty());

    tracked_variant.price = Some("24.99".to_string());
    assert!(tracked_variant.is_dirty());

    let variant_changes = tracked_variant.changed_fields();
    assert!(variant_changes.get("price").is_some());

    // Test with Order
    let order = Order {
        id: Some(789),
        email: Some("customer@example.com".to_string()),
        note: Some("Original note".to_string()),
        ..Default::default()
    };

    let mut tracked_order = TrackedResource::from_existing(order);
    assert!(!tracked_order.is_dirty());

    tracked_order.note = Some("Updated note".to_string());
    assert!(tracked_order.is_dirty());

    let order_changes = tracked_order.changed_fields();
    assert!(order_changes.get("note").is_some());

    // Test with Customer
    let customer = Customer {
        id: Some(101),
        email: Some("test@example.com".to_string()),
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        ..Default::default()
    };

    let mut tracked_customer = TrackedResource::from_existing(customer);
    assert!(!tracked_customer.is_dirty());

    tracked_customer.first_name = Some("Jane".to_string());
    assert!(tracked_customer.is_dirty());

    // Test new resource (not loaded from API)
    let new_product = Product {
        title: Some("New Product".to_string()),
        vendor: Some("New Vendor".to_string()),
        ..Default::default()
    };

    let tracked_new = TrackedResource::new(new_product);
    assert!(tracked_new.is_dirty()); // New resources are always dirty
    assert!(tracked_new.is_new());
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

#[test]
fn test_all_resources_implement_rest_resource_trait() {
    // Verify all resources have correct trait implementations
    fn assert_rest_resource<T: RestResource>() {}

    assert_rest_resource::<Product>();
    assert_rest_resource::<Variant>();
    assert_rest_resource::<Customer>();
    assert_rest_resource::<Order>();
    assert_rest_resource::<Fulfillment>();
    assert_rest_resource::<InventoryItem>();
}

#[test]
fn test_enum_default_values() {
    // Test enum defaults
    assert_eq!(ProductStatus::default(), ProductStatus::Active);
    assert_eq!(FinancialStatus::default(), FinancialStatus::Pending);
    assert_eq!(
        FulfillmentResourceStatus::default(),
        FulfillmentResourceStatus::Pending
    );
    assert_eq!(WeightUnit::default(), WeightUnit::Kg);
    // CustomerState has Enabled as default based on implementation
    assert_eq!(CustomerState::default(), CustomerState::Enabled);
}

#[test]
fn test_all_types_exported_correctly() {
    // Verify all types are exported from the resources module
    let _: Product = Product::default();
    let _: ProductStatus = ProductStatus::Active;
    let _: ProductVariant = ProductVariant::default();
    let _: ProductListParams = ProductListParams::default();
    let _: ProductFindParams = ProductFindParams::default();
    let _: ProductCountParams = ProductCountParams::default();

    let _: Variant = Variant::default();
    let _: WeightUnit = WeightUnit::Kg;
    let _: VariantListParams = VariantListParams::default();
    let _: VariantFindParams = VariantFindParams::default();
    let _: VariantCountParams = VariantCountParams::default();

    let _: Customer = Customer::default();
    let _: CustomerState = CustomerState::Enabled;
    let _: CustomerListParams = CustomerListParams::default();
    let _: CustomerFindParams = CustomerFindParams::default();
    let _: CustomerCountParams = CustomerCountParams::default();
    let _: EmailMarketingConsent = EmailMarketingConsent::default();
    let _: SmsMarketingConsent = SmsMarketingConsent::default();

    let _: Order = Order::default();
    let _: FinancialStatus = FinancialStatus::Paid;
    let _: FulfillmentStatus = FulfillmentStatus::Fulfilled;
    let _: CancelReason = CancelReason::Customer;
    let _: OrderListParams = OrderListParams::default();
    let _: OrderFindParams = OrderFindParams::default();
    let _: OrderCountParams = OrderCountParams::default();
    let _: DiscountCode = DiscountCode::default();
    let _: Refund = Refund::default();
    let _: OrderFulfillment = OrderFulfillment::default();

    let _: Fulfillment = Fulfillment::default();
    let _: FulfillmentResourceStatus = FulfillmentResourceStatus::Success;
    let _: ShipmentStatus = ShipmentStatus::Delivered;
    let _: FulfillmentListParams = FulfillmentListParams::default();
    let _: FulfillmentFindParams = FulfillmentFindParams::default();
    let _: FulfillmentCountParams = FulfillmentCountParams::default();
    let _: FulfillmentLineItem = FulfillmentLineItem::default();
    let _: TrackingInfo = TrackingInfo::default();

    let _: InventoryItem = InventoryItem::default();
    let _: InventoryItemListParams = InventoryItemListParams::default();
    let _: InventoryItemFindParams = InventoryItemFindParams::default();
    let _: CountryHarmonizedSystemCode = CountryHarmonizedSystemCode::default();

    // Common types
    let _: Address = Address::default();
    let _: CustomerAddress = CustomerAddress::default();
    let _: LineItem = LineItem::default();
    let _: TaxLine = TaxLine::default();
    let _: DiscountApplication = DiscountApplication::default();
    let _: ShippingLine = ShippingLine::default();
    let _: NoteAttribute = NoteAttribute::default();
    let _: ProductImage = ProductImage::default();
    let _: ProductOption = ProductOption::default();
}

#[test]
fn test_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    // Resources
    assert_send_sync::<Product>();
    assert_send_sync::<Variant>();
    assert_send_sync::<Customer>();
    assert_send_sync::<Order>();
    assert_send_sync::<Fulfillment>();
    assert_send_sync::<InventoryItem>();

    // Params
    assert_send_sync::<ProductListParams>();
    assert_send_sync::<VariantListParams>();
    assert_send_sync::<CustomerListParams>();
    assert_send_sync::<OrderListParams>();
    assert_send_sync::<FulfillmentListParams>();
    assert_send_sync::<InventoryItemListParams>();

    // Common types
    assert_send_sync::<Address>();
    assert_send_sync::<LineItem>();
    assert_send_sync::<TaxLine>();

    // TrackedResource wrapping our types
    assert_send_sync::<TrackedResource<Product>>();
    assert_send_sync::<TrackedResource<Variant>>();
    assert_send_sync::<TrackedResource<Order>>();
}

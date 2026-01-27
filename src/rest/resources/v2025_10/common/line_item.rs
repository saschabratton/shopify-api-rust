//! Line item and related types for orders.
//!
//! This module provides structs for order line items, tax lines, discount
//! applications, and shipping lines.

use serde::{Deserialize, Serialize};

/// A tax applied to an order or line item.
///
/// Contains information about a specific tax charge including the title,
/// price, and rate.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::rest::resources::v2025_10::common::TaxLine;
///
/// let tax = TaxLine {
///     title: Some("State Tax".to_string()),
///     price: Some("5.99".to_string()),
///     rate: Some(0.08),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TaxLine {
    /// The name of the tax (e.g., "State Tax", "VAT").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The tax amount as a string to preserve decimal precision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The tax rate as a decimal (e.g., 0.08 for 8%).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,

    /// The price in multiple currencies.
    /// Uses `serde_json::Value` for flexibility as this is a complex nested structure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_set: Option<serde_json::Value>,
}

/// A discount applied to an order.
///
/// Represents how a discount was applied, including the type, value,
/// and targeting information.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountApplication {
    /// The type of discount (e.g., `discount_code`, `manual`, `script`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub discount_type: Option<String>,

    /// The value of the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The type of value (e.g., `percentage`, `fixed_amount`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,

    /// How the discount is allocated (e.g., "across", "each", "one").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation_method: Option<String>,

    /// How line items are selected for the discount (e.g., "all", "entitled", "explicit").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_selection: Option<String>,

    /// The type of line the discount applies to (e.g., `line_item`, `shipping_line`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,

    /// The discount code used (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The title of the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// A description of the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// An allocated discount amount on a line item.
///
/// Links a discount application to the specific amount allocated
/// to a line item.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct DiscountAllocation {
    /// The allocated discount amount as a string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// The index of the discount application in the order's `discount_applications` array.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_application_index: Option<i64>,

    /// The allocated amount in multiple currencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_set: Option<serde_json::Value>,
}

/// A custom property on a line item.
///
/// Used for customization options like engraving, gift messages, etc.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LineItemProperty {
    /// The name of the property.
    pub name: String,

    /// The value of the property.
    pub value: String,
}

/// A line item in an order.
///
/// Represents a single product or variant being purchased, including
/// quantity, price, and applied discounts/taxes.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::rest::resources::v2025_10::common::LineItem;
///
/// let line_item = LineItem {
///     id: Some(123456),
///     variant_id: Some(789012),
///     product_id: Some(345678),
///     title: Some("Cool T-Shirt".to_string()),
///     quantity: Some(2),
///     price: Some("29.99".to_string()),
///     sku: Some("TSHIRT-BLUE-M".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LineItem {
    /// The unique identifier of the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The ID of the product variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<u64>,

    /// The ID of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_id: Option<u64>,

    /// The title of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The quantity of items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<i64>,

    /// The price per item as a string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The SKU of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,

    /// The title of the variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_title: Option<String>,

    /// The vendor of the product.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,

    /// The weight of the item in grams.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grams: Option<i64>,

    /// Whether the item is taxable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxable: Option<bool>,

    /// Whether the item is a gift card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gift_card: Option<bool>,

    /// Whether the item requires shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_shipping: Option<bool>,

    /// The name of the product and variant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Custom properties on the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<LineItemProperty>>,

    /// Tax lines applied to this line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_lines: Option<Vec<TaxLine>>,

    /// Discount allocations for this line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_allocations: Option<Vec<DiscountAllocation>>,

    /// The total discount amount on this line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_discount: Option<String>,

    /// The fulfillment service for the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_service: Option<String>,

    /// The fulfillment status of the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_status: Option<String>,

    /// Whether the product still exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_exists: Option<bool>,

    /// The quantity that can still be fulfilled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillable_quantity: Option<i64>,

    /// The total duties on the line item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duties: Option<Vec<serde_json::Value>>,

    /// The price in multiple currencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_set: Option<serde_json::Value>,

    /// The total discount in multiple currencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_discount_set: Option<serde_json::Value>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_graphql_api_id: Option<String>,
}

/// A shipping line on an order.
///
/// Represents a shipping method applied to the order, including
/// the price and carrier information.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ShippingLine {
    /// The unique identifier of the shipping line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The title of the shipping method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The price of shipping as a string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The code for the shipping method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// The source of the shipping rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// The phone number for the shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// The carrier identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carrier_identifier: Option<String>,

    /// Tax lines for the shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_lines: Option<Vec<TaxLine>>,

    /// The shipping price in multiple currencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_set: Option<serde_json::Value>,

    /// Discount allocations for shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discount_allocations: Option<Vec<DiscountAllocation>>,

    /// The discounted price of shipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discounted_price: Option<String>,

    /// The discounted price in multiple currencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discounted_price_set: Option<serde_json::Value>,
}

/// A custom note attribute on an order.
///
/// Used for storing additional order metadata as key-value pairs.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct NoteAttribute {
    /// The attribute name.
    pub name: String,

    /// The attribute value.
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tax_line_serialization() {
        let tax_line = TaxLine {
            title: Some("State Tax".to_string()),
            price: Some("8.50".to_string()),
            rate: Some(0.085),
            price_set: None,
        };

        let json = serde_json::to_string(&tax_line).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["title"], "State Tax");
        assert_eq!(parsed["price"], "8.50");
        assert_eq!(parsed["rate"], 0.085);
        assert!(parsed.get("price_set").is_none());
    }

    #[test]
    fn test_discount_application_with_type_rename() {
        let discount = DiscountApplication {
            discount_type: Some("discount_code".to_string()),
            value: Some("10.00".to_string()),
            value_type: Some("fixed_amount".to_string()),
            allocation_method: Some("across".to_string()),
            target_selection: Some("all".to_string()),
            target_type: Some("line_item".to_string()),
            code: Some("SAVE10".to_string()),
            title: Some("$10 Off".to_string()),
            description: Some("Save $10 on your order".to_string()),
        };

        let json = serde_json::to_string(&discount).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify the type field is renamed from discount_type
        assert_eq!(parsed["type"], "discount_code");
        assert_eq!(parsed["value"], "10.00");
        assert_eq!(parsed["code"], "SAVE10");

        // Verify deserialization works with the renamed field
        let json_with_type = r#"{
            "type": "manual",
            "value": "5.00",
            "value_type": "percentage"
        }"#;
        let deserialized: DiscountApplication = serde_json::from_str(json_with_type).unwrap();
        assert_eq!(deserialized.discount_type, Some("manual".to_string()));
    }

    #[test]
    fn test_line_item_with_nested_tax_lines() {
        let line_item = LineItem {
            id: Some(1234567890),
            variant_id: Some(9876543210),
            product_id: Some(5555555555),
            title: Some("Awesome Product".to_string()),
            quantity: Some(3),
            price: Some("49.99".to_string()),
            sku: Some("AWESOME-001".to_string()),
            variant_title: Some("Large / Blue".to_string()),
            vendor: Some("Cool Vendor".to_string()),
            grams: Some(500),
            taxable: Some(true),
            gift_card: Some(false),
            requires_shipping: Some(true),
            name: Some("Awesome Product - Large / Blue".to_string()),
            tax_lines: Some(vec![
                TaxLine {
                    title: Some("State Tax".to_string()),
                    price: Some("4.00".to_string()),
                    rate: Some(0.08),
                    price_set: None,
                },
                TaxLine {
                    title: Some("City Tax".to_string()),
                    price: Some("1.25".to_string()),
                    rate: Some(0.025),
                    price_set: None,
                },
            ]),
            discount_allocations: Some(vec![DiscountAllocation {
                amount: Some("5.00".to_string()),
                discount_application_index: Some(0),
                amount_set: None,
            }]),
            total_discount: Some("5.00".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&line_item).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["id"], 1234567890u64);
        assert_eq!(parsed["title"], "Awesome Product");
        assert_eq!(parsed["quantity"], 3);
        assert_eq!(parsed["price"], "49.99");
        assert!(parsed["taxable"].as_bool().unwrap());
        assert!(!parsed["gift_card"].as_bool().unwrap());

        // Check nested tax_lines
        let tax_lines = parsed["tax_lines"].as_array().unwrap();
        assert_eq!(tax_lines.len(), 2);
        assert_eq!(tax_lines[0]["title"], "State Tax");
        assert_eq!(tax_lines[1]["title"], "City Tax");

        // Check discount allocations
        let discount_allocations = parsed["discount_allocations"].as_array().unwrap();
        assert_eq!(discount_allocations.len(), 1);
        assert_eq!(discount_allocations[0]["amount"], "5.00");
    }

    #[test]
    fn test_line_item_deserialization() {
        let json = r#"{
            "id": 11111,
            "variant_id": 22222,
            "product_id": 33333,
            "title": "Test Product",
            "quantity": 1,
            "price": "19.99",
            "taxable": true,
            "tax_lines": [
                {
                    "title": "GST",
                    "price": "1.00",
                    "rate": 0.05
                }
            ]
        }"#;

        let line_item: LineItem = serde_json::from_str(json).unwrap();

        assert_eq!(line_item.id, Some(11111));
        assert_eq!(line_item.title, Some("Test Product".to_string()));
        assert_eq!(line_item.quantity, Some(1));
        assert_eq!(line_item.price, Some("19.99".to_string()));

        let tax_lines = line_item.tax_lines.unwrap();
        assert_eq!(tax_lines.len(), 1);
        assert_eq!(tax_lines[0].title, Some("GST".to_string()));
        assert_eq!(tax_lines[0].rate, Some(0.05));
    }

    #[test]
    fn test_shipping_line_serialization() {
        let shipping = ShippingLine {
            id: Some(9999),
            title: Some("Standard Shipping".to_string()),
            price: Some("5.99".to_string()),
            code: Some("STANDARD".to_string()),
            source: Some("shopify".to_string()),
            carrier_identifier: Some("ups".to_string()),
            tax_lines: Some(vec![TaxLine {
                title: Some("Shipping Tax".to_string()),
                price: Some("0.48".to_string()),
                rate: Some(0.08),
                price_set: None,
            }]),
            ..Default::default()
        };

        let json = serde_json::to_string(&shipping).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["title"], "Standard Shipping");
        assert_eq!(parsed["price"], "5.99");
        assert_eq!(parsed["tax_lines"][0]["title"], "Shipping Tax");
    }

    #[test]
    fn test_note_attribute_serialization() {
        let note = NoteAttribute {
            name: "gift_message".to_string(),
            value: "Happy Birthday!".to_string(),
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["name"], "gift_message");
        assert_eq!(parsed["value"], "Happy Birthday!");
    }

    #[test]
    fn test_line_item_property_serialization() {
        let property = LineItemProperty {
            name: "engraving".to_string(),
            value: "J + M".to_string(),
        };

        let json = serde_json::to_string(&property).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["name"], "engraving");
        assert_eq!(parsed["value"], "J + M");
    }
}

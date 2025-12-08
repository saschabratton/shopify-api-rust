//! Address types for orders and customers.
//!
//! This module provides address structs used for billing/shipping addresses
//! in orders and customer addresses.

use serde::{Deserialize, Serialize};

/// A physical address used for billing or shipping.
///
/// This struct is used for `billing_address` and `shipping_address` fields
/// in orders. All fields are optional to support partial address data.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::Address;
///
/// let address = Address {
///     first_name: Some("John".to_string()),
///     last_name: Some("Doe".to_string()),
///     address1: Some("123 Main St".to_string()),
///     city: Some("New York".to_string()),
///     province: Some("New York".to_string()),
///     province_code: Some("NY".to_string()),
///     country: Some("United States".to_string()),
///     country_code: Some("US".to_string()),
///     zip: Some("10001".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Address {
    /// The first name of the person at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// The last name of the person at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// The company name at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,

    /// The first line of the address (street address).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,

    /// The second line of the address (apartment, suite, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,

    /// The city, town, or village.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// The province, state, or region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province: Option<String>,

    /// The two-letter province or state code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province_code: Option<String>,

    /// The country name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// The two-letter country code (ISO 3166-1 alpha-2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,

    /// The postal or ZIP code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,

    /// The phone number at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// The full name of the person at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The latitude of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,

    /// The longitude of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

/// An address associated with a customer.
///
/// Extends the base `Address` with customer-specific fields like `id`,
/// `customer_id`, and `default` flag.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::CustomerAddress;
///
/// let address = CustomerAddress {
///     id: Some(123456),
///     customer_id: Some(789012),
///     default: Some(true),
///     first_name: Some("John".to_string()),
///     last_name: Some("Doe".to_string()),
///     address1: Some("123 Main St".to_string()),
///     city: Some("New York".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CustomerAddress {
    /// The unique identifier of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The ID of the customer this address belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<u64>,

    /// Whether this is the customer's default address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,

    /// The first name of the person at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// The last name of the person at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// The company name at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,

    /// The first line of the address (street address).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,

    /// The second line of the address (apartment, suite, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,

    /// The city, town, or village.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// The province, state, or region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province: Option<String>,

    /// The two-letter province or state code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub province_code: Option<String>,

    /// The country name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// The two-letter country code (ISO 3166-1 alpha-2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,

    /// The postal or ZIP code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,

    /// The phone number at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// The full name of the person at the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The latitude of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,

    /// The longitude of the address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_serialization_with_all_fields() {
        let address = Address {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            company: Some("Acme Inc".to_string()),
            address1: Some("123 Main St".to_string()),
            address2: Some("Apt 4B".to_string()),
            city: Some("New York".to_string()),
            province: Some("New York".to_string()),
            province_code: Some("NY".to_string()),
            country: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            zip: Some("10001".to_string()),
            phone: Some("+1-555-555-5555".to_string()),
            name: Some("John Doe".to_string()),
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
        };

        let json = serde_json::to_string(&address).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["first_name"], "John");
        assert_eq!(parsed["last_name"], "Doe");
        assert_eq!(parsed["company"], "Acme Inc");
        assert_eq!(parsed["address1"], "123 Main St");
        assert_eq!(parsed["address2"], "Apt 4B");
        assert_eq!(parsed["city"], "New York");
        assert_eq!(parsed["province"], "New York");
        assert_eq!(parsed["province_code"], "NY");
        assert_eq!(parsed["country"], "United States");
        assert_eq!(parsed["country_code"], "US");
        assert_eq!(parsed["zip"], "10001");
        assert_eq!(parsed["phone"], "+1-555-555-5555");
        assert_eq!(parsed["name"], "John Doe");
        assert_eq!(parsed["latitude"], 40.7128);
        assert_eq!(parsed["longitude"], -74.0060);
    }

    #[test]
    fn test_address_serialization_with_optional_fields_omitted() {
        let address = Address {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            address1: Some("456 Oak Ave".to_string()),
            city: Some("Los Angeles".to_string()),
            country: Some("United States".to_string()),
            zip: Some("90001".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&address).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Present fields
        assert_eq!(parsed["first_name"], "Jane");
        assert_eq!(parsed["last_name"], "Smith");
        assert_eq!(parsed["address1"], "456 Oak Ave");
        assert_eq!(parsed["city"], "Los Angeles");
        assert_eq!(parsed["country"], "United States");
        assert_eq!(parsed["zip"], "90001");

        // Omitted fields should not be present
        assert!(parsed.get("company").is_none());
        assert!(parsed.get("address2").is_none());
        assert!(parsed.get("province").is_none());
        assert!(parsed.get("province_code").is_none());
        assert!(parsed.get("country_code").is_none());
        assert!(parsed.get("phone").is_none());
        assert!(parsed.get("name").is_none());
        assert!(parsed.get("latitude").is_none());
        assert!(parsed.get("longitude").is_none());
    }

    #[test]
    fn test_customer_address_with_extended_fields() {
        let address = CustomerAddress {
            id: Some(123456789),
            customer_id: Some(987654321),
            default: Some(true),
            first_name: Some("Alice".to_string()),
            last_name: Some("Johnson".to_string()),
            address1: Some("789 Pine Rd".to_string()),
            city: Some("Chicago".to_string()),
            province: Some("Illinois".to_string()),
            province_code: Some("IL".to_string()),
            country: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            zip: Some("60601".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&address).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Customer-specific fields
        assert_eq!(parsed["id"], 123456789);
        assert_eq!(parsed["customer_id"], 987654321);
        assert_eq!(parsed["default"], true);

        // Base address fields
        assert_eq!(parsed["first_name"], "Alice");
        assert_eq!(parsed["last_name"], "Johnson");
        assert_eq!(parsed["city"], "Chicago");
    }

    #[test]
    fn test_address_deserialization() {
        let json = r#"{
            "first_name": "Bob",
            "last_name": "Williams",
            "address1": "321 Elm St",
            "city": "Seattle",
            "province": "Washington",
            "province_code": "WA",
            "country": "United States",
            "country_code": "US",
            "zip": "98101"
        }"#;

        let address: Address = serde_json::from_str(json).unwrap();

        assert_eq!(address.first_name, Some("Bob".to_string()));
        assert_eq!(address.last_name, Some("Williams".to_string()));
        assert_eq!(address.address1, Some("321 Elm St".to_string()));
        assert_eq!(address.city, Some("Seattle".to_string()));
        assert_eq!(address.province, Some("Washington".to_string()));
        assert_eq!(address.province_code, Some("WA".to_string()));
        assert_eq!(address.country, Some("United States".to_string()));
        assert_eq!(address.country_code, Some("US".to_string()));
        assert_eq!(address.zip, Some("98101".to_string()));
        // Optional fields not in JSON should be None
        assert_eq!(address.company, None);
        assert_eq!(address.latitude, None);
    }

    #[test]
    fn test_customer_address_deserialization() {
        let json = r#"{
            "id": 5551234,
            "customer_id": 7778899,
            "default": false,
            "first_name": "Carol",
            "last_name": "Davis",
            "address1": "100 Market St",
            "city": "San Francisco",
            "province": "California",
            "province_code": "CA",
            "country": "United States",
            "country_code": "US",
            "zip": "94102"
        }"#;

        let address: CustomerAddress = serde_json::from_str(json).unwrap();

        assert_eq!(address.id, Some(5551234));
        assert_eq!(address.customer_id, Some(7778899));
        assert_eq!(address.default, Some(false));
        assert_eq!(address.first_name, Some("Carol".to_string()));
        assert_eq!(address.city, Some("San Francisco".to_string()));
    }
}

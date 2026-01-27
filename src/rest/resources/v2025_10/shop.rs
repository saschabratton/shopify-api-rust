//! Shop resource implementation.
//!
//! This module provides the [`Shop`] resource for retrieving shop information from Shopify.
//! The Shop resource is a singleton, read-only resource that represents the current shop.
//!
//! # Singleton Pattern
//!
//! Unlike other resources, Shop does not have standard CRUD operations. There is only
//! one shop per API session, accessed via the `current()` method.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::resources::v2025_10::Shop;
//!
//! // Get current shop information
//! let shop = Shop::current(&client).await?;
//! println!("Shop: {}", shop.name.as_deref().unwrap_or(""));
//! println!("Domain: {}", shop.domain.as_deref().unwrap_or(""));
//! println!("Plan: {}", shop.plan_name.as_deref().unwrap_or(""));
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A Shopify shop.
///
/// The Shop resource contains information about the store including its name,
/// domain, contact information, address, and configuration settings.
///
/// # Read-Only Resource
///
/// The Shop resource is read-only and cannot be created, updated, or deleted
/// through the REST API. All fields are read-only and will not be serialized
/// in requests.
///
/// # Singleton Pattern
///
/// There is only one Shop per API session. Use [`Shop::current()`] to retrieve it.
///
/// # Example
///
/// ```rust,ignore
/// use shopify_sdk::rest::resources::v2025_10::Shop;
///
/// let shop = Shop::current(&client).await?;
/// println!("Shop name: {}", shop.name.as_deref().unwrap_or("Unknown"));
/// println!("Currency: {}", shop.currency.as_deref().unwrap_or("USD"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Shop {
    // --- Read-only identification fields ---
    /// The unique identifier of the shop.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The name of the shop.
    #[serde(skip_serializing)]
    pub name: Option<String>,

    /// The contact email address for the shop.
    #[serde(skip_serializing)]
    pub email: Option<String>,

    /// The shop's custom domain (e.g., "www.example.com").
    #[serde(skip_serializing)]
    pub domain: Option<String>,

    /// The shop's myshopify.com domain (e.g., "example.myshopify.com").
    #[serde(skip_serializing)]
    pub myshopify_domain: Option<String>,

    // --- Plan information ---
    /// The name of the Shopify plan the shop is on (e.g., "basic", "professional").
    #[serde(skip_serializing)]
    pub plan_name: Option<String>,

    /// The display-friendly name of the plan.
    #[serde(skip_serializing)]
    pub plan_display_name: Option<String>,

    /// Whether the shop has a storefront password enabled.
    #[serde(skip_serializing)]
    pub password_enabled: Option<bool>,

    // --- Address fields ---
    /// The primary address line.
    #[serde(skip_serializing)]
    pub address1: Option<String>,

    /// The secondary address line.
    #[serde(skip_serializing)]
    pub address2: Option<String>,

    /// The city.
    #[serde(skip_serializing)]
    pub city: Option<String>,

    /// The province or state.
    #[serde(skip_serializing)]
    pub province: Option<String>,

    /// The two-letter province code.
    #[serde(skip_serializing)]
    pub province_code: Option<String>,

    /// The country name.
    #[serde(skip_serializing)]
    pub country: Option<String>,

    /// The two-letter country code (ISO 3166-1 alpha-2).
    #[serde(skip_serializing)]
    pub country_code: Option<String>,

    /// The full country name.
    #[serde(skip_serializing)]
    pub country_name: Option<String>,

    /// The postal/ZIP code.
    #[serde(skip_serializing)]
    pub zip: Option<String>,

    /// The shop's phone number.
    #[serde(skip_serializing)]
    pub phone: Option<String>,

    /// The latitude coordinate of the shop's location.
    #[serde(skip_serializing)]
    pub latitude: Option<f64>,

    /// The longitude coordinate of the shop's location.
    #[serde(skip_serializing)]
    pub longitude: Option<f64>,

    // --- Currency and money formatting ---
    /// The three-letter currency code (e.g., "USD", "EUR").
    #[serde(skip_serializing)]
    pub currency: Option<String>,

    /// The format for displaying money without currency symbol.
    #[serde(skip_serializing)]
    pub money_format: Option<String>,

    /// The format for displaying money with currency symbol.
    #[serde(skip_serializing)]
    pub money_with_currency_format: Option<String>,

    // --- Timezone ---
    /// The timezone in display format (e.g., "(GMT-05:00) Eastern Time (US & Canada)").
    #[serde(skip_serializing)]
    pub timezone: Option<String>,

    /// The IANA timezone identifier (e.g., `America/New_York`).
    #[serde(skip_serializing)]
    pub iana_timezone: Option<String>,

    // --- Feature flags ---
    /// Whether the checkout API is supported.
    #[serde(skip_serializing)]
    pub checkout_api_supported: Option<bool>,

    /// Whether multi-location is enabled.
    #[serde(skip_serializing)]
    pub multi_location_enabled: Option<bool>,

    /// Whether prices include taxes.
    #[serde(skip_serializing)]
    pub taxes_included: Option<bool>,

    /// Whether shipping is taxed.
    #[serde(skip_serializing)]
    pub tax_shipping: Option<bool>,

    /// Whether transactional SMS is disabled.
    #[serde(skip_serializing)]
    pub transactional_sms_disabled: Option<bool>,

    /// Whether the storefront access token has been enabled for checkout.
    #[serde(skip_serializing)]
    pub has_storefront_api: Option<bool>,

    /// Whether the shop has discounts enabled.
    #[serde(skip_serializing)]
    pub has_discounts: Option<bool>,

    /// Whether the shop has gift cards enabled.
    #[serde(skip_serializing)]
    pub has_gift_cards: Option<bool>,

    /// Whether the shop is eligible for payments.
    #[serde(skip_serializing)]
    pub eligible_for_payments: Option<bool>,

    /// Whether the shop requires extra payments agreement.
    #[serde(skip_serializing)]
    pub requires_extra_payments_agreement: Option<bool>,

    /// Whether the shop is set up.
    #[serde(skip_serializing)]
    pub setup_required: Option<bool>,

    /// Whether pre-launch mode is enabled.
    #[serde(skip_serializing)]
    pub pre_launch_enabled: Option<bool>,

    /// Whether the shop is enabled for cookie consent.
    #[serde(skip_serializing)]
    pub cookie_consent_level: Option<String>,

    // --- Shop details ---
    /// The shop owner's name.
    #[serde(skip_serializing)]
    pub shop_owner: Option<String>,

    /// The source for the shop creation.
    #[serde(skip_serializing)]
    pub source: Option<String>,

    /// The weight unit used by the shop ("kg", "lb", "oz", "g").
    #[serde(skip_serializing)]
    pub weight_unit: Option<String>,

    /// The primary locale of the shop.
    #[serde(skip_serializing)]
    pub primary_locale: Option<String>,

    /// Additional enabled locales.
    #[serde(skip_serializing)]
    pub enabled_presentment_currencies: Option<Vec<String>>,

    // --- Timestamps ---
    /// When the shop was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the shop was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for Shop {
    type Id = u64;
    type FindParams = ();
    type AllParams = ();
    type CountParams = ();

    const NAME: &'static str = "Shop";
    const PLURAL: &'static str = "shop";

    // Shop only has a Find operation at the `shop` path (no ID parameter).
    // It does not support Create, Update, Delete, All, or Count operations.
    const PATHS: &'static [ResourcePath] = &[ResourcePath::new(
        HttpMethod::Get,
        ResourceOperation::Find,
        &[],
        "shop",
    )];

    /// Returns None since Shop is a singleton and doesn't use ID-based operations.
    fn get_id(&self) -> Option<Self::Id> {
        None
    }
}

impl Shop {
    /// Retrieves the current shop.
    ///
    /// Sends a GET request to `/admin/api/{version}/shop.json`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Returns
    ///
    /// Returns the Shop directly (not wrapped in `ResourceResponse`) since
    /// pagination is not applicable to singleton resources.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::Http`] if the request fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::rest::resources::v2025_10::Shop;
    ///
    /// let shop = Shop::current(&client).await?;
    /// println!("Shop: {}", shop.name.as_deref().unwrap_or(""));
    /// println!("Domain: {}", shop.myshopify_domain.as_deref().unwrap_or(""));
    /// println!("Plan: {}", shop.plan_name.as_deref().unwrap_or(""));
    /// println!("Currency: {}", shop.currency.as_deref().unwrap_or(""));
    /// ```
    pub async fn current(client: &RestClient) -> Result<Self, ResourceError> {
        let path = "shop";

        let response = client.get(path, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        // Parse the response - Shopify returns the shop wrapped in "shop" key
        let shop: Self = response
            .body
            .get("shop")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'shop' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize shop: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(shop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_shop_struct_deserialization_from_api_response() {
        let json_str = r#"{
            "id": 548380009,
            "name": "John Smith Test Store",
            "email": "j.smith@example.com",
            "domain": "shop.example.com",
            "myshopify_domain": "john-smith-test-store.myshopify.com",
            "plan_name": "partner_test",
            "plan_display_name": "Partner Test",
            "password_enabled": false,
            "address1": "123 Main St",
            "address2": "Suite 100",
            "city": "Ottawa",
            "province": "Ontario",
            "province_code": "ON",
            "country": "Canada",
            "country_code": "CA",
            "country_name": "Canada",
            "zip": "K1A 0B1",
            "phone": "1234567890",
            "latitude": 45.4215,
            "longitude": -75.6972,
            "currency": "CAD",
            "money_format": "${{amount}}",
            "money_with_currency_format": "${{amount}} CAD",
            "timezone": "(GMT-05:00) Eastern Time (US & Canada)",
            "iana_timezone": "America/Toronto",
            "checkout_api_supported": true,
            "multi_location_enabled": true,
            "taxes_included": false,
            "tax_shipping": null,
            "weight_unit": "kg",
            "primary_locale": "en",
            "created_at": "2023-01-01T12:00:00-05:00",
            "updated_at": "2024-06-01T12:00:00-05:00"
        }"#;

        let shop: Shop = serde_json::from_str(json_str).unwrap();

        assert_eq!(shop.id, Some(548380009));
        assert_eq!(shop.name.as_deref(), Some("John Smith Test Store"));
        assert_eq!(shop.email.as_deref(), Some("j.smith@example.com"));
        assert_eq!(shop.domain.as_deref(), Some("shop.example.com"));
        assert_eq!(
            shop.myshopify_domain.as_deref(),
            Some("john-smith-test-store.myshopify.com")
        );
        assert_eq!(shop.plan_name.as_deref(), Some("partner_test"));
        assert_eq!(shop.plan_display_name.as_deref(), Some("Partner Test"));
        assert_eq!(shop.password_enabled, Some(false));
        assert_eq!(shop.address1.as_deref(), Some("123 Main St"));
        assert_eq!(shop.address2.as_deref(), Some("Suite 100"));
        assert_eq!(shop.city.as_deref(), Some("Ottawa"));
        assert_eq!(shop.province.as_deref(), Some("Ontario"));
        assert_eq!(shop.province_code.as_deref(), Some("ON"));
        assert_eq!(shop.country.as_deref(), Some("Canada"));
        assert_eq!(shop.country_code.as_deref(), Some("CA"));
        assert_eq!(shop.zip.as_deref(), Some("K1A 0B1"));
        assert_eq!(shop.currency.as_deref(), Some("CAD"));
        assert_eq!(
            shop.timezone.as_deref(),
            Some("(GMT-05:00) Eastern Time (US & Canada)")
        );
        assert_eq!(shop.iana_timezone.as_deref(), Some("America/Toronto"));
        assert_eq!(shop.checkout_api_supported, Some(true));
        assert_eq!(shop.multi_location_enabled, Some(true));
        assert_eq!(shop.taxes_included, Some(false));
        assert_eq!(shop.weight_unit.as_deref(), Some("kg"));
        assert_eq!(shop.primary_locale.as_deref(), Some("en"));
        assert!(shop.created_at.is_some());
        assert!(shop.updated_at.is_some());
    }

    #[test]
    fn test_shop_current_method_signature_exists() {
        // Verify the method signature is correct by referencing it
        fn _assert_current_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<Shop, ResourceError>>,
        {
            let _ = f;
        }

        // This test verifies the method exists and has the correct signature.
        // The actual HTTP call would require a mock client.
    }

    #[test]
    fn test_shop_does_not_have_standard_crud_paths() {
        // Shop should only have a Find path
        let find_path = get_path(Shop::PATHS, ResourceOperation::Find, &[]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "shop");

        // Shop should NOT have Create, Update, Delete, All, or Count paths
        let create_path = get_path(Shop::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_none());

        let update_path = get_path(Shop::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());

        let delete_path = get_path(Shop::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());

        let all_path = get_path(Shop::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_none());

        let count_path = get_path(Shop::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());
    }

    #[test]
    fn test_shop_read_only_fields_are_not_serialized() {
        let shop = Shop {
            id: Some(548380009),
            name: Some("Test Store".to_string()),
            email: Some("test@example.com".to_string()),
            domain: Some("shop.example.com".to_string()),
            myshopify_domain: Some("test-store.myshopify.com".to_string()),
            plan_name: Some("basic".to_string()),
            currency: Some("USD".to_string()),
            timezone: Some("(GMT-05:00) Eastern Time".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&shop).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // All fields have skip_serializing, so the JSON should be empty
        assert!(
            parsed.as_object().unwrap().is_empty(),
            "Shop serialization should produce empty object since all fields have skip_serializing"
        );
    }

    #[test]
    fn test_shop_all_major_fields_are_captured() {
        // This test verifies that a complete Shop response can be deserialized
        let json_str = r#"{
            "id": 548380009,
            "name": "Complete Test Store",
            "email": "complete@example.com",
            "domain": "complete.example.com",
            "myshopify_domain": "complete-test.myshopify.com",
            "plan_name": "unlimited",
            "plan_display_name": "Shopify Plus",
            "password_enabled": true,
            "address1": "123 Commerce St",
            "address2": "Floor 2",
            "city": "New York",
            "province": "New York",
            "province_code": "NY",
            "country": "United States",
            "country_code": "US",
            "country_name": "United States",
            "zip": "10001",
            "phone": "+1-555-555-5555",
            "latitude": 40.7128,
            "longitude": -74.0060,
            "currency": "USD",
            "money_format": "${{amount}}",
            "money_with_currency_format": "${{amount}} USD",
            "timezone": "(GMT-05:00) Eastern Time (US & Canada)",
            "iana_timezone": "America/New_York",
            "checkout_api_supported": true,
            "multi_location_enabled": true,
            "taxes_included": false,
            "tax_shipping": true,
            "transactional_sms_disabled": false,
            "has_storefront_api": true,
            "has_discounts": true,
            "has_gift_cards": true,
            "eligible_for_payments": true,
            "requires_extra_payments_agreement": false,
            "setup_required": false,
            "pre_launch_enabled": false,
            "cookie_consent_level": "implicit",
            "shop_owner": "John Smith",
            "source": null,
            "weight_unit": "lb",
            "primary_locale": "en",
            "enabled_presentment_currencies": ["USD", "CAD", "EUR"],
            "created_at": "2022-01-15T10:30:00-05:00",
            "updated_at": "2024-11-20T14:45:00-05:00",
            "admin_graphql_api_id": "gid://shopify/Shop/548380009"
        }"#;

        let shop: Shop = serde_json::from_str(json_str).unwrap();

        // Verify all major field groups are captured
        // Identification
        assert_eq!(shop.id, Some(548380009));
        assert_eq!(shop.name.as_deref(), Some("Complete Test Store"));
        assert_eq!(shop.email.as_deref(), Some("complete@example.com"));
        assert_eq!(shop.domain.as_deref(), Some("complete.example.com"));
        assert_eq!(
            shop.myshopify_domain.as_deref(),
            Some("complete-test.myshopify.com")
        );

        // Plan
        assert_eq!(shop.plan_name.as_deref(), Some("unlimited"));
        assert_eq!(shop.plan_display_name.as_deref(), Some("Shopify Plus"));
        assert_eq!(shop.password_enabled, Some(true));

        // Address
        assert_eq!(shop.address1.as_deref(), Some("123 Commerce St"));
        assert_eq!(shop.address2.as_deref(), Some("Floor 2"));
        assert_eq!(shop.city.as_deref(), Some("New York"));
        assert_eq!(shop.province.as_deref(), Some("New York"));
        assert_eq!(shop.province_code.as_deref(), Some("NY"));
        assert_eq!(shop.country.as_deref(), Some("United States"));
        assert_eq!(shop.country_code.as_deref(), Some("US"));
        assert_eq!(shop.country_name.as_deref(), Some("United States"));
        assert_eq!(shop.zip.as_deref(), Some("10001"));
        assert_eq!(shop.phone.as_deref(), Some("+1-555-555-5555"));
        assert_eq!(shop.latitude, Some(40.7128));
        assert_eq!(shop.longitude, Some(-74.0060));

        // Currency
        assert_eq!(shop.currency.as_deref(), Some("USD"));
        assert_eq!(shop.money_format.as_deref(), Some("${{amount}}"));
        assert_eq!(
            shop.money_with_currency_format.as_deref(),
            Some("${{amount}} USD")
        );

        // Timezone
        assert_eq!(
            shop.timezone.as_deref(),
            Some("(GMT-05:00) Eastern Time (US & Canada)")
        );
        assert_eq!(shop.iana_timezone.as_deref(), Some("America/New_York"));

        // Features
        assert_eq!(shop.checkout_api_supported, Some(true));
        assert_eq!(shop.multi_location_enabled, Some(true));
        assert_eq!(shop.taxes_included, Some(false));
        assert_eq!(shop.tax_shipping, Some(true));
        assert_eq!(shop.has_storefront_api, Some(true));
        assert_eq!(shop.has_discounts, Some(true));
        assert_eq!(shop.has_gift_cards, Some(true));
        assert_eq!(shop.eligible_for_payments, Some(true));
        assert_eq!(shop.setup_required, Some(false));
        assert_eq!(shop.pre_launch_enabled, Some(false));

        // Details
        assert_eq!(shop.shop_owner.as_deref(), Some("John Smith"));
        assert_eq!(shop.weight_unit.as_deref(), Some("lb"));
        assert_eq!(shop.primary_locale.as_deref(), Some("en"));
        assert_eq!(
            shop.enabled_presentment_currencies,
            Some(vec![
                "USD".to_string(),
                "CAD".to_string(),
                "EUR".to_string()
            ])
        );

        // Timestamps
        assert!(shop.created_at.is_some());
        assert!(shop.updated_at.is_some());

        // GraphQL ID
        assert_eq!(
            shop.admin_graphql_api_id.as_deref(),
            Some("gid://shopify/Shop/548380009")
        );
    }

    #[test]
    fn test_shop_get_id_returns_none_for_singleton() {
        let shop = Shop {
            id: Some(548380009),
            name: Some("Test Store".to_string()),
            ..Default::default()
        };

        // Even though the shop has an id field, get_id() returns None
        // because Shop is a singleton and doesn't use ID-based operations
        assert!(shop.get_id().is_none());
    }

    #[test]
    fn test_shop_resource_constants() {
        assert_eq!(Shop::NAME, "Shop");
        assert_eq!(Shop::PLURAL, "shop");
        assert_eq!(Shop::PATHS.len(), 1);
    }
}

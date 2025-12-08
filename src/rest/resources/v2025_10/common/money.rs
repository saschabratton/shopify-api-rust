//! Money-related types for currency and pricing.
//!
//! This module provides types for handling money amounts in multiple currencies,
//! as used in Shopify's `price_set` fields.

use serde::{Deserialize, Serialize};

/// A money amount with currency information.
///
/// Used within `MoneySet` to represent amounts in specific currencies.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Money {
    /// The monetary amount as a string to preserve decimal precision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,

    /// The three-letter ISO 4217 currency code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
}

/// A set of money amounts in shop and presentment currencies.
///
/// Shopify returns `price_set` fields with both the shop's base currency
/// and the customer's presentment currency.
///
/// # Example
///
/// ```rust
/// use shopify_api::rest::resources::v2025_10::common::{MoneySet, Money};
///
/// let price_set = MoneySet {
///     shop_money: Some(Money {
///         amount: Some("19.99".to_string()),
///         currency_code: Some("USD".to_string()),
///     }),
///     presentment_money: Some(Money {
///         amount: Some("25.99".to_string()),
///         currency_code: Some("CAD".to_string()),
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MoneySet {
    /// The amount in the shop's base currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shop_money: Option<Money>,

    /// The amount in the customer's presentment currency.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentment_money: Option<Money>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_set_serialization() {
        let money_set = MoneySet {
            shop_money: Some(Money {
                amount: Some("99.99".to_string()),
                currency_code: Some("USD".to_string()),
            }),
            presentment_money: Some(Money {
                amount: Some("129.99".to_string()),
                currency_code: Some("CAD".to_string()),
            }),
        };

        let json = serde_json::to_string(&money_set).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["shop_money"]["amount"], "99.99");
        assert_eq!(parsed["shop_money"]["currency_code"], "USD");
        assert_eq!(parsed["presentment_money"]["amount"], "129.99");
        assert_eq!(parsed["presentment_money"]["currency_code"], "CAD");
    }

    #[test]
    fn test_money_set_deserialization() {
        let json = r#"{
            "shop_money": {
                "amount": "50.00",
                "currency_code": "EUR"
            },
            "presentment_money": {
                "amount": "55.00",
                "currency_code": "GBP"
            }
        }"#;

        let money_set: MoneySet = serde_json::from_str(json).unwrap();

        assert_eq!(
            money_set.shop_money.as_ref().unwrap().amount,
            Some("50.00".to_string())
        );
        assert_eq!(
            money_set.shop_money.as_ref().unwrap().currency_code,
            Some("EUR".to_string())
        );
    }
}

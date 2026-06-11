//! Billing-related types for charge resources.
//!
//! This module provides shared types used by billing resources:
//! - `ApplicationCharge` (one-time charges)
//! - `RecurringApplicationCharge` (subscription charges)
//! - `UsageCharge` (usage-based billing)

use serde::{Deserialize, Serialize};

/// The status of a billing charge.
///
/// Represents the lifecycle state of an application charge or recurring charge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChargeStatus {
    /// The charge is awaiting merchant approval.
    #[default]
    Pending,

    /// The merchant has accepted the charge (for recurring charges).
    Accepted,

    /// The charge is active and billing.
    Active,

    /// The merchant declined the charge.
    Declined,

    /// The charge has expired without action.
    Expired,

    /// The charge was cancelled.
    Cancelled,

    /// The charge has been frozen (for recurring charges).
    Frozen,
}

impl ChargeStatus {
    /// Returns `true` if the charge is pending merchant approval.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Returns `true` if the charge is active and billing.
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns `true` if the charge has been cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    /// Returns `true` if the charge was declined by the merchant.
    #[must_use]
    pub fn is_declined(&self) -> bool {
        matches!(self, Self::Declined)
    }

    /// Returns `true` if the charge has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        matches!(self, Self::Expired)
    }
}

/// Currency information for a charge.
///
/// Shopify returns currency as a nested object with a `currency` field
/// containing the ISO 4217 currency code.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ChargeCurrency {
    /// The three-letter ISO 4217 currency code (e.g., "USD", "EUR").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

impl ChargeCurrency {
    /// Creates a new `ChargeCurrency` with the given currency code.
    #[must_use]
    pub fn new(currency: impl Into<String>) -> Self {
        Self {
            currency: Some(currency.into()),
        }
    }

    /// Returns the currency code if present.
    #[must_use]
    pub fn code(&self) -> Option<&str> {
        self.currency.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charge_status_serialization() {
        let status = ChargeStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"active\"");

        let status = ChargeStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"pending\"");
    }

    #[test]
    fn test_charge_status_deserialization() {
        let status: ChargeStatus = serde_json::from_str("\"active\"").unwrap();
        assert_eq!(status, ChargeStatus::Active);

        let status: ChargeStatus = serde_json::from_str("\"declined\"").unwrap();
        assert_eq!(status, ChargeStatus::Declined);

        let status: ChargeStatus = serde_json::from_str("\"cancelled\"").unwrap();
        assert_eq!(status, ChargeStatus::Cancelled);
    }

    #[test]
    fn test_charge_status_helper_methods() {
        assert!(ChargeStatus::Pending.is_pending());
        assert!(!ChargeStatus::Active.is_pending());

        assert!(ChargeStatus::Active.is_active());
        assert!(!ChargeStatus::Pending.is_active());

        assert!(ChargeStatus::Cancelled.is_cancelled());
        assert!(!ChargeStatus::Active.is_cancelled());

        assert!(ChargeStatus::Declined.is_declined());
        assert!(ChargeStatus::Expired.is_expired());
    }

    #[test]
    fn test_charge_currency_serialization() {
        let currency = ChargeCurrency::new("USD");
        let json = serde_json::to_string(&currency).unwrap();
        assert!(json.contains("\"currency\":\"USD\""));
    }

    #[test]
    fn test_charge_currency_deserialization() {
        let json = r#"{"currency": "EUR"}"#;
        let currency: ChargeCurrency = serde_json::from_str(json).unwrap();
        assert_eq!(currency.code(), Some("EUR"));
    }

    #[test]
    fn test_charge_currency_code_method() {
        let currency = ChargeCurrency::new("CAD");
        assert_eq!(currency.code(), Some("CAD"));

        let empty = ChargeCurrency::default();
        assert_eq!(empty.code(), None);
    }
}

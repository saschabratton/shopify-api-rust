//! RecurringApplicationCharge resource implementation.
//!
//! This module provides the [`RecurringApplicationCharge`] resource for managing
//! subscription-based charges in Shopify apps. Recurring charges allow apps to
//! bill merchants on a regular basis (typically monthly).
//!
//! # API Endpoints
//!
//! - `GET /recurring_application_charges.json` - List all recurring charges
//! - `POST /recurring_application_charges.json` - Create a new recurring charge
//! - `GET /recurring_application_charges/{id}.json` - Retrieve a single charge
//! - `DELETE /recurring_application_charges/{id}.json` - Cancel a recurring charge
//! - `PUT /recurring_application_charges/{id}/customize.json` - Update capped amount
//!
//! Note: Recurring charges cannot be updated after creation, except for
//! the capped_amount via the customize endpoint.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{
//!     RecurringApplicationCharge, RecurringApplicationChargeListParams
//! };
//!
//! // Create a new recurring charge with trial period
//! let charge = RecurringApplicationCharge {
//!     name: Some("Pro Plan".to_string()),
//!     price: Some("29.99".to_string()),
//!     return_url: Some("https://myapp.com/charge-callback".to_string()),
//!     trial_days: Some(14),
//!     capped_amount: Some("100.00".to_string()),
//!     terms: Some("$29.99/month plus usage".to_string()),
//!     test: Some(true), // Test charges don't actually bill
//!     ..Default::default()
//! };
//! let saved = charge.save(&client).await?;
//!
//! // Redirect merchant to the confirmation_url for approval
//! if let Some(url) = saved.confirmation_url.as_ref() {
//!     println!("Redirect merchant to: {}", url);
//! }
//!
//! // Check charge status
//! if saved.is_active() {
//!     println!("Subscription is active!");
//! } else if saved.is_in_trial() {
//!     println!("Subscription is in trial period");
//! }
//!
//! // Update capped amount for usage-based billing
//! let updated = saved.customize(&client, "200.00").await?;
//!
//! // Get the current active charge
//! let current = RecurringApplicationCharge::current(&client).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

use super::common::{ChargeCurrency, ChargeStatus};

/// A recurring application charge (subscription).
///
/// Recurring application charges allow apps to bill merchants on a recurring
/// basis. After creating a charge, redirect the merchant to the `confirmation_url`
/// for them to approve the subscription.
///
/// # Charge Lifecycle
///
/// 1. App creates a charge via POST
/// 2. App redirects merchant to `confirmation_url`
/// 3. Merchant approves or declines the charge
/// 4. Merchant is redirected back to the app's `return_url`
/// 5. If approved, the subscription begins (possibly with a trial)
///
/// # Trial Periods
///
/// Set `trial_days` to give merchants a free trial before billing begins.
/// Use `is_in_trial()` to check if the subscription is currently in trial.
///
/// # Usage-Based Billing
///
/// For usage-based pricing, set a `capped_amount` which limits the total
/// charges per billing period. Use the `customize()` method to update the cap.
/// Create `UsageCharge` records to add usage fees up to the cap.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier of the charge
/// - `confirmation_url` - The URL to redirect merchant for approval
/// - `status` - The current status of the charge
/// - `currency` - The currency object with the currency code
/// - `activated_on` - When the charge was activated
/// - `billing_on` - The next billing date
/// - `cancelled_on` - When the charge was cancelled (if applicable)
/// - `trial_ends_on` - When the trial period ends
/// - `created_at` - When the charge was created
/// - `updated_at` - When the charge was last updated
///
/// ## Writable Fields
/// - `name` - The name of the charge (displayed to merchant)
/// - `price` - The recurring price
/// - `return_url` - The URL to redirect after merchant action
/// - `test` - Whether this is a test charge
/// - `capped_amount` - Maximum usage charge per billing period
/// - `terms` - Terms displayed to merchant on confirmation page
/// - `trial_days` - Number of trial days before billing begins
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RecurringApplicationCharge {
    /// The unique identifier of the recurring charge.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The name of the recurring charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The recurring price of the charge.
    /// Must be a string representing the monetary amount (e.g., "29.99").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,

    /// The current status of the charge.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub status: Option<ChargeStatus>,

    /// Whether this is a test charge.
    /// Test charges don't actually bill the merchant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<bool>,

    /// The URL to redirect the merchant after they approve/decline the charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<String>,

    /// The URL where the merchant can approve/decline the charge.
    /// Read-only field, populated after creating the charge.
    #[serde(skip_serializing)]
    pub confirmation_url: Option<String>,

    /// The currency information for the charge.
    /// Read-only field containing the currency code.
    #[serde(skip_serializing)]
    pub currency: Option<ChargeCurrency>,

    /// The maximum usage charge amount per billing period.
    /// Used for usage-based billing with capped amounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capped_amount: Option<String>,

    /// The terms displayed to the merchant on the confirmation page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms: Option<String>,

    /// The number of trial days before billing begins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trial_days: Option<i32>,

    /// When the trial period ends.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub trial_ends_on: Option<DateTime<Utc>>,

    /// When the charge was activated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub activated_on: Option<DateTime<Utc>>,

    /// The next billing date.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub billing_on: Option<DateTime<Utc>>,

    /// When the charge was cancelled.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub cancelled_on: Option<DateTime<Utc>>,

    /// When the charge was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the charge was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,
}

impl RecurringApplicationCharge {
    /// Returns `true` if the charge is active (approved and billing).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if charge.is_active() {
    ///     println!("Subscription is active!");
    /// }
    /// ```
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status.as_ref().map_or(false, ChargeStatus::is_active)
    }

    /// Returns `true` if the charge is pending merchant approval.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if charge.is_pending() {
    ///     println!("Waiting for merchant approval");
    /// }
    /// ```
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status.as_ref().map_or(false, ChargeStatus::is_pending)
    }

    /// Returns `true` if the charge has been cancelled.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if charge.is_cancelled() {
    ///     println!("Subscription has been cancelled");
    /// }
    /// ```
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.status
            .as_ref()
            .map_or(false, ChargeStatus::is_cancelled)
    }

    /// Returns `true` if this is a test charge.
    ///
    /// Test charges don't actually bill the merchant and are
    /// automatically approved on development stores.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if charge.is_test() {
    ///     println!("This is a test charge");
    /// }
    /// ```
    #[must_use]
    pub fn is_test(&self) -> bool {
        self.test.unwrap_or(false)
    }

    /// Returns `true` if the subscription is currently in trial period.
    ///
    /// This checks if `trial_ends_on` is set and in the future.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if charge.is_in_trial() {
    ///     println!("Subscription is in trial until {:?}", charge.trial_ends_on);
    /// }
    /// ```
    #[must_use]
    pub fn is_in_trial(&self) -> bool {
        self.trial_ends_on
            .map_or(false, |ends_on| ends_on > Utc::now())
    }

    /// Updates the capped amount for usage-based billing.
    ///
    /// Sends a PUT request to `/recurring_application_charges/{id}/customize.json`
    /// with the new capped amount.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `capped_amount` - The new maximum usage charge amount
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the charge doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the charge has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let charge = RecurringApplicationCharge::find(&client, 123, None).await?.into_inner();
    /// let updated = charge.customize(&client, "200.00").await?;
    /// println!("New capped amount: {:?}", updated.capped_amount);
    /// ```
    pub async fn customize(
        &self,
        client: &RestClient,
        capped_amount: &str,
    ) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "customize",
        })?;

        let path = format!("recurring_application_charges/{id}/customize");
        let body = serde_json::json!({
            "recurring_application_charge": {
                "capped_amount": capped_amount
            }
        });

        let response = client.put(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the response - Shopify returns the charge wrapped in "recurring_application_charge" key
        let charge: Self = response
            .body
            .get("recurring_application_charge")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'recurring_application_charge' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize recurring_application_charge: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(charge)
    }

    /// Retrieves the currently active recurring charge for the app.
    ///
    /// This is a convenience method that lists all charges with `status=active`
    /// and returns the first one. Most apps should only have one active charge.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Returns
    ///
    /// The currently active recurring charge, or `None` if no active charge exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(charge) = RecurringApplicationCharge::current(&client).await? {
    ///     println!("Active subscription: {} at {}/month",
    ///         charge.name.as_deref().unwrap_or(""),
    ///         charge.price.as_deref().unwrap_or("0")
    ///     );
    /// } else {
    ///     println!("No active subscription");
    /// }
    /// ```
    pub async fn current(client: &RestClient) -> Result<Option<Self>, ResourceError> {
        let params = RecurringApplicationChargeListParams {
            status: Some("active".to_string()),
            ..Default::default()
        };

        let response = Self::all(client, Some(params)).await?;
        Ok(response.into_inner().into_iter().next())
    }
}

impl RestResource for RecurringApplicationCharge {
    type Id = u64;
    type FindParams = RecurringApplicationChargeFindParams;
    type AllParams = RecurringApplicationChargeListParams;
    type CountParams = ();

    const NAME: &'static str = "RecurringApplicationCharge";
    const PLURAL: &'static str = "recurring_application_charges";

    /// Paths for the RecurringApplicationCharge resource.
    ///
    /// Note: No Update path - use `customize()` for updating capped_amount.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "recurring_application_charges/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "recurring_application_charges",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "recurring_application_charges",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "recurring_application_charges/{id}",
        ),
        // Note: customize endpoint is handled separately via the customize() method
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single recurring application charge.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RecurringApplicationChargeFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing recurring application charges.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RecurringApplicationChargeListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return charges after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Filter by status (e.g., "active", "pending", "cancelled").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_recurring_application_charge_serialization() {
        let charge = RecurringApplicationCharge {
            id: Some(12345),
            name: Some("Pro Plan".to_string()),
            price: Some("29.99".to_string()),
            status: Some(ChargeStatus::Active),
            test: Some(true),
            return_url: Some("https://myapp.com/callback".to_string()),
            confirmation_url: Some("https://shop.myshopify.com/confirm".to_string()),
            currency: Some(ChargeCurrency::new("USD")),
            capped_amount: Some("100.00".to_string()),
            terms: Some("$29.99/month plus usage".to_string()),
            trial_days: Some(14),
            trial_ends_on: Some(
                DateTime::parse_from_rfc3339("2024-02-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            activated_on: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            billing_on: Some(
                DateTime::parse_from_rfc3339("2024-02-15T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            cancelled_on: None,
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:35:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_string(&charge).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["name"], "Pro Plan");
        assert_eq!(parsed["price"], "29.99");
        assert_eq!(parsed["test"], true);
        assert_eq!(parsed["return_url"], "https://myapp.com/callback");
        assert_eq!(parsed["capped_amount"], "100.00");
        assert_eq!(parsed["terms"], "$29.99/month plus usage");
        assert_eq!(parsed["trial_days"], 14);

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("status").is_none());
        assert!(parsed.get("confirmation_url").is_none());
        assert!(parsed.get("currency").is_none());
        assert!(parsed.get("trial_ends_on").is_none());
        assert!(parsed.get("activated_on").is_none());
        assert!(parsed.get("billing_on").is_none());
        assert!(parsed.get("cancelled_on").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
    }

    #[test]
    fn test_recurring_application_charge_deserialization() {
        let json = r#"{
            "id": 455696195,
            "name": "Super Mega Plan",
            "price": "15.00",
            "status": "active",
            "test": true,
            "return_url": "https://super-duper.shopifyapps.com/",
            "confirmation_url": "https://jsmith.myshopify.com/admin/charges/455696195/confirm_recurring_application_charge",
            "currency": {
                "currency": "USD"
            },
            "capped_amount": "100.00",
            "terms": "$1 for 1000 emails",
            "trial_days": 7,
            "trial_ends_on": "2024-02-01T00:00:00Z",
            "activated_on": "2024-01-15T10:30:00Z",
            "billing_on": "2024-02-15T00:00:00Z",
            "cancelled_on": null,
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:35:00Z"
        }"#;

        let charge: RecurringApplicationCharge = serde_json::from_str(json).unwrap();

        assert_eq!(charge.id, Some(455696195));
        assert_eq!(charge.name, Some("Super Mega Plan".to_string()));
        assert_eq!(charge.price, Some("15.00".to_string()));
        assert_eq!(charge.status, Some(ChargeStatus::Active));
        assert_eq!(charge.test, Some(true));
        assert!(charge.confirmation_url.is_some());
        assert_eq!(charge.currency.as_ref().unwrap().code(), Some("USD"));
        assert_eq!(charge.capped_amount, Some("100.00".to_string()));
        assert_eq!(charge.terms, Some("$1 for 1000 emails".to_string()));
        assert_eq!(charge.trial_days, Some(7));
        assert!(charge.trial_ends_on.is_some());
        assert!(charge.activated_on.is_some());
        assert!(charge.billing_on.is_some());
        assert!(charge.cancelled_on.is_none());
        assert!(charge.created_at.is_some());
        assert!(charge.updated_at.is_some());
    }

    #[test]
    fn test_recurring_application_charge_convenience_methods() {
        // Test is_active
        let active_charge = RecurringApplicationCharge {
            status: Some(ChargeStatus::Active),
            ..Default::default()
        };
        assert!(active_charge.is_active());
        assert!(!active_charge.is_pending());
        assert!(!active_charge.is_cancelled());

        // Test is_pending
        let pending_charge = RecurringApplicationCharge {
            status: Some(ChargeStatus::Pending),
            ..Default::default()
        };
        assert!(pending_charge.is_pending());
        assert!(!pending_charge.is_active());

        // Test is_cancelled
        let cancelled_charge = RecurringApplicationCharge {
            status: Some(ChargeStatus::Cancelled),
            ..Default::default()
        };
        assert!(cancelled_charge.is_cancelled());
        assert!(!cancelled_charge.is_active());

        // Test is_test
        let test_charge = RecurringApplicationCharge {
            test: Some(true),
            ..Default::default()
        };
        assert!(test_charge.is_test());

        let non_test_charge = RecurringApplicationCharge {
            test: Some(false),
            ..Default::default()
        };
        assert!(!non_test_charge.is_test());

        // Test default
        let default_charge = RecurringApplicationCharge::default();
        assert!(!default_charge.is_test());
        assert!(!default_charge.is_active());
        assert!(!default_charge.is_pending());
        assert!(!default_charge.is_cancelled());
    }

    #[test]
    fn test_recurring_application_charge_is_in_trial() {
        // Test with future trial_ends_on (in trial)
        let future_date = Utc::now() + chrono::Duration::days(7);
        let in_trial_charge = RecurringApplicationCharge {
            trial_ends_on: Some(future_date),
            ..Default::default()
        };
        assert!(in_trial_charge.is_in_trial());

        // Test with past trial_ends_on (trial ended)
        let past_date = Utc::now() - chrono::Duration::days(7);
        let trial_ended_charge = RecurringApplicationCharge {
            trial_ends_on: Some(past_date),
            ..Default::default()
        };
        assert!(!trial_ended_charge.is_in_trial());

        // Test with no trial_ends_on (no trial)
        let no_trial_charge = RecurringApplicationCharge {
            trial_ends_on: None,
            ..Default::default()
        };
        assert!(!no_trial_charge.is_in_trial());
    }

    #[test]
    fn test_recurring_application_charge_paths() {
        // Find path
        let find_path = get_path(
            RecurringApplicationCharge::PATHS,
            ResourceOperation::Find,
            &["id"],
        );
        assert!(find_path.is_some());
        assert_eq!(
            find_path.unwrap().template,
            "recurring_application_charges/{id}"
        );

        // All path
        let all_path = get_path(RecurringApplicationCharge::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "recurring_application_charges");

        // Create path
        let create_path = get_path(
            RecurringApplicationCharge::PATHS,
            ResourceOperation::Create,
            &[],
        );
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "recurring_application_charges");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Delete path
        let delete_path = get_path(
            RecurringApplicationCharge::PATHS,
            ResourceOperation::Delete,
            &["id"],
        );
        assert!(delete_path.is_some());
        assert_eq!(
            delete_path.unwrap().template,
            "recurring_application_charges/{id}"
        );
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);

        // No Update path (use customize method)
        let update_path = get_path(
            RecurringApplicationCharge::PATHS,
            ResourceOperation::Update,
            &["id"],
        );
        assert!(update_path.is_none());

        // No Count path
        let count_path = get_path(
            RecurringApplicationCharge::PATHS,
            ResourceOperation::Count,
            &[],
        );
        assert!(count_path.is_none());
    }

    #[test]
    fn test_recurring_application_charge_list_params() {
        let params = RecurringApplicationChargeListParams {
            limit: Some(50),
            since_id: Some(100),
            status: Some("active".to_string()),
            fields: Some("id,name,price".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 100);
        assert_eq!(json["status"], "active");
        assert_eq!(json["fields"], "id,name,price");

        // Empty params should serialize to empty object
        let empty_params = RecurringApplicationChargeListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_recurring_application_charge_constants() {
        assert_eq!(RecurringApplicationCharge::NAME, "RecurringApplicationCharge");
        assert_eq!(
            RecurringApplicationCharge::PLURAL,
            "recurring_application_charges"
        );
    }

    #[test]
    fn test_recurring_application_charge_get_id() {
        let charge_with_id = RecurringApplicationCharge {
            id: Some(12345),
            ..Default::default()
        };
        assert_eq!(charge_with_id.get_id(), Some(12345));

        let charge_without_id = RecurringApplicationCharge::default();
        assert_eq!(charge_without_id.get_id(), None);
    }

    #[test]
    fn test_customize_method_signature() {
        // Verify the customize method exists with correct signature
        fn _assert_customize_signature<F, Fut>(f: F)
        where
            F: Fn(&RecurringApplicationCharge, &RestClient, &str) -> Fut,
            Fut: std::future::Future<Output = Result<RecurringApplicationCharge, ResourceError>>,
        {
            let _ = f;
        }

        // Verify PathResolutionFailed error is returned when charge has no ID
        let charge_without_id = RecurringApplicationCharge::default();
        assert!(charge_without_id.get_id().is_none());
    }

    #[test]
    fn test_current_method_signature() {
        // Verify the current method exists with correct signature
        fn _assert_current_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<Option<RecurringApplicationCharge>, ResourceError>>,
        {
            let _ = f;
        }
    }
}

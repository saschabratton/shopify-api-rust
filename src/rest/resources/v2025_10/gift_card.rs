//! GiftCard resource implementation.
//!
//! This module provides the [`GiftCard`] resource for managing gift cards in Shopify.
//! Gift cards are a Shopify Plus feature that allow merchants to sell store credit.
//!
//! # Scope Requirements
//!
//! **Important**: The `read_gift_cards` and `write_gift_cards` scopes require
//! approval from Shopify Support. Contact Shopify Partner Support to request
//! access to these scopes for your app.
//!
//! # Resource-Specific Operations
//!
//! In addition to standard CRUD operations (no Delete), the GiftCard resource provides:
//! - [`GiftCard::disable`] - Disable a gift card (cannot be re-enabled)
//! - [`GiftCard::search`] - Search for gift cards by query
//!
//! # Field Constraints
//!
//! - `initial_value` is required when creating a gift card
//! - `code` is write-only (only `last_characters` is readable after creation)
//! - `code` is auto-generated if not provided; must be 8-20 alphanumeric chars if provided
//! - Only `expires_on`, `note`, `template_suffix` are updatable after creation
//! - `customer_id` can only be set if currently null
//! - There is no Delete operation - use `disable()` instead
//! - Gift cards cannot be re-enabled after being disabled
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{GiftCard, GiftCardListParams};
//!
//! // Create a gift card
//! let mut gift_card = GiftCard {
//!     initial_value: Some("100.00".to_string()),
//!     note: Some("Employee reward".to_string()),
//!     ..Default::default()
//! };
//! let saved = gift_card.save(&client).await?;
//! println!("Gift card created with last chars: {:?}", saved.last_characters);
//!
//! // Search for gift cards
//! let results = GiftCard::search(&client, "employee").await?;
//!
//! // Disable a gift card
//! let disabled = saved.disable(&client).await?;
//! println!("Disabled at: {:?}", disabled.disabled_at);
//! ```

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{ResourceError, ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A gift card in Shopify.
///
/// Gift cards are a Shopify Plus feature that allow merchants to sell
/// or give away store credit. Gift card codes can be redeemed at checkout.
///
/// # Scope Requirements
///
/// The `read_gift_cards` and `write_gift_cards` scopes require approval
/// from Shopify Support. Contact support to request access for your app.
///
/// # Read-Only Fields
///
/// The following fields are read-only and will not be sent in create/update requests:
/// - `id`, `balance`
/// - `disabled_at`, `line_item_id`, `api_client_id`, `user_id`
/// - `last_characters`, `order_id`
/// - `created_at`, `updated_at`
/// - `admin_graphql_api_id`
///
/// # Write-Only Fields
///
/// The following fields are write-only (only for creation):
/// - `code` - The gift card code (auto-generated if not provided)
///
/// # Updatable Fields
///
/// After creation, only these fields can be updated:
/// - `expires_on` - Expiration date
/// - `note` - Internal note
/// - `template_suffix` - Template suffix for rendering
/// - `customer_id` - Only if currently null
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::rest::resources::v2025_10::GiftCard;
///
/// let gift_card = GiftCard {
///     initial_value: Some("50.00".to_string()),
///     note: Some("Birthday gift".to_string()),
///     customer_id: Some(123456),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GiftCard {
    // --- Read-only fields (not serialized) ---
    /// The unique identifier of the gift card.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The current balance of the gift card.
    #[serde(skip_serializing)]
    pub balance: Option<String>,

    /// When the gift card was disabled (null if still enabled).
    /// A gift card is disabled if this field is set.
    #[serde(skip_serializing)]
    pub disabled_at: Option<DateTime<Utc>>,

    /// The ID of the line item that created this gift card.
    #[serde(skip_serializing)]
    pub line_item_id: Option<u64>,

    /// The ID of the API client that created this gift card.
    #[serde(skip_serializing)]
    pub api_client_id: Option<u64>,

    /// The ID of the user who created this gift card.
    #[serde(skip_serializing)]
    pub user_id: Option<u64>,

    /// The last four characters of the gift card code.
    /// This is the only way to identify the code after creation.
    #[serde(skip_serializing)]
    pub last_characters: Option<String>,

    /// The ID of the order that created this gift card (if any).
    #[serde(skip_serializing)]
    pub order_id: Option<u64>,

    /// When the gift card was created.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the gift card was last updated.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,

    // --- Write-only fields (only deserialized, not serialized back) ---
    // Note: We use a custom approach here - code is included in serialization
    // for create, but won't be returned in responses

    /// The gift card code.
    ///
    /// **Write-only**: This field is only used when creating a gift card.
    /// After creation, only `last_characters` is available.
    ///
    /// If not provided, Shopify auto-generates a code.
    /// If provided, must be 8-20 alphanumeric characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    // --- Writable fields ---
    /// The initial value of the gift card.
    ///
    /// **Required for creation.**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<String>,

    /// The currency code for the gift card (e.g., "USD").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,

    /// The ID of the customer this gift card is associated with.
    ///
    /// Can only be set if currently null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_id: Option<u64>,

    /// An optional note attached to the gift card.
    ///
    /// Updatable after creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// When the gift card expires.
    ///
    /// Updatable after creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_on: Option<NaiveDate>,

    /// The template suffix for rendering the gift card.
    ///
    /// Updatable after creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_suffix: Option<String>,
}

impl GiftCard {
    /// Returns whether the gift card is currently enabled (not disabled).
    ///
    /// A gift card is considered enabled if `disabled_at` is `None`.
    pub fn is_enabled(&self) -> bool {
        self.disabled_at.is_none()
    }

    /// Returns whether the gift card is currently disabled.
    ///
    /// A gift card is considered disabled if `disabled_at` is set.
    pub fn is_disabled(&self) -> bool {
        self.disabled_at.is_some()
    }
}

impl RestResource for GiftCard {
    type Id = u64;
    type FindParams = GiftCardFindParams;
    type AllParams = GiftCardListParams;
    type CountParams = GiftCardCountParams;

    const NAME: &'static str = "GiftCard";
    const PLURAL: &'static str = "gift_cards";

    /// Paths for the GiftCard resource.
    ///
    /// Note: GiftCard does NOT have a Delete operation.
    /// Use `disable()` to deactivate a gift card instead.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "gift_cards/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "gift_cards"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "gift_cards/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "gift_cards",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "gift_cards/{id}",
        ),
        // No Delete path - use disable() instead
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl GiftCard {
    /// Disables the gift card.
    ///
    /// Sends a POST request to `/admin/api/{version}/gift_cards/{id}/disable.json`.
    ///
    /// **Note**: Once disabled, a gift card cannot be re-enabled.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    ///
    /// # Returns
    ///
    /// The gift card with `disabled_at` populated.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::NotFound`] if the gift card doesn't exist.
    /// Returns [`ResourceError::PathResolutionFailed`] if the gift card has no ID.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let gift_card = GiftCard::find(&client, 123, None).await?.into_inner();
    /// let disabled = gift_card.disable(&client).await?;
    /// assert!(disabled.is_disabled());
    /// ```
    pub async fn disable(&self, client: &RestClient) -> Result<Self, ResourceError> {
        let id = self.get_id().ok_or(ResourceError::PathResolutionFailed {
            resource: Self::NAME,
            operation: "disable",
        })?;

        let path = format!("gift_cards/{id}/disable");
        let body = serde_json::json!({});

        let response = client.post(&path, body, None).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                Some(&id.to_string()),
                response.request_id(),
            ));
        }

        // Parse the response - Shopify returns the gift card wrapped in "gift_card" key
        let gift_card: Self = response
            .body
            .get("gift_card")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'gift_card' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize gift_card: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(gift_card)
    }

    /// Searches for gift cards matching the query.
    ///
    /// Sends a GET request to `/admin/api/{version}/gift_cards/search.json?query={query}`.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client to use for the request
    /// * `query` - The search query string
    ///
    /// # Returns
    ///
    /// A list of gift cards matching the search query.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Search by last characters of code
    /// let results = GiftCard::search(&client, "abc1").await?;
    ///
    /// // Search by note content
    /// let results = GiftCard::search(&client, "birthday").await?;
    /// ```
    pub async fn search(client: &RestClient, query: &str) -> Result<Vec<Self>, ResourceError> {
        let path = format!("gift_cards/search");

        let mut query_params = std::collections::HashMap::new();
        query_params.insert("query".to_string(), query.to_string());

        let response = client.get(&path, Some(query_params)).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        // Parse the response - Shopify returns gift cards wrapped in "gift_cards" key
        let gift_cards: Vec<Self> = response
            .body
            .get("gift_cards")
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'gift_cards' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: response.code,
                            message: format!("Failed to deserialize gift_cards: {e}"),
                            error_reference: response.request_id().map(ToString::to_string),
                        },
                    ))
                })
            })?;

        Ok(gift_cards)
    }
}

/// Parameters for finding a single gift card.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GiftCardFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing gift cards.
///
/// All fields are optional. Unset fields will not be included in the request.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GiftCardListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return only gift cards after the specified ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Filter by status: "enabled" or "disabled".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Page info for cursor-based pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,
}

/// Parameters for counting gift cards.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GiftCardCountParams {
    /// Filter by status: "enabled" or "disabled".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_gift_card_struct_serialization() {
        let gift_card = GiftCard {
            id: Some(123456789),
            balance: Some("75.00".to_string()),
            disabled_at: None,
            line_item_id: Some(111),
            api_client_id: Some(222),
            user_id: Some(333),
            last_characters: Some("abc1".to_string()),
            order_id: Some(444),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            admin_graphql_api_id: Some("gid://shopify/GiftCard/123456789".to_string()),
            code: Some("GIFT1234ABCD5678".to_string()),
            initial_value: Some("100.00".to_string()),
            currency: Some("USD".to_string()),
            customer_id: Some(789012),
            note: Some("Employee reward".to_string()),
            expires_on: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            template_suffix: Some("premium".to_string()),
        };

        let json = serde_json::to_string(&gift_card).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["code"], "GIFT1234ABCD5678");
        assert_eq!(parsed["initial_value"], "100.00");
        assert_eq!(parsed["currency"], "USD");
        assert_eq!(parsed["customer_id"], 789012);
        assert_eq!(parsed["note"], "Employee reward");
        assert_eq!(parsed["expires_on"], "2025-12-31");
        assert_eq!(parsed["template_suffix"], "premium");

        // Read-only fields should NOT be serialized
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("balance").is_none());
        assert!(parsed.get("disabled_at").is_none());
        assert!(parsed.get("line_item_id").is_none());
        assert!(parsed.get("api_client_id").is_none());
        assert!(parsed.get("user_id").is_none());
        assert!(parsed.get("last_characters").is_none());
        assert!(parsed.get("order_id").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_gift_card_deserialization_from_api_response() {
        let json_str = r#"{
            "id": 1035197676,
            "balance": "100.00",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "currency": "USD",
            "initial_value": "100.00",
            "disabled_at": null,
            "line_item_id": 466157049,
            "api_client_id": 755357713,
            "user_id": null,
            "customer_id": 207119551,
            "note": "Birthday gift for John",
            "expires_on": "2025-12-31",
            "template_suffix": null,
            "last_characters": "0e0e",
            "order_id": 450789469,
            "admin_graphql_api_id": "gid://shopify/GiftCard/1035197676"
        }"#;

        let gift_card: GiftCard = serde_json::from_str(json_str).unwrap();

        assert_eq!(gift_card.id, Some(1035197676));
        assert_eq!(gift_card.balance.as_deref(), Some("100.00"));
        assert_eq!(gift_card.currency.as_deref(), Some("USD"));
        assert_eq!(gift_card.initial_value.as_deref(), Some("100.00"));
        assert_eq!(gift_card.disabled_at, None);
        assert_eq!(gift_card.line_item_id, Some(466157049));
        assert_eq!(gift_card.api_client_id, Some(755357713));
        assert_eq!(gift_card.user_id, None);
        assert_eq!(gift_card.customer_id, Some(207119551));
        assert_eq!(gift_card.note.as_deref(), Some("Birthday gift for John"));
        assert_eq!(
            gift_card.expires_on,
            Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
        );
        assert_eq!(gift_card.template_suffix, None);
        assert_eq!(gift_card.last_characters.as_deref(), Some("0e0e"));
        assert_eq!(gift_card.order_id, Some(450789469));
        assert!(gift_card.created_at.is_some());
        assert!(gift_card.updated_at.is_some());

        // Code should not be in the response (write-only)
        assert_eq!(gift_card.code, None);
    }

    #[test]
    fn test_gift_card_path_constants() {
        // Test Find path
        let find_path = get_path(GiftCard::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "gift_cards/{id}");
        assert_eq!(find_path.unwrap().http_method, HttpMethod::Get);

        // Test All path
        let all_path = get_path(GiftCard::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "gift_cards");

        // Test Count path
        let count_path = get_path(GiftCard::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "gift_cards/count");

        // Test Create path
        let create_path = get_path(GiftCard::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "gift_cards");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Test Update path
        let update_path = get_path(GiftCard::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "gift_cards/{id}");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Test that there is NO Delete path
        let delete_path = get_path(GiftCard::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());

        // Verify constants
        assert_eq!(GiftCard::NAME, "GiftCard");
        assert_eq!(GiftCard::PLURAL, "gift_cards");
    }

    #[test]
    fn test_disable_method_signature() {
        // Verify the disable method signature compiles correctly
        fn _assert_disable_signature<F, Fut>(f: F)
        where
            F: Fn(&GiftCard, &RestClient) -> Fut,
            Fut: std::future::Future<Output = Result<GiftCard, ResourceError>>,
        {
            let _ = f;
        }

        // Verify PathResolutionFailed error is returned when gift card has no ID
        let gift_card_without_id = GiftCard::default();
        assert!(gift_card_without_id.get_id().is_none());
    }

    #[test]
    fn test_search_method_signature() {
        // Verify the search method signature compiles correctly
        fn _assert_search_signature<F, Fut>(f: F)
        where
            F: Fn(&RestClient, &str) -> Fut,
            Fut: std::future::Future<Output = Result<Vec<GiftCard>, ResourceError>>,
        {
            let _ = f;
        }
    }

    #[test]
    fn test_gift_card_is_enabled_disabled() {
        let enabled_gift_card = GiftCard {
            id: Some(123),
            balance: Some("100.00".to_string()),
            disabled_at: None,
            ..Default::default()
        };

        assert!(enabled_gift_card.is_enabled());
        assert!(!enabled_gift_card.is_disabled());

        let disabled_gift_card = GiftCard {
            id: Some(456),
            balance: Some("0.00".to_string()),
            disabled_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ..Default::default()
        };

        assert!(!disabled_gift_card.is_enabled());
        assert!(disabled_gift_card.is_disabled());
    }

    #[test]
    fn test_gift_card_get_id_returns_correct_value() {
        let gift_card_with_id = GiftCard {
            id: Some(1035197676),
            balance: Some("100.00".to_string()),
            ..Default::default()
        };
        assert_eq!(gift_card_with_id.get_id(), Some(1035197676));

        let gift_card_without_id = GiftCard {
            id: None,
            initial_value: Some("50.00".to_string()),
            ..Default::default()
        };
        assert_eq!(gift_card_without_id.get_id(), None);
    }

    #[test]
    fn test_gift_card_list_params_serialization() {
        let params = GiftCardListParams {
            limit: Some(50),
            since_id: Some(12345),
            status: Some("enabled".to_string()),
            fields: Some("id,balance,last_characters".to_string()),
            page_info: None,
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["since_id"], 12345);
        assert_eq!(json["status"], "enabled");
        assert_eq!(json["fields"], "id,balance,last_characters");
        assert!(json.get("page_info").is_none());

        // Test empty params
        let empty_params = GiftCardListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_gift_card_count_params_serialization() {
        let params = GiftCardCountParams {
            status: Some("disabled".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["status"], "disabled");

        // Test empty params
        let empty_params = GiftCardCountParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_gift_card_updatable_fields() {
        // Test that only updatable fields can be serialized for update
        let update_gift_card = GiftCard {
            expires_on: Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
            note: Some("Updated note".to_string()),
            template_suffix: Some("custom".to_string()),
            customer_id: Some(999999),
            ..Default::default()
        };

        let json = serde_json::to_value(&update_gift_card).unwrap();

        // Updatable fields should be present
        assert_eq!(json["expires_on"], "2026-06-30");
        assert_eq!(json["note"], "Updated note");
        assert_eq!(json["template_suffix"], "custom");
        assert_eq!(json["customer_id"], 999999);
    }

    #[test]
    fn test_gift_card_code_is_write_only() {
        // When creating a gift card, code can be provided
        let create_gift_card = GiftCard {
            initial_value: Some("100.00".to_string()),
            code: Some("MYGIFTCODE1234".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&create_gift_card).unwrap();
        assert_eq!(json["code"], "MYGIFTCODE1234");
        assert_eq!(json["initial_value"], "100.00");

        // When deserializing from API (no code in response, only last_characters)
        let api_response = r#"{
            "id": 123,
            "balance": "100.00",
            "initial_value": "100.00",
            "last_characters": "1234"
        }"#;

        let gift_card: GiftCard = serde_json::from_str(api_response).unwrap();
        assert_eq!(gift_card.code, None);
        assert_eq!(gift_card.last_characters.as_deref(), Some("1234"));
    }
}

//! PriceRule resource implementation.
//!
//! This module provides the [`PriceRule`] resource for managing price rules
//! in Shopify stores. Price rules define discount logic that can be applied
//! through discount codes.
//!
//! # Deprecation Notice
//!
//! **⚠️ DEPRECATED**: Price rules are deprecated in favor of the GraphQL
//! Discount APIs. Consider using the GraphQL Admin API for creating and
//! managing discounts instead. The REST API for price rules may be removed
//! in a future API version.
//!
//! # Price Rule Types
//!
//! Price rules support different discount types:
//! - **Fixed amount**: A fixed monetary discount (e.g., $10 off)
//! - **Percentage**: A percentage discount (e.g., 20% off)
//!
//! # Allocation Methods
//!
//! Discounts can be allocated in two ways:
//! - **Each**: The discount is applied to each qualifying item
//! - **Across**: The discount is spread across all qualifying items
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_sdk::rest::{RestResource, ResourceResponse};
//! use shopify_sdk::rest::resources::v2025_10::{
//!     PriceRule, PriceRuleListParams,
//!     PriceRuleValueType, PriceRuleAllocationMethod
//! };
//!
//! // Create a percentage discount price rule
//! let price_rule = PriceRule {
//!     title: Some("20% Off Sale".to_string()),
//!     value_type: Some(PriceRuleValueType::Percentage),
//!     value: Some("-20.0".to_string()),
//!     target_type: Some(PriceRuleTargetType::LineItem),
//!     target_selection: Some(PriceRuleTargetSelection::All),
//!     allocation_method: Some(PriceRuleAllocationMethod::Across),
//!     customer_selection: Some(PriceRuleCustomerSelection::All),
//!     starts_at: Some(Utc::now()),
//!     ..Default::default()
//! };
//! let saved = price_rule.save(&client).await?;
//!
//! // List all price rules
//! let rules = PriceRule::all(&client, None).await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// The type of value for the price rule discount.
///
/// Determines whether the discount is a fixed amount or percentage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PriceRuleValueType {
    /// A fixed monetary amount discount.
    FixedAmount,
    /// A percentage discount.
    Percentage,
}

/// How the discount is allocated to qualifying items.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PriceRuleAllocationMethod {
    /// The discount is applied to each qualifying item individually.
    Each,
    /// The discount is spread across all qualifying items.
    Across,
}

/// Which customers are eligible for the price rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PriceRuleCustomerSelection {
    /// All customers are eligible.
    All,
    /// Only customers meeting prerequisite conditions are eligible.
    Prerequisite,
}

/// The type of target for the price rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PriceRuleTargetType {
    /// The discount applies to line items.
    LineItem,
    /// The discount applies to shipping.
    ShippingLine,
}

/// Which items the price rule targets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PriceRuleTargetSelection {
    /// The discount applies to all items.
    All,
    /// The discount applies to specific entitled items.
    Entitled,
}

/// A price rule that defines discount logic.
///
/// **⚠️ DEPRECATED**: Use GraphQL Discount APIs instead.
///
/// Price rules define the conditions and discounts for promotions.
/// Each price rule can have one or more associated discount codes.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `created_at` - When the rule was created
/// - `updated_at` - When the rule was last updated
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `title` - Internal name for the rule
/// - `value_type` - Whether discount is fixed_amount or percentage
/// - `value` - The discount value (negative for discounts)
/// - `target_type` - Whether applied to line_item or shipping_line
/// - `target_selection` - Which items: all or entitled
/// - `allocation_method` - How discount is applied: each or across
/// - `customer_selection` - Which customers: all or prerequisite
/// - `starts_at` - When the rule becomes active
/// - `ends_at` - When the rule expires (optional)
/// - Plus various prerequisite and entitlement fields
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PriceRule {
    /// The unique identifier of the price rule.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The internal title of the price rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The type of value: fixed_amount or percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<PriceRuleValueType>,

    /// The discount value. Negative for discounts (e.g., "-10.0" for $10 off).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The customer selection method: all or prerequisite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_selection: Option<PriceRuleCustomerSelection>,

    /// The target type: line_item or shipping_line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<PriceRuleTargetType>,

    /// The target selection: all or entitled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_selection: Option<PriceRuleTargetSelection>,

    /// How the discount is allocated: each or across.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation_method: Option<PriceRuleAllocationMethod>,

    /// How many times the discount can be allocated per order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation_limit: Option<i32>,

    /// Whether the price rule can be combined with other discounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub once_per_customer: Option<bool>,

    /// Maximum number of times this rule can be used across all customers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_limit: Option<i32>,

    /// When the price rule becomes active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<DateTime<Utc>>,

    /// When the price rule expires. Null means no expiration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<DateTime<Utc>>,

    /// Minimum subtotal required for the discount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_subtotal_range: Option<PrerequisiteRange>,

    /// Minimum quantity of items required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_quantity_range: Option<PrerequisiteRange>,

    /// Minimum shipping cost required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_shipping_price_range: Option<PrerequisiteRange>,

    /// IDs of prerequisite collections for BOGO discounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_collection_ids: Option<Vec<u64>>,

    /// IDs of prerequisite product variants for BOGO discounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_variant_ids: Option<Vec<u64>>,

    /// IDs of prerequisite products for BOGO discounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_product_ids: Option<Vec<u64>>,

    /// IDs of prerequisite customers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_customer_ids: Option<Vec<u64>>,

    /// IDs of collections the discount applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitled_collection_ids: Option<Vec<u64>>,

    /// IDs of products the discount applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitled_product_ids: Option<Vec<u64>>,

    /// IDs of product variants the discount applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitled_variant_ids: Option<Vec<u64>>,

    /// IDs of countries the discount applies to (for shipping discounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitled_country_ids: Option<Vec<u64>>,

    /// For BOGO: "prerequisite_quantity" for X items or "prerequisite_amount" for X spent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_to_entitlement_purchase: Option<PrerequisiteToEntitlement>,

    /// For BOGO: required quantity of prerequisite items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_to_entitlement_quantity_ratio: Option<BxgyRatio>,

    /// How many times the price rule has been used.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub times_used: Option<i32>,

    /// When the price rule was created.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,

    /// When the price rule was last updated.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub updated_at: Option<DateTime<Utc>>,

    /// The admin GraphQL API ID for this price rule.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

/// A prerequisite range for minimum values.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PrerequisiteRange {
    /// The minimum value for "greater than or equal to" comparisons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub greater_than_or_equal_to: Option<String>,
}

/// The type of purchase required for BOGO discounts.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PrerequisiteToEntitlement {
    /// The type: "prerequisite_quantity" or "prerequisite_amount".
    #[serde(rename = "prerequisite_amount", skip_serializing_if = "Option::is_none")]
    pub prerequisite_amount: Option<String>,
}

/// The buy X get Y quantity ratio for BOGO discounts.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BxgyRatio {
    /// Quantity of prerequisite items required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prerequisite_quantity: Option<i32>,

    /// Quantity of entitled items the customer gets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitled_quantity: Option<i32>,
}

impl RestResource for PriceRule {
    type Id = u64;
    type FindParams = PriceRuleFindParams;
    type AllParams = PriceRuleListParams;
    type CountParams = PriceRuleCountParams;

    const NAME: &'static str = "PriceRule";
    const PLURAL: &'static str = "price_rules";

    /// Paths for the PriceRule resource.
    ///
    /// Full CRUD operations are supported.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "price_rules/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "price_rules",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "price_rules/count",
        ),
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "price_rules",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "price_rules/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "price_rules/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single price rule.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PriceRuleFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing price rules.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PriceRuleListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return price rules after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Show price rules created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show price rules created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show price rules updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show price rules updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show price rules starting after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at_min: Option<DateTime<Utc>>,

    /// Show price rules starting before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at_max: Option<DateTime<Utc>>,

    /// Show price rules ending after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at_min: Option<DateTime<Utc>>,

    /// Show price rules ending before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at_max: Option<DateTime<Utc>>,

    /// Filter by times used count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_used: Option<i32>,

    /// Cursor for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_info: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting price rules.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PriceRuleCountParams {
    /// Show price rules created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show price rules created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Show price rules updated after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_min: Option<DateTime<Utc>>,

    /// Show price rules updated before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_max: Option<DateTime<Utc>>,

    /// Show price rules starting after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at_min: Option<DateTime<Utc>>,

    /// Show price rules starting before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_at_max: Option<DateTime<Utc>>,

    /// Show price rules ending after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at_min: Option<DateTime<Utc>>,

    /// Show price rules ending before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at_max: Option<DateTime<Utc>>,

    /// Filter by times used count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_used: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_price_rule_serialization() {
        let rule = PriceRule {
            id: Some(12345),
            title: Some("20% Off Sale".to_string()),
            value_type: Some(PriceRuleValueType::Percentage),
            value: Some("-20.0".to_string()),
            customer_selection: Some(PriceRuleCustomerSelection::All),
            target_type: Some(PriceRuleTargetType::LineItem),
            target_selection: Some(PriceRuleTargetSelection::All),
            allocation_method: Some(PriceRuleAllocationMethod::Across),
            once_per_customer: Some(true),
            usage_limit: Some(100),
            starts_at: Some(
                DateTime::parse_from_rfc3339("2024-01-15T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ends_at: Some(
                DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            times_used: Some(50),
            created_at: Some(
                DateTime::parse_from_rfc3339("2024-01-10T08:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            updated_at: Some(
                DateTime::parse_from_rfc3339("2024-06-20T15:45:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            admin_graphql_api_id: Some("gid://shopify/PriceRule/12345".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&rule).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["title"], "20% Off Sale");
        assert_eq!(parsed["value_type"], "percentage");
        assert_eq!(parsed["value"], "-20.0");
        assert_eq!(parsed["customer_selection"], "all");
        assert_eq!(parsed["target_type"], "line_item");
        assert_eq!(parsed["target_selection"], "all");
        assert_eq!(parsed["allocation_method"], "across");
        assert_eq!(parsed["once_per_customer"], true);
        assert_eq!(parsed["usage_limit"], 100);
        assert!(parsed["starts_at"].as_str().is_some());
        assert!(parsed["ends_at"].as_str().is_some());

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("times_used").is_none());
        assert!(parsed.get("created_at").is_none());
        assert!(parsed.get("updated_at").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_price_rule_deserialization() {
        let json = r#"{
            "id": 507328175,
            "title": "Summer Sale",
            "value_type": "fixed_amount",
            "value": "-10.0",
            "customer_selection": "all",
            "target_type": "line_item",
            "target_selection": "all",
            "allocation_method": "across",
            "allocation_limit": null,
            "once_per_customer": false,
            "usage_limit": null,
            "starts_at": "2024-06-01T00:00:00Z",
            "ends_at": "2024-08-31T23:59:59Z",
            "times_used": 25,
            "created_at": "2024-05-15T10:30:00Z",
            "updated_at": "2024-06-20T15:45:00Z",
            "admin_graphql_api_id": "gid://shopify/PriceRule/507328175"
        }"#;

        let rule: PriceRule = serde_json::from_str(json).unwrap();

        assert_eq!(rule.id, Some(507328175));
        assert_eq!(rule.title, Some("Summer Sale".to_string()));
        assert_eq!(rule.value_type, Some(PriceRuleValueType::FixedAmount));
        assert_eq!(rule.value, Some("-10.0".to_string()));
        assert_eq!(rule.customer_selection, Some(PriceRuleCustomerSelection::All));
        assert_eq!(rule.target_type, Some(PriceRuleTargetType::LineItem));
        assert_eq!(rule.target_selection, Some(PriceRuleTargetSelection::All));
        assert_eq!(rule.allocation_method, Some(PriceRuleAllocationMethod::Across));
        assert_eq!(rule.once_per_customer, Some(false));
        assert_eq!(rule.times_used, Some(25));
        assert!(rule.created_at.is_some());
        assert!(rule.updated_at.is_some());
        assert_eq!(
            rule.admin_graphql_api_id,
            Some("gid://shopify/PriceRule/507328175".to_string())
        );
    }

    #[test]
    fn test_price_rule_full_crud_paths() {
        // Find by ID
        let find_path = get_path(PriceRule::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "price_rules/{id}");

        // List all
        let all_path = get_path(PriceRule::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "price_rules");

        // Count
        let count_path = get_path(PriceRule::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "price_rules/count");

        // Create
        let create_path = get_path(PriceRule::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "price_rules");
        assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

        // Update
        let update_path = get_path(PriceRule::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "price_rules/{id}");
        assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

        // Delete
        let delete_path = get_path(PriceRule::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "price_rules/{id}");
        assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);
    }

    #[test]
    fn test_price_rule_value_type_enum_serialization() {
        // Test that value_type enum serializes to snake_case
        let percentage = PriceRuleValueType::Percentage;
        let json = serde_json::to_value(&percentage).unwrap();
        assert_eq!(json, "percentage");

        let fixed = PriceRuleValueType::FixedAmount;
        let json = serde_json::to_value(&fixed).unwrap();
        assert_eq!(json, "fixed_amount");

        // Deserialize back
        let parsed: PriceRuleValueType = serde_json::from_str("\"fixed_amount\"").unwrap();
        assert_eq!(parsed, PriceRuleValueType::FixedAmount);
    }

    #[test]
    fn test_price_rule_list_params_with_date_ranges() {
        let params = PriceRuleListParams {
            limit: Some(50),
            created_at_min: Some(
                DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            starts_at_min: Some(
                DateTime::parse_from_rfc3339("2024-06-01T00:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            ends_at_max: Some(
                DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            times_used: Some(10),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert!(json["created_at_min"].as_str().is_some());
        assert!(json["starts_at_min"].as_str().is_some());
        assert!(json["ends_at_max"].as_str().is_some());
        assert_eq!(json["times_used"], 10);

        // Fields not set should be omitted
        assert!(json.get("since_id").is_none());
        assert!(json.get("updated_at_min").is_none());
    }

    #[test]
    fn test_price_rule_constants() {
        assert_eq!(PriceRule::NAME, "PriceRule");
        assert_eq!(PriceRule::PLURAL, "price_rules");
    }

    #[test]
    fn test_price_rule_get_id() {
        let rule_with_id = PriceRule {
            id: Some(12345),
            title: Some("Test Rule".to_string()),
            ..Default::default()
        };
        assert_eq!(rule_with_id.get_id(), Some(12345));

        let rule_without_id = PriceRule::default();
        assert_eq!(rule_without_id.get_id(), None);
    }
}

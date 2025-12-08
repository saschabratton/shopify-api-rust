//! FulfillmentService resource implementation.
//!
//! This module provides the [`FulfillmentService`] resource for managing
//! third-party fulfillment services that can fulfill orders on behalf of
//! a merchant.
//!
//! # No Count Endpoint
//!
//! Note that `FulfillmentService` does not have a count endpoint. Use the
//! `all` method with a limit to retrieve fulfillment services.
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{
//!     FulfillmentService, FulfillmentServiceListParams
//! };
//!
//! // List all fulfillment services
//! let services = FulfillmentService::all(&client, None).await?;
//!
//! // List services created by the current app
//! let params = FulfillmentServiceListParams {
//!     scope: Some("current_client".to_string()),
//!     ..Default::default()
//! };
//! let my_services = FulfillmentService::all(&client, Some(params)).await?;
//!
//! // Create a fulfillment service
//! let service = FulfillmentService {
//!     name: Some("My Fulfillment".to_string()),
//!     callback_url: Some("https://myapp.com/fulfillment".to_string()),
//!     inventory_management: Some(true),
//!     tracking_support: Some(true),
//!     ..Default::default()
//! };
//! let saved = service.save(&client).await?;
//! ```

use serde::{Deserialize, Serialize};

use crate::rest::{ResourceOperation, ResourcePath, RestResource};
use crate::HttpMethod;

/// A third-party fulfillment service.
///
/// Fulfillment services handle the picking, packing, and shipping of orders
/// on behalf of merchants. They can manage their own inventory and provide
/// tracking information.
///
/// # Fields
///
/// ## Read-Only Fields
/// - `id` - The unique identifier
/// - `location_id` - The location ID where this service operates
/// - `provider_id` - The provider ID (if applicable)
/// - `admin_graphql_api_id` - The GraphQL API ID
///
/// ## Writable Fields
/// - `name` - Display name of the service
/// - `handle` - URL-friendly identifier
/// - `callback_url` - URL for fulfillment callbacks
/// - `tracking_support` - Whether service provides tracking
/// - `inventory_management` - Whether service manages inventory
/// - `requires_shipping_method` - Whether shipping method is required
/// - `fulfillment_orders_opt_in` - Whether service uses fulfillment orders
/// - `permits_sku_sharing` - Whether SKUs can be shared
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentService {
    /// The unique identifier of the fulfillment service.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The name of the fulfillment service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The URL-friendly identifier for the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,

    /// The callback URL for fulfillment requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,

    /// The location ID associated with this service.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub location_id: Option<u64>,

    /// The provider ID (if using a standard provider).
    /// Read-only field.
    #[serde(skip_serializing)]
    pub provider_id: Option<String>,

    /// Whether the service provides tracking information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_support: Option<bool>,

    /// Whether the service manages inventory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_management: Option<bool>,

    /// Whether a shipping method is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_shipping_method: Option<bool>,

    /// Whether the service has opted into fulfillment orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment_orders_opt_in: Option<bool>,

    /// Whether SKU sharing is permitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permits_sku_sharing: Option<bool>,

    /// The admin GraphQL API ID for this service.
    /// Read-only field.
    #[serde(skip_serializing)]
    pub admin_graphql_api_id: Option<String>,
}

impl RestResource for FulfillmentService {
    type Id = u64;
    type FindParams = FulfillmentServiceFindParams;
    type AllParams = FulfillmentServiceListParams;
    type CountParams = ();

    const NAME: &'static str = "FulfillmentService";
    const PLURAL: &'static str = "fulfillment_services";

    /// Paths for the FulfillmentService resource.
    ///
    /// CRUD without Count - no count endpoint exists.
    const PATHS: &'static [ResourcePath] = &[
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "fulfillment_services/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &[],
            "fulfillment_services",
        ),
        // Note: No Count path - not supported by the API
        ResourcePath::new(
            HttpMethod::Post,
            ResourceOperation::Create,
            &[],
            "fulfillment_services",
        ),
        ResourcePath::new(
            HttpMethod::Put,
            ResourceOperation::Update,
            &["id"],
            "fulfillment_services/{id}",
        ),
        ResourcePath::new(
            HttpMethod::Delete,
            ResourceOperation::Delete,
            &["id"],
            "fulfillment_services/{id}",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

/// Parameters for finding a single fulfillment service.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentServiceFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing fulfillment services.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct FulfillmentServiceListParams {
    /// Filter services by scope.
    /// Valid values: "all" (all services) or "current_client" (services by current app).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ResourceOperation};

    #[test]
    fn test_fulfillment_service_serialization() {
        let service = FulfillmentService {
            id: Some(61629186),
            name: Some("My Fulfillment".to_string()),
            handle: Some("my-fulfillment".to_string()),
            callback_url: Some("https://myapp.com/fulfillment".to_string()),
            location_id: Some(655441491),
            provider_id: Some("provider_123".to_string()),
            tracking_support: Some(true),
            inventory_management: Some(true),
            requires_shipping_method: Some(false),
            fulfillment_orders_opt_in: Some(true),
            permits_sku_sharing: Some(false),
            admin_graphql_api_id: Some("gid://shopify/FulfillmentService/61629186".to_string()),
        };

        let json = serde_json::to_string(&service).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Writable fields should be present
        assert_eq!(parsed["name"], "My Fulfillment");
        assert_eq!(parsed["handle"], "my-fulfillment");
        assert_eq!(parsed["callback_url"], "https://myapp.com/fulfillment");
        assert_eq!(parsed["tracking_support"], true);
        assert_eq!(parsed["inventory_management"], true);
        assert_eq!(parsed["requires_shipping_method"], false);
        assert_eq!(parsed["fulfillment_orders_opt_in"], true);
        assert_eq!(parsed["permits_sku_sharing"], false);

        // Read-only fields should be omitted
        assert!(parsed.get("id").is_none());
        assert!(parsed.get("location_id").is_none());
        assert!(parsed.get("provider_id").is_none());
        assert!(parsed.get("admin_graphql_api_id").is_none());
    }

    #[test]
    fn test_fulfillment_service_deserialization() {
        let json = r#"{
            "id": 61629186,
            "name": "My Fulfillment Service",
            "handle": "my-fulfillment-service",
            "callback_url": "https://myapp.com/fulfillment",
            "location_id": 655441491,
            "provider_id": null,
            "tracking_support": true,
            "inventory_management": true,
            "requires_shipping_method": false,
            "fulfillment_orders_opt_in": true,
            "permits_sku_sharing": false,
            "admin_graphql_api_id": "gid://shopify/FulfillmentService/61629186"
        }"#;

        let service: FulfillmentService = serde_json::from_str(json).unwrap();

        assert_eq!(service.id, Some(61629186));
        assert_eq!(service.name, Some("My Fulfillment Service".to_string()));
        assert_eq!(service.handle, Some("my-fulfillment-service".to_string()));
        assert_eq!(
            service.callback_url,
            Some("https://myapp.com/fulfillment".to_string())
        );
        assert_eq!(service.location_id, Some(655441491));
        assert!(service.provider_id.is_none());
        assert_eq!(service.tracking_support, Some(true));
        assert_eq!(service.inventory_management, Some(true));
        assert_eq!(service.requires_shipping_method, Some(false));
        assert_eq!(service.fulfillment_orders_opt_in, Some(true));
        assert_eq!(service.permits_sku_sharing, Some(false));
        assert_eq!(
            service.admin_graphql_api_id,
            Some("gid://shopify/FulfillmentService/61629186".to_string())
        );
    }

    #[test]
    fn test_fulfillment_service_crud_without_count() {
        // Find by ID
        let find_path = get_path(FulfillmentService::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "fulfillment_services/{id}");

        // List all
        let all_path = get_path(FulfillmentService::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "fulfillment_services");

        // No count path
        let count_path = get_path(FulfillmentService::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_none());

        // Create
        let create_path = get_path(FulfillmentService::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_some());
        assert_eq!(create_path.unwrap().template, "fulfillment_services");

        // Update
        let update_path = get_path(FulfillmentService::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_some());
        assert_eq!(update_path.unwrap().template, "fulfillment_services/{id}");

        // Delete
        let delete_path = get_path(FulfillmentService::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_some());
        assert_eq!(delete_path.unwrap().template, "fulfillment_services/{id}");
    }

    #[test]
    fn test_fulfillment_service_list_params() {
        let params = FulfillmentServiceListParams {
            scope: Some("current_client".to_string()),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["scope"], "current_client");

        // Test with "all" scope
        let params_all = FulfillmentServiceListParams {
            scope: Some("all".to_string()),
        };
        let json_all = serde_json::to_value(&params_all).unwrap();
        assert_eq!(json_all["scope"], "all");

        // Empty params
        let empty_params = FulfillmentServiceListParams::default();
        let empty_json = serde_json::to_value(&empty_params).unwrap();
        assert_eq!(empty_json, serde_json::json!({}));
    }

    #[test]
    fn test_fulfillment_service_constants() {
        assert_eq!(FulfillmentService::NAME, "FulfillmentService");
        assert_eq!(FulfillmentService::PLURAL, "fulfillment_services");
    }

    #[test]
    fn test_fulfillment_service_get_id() {
        let service_with_id = FulfillmentService {
            id: Some(61629186),
            name: Some("Test Service".to_string()),
            ..Default::default()
        };
        assert_eq!(service_with_id.get_id(), Some(61629186));

        let service_without_id = FulfillmentService::default();
        assert_eq!(service_without_id.get_id(), None);
    }
}

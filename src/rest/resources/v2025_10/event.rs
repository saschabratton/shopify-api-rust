//! Event resource implementation.
//!
//! This module provides the [`Event`] resource for viewing events that have
//! occurred in a Shopify store. Events represent actions taken on resources
//! like products, orders, and articles.
//!
//! # Read-Only Resource
//!
//! Events implement [`ReadOnlyResource`](crate::rest::ReadOnlyResource) - they
//! can only be retrieved, not created, updated, or deleted. Events are
//! automatically recorded by Shopify when actions occur.
//!
//! # Polymorphic Owner Paths
//!
//! Events support optional polymorphic paths based on the owner resource:
//! - Global: `/events.json` (all events)
//! - Orders: `/orders/{order_id}/events.json`
//! - Products: `/products/{product_id}/events.json`
//!
//! # Example
//!
//! ```rust,ignore
//! use shopify_api::rest::{RestResource, ResourceResponse};
//! use shopify_api::rest::resources::v2025_10::{Event, EventListParams};
//!
//! // List all events
//! let events = Event::all(&client, None).await?;
//! for event in events.iter() {
//!     println!("{}: {} {}", event.verb.as_deref().unwrap_or(""),
//!         event.subject_type.as_deref().unwrap_or(""),
//!         event.subject_id.unwrap_or(0));
//! }
//!
//! // List events for a specific order
//! let events = Event::all_for_owner(
//!     &client,
//!     "order_id",
//!     450789469,
//!     None
//! ).await?;
//!
//! // Filter events by verb
//! let params = EventListParams {
//!     verb: Some("create".to_string()),
//!     filter: Some("Product".to_string()),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let events = Event::all(&client, Some(params)).await?;
//! ```

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::clients::RestClient;
use crate::rest::{
    build_path, get_path, ReadOnlyResource, ResourceError, ResourceOperation, ResourcePath,
    ResourceResponse, RestResource,
};
use crate::HttpMethod;

/// An event representing an action in the store.
///
/// Events are read-only records of actions that have occurred on resources.
/// They are automatically created by Shopify and cannot be modified.
///
/// # Read-Only Resource
///
/// This resource implements [`ReadOnlyResource`] - only GET operations are
/// available. Events are recorded automatically when actions occur.
///
/// # Fields
///
/// All fields are read-only:
/// - `id` - The unique identifier
/// - `subject_id` - The ID of the resource the event is for
/// - `subject_type` - The type of resource (e.g., "Product", "Order")
/// - `verb` - The action that occurred (e.g., "create", "update", "destroy")
/// - `arguments` - Any additional arguments for the event
/// - `message` - Human-readable description of the event
/// - `description` - Detailed description of the event
/// - `body` - The body of the event (for some event types)
/// - `path` - The relative path to the resource
/// - `created_at` - When the event occurred
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Event {
    /// The unique identifier of the event.
    #[serde(skip_serializing)]
    pub id: Option<u64>,

    /// The ID of the resource the event is for.
    #[serde(skip_serializing)]
    pub subject_id: Option<u64>,

    /// The type of resource (e.g., "Product", "Order", "Article").
    #[serde(skip_serializing)]
    pub subject_type: Option<String>,

    /// The action that occurred (e.g., "create", "update", "destroy").
    #[serde(skip_serializing)]
    pub verb: Option<String>,

    /// Additional arguments for the event.
    #[serde(skip_serializing)]
    pub arguments: Option<Vec<String>>,

    /// Human-readable description of the event.
    #[serde(skip_serializing)]
    pub message: Option<String>,

    /// Detailed description of the event.
    #[serde(skip_serializing)]
    pub description: Option<String>,

    /// The body of the event (for some event types).
    #[serde(skip_serializing)]
    pub body: Option<String>,

    /// The relative path to the resource.
    #[serde(skip_serializing)]
    pub path: Option<String>,

    /// When the event occurred.
    #[serde(skip_serializing)]
    pub created_at: Option<DateTime<Utc>>,
}

impl Event {
    /// Lists events for a specific owner resource.
    ///
    /// This supports polymorphic paths for different resource types.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client
    /// * `owner_id_name` - The name of the owner ID parameter (e.g., "order_id", "product_id")
    /// * `owner_id` - The owner's ID
    /// * `params` - Optional list parameters
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Events for an order
    /// let events = Event::all_for_owner(&client, "order_id", 450789469, None).await?;
    ///
    /// // Events for a product
    /// let events = Event::all_for_owner(&client, "product_id", 632910392, None).await?;
    /// ```
    pub async fn all_for_owner<OwnerId: std::fmt::Display + Send>(
        client: &RestClient,
        owner_id_name: &str,
        owner_id: OwnerId,
        params: Option<EventListParams>,
    ) -> Result<ResourceResponse<Vec<Self>>, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert(owner_id_name, owner_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::All, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "all",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Build query params
        let query = params
            .map(|p| {
                let value = serde_json::to_value(&p).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: 400,
                            message: format!("Failed to serialize params: {e}"),
                            error_reference: None,
                        },
                    ))
                })?;

                let mut query = HashMap::new();
                if let serde_json::Value::Object(map) = value {
                    for (key, val) in map {
                        match val {
                            serde_json::Value::String(s) => {
                                query.insert(key, s);
                            }
                            serde_json::Value::Number(n) => {
                                query.insert(key, n.to_string());
                            }
                            serde_json::Value::Bool(b) => {
                                query.insert(key, b.to_string());
                            }
                            _ => {}
                        }
                    }
                }
                Ok::<_, ResourceError>(query)
            })
            .transpose()?
            .filter(|q| !q.is_empty());

        let response = client.get(&url, query).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        let key = Self::PLURAL;
        ResourceResponse::from_http_response(response, key)
    }

    /// Counts events for a specific owner resource.
    ///
    /// # Arguments
    ///
    /// * `client` - The REST client
    /// * `owner_id_name` - The name of the owner ID parameter
    /// * `owner_id` - The owner's ID
    /// * `params` - Optional count parameters
    pub async fn count_for_owner<OwnerId: std::fmt::Display + Send>(
        client: &RestClient,
        owner_id_name: &str,
        owner_id: OwnerId,
        params: Option<EventCountParams>,
    ) -> Result<u64, ResourceError> {
        let mut ids: HashMap<&str, String> = HashMap::new();
        ids.insert(owner_id_name, owner_id.to_string());

        let available_ids: Vec<&str> = ids.keys().copied().collect();
        let path = get_path(Self::PATHS, ResourceOperation::Count, &available_ids).ok_or(
            ResourceError::PathResolutionFailed {
                resource: Self::NAME,
                operation: "count",
            },
        )?;

        let url = build_path(path.template, &ids);

        // Build query params
        let query = params
            .map(|p| {
                let value = serde_json::to_value(&p).map_err(|e| {
                    ResourceError::Http(crate::clients::HttpError::Response(
                        crate::clients::HttpResponseError {
                            code: 400,
                            message: format!("Failed to serialize params: {e}"),
                            error_reference: None,
                        },
                    ))
                })?;

                let mut query = HashMap::new();
                if let serde_json::Value::Object(map) = value {
                    for (key, val) in map {
                        match val {
                            serde_json::Value::String(s) => {
                                query.insert(key, s);
                            }
                            serde_json::Value::Number(n) => {
                                query.insert(key, n.to_string());
                            }
                            serde_json::Value::Bool(b) => {
                                query.insert(key, b.to_string());
                            }
                            _ => {}
                        }
                    }
                }
                Ok::<_, ResourceError>(query)
            })
            .transpose()?
            .filter(|q| !q.is_empty());

        let response = client.get(&url, query).await?;

        if !response.is_ok() {
            return Err(ResourceError::from_http_response(
                response.code,
                &response.body,
                Self::NAME,
                None,
                response.request_id(),
            ));
        }

        let count = response
            .body
            .get("count")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                ResourceError::Http(crate::clients::HttpError::Response(
                    crate::clients::HttpResponseError {
                        code: response.code,
                        message: "Missing 'count' in response".to_string(),
                        error_reference: response.request_id().map(ToString::to_string),
                    },
                ))
            })?;

        Ok(count)
    }
}

impl RestResource for Event {
    type Id = u64;
    type FindParams = EventFindParams;
    type AllParams = EventListParams;
    type CountParams = EventCountParams;

    const NAME: &'static str = "Event";
    const PLURAL: &'static str = "events";

    /// Paths for the Event resource.
    ///
    /// Supports both global paths and polymorphic owner paths.
    /// Only GET operations - events are read-only.
    const PATHS: &'static [ResourcePath] = &[
        // Global paths
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Find,
            &["id"],
            "events/{id}",
        ),
        ResourcePath::new(HttpMethod::Get, ResourceOperation::All, &[], "events"),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &[],
            "events/count",
        ),
        // Polymorphic owner paths - orders
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["order_id"],
            "orders/{order_id}/events",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["order_id"],
            "orders/{order_id}/events/count",
        ),
        // Polymorphic owner paths - products
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::All,
            &["product_id"],
            "products/{product_id}/events",
        ),
        ResourcePath::new(
            HttpMethod::Get,
            ResourceOperation::Count,
            &["product_id"],
            "products/{product_id}/events/count",
        ),
    ];

    fn get_id(&self) -> Option<Self::Id> {
        self.id
    }
}

impl ReadOnlyResource for Event {}

/// Parameters for finding a single event.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct EventFindParams {
    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for listing events.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct EventListParams {
    /// Maximum number of results to return (default: 50, max: 250).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Return events after this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_id: Option<u64>,

    /// Show events created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show events created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,

    /// Filter events by subject type (e.g., "Product", "Order").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    /// Filter events by verb (e.g., "create", "update", "destroy").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verb: Option<String>,

    /// Comma-separated list of fields to include in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,
}

/// Parameters for counting events.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct EventCountParams {
    /// Show events created after this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_min: Option<DateTime<Utc>>,

    /// Show events created before this date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at_max: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::{get_path, ReadOnlyResource, ResourceOperation, RestResource};

    #[test]
    fn test_event_implements_read_only_resource() {
        // This test verifies that Event implements ReadOnlyResource
        fn assert_read_only<T: ReadOnlyResource>() {}
        assert_read_only::<Event>();
    }

    #[test]
    fn test_event_deserialization() {
        let json = r#"{
            "id": 677313116,
            "subject_id": 921728736,
            "subject_type": "Product",
            "verb": "create",
            "arguments": ["IPod Touch 8GB", "White"],
            "message": "IPod Touch 8GB was created with colors White.",
            "description": "Product created",
            "body": null,
            "path": "/admin/products/921728736",
            "created_at": "2024-06-15T10:30:00Z"
        }"#;

        let event: Event = serde_json::from_str(json).unwrap();

        assert_eq!(event.id, Some(677313116));
        assert_eq!(event.subject_id, Some(921728736));
        assert_eq!(event.subject_type, Some("Product".to_string()));
        assert_eq!(event.verb, Some("create".to_string()));
        assert!(event.arguments.is_some());
        let args = event.arguments.unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "IPod Touch 8GB");
        assert_eq!(args[1], "White");
        assert_eq!(
            event.message,
            Some("IPod Touch 8GB was created with colors White.".to_string())
        );
        assert_eq!(event.description, Some("Product created".to_string()));
        assert!(event.body.is_none());
        assert_eq!(
            event.path,
            Some("/admin/products/921728736".to_string())
        );
        assert!(event.created_at.is_some());
    }

    #[test]
    fn test_event_read_only_paths() {
        // Global paths
        let find_path = get_path(Event::PATHS, ResourceOperation::Find, &["id"]);
        assert!(find_path.is_some());
        assert_eq!(find_path.unwrap().template, "events/{id}");

        let all_path = get_path(Event::PATHS, ResourceOperation::All, &[]);
        assert!(all_path.is_some());
        assert_eq!(all_path.unwrap().template, "events");

        let count_path = get_path(Event::PATHS, ResourceOperation::Count, &[]);
        assert!(count_path.is_some());
        assert_eq!(count_path.unwrap().template, "events/count");

        // No create, update, or delete paths
        let create_path = get_path(Event::PATHS, ResourceOperation::Create, &[]);
        assert!(create_path.is_none());

        let update_path = get_path(Event::PATHS, ResourceOperation::Update, &["id"]);
        assert!(update_path.is_none());

        let delete_path = get_path(Event::PATHS, ResourceOperation::Delete, &["id"]);
        assert!(delete_path.is_none());
    }

    #[test]
    fn test_event_polymorphic_owner_paths() {
        // Order events path
        let order_events = get_path(Event::PATHS, ResourceOperation::All, &["order_id"]);
        assert!(order_events.is_some());
        assert_eq!(
            order_events.unwrap().template,
            "orders/{order_id}/events"
        );

        // Order events count path
        let order_count = get_path(Event::PATHS, ResourceOperation::Count, &["order_id"]);
        assert!(order_count.is_some());
        assert_eq!(
            order_count.unwrap().template,
            "orders/{order_id}/events/count"
        );

        // Product events path
        let product_events = get_path(Event::PATHS, ResourceOperation::All, &["product_id"]);
        assert!(product_events.is_some());
        assert_eq!(
            product_events.unwrap().template,
            "products/{product_id}/events"
        );

        // Product events count path
        let product_count = get_path(Event::PATHS, ResourceOperation::Count, &["product_id"]);
        assert!(product_count.is_some());
        assert_eq!(
            product_count.unwrap().template,
            "products/{product_id}/events/count"
        );
    }

    #[test]
    fn test_event_list_params() {
        let params = EventListParams {
            limit: Some(50),
            filter: Some("Product".to_string()),
            verb: Some("create".to_string()),
            since_id: Some(100),
            ..Default::default()
        };

        let json = serde_json::to_value(&params).unwrap();

        assert_eq!(json["limit"], 50);
        assert_eq!(json["filter"], "Product");
        assert_eq!(json["verb"], "create");
        assert_eq!(json["since_id"], 100);
    }

    #[test]
    fn test_event_constants() {
        assert_eq!(Event::NAME, "Event");
        assert_eq!(Event::PLURAL, "events");
    }

    #[test]
    fn test_event_get_id() {
        let event_with_id = Event {
            id: Some(677313116),
            verb: Some("create".to_string()),
            ..Default::default()
        };
        assert_eq!(event_with_id.get_id(), Some(677313116));

        let event_without_id = Event::default();
        assert_eq!(event_without_id.get_id(), None);
    }
}

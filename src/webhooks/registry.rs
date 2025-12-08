//! Webhook registry for managing webhook registrations.
//!
//! This module provides the [`WebhookRegistry`] struct for storing and managing
//! webhook registrations locally, then syncing them with Shopify via GraphQL API.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::webhooks::{WebhookRegistry, WebhookRegistrationBuilder};
//! use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
//!
//! let mut registry = WebhookRegistry::new();
//!
//! // Add registrations
//! registry.add_registration(
//!     WebhookRegistrationBuilder::new(
//!         WebhookTopic::OrdersCreate,
//!         "/webhooks/orders/create".to_string(),
//!     )
//!     .build()
//! );
//!
//! // Get a registration
//! let registration = registry.get_registration(&WebhookTopic::OrdersCreate);
//! assert!(registration.is_some());
//! ```

use std::collections::HashMap;

use crate::auth::Session;
use crate::clients::GraphqlClient;
use crate::config::ShopifyConfig;

use super::errors::WebhookError;
use super::types::{WebhookRegistration, WebhookRegistrationResult, WebhookTopic};

/// Registry for managing webhook subscriptions.
///
/// `WebhookRegistry` stores webhook registrations in memory and provides
/// methods to sync them with Shopify via the GraphQL Admin API.
///
/// # Two-Phase Pattern
///
/// The registry follows a two-phase pattern:
///
/// 1. **Add Registration (Local)**: Use [`add_registration`](Self::add_registration)
///    to store webhook configuration in the in-memory registry
/// 2. **Register with Shopify (Remote)**: Use [`register`](Self::register) or
///    [`register_all`](Self::register_all) to sync registrations with Shopify
///
/// This pattern allows apps to configure webhooks at startup and register
/// them later when a valid session is available.
///
/// # Thread Safety
///
/// `WebhookRegistry` is `Send + Sync`, making it safe to share across threads.
///
/// # Smart Registration
///
/// When registering webhooks, the registry performs "smart registration":
/// - Queries existing subscriptions from Shopify
/// - Compares configuration to detect changes
/// - Only creates/updates when necessary
/// - Avoids unnecessary API calls
///
/// # Example
///
/// ```rust
/// use shopify_api::webhooks::{WebhookRegistry, WebhookRegistrationBuilder};
/// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
///
/// // Create a registry and add registrations
/// let mut registry = WebhookRegistry::new();
///
/// registry.add_registration(
///     WebhookRegistrationBuilder::new(
///         WebhookTopic::OrdersCreate,
///         "/api/webhooks/orders".to_string(),
///     )
///     .build()
/// );
///
/// // Later, when you have a session:
/// // let results = registry.register_all(&session, &config).await?;
/// ```
#[derive(Debug, Default)]
pub struct WebhookRegistry {
    /// Internal storage for webhook registrations, keyed by topic.
    registrations: HashMap<WebhookTopic, WebhookRegistration>,
}

// Verify WebhookRegistry is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WebhookRegistry>();
};

impl WebhookRegistry {
    /// Creates a new empty webhook registry.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::WebhookRegistry;
    ///
    /// let registry = WebhookRegistry::new();
    /// assert!(registry.list_registrations().is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
        }
    }

    /// Adds a webhook registration to the registry.
    ///
    /// If a registration for the same topic already exists, it will be replaced.
    /// Returns `&mut Self` to allow method chaining.
    ///
    /// # Arguments
    ///
    /// * `registration` - The webhook registration to add
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::{WebhookRegistry, WebhookRegistrationBuilder};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    ///
    /// // Method chaining
    /// registry
    ///     .add_registration(
    ///         WebhookRegistrationBuilder::new(
    ///             WebhookTopic::OrdersCreate,
    ///             "/webhooks/orders/create".to_string(),
    ///         )
    ///         .build()
    ///     )
    ///     .add_registration(
    ///         WebhookRegistrationBuilder::new(
    ///             WebhookTopic::ProductsUpdate,
    ///             "/webhooks/products/update".to_string(),
    ///         )
    ///         .build()
    ///     );
    ///
    /// assert_eq!(registry.list_registrations().len(), 2);
    /// ```
    pub fn add_registration(&mut self, registration: WebhookRegistration) -> &mut Self {
        self.registrations
            .insert(registration.topic, registration);
        self
    }

    /// Gets a webhook registration by topic.
    ///
    /// Returns `None` if no registration exists for the given topic.
    ///
    /// # Arguments
    ///
    /// * `topic` - The webhook topic to look up
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::{WebhookRegistry, WebhookRegistrationBuilder};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// registry.add_registration(
    ///     WebhookRegistrationBuilder::new(
    ///         WebhookTopic::OrdersCreate,
    ///         "/webhooks".to_string(),
    ///     )
    ///     .build()
    /// );
    ///
    /// // Found
    /// assert!(registry.get_registration(&WebhookTopic::OrdersCreate).is_some());
    ///
    /// // Not found
    /// assert!(registry.get_registration(&WebhookTopic::ProductsCreate).is_none());
    /// ```
    #[must_use]
    pub fn get_registration(&self, topic: &WebhookTopic) -> Option<&WebhookRegistration> {
        self.registrations.get(topic)
    }

    /// Lists all webhook registrations in the registry.
    ///
    /// Returns a vector of references to all registrations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::webhooks::{WebhookRegistry, WebhookRegistrationBuilder};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// registry
    ///     .add_registration(
    ///         WebhookRegistrationBuilder::new(
    ///             WebhookTopic::OrdersCreate,
    ///             "/webhooks/orders".to_string(),
    ///         )
    ///         .build()
    ///     )
    ///     .add_registration(
    ///         WebhookRegistrationBuilder::new(
    ///             WebhookTopic::ProductsCreate,
    ///             "/webhooks/products".to_string(),
    ///         )
    ///         .build()
    ///     );
    ///
    /// let registrations = registry.list_registrations();
    /// assert_eq!(registrations.len(), 2);
    /// ```
    #[must_use]
    pub fn list_registrations(&self) -> Vec<&WebhookRegistration> {
        self.registrations.values().collect()
    }

    /// Registers a single webhook with Shopify.
    ///
    /// This method performs "smart registration":
    /// - Queries existing subscriptions from Shopify
    /// - Compares configuration to detect changes
    /// - Creates new subscription if none exists
    /// - Updates existing subscription if configuration differs
    /// - Returns `AlreadyRegistered` if configuration matches
    ///
    /// # Arguments
    ///
    /// * `session` - The authenticated session for API calls
    /// * `config` - The SDK configuration (must have `host` set)
    /// * `topic` - The webhook topic to register
    ///
    /// # Errors
    ///
    /// Returns `WebhookError::HostNotConfigured` if `config.host()` is `None`.
    /// Returns `WebhookError::RegistrationNotFound` if the topic is not in the registry.
    /// Returns `WebhookError::GraphqlError` for underlying API errors.
    /// Returns `WebhookError::ShopifyError` for userErrors in the response.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::webhooks::{WebhookRegistry, WebhookRegistrationBuilder};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// registry.add_registration(
    ///     WebhookRegistrationBuilder::new(
    ///         WebhookTopic::OrdersCreate,
    ///         "/webhooks/orders".to_string(),
    ///     )
    ///     .build()
    /// );
    ///
    /// let result = registry.register(&session, &config, &WebhookTopic::OrdersCreate).await?;
    /// ```
    pub async fn register(
        &self,
        session: &Session,
        config: &ShopifyConfig,
        topic: &WebhookTopic,
    ) -> Result<WebhookRegistrationResult, WebhookError> {
        // Check that host is configured
        let host = config.host().ok_or(WebhookError::HostNotConfigured)?;

        // Check that registration exists
        let registration = self
            .get_registration(topic)
            .ok_or_else(|| WebhookError::RegistrationNotFound {
                topic: topic.clone(),
            })?;

        // Construct callback URL
        let callback_url = format!("{}{}", host.as_ref(), registration.path);

        // Convert topic to GraphQL format
        let graphql_topic = topic_to_graphql_format(topic);

        // Create GraphQL client
        let client = GraphqlClient::new(session, Some(config));

        // Query existing webhook subscription
        let existing = self
            .query_existing_subscription(&client, &graphql_topic)
            .await?;

        match existing {
            Some((id, existing_config)) => {
                // Compare configurations
                if self.config_matches(&existing_config, &callback_url, registration) {
                    Ok(WebhookRegistrationResult::AlreadyRegistered { id })
                } else {
                    // Update existing subscription
                    self.update_subscription(&client, &id, &callback_url, registration)
                        .await
                }
            }
            None => {
                // Create new subscription
                self.create_subscription(&client, &graphql_topic, &callback_url, registration)
                    .await
            }
        }
    }

    /// Registers all webhooks in the registry with Shopify.
    ///
    /// Iterates through all registrations and calls [`register`](Self::register) for each.
    /// Continues processing even if individual registrations fail.
    ///
    /// # Arguments
    ///
    /// * `session` - The authenticated session for API calls
    /// * `config` - The SDK configuration (must have `host` set)
    ///
    /// # Errors
    ///
    /// Returns `WebhookError::HostNotConfigured` if `config.host()` is `None`.
    /// Individual registration failures are captured in `WebhookRegistrationResult::Failed`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::webhooks::{WebhookRegistry, WebhookRegistrationBuilder, WebhookRegistrationResult};
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// registry.add_registration(/* ... */);
    ///
    /// let results = registry.register_all(&session, &config).await?;
    /// for result in results {
    ///     match result {
    ///         WebhookRegistrationResult::Created { id } => println!("Created: {}", id),
    ///         WebhookRegistrationResult::Failed(err) => println!("Failed: {}", err),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn register_all(
        &self,
        session: &Session,
        config: &ShopifyConfig,
    ) -> Result<Vec<WebhookRegistrationResult>, WebhookError> {
        // Check that host is configured first (fail fast)
        if config.host().is_none() {
            return Err(WebhookError::HostNotConfigured);
        }

        let mut results = Vec::new();

        for registration in self.registrations.values() {
            let result = match self.register(session, config, &registration.topic).await {
                Ok(result) => result,
                Err(error) => WebhookRegistrationResult::Failed(error),
            };
            results.push(result);
        }

        Ok(results)
    }

    /// Unregisters a webhook from Shopify.
    ///
    /// Queries for the existing webhook subscription and deletes it.
    ///
    /// # Arguments
    ///
    /// * `session` - The authenticated session for API calls
    /// * `config` - The SDK configuration
    /// * `topic` - The webhook topic to unregister
    ///
    /// # Errors
    ///
    /// Returns `WebhookError::SubscriptionNotFound` if the webhook doesn't exist in Shopify.
    /// Returns `WebhookError::GraphqlError` for underlying API errors.
    /// Returns `WebhookError::ShopifyError` for userErrors in the response.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::webhooks::WebhookRegistry;
    /// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let registry = WebhookRegistry::new();
    /// registry.unregister(&session, &config, &WebhookTopic::OrdersCreate).await?;
    /// ```
    pub async fn unregister(
        &self,
        session: &Session,
        config: &ShopifyConfig,
        topic: &WebhookTopic,
    ) -> Result<(), WebhookError> {
        // Convert topic to GraphQL format
        let graphql_topic = topic_to_graphql_format(topic);

        // Create GraphQL client
        let client = GraphqlClient::new(session, Some(config));

        // Query existing webhook subscription
        let existing = self
            .query_existing_subscription(&client, &graphql_topic)
            .await?;

        match existing {
            Some((id, _)) => {
                // Delete the subscription
                self.delete_subscription(&client, &id).await
            }
            None => Err(WebhookError::SubscriptionNotFound {
                topic: topic.clone(),
            }),
        }
    }

    /// Unregisters all webhooks in the registry from Shopify.
    ///
    /// Iterates through all registrations and calls [`unregister`](Self::unregister) for each.
    /// Continues processing even if individual unregistrations fail.
    ///
    /// # Arguments
    ///
    /// * `session` - The authenticated session for API calls
    /// * `config` - The SDK configuration
    ///
    /// # Errors
    ///
    /// Returns the first error encountered, but continues processing all registrations.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_api::webhooks::WebhookRegistry;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// // ... add registrations ...
    ///
    /// registry.unregister_all(&session, &config).await?;
    /// ```
    pub async fn unregister_all(
        &self,
        session: &Session,
        config: &ShopifyConfig,
    ) -> Result<(), WebhookError> {
        let mut first_error: Option<WebhookError> = None;

        for registration in self.registrations.values() {
            if let Err(error) = self.unregister(session, config, &registration.topic).await {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Queries Shopify for an existing webhook subscription by topic.
    async fn query_existing_subscription(
        &self,
        client: &GraphqlClient,
        graphql_topic: &str,
    ) -> Result<Option<(String, ExistingWebhookConfig)>, WebhookError> {
        let query = format!(
            r#"
            query {{
                webhookSubscriptions(first: 1, topics: [{topic}]) {{
                    edges {{
                        node {{
                            id
                            endpoint {{
                                ... on WebhookHttpEndpoint {{
                                    callbackUrl
                                }}
                            }}
                            includeFields
                            metafieldNamespaces
                            filter
                        }}
                    }}
                }}
            }}
            "#,
            topic = graphql_topic
        );

        let response = client.query(&query, None, None, None).await?;

        // Parse the response
        let edges = response.body["data"]["webhookSubscriptions"]["edges"]
            .as_array()
            .ok_or_else(|| WebhookError::ShopifyError {
                message: "Invalid response structure".to_string(),
            })?;

        if edges.is_empty() {
            return Ok(None);
        }

        let node = &edges[0]["node"];
        let id = node["id"]
            .as_str()
            .ok_or_else(|| WebhookError::ShopifyError {
                message: "Missing webhook ID".to_string(),
            })?
            .to_string();

        let callback_url = node["endpoint"]["callbackUrl"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let include_fields = node["includeFields"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let metafield_namespaces = node["metafieldNamespaces"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        let filter = node["filter"].as_str().map(String::from);

        Ok(Some((
            id,
            ExistingWebhookConfig {
                callback_url,
                include_fields,
                metafield_namespaces,
                filter,
            },
        )))
    }

    /// Compares existing webhook configuration with desired configuration.
    fn config_matches(
        &self,
        existing: &ExistingWebhookConfig,
        callback_url: &str,
        registration: &WebhookRegistration,
    ) -> bool {
        existing.callback_url == callback_url
            && existing.include_fields == registration.include_fields
            && existing.metafield_namespaces == registration.metafield_namespaces
            && existing.filter == registration.filter
    }

    /// Creates a new webhook subscription in Shopify.
    async fn create_subscription(
        &self,
        client: &GraphqlClient,
        graphql_topic: &str,
        callback_url: &str,
        registration: &WebhookRegistration,
    ) -> Result<WebhookRegistrationResult, WebhookError> {
        let include_fields_input = registration
            .include_fields
            .as_ref()
            .map(|fields| {
                let quoted: Vec<String> = fields.iter().map(|f| format!("\"{}\"", f)).collect();
                format!(", includeFields: [{}]", quoted.join(", "))
            })
            .unwrap_or_default();

        let metafield_namespaces_input = registration
            .metafield_namespaces
            .as_ref()
            .map(|ns| {
                let quoted: Vec<String> = ns.iter().map(|n| format!("\"{}\"", n)).collect();
                format!(", metafieldNamespaces: [{}]", quoted.join(", "))
            })
            .unwrap_or_default();

        let filter_input = registration
            .filter
            .as_ref()
            .map(|f| format!(", filter: \"{}\"", f))
            .unwrap_or_default();

        let mutation = format!(
            r#"
            mutation {{
                webhookSubscriptionCreate(
                    topic: {topic},
                    webhookSubscription: {{
                        callbackUrl: "{callback_url}"{include_fields}{metafield_namespaces}{filter}
                    }}
                ) {{
                    webhookSubscription {{
                        id
                    }}
                    userErrors {{
                        field
                        message
                    }}
                }}
            }}
            "#,
            topic = graphql_topic,
            callback_url = callback_url,
            include_fields = include_fields_input,
            metafield_namespaces = metafield_namespaces_input,
            filter = filter_input
        );

        let response = client.query(&mutation, None, None, None).await?;

        // Check for userErrors
        let user_errors = &response.body["data"]["webhookSubscriptionCreate"]["userErrors"];
        if let Some(errors) = user_errors.as_array() {
            if !errors.is_empty() {
                let messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e["message"].as_str().map(String::from))
                    .collect();
                return Err(WebhookError::ShopifyError {
                    message: messages.join("; "),
                });
            }
        }

        // Get the created subscription ID
        let id = response.body["data"]["webhookSubscriptionCreate"]["webhookSubscription"]["id"]
            .as_str()
            .ok_or_else(|| WebhookError::ShopifyError {
                message: "Missing webhook subscription ID in response".to_string(),
            })?
            .to_string();

        Ok(WebhookRegistrationResult::Created { id })
    }

    /// Updates an existing webhook subscription in Shopify.
    async fn update_subscription(
        &self,
        client: &GraphqlClient,
        id: &str,
        callback_url: &str,
        registration: &WebhookRegistration,
    ) -> Result<WebhookRegistrationResult, WebhookError> {
        let include_fields_input = registration
            .include_fields
            .as_ref()
            .map(|fields| {
                let quoted: Vec<String> = fields.iter().map(|f| format!("\"{}\"", f)).collect();
                format!(", includeFields: [{}]", quoted.join(", "))
            })
            .unwrap_or_default();

        let metafield_namespaces_input = registration
            .metafield_namespaces
            .as_ref()
            .map(|ns| {
                let quoted: Vec<String> = ns.iter().map(|n| format!("\"{}\"", n)).collect();
                format!(", metafieldNamespaces: [{}]", quoted.join(", "))
            })
            .unwrap_or_default();

        let filter_input = registration
            .filter
            .as_ref()
            .map(|f| format!(", filter: \"{}\"", f))
            .unwrap_or_default();

        let mutation = format!(
            r#"
            mutation {{
                webhookSubscriptionUpdate(
                    id: "{id}",
                    webhookSubscription: {{
                        callbackUrl: "{callback_url}"{include_fields}{metafield_namespaces}{filter}
                    }}
                ) {{
                    webhookSubscription {{
                        id
                    }}
                    userErrors {{
                        field
                        message
                    }}
                }}
            }}
            "#,
            id = id,
            callback_url = callback_url,
            include_fields = include_fields_input,
            metafield_namespaces = metafield_namespaces_input,
            filter = filter_input
        );

        let response = client.query(&mutation, None, None, None).await?;

        // Check for userErrors
        let user_errors = &response.body["data"]["webhookSubscriptionUpdate"]["userErrors"];
        if let Some(errors) = user_errors.as_array() {
            if !errors.is_empty() {
                let messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e["message"].as_str().map(String::from))
                    .collect();
                return Err(WebhookError::ShopifyError {
                    message: messages.join("; "),
                });
            }
        }

        Ok(WebhookRegistrationResult::Updated { id: id.to_string() })
    }

    /// Deletes a webhook subscription from Shopify.
    async fn delete_subscription(
        &self,
        client: &GraphqlClient,
        id: &str,
    ) -> Result<(), WebhookError> {
        let mutation = format!(
            r#"
            mutation {{
                webhookSubscriptionDelete(id: "{id}") {{
                    deletedWebhookSubscriptionId
                    userErrors {{
                        field
                        message
                    }}
                }}
            }}
            "#,
            id = id
        );

        let response = client.query(&mutation, None, None, None).await?;

        // Check for userErrors
        let user_errors = &response.body["data"]["webhookSubscriptionDelete"]["userErrors"];
        if let Some(errors) = user_errors.as_array() {
            if !errors.is_empty() {
                let messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e["message"].as_str().map(String::from))
                    .collect();
                return Err(WebhookError::ShopifyError {
                    message: messages.join("; "),
                });
            }
        }

        Ok(())
    }
}

/// Internal struct for holding existing webhook configuration from Shopify.
struct ExistingWebhookConfig {
    callback_url: String,
    include_fields: Option<Vec<String>>,
    metafield_namespaces: Option<Vec<String>>,
    filter: Option<String>,
}

/// Converts a `WebhookTopic` to GraphQL enum format.
///
/// Transforms the serde format (e.g., "orders/create") to the GraphQL
/// enum format (e.g., "ORDERS_CREATE").
///
/// # Example
///
/// ```rust,ignore
/// use shopify_api::rest::resources::v2025_10::common::WebhookTopic;
///
/// let graphql_format = topic_to_graphql_format(&WebhookTopic::OrdersCreate);
/// assert_eq!(graphql_format, "ORDERS_CREATE");
/// ```
fn topic_to_graphql_format(topic: &WebhookTopic) -> String {
    // Serialize topic to get the serde format (e.g., "orders/create")
    let json_str = serde_json::to_string(topic).unwrap_or_default();

    // Remove quotes, replace "/" and "_" with "_", and uppercase
    json_str
        .trim_matches('"')
        .replace('/', "_")
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_registry_new_creates_empty_registry() {
        let registry = WebhookRegistry::new();
        assert!(registry.list_registrations().is_empty());
    }

    #[test]
    fn test_add_registration_stores_registration() {
        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            super::super::types::WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                "/webhooks/orders".to_string(),
            )
            .build(),
        );

        assert_eq!(registry.list_registrations().len(), 1);
        assert!(registry.get_registration(&WebhookTopic::OrdersCreate).is_some());
    }

    #[test]
    fn test_add_registration_overwrites_same_topic() {
        let mut registry = WebhookRegistry::new();

        // Add first registration
        registry.add_registration(
            super::super::types::WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                "/webhooks/v1/orders".to_string(),
            )
            .build(),
        );

        // Add second registration with same topic
        registry.add_registration(
            super::super::types::WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                "/webhooks/v2/orders".to_string(),
            )
            .build(),
        );

        assert_eq!(registry.list_registrations().len(), 1);

        let registration = registry.get_registration(&WebhookTopic::OrdersCreate).unwrap();
        assert_eq!(registration.path, "/webhooks/v2/orders");
    }

    #[test]
    fn test_get_registration_returns_none_for_missing_topic() {
        let registry = WebhookRegistry::new();
        assert!(registry.get_registration(&WebhookTopic::OrdersCreate).is_none());
    }

    #[test]
    fn test_list_registrations_returns_all() {
        let mut registry = WebhookRegistry::new();

        registry
            .add_registration(
                super::super::types::WebhookRegistrationBuilder::new(
                    WebhookTopic::OrdersCreate,
                    "/webhooks/orders".to_string(),
                )
                .build(),
            )
            .add_registration(
                super::super::types::WebhookRegistrationBuilder::new(
                    WebhookTopic::ProductsCreate,
                    "/webhooks/products".to_string(),
                )
                .build(),
            )
            .add_registration(
                super::super::types::WebhookRegistrationBuilder::new(
                    WebhookTopic::CustomersCreate,
                    "/webhooks/customers".to_string(),
                )
                .build(),
            );

        let registrations = registry.list_registrations();
        assert_eq!(registrations.len(), 3);
    }

    #[test]
    fn test_webhook_registry_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WebhookRegistry>();
    }

    #[test]
    fn test_topic_to_graphql_format_orders_create() {
        let topic = WebhookTopic::OrdersCreate;
        let graphql_format = topic_to_graphql_format(&topic);
        assert_eq!(graphql_format, "ORDERS_CREATE");
    }

    #[test]
    fn test_topic_to_graphql_format_products_update() {
        let topic = WebhookTopic::ProductsUpdate;
        let graphql_format = topic_to_graphql_format(&topic);
        assert_eq!(graphql_format, "PRODUCTS_UPDATE");
    }

    #[test]
    fn test_topic_to_graphql_format_app_uninstalled() {
        let topic = WebhookTopic::AppUninstalled;
        let graphql_format = topic_to_graphql_format(&topic);
        assert_eq!(graphql_format, "APP_UNINSTALLED");
    }

    #[test]
    fn test_topic_to_graphql_format_inventory_levels_update() {
        let topic = WebhookTopic::InventoryLevelsUpdate;
        let graphql_format = topic_to_graphql_format(&topic);
        assert_eq!(graphql_format, "INVENTORY_LEVELS_UPDATE");
    }

    #[test]
    fn test_add_registration_returns_mut_self_for_chaining() {
        let mut registry = WebhookRegistry::new();

        // Test method chaining
        let chain_result = registry
            .add_registration(
                super::super::types::WebhookRegistrationBuilder::new(
                    WebhookTopic::OrdersCreate,
                    "/webhooks/orders".to_string(),
                )
                .build(),
            )
            .add_registration(
                super::super::types::WebhookRegistrationBuilder::new(
                    WebhookTopic::ProductsCreate,
                    "/webhooks/products".to_string(),
                )
                .build(),
            );

        // Verify chaining worked
        assert_eq!(chain_result.list_registrations().len(), 2);
    }
}

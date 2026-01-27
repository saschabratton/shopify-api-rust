//! Webhook registry for managing webhook registrations.
//!
//! This module provides the [`WebhookRegistry`] struct for storing and managing
//! webhook registrations locally, then syncing them with Shopify via GraphQL API.
//!
//! # Example
//!
//! ```rust
//! use shopify_sdk::webhooks::{WebhookRegistry, WebhookRegistrationBuilder, WebhookDeliveryMethod};
//! use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
//!
//! let mut registry = WebhookRegistry::new();
//!
//! // Add registrations with HTTP delivery
//! registry.add_registration(
//!     WebhookRegistrationBuilder::new(
//!         WebhookTopic::OrdersCreate,
//!         WebhookDeliveryMethod::Http {
//!             uri: "https://example.com/webhooks/orders/create".to_string(),
//!         },
//!     )
//!     .build()
//! );
//!
//! // Add registrations with EventBridge delivery
//! registry.add_registration(
//!     WebhookRegistrationBuilder::new(
//!         WebhookTopic::ProductsUpdate,
//!         WebhookDeliveryMethod::EventBridge {
//!             arn: "arn:aws:events:us-east-1::event-source/aws.partner/shopify.com/123/source".to_string(),
//!         },
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
use super::types::{
    WebhookRegistrationBuilder,
    WebhookDeliveryMethod, WebhookHandler, WebhookRegistration, WebhookRegistrationResult,
    WebhookTopic,
};
use super::verification::{verify_webhook, WebhookRequest};

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
/// # Delivery Methods
///
/// The registry supports three delivery methods:
/// - **HTTP**: Webhooks delivered via HTTP POST to a callback URL
/// - **EventBridge**: Webhooks delivered to Amazon EventBridge
/// - **Pub/Sub**: Webhooks delivered to Google Cloud Pub/Sub
///
/// # Example
///
/// ```rust
/// use shopify_sdk::webhooks::{WebhookRegistry, WebhookRegistrationBuilder, WebhookDeliveryMethod};
/// use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
///
/// // Create a registry and add registrations
/// let mut registry = WebhookRegistry::new();
///
/// registry.add_registration(
///     WebhookRegistrationBuilder::new(
///         WebhookTopic::OrdersCreate,
///         WebhookDeliveryMethod::Http {
///             uri: "https://example.com/api/webhooks/orders".to_string(),
///         },
///     )
///     .build()
/// );
///
/// // Later, when you have a session:
/// // let results = registry.register_all(&session, &config).await?;
/// ```
#[derive(Default)]
pub struct WebhookRegistry {
    /// Internal storage for webhook registrations, keyed by topic.
    registrations: HashMap<WebhookTopic, WebhookRegistration>,
    /// Internal storage for webhook handlers, keyed by topic.
    handlers: HashMap<WebhookTopic, Box<dyn WebhookHandler>>,
}

// Implement Debug manually since trait objects don't implement Debug
impl std::fmt::Debug for WebhookRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebhookRegistry")
            .field("registrations", &self.registrations)
            .field("handlers", &format!("<{} handlers>", self.handlers.len()))
            .finish()
    }
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
    /// use shopify_sdk::webhooks::WebhookRegistry;
    ///
    /// let registry = WebhookRegistry::new();
    /// assert!(registry.list_registrations().is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            registrations: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// Adds a webhook registration to the registry.
    ///
    /// If a registration for the same topic already exists, it will be replaced.
    /// If the registration contains a handler, the handler is extracted and stored
    /// separately in the handlers map.
    /// Returns `&mut Self` to allow method chaining.
    ///
    /// # Arguments
    ///
    /// * `registration` - The webhook registration to add
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::webhooks::{WebhookRegistry, WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    ///
    /// // Method chaining with different delivery methods
    /// registry
    ///     .add_registration(
    ///         WebhookRegistrationBuilder::new(
    ///             WebhookTopic::OrdersCreate,
    ///             WebhookDeliveryMethod::Http {
    ///                 uri: "https://example.com/webhooks/orders/create".to_string(),
    ///             },
    ///         )
    ///         .build()
    ///     )
    ///     .add_registration(
    ///         WebhookRegistrationBuilder::new(
    ///             WebhookTopic::ProductsUpdate,
    ///             WebhookDeliveryMethod::EventBridge {
    ///                 arn: "arn:aws:events:us-east-1::event-source/test".to_string(),
    ///             },
    ///         )
    ///         .build()
    ///     );
    ///
    /// assert_eq!(registry.list_registrations().len(), 2);
    /// ```
    pub fn add_registration(&mut self, mut registration: WebhookRegistration) -> &mut Self {
        let topic = registration.topic;

        // Extract handler if present and store separately
        if let Some(handler) = registration.handler.take() {
            self.handlers.insert(topic, handler);
        }

        self.registrations.insert(topic, registration);
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
    /// use shopify_sdk::webhooks::{WebhookRegistry, WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// registry.add_registration(
    ///     WebhookRegistrationBuilder::new(
    ///         WebhookTopic::OrdersCreate,
    ///         WebhookDeliveryMethod::Http {
    ///             uri: "https://example.com/webhooks".to_string(),
    ///         },
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
    /// use shopify_sdk::webhooks::{WebhookRegistry, WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// registry
    ///     .add_registration(
    ///         WebhookRegistrationBuilder::new(
    ///             WebhookTopic::OrdersCreate,
    ///             WebhookDeliveryMethod::Http {
    ///                 uri: "https://example.com/webhooks/orders".to_string(),
    ///             },
    ///         )
    ///         .build()
    ///     )
    ///     .add_registration(
    ///         WebhookRegistrationBuilder::new(
    ///             WebhookTopic::ProductsCreate,
    ///             WebhookDeliveryMethod::PubSub {
    ///                 project_id: "my-project".to_string(),
    ///                 topic_id: "webhooks".to_string(),
    ///             },
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

    /// Processes an incoming webhook request.
    ///
    /// This method verifies the webhook signature, looks up the appropriate handler,
    /// parses the payload, and invokes the handler.
    ///
    /// # Flow
    ///
    /// 1. Verify the webhook signature using [`verify_webhook`]
    /// 2. Look up the handler by topic
    /// 3. Parse the request body as JSON
    /// 4. Invoke the handler with the context and payload
    ///
    /// # Arguments
    ///
    /// * `config` - The Shopify configuration containing the API secret key
    /// * `request` - The incoming webhook request
    ///
    /// # Errors
    ///
    /// Returns `WebhookError::InvalidHmac` if signature verification fails.
    /// Returns `WebhookError::NoHandlerForTopic` if no handler is registered for the topic.
    /// Returns `WebhookError::PayloadParseError` if the body cannot be parsed as JSON.
    /// Returns any error returned by the handler.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::webhooks::{WebhookRegistry, WebhookRequest};
    ///
    /// let registry = WebhookRegistry::new();
    /// // ... add registrations with handlers ...
    ///
    /// // Process incoming webhook
    /// registry.process(&config, &request).await?;
    /// ```
    pub async fn process(
        &self,
        config: &ShopifyConfig,
        request: &WebhookRequest,
    ) -> Result<(), WebhookError> {
        // Step 1: Verify webhook signature and get context
        let context = verify_webhook(config, request)?;

        // Step 2: Look up handler by topic
        let handler = match context.topic() {
            Some(topic) => self.handlers.get(&topic),
            None => None,
        };

        let handler = handler.ok_or_else(|| WebhookError::NoHandlerForTopic {
            topic: context.topic_raw().to_string(),
        })?;

        // Step 3: Parse request body as JSON
        let payload: serde_json::Value =
            serde_json::from_slice(request.body()).map_err(|e| WebhookError::PayloadParseError {
                message: e.to_string(),
            })?;

        // Step 4: Invoke handler
        handler.handle(context, payload).await
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
    /// * `config` - The SDK configuration
    /// * `topic` - The webhook topic to register
    ///
    /// # Errors
    ///
    /// Returns `WebhookError::RegistrationNotFound` if the topic is not in the registry.
    /// Returns `WebhookError::GraphqlError` for underlying API errors.
    /// Returns `WebhookError::ShopifyError` for userErrors in the response.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::webhooks::{WebhookRegistry, WebhookRegistrationBuilder, WebhookDeliveryMethod};
    /// use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// registry.add_registration(
    ///     WebhookRegistrationBuilder::new(
    ///         WebhookTopic::OrdersCreate,
    ///         WebhookDeliveryMethod::Http {
    ///             uri: "https://example.com/webhooks/orders".to_string(),
    ///         },
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
        // Check that registration exists
        let registration = self
            .get_registration(topic)
            .ok_or_else(|| WebhookError::RegistrationNotFound {
                topic: topic.clone(),
            })?;

        // Convert topic to GraphQL format
        let graphql_topic = topic_to_graphql_format(topic);

        // Create GraphQL client
        let client = GraphqlClient::new(session, Some(config));

        // Query existing webhook subscription
        let existing = self
            .query_existing_subscription(&client, &graphql_topic, &registration.delivery_method)
            .await?;

        match existing {
            Some((id, existing_config)) => {
                // Compare configurations
                if self.config_matches(&existing_config, registration) {
                    Ok(WebhookRegistrationResult::AlreadyRegistered { id })
                } else {
                    // Update existing subscription
                    self.update_subscription(&client, &id, registration).await
                }
            }
            None => {
                // Create new subscription
                self.create_subscription(&client, &graphql_topic, registration)
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
    /// * `config` - The SDK configuration
    ///
    /// # Returns
    ///
    /// A vector of results for each registration.
    /// Individual registration failures are captured in `WebhookRegistrationResult::Failed`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shopify_sdk::webhooks::{WebhookRegistry, WebhookRegistrationBuilder, WebhookRegistrationResult, WebhookDeliveryMethod};
    /// use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
    ///
    /// let mut registry = WebhookRegistry::new();
    /// registry.add_registration(/* ... */);
    ///
    /// let results = registry.register_all(&session, &config).await;
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
    ) -> Vec<WebhookRegistrationResult> {
        let mut results = Vec::new();

        for registration in self.registrations.values() {
            let result = match self.register(session, config, &registration.topic).await {
                Ok(result) => result,
                Err(error) => WebhookRegistrationResult::Failed(error),
            };
            results.push(result);
        }

        results
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
    /// use shopify_sdk::webhooks::WebhookRegistry;
    /// use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
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
        // Get the registration to know the delivery method
        let registration = self
            .get_registration(topic)
            .ok_or_else(|| WebhookError::RegistrationNotFound {
                topic: topic.clone(),
            })?;

        // Convert topic to GraphQL format
        let graphql_topic = topic_to_graphql_format(topic);

        // Create GraphQL client
        let client = GraphqlClient::new(session, Some(config));

        // Query existing webhook subscription
        let existing = self
            .query_existing_subscription(&client, &graphql_topic, &registration.delivery_method)
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
    /// use shopify_sdk::webhooks::WebhookRegistry;
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

    /// Queries Shopify for an existing webhook subscription by topic and delivery method.
    async fn query_existing_subscription(
        &self,
        client: &GraphqlClient,
        graphql_topic: &str,
        delivery_method: &WebhookDeliveryMethod,
    ) -> Result<Option<(String, ExistingWebhookConfig)>, WebhookError> {
        let query = format!(
            r#"
            query {{
                webhookSubscriptions(first: 25, topics: [{topic}]) {{
                    edges {{
                        node {{
                            id
                            endpoint {{
                                ... on WebhookHttpEndpoint {{
                                    callbackUrl
                                }}
                                ... on WebhookEventBridgeEndpoint {{
                                    arn
                                }}
                                ... on WebhookPubSubEndpoint {{
                                    pubSubProject
                                    pubSubTopic
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

        // Find a matching subscription by delivery method
        for edge in edges {
            let node = &edge["node"];
            let endpoint = &node["endpoint"];

            // Parse endpoint and check if it matches the desired delivery method
            let parsed_delivery_method = if let Some(uri) = endpoint["callbackUrl"].as_str() {
                Some(WebhookDeliveryMethod::Http {
                    uri: uri.to_string(),
                })
            } else if let Some(arn) = endpoint["arn"].as_str() {
                Some(WebhookDeliveryMethod::EventBridge {
                    arn: arn.to_string(),
                })
            } else if let (Some(project), Some(topic)) = (
                endpoint["pubSubProject"].as_str(),
                endpoint["pubSubTopic"].as_str(),
            ) {
                Some(WebhookDeliveryMethod::PubSub {
                    project_id: project.to_string(),
                    topic_id: topic.to_string(),
                })
            } else {
                None
            };

            // Check if the delivery method type matches (we compare full method for exact match later)
            if let Some(ref parsed_method) = parsed_delivery_method {
                let type_matches = match (parsed_method, delivery_method) {
                    (WebhookDeliveryMethod::Http { .. }, WebhookDeliveryMethod::Http { .. }) => {
                        true
                    }
                    (
                        WebhookDeliveryMethod::EventBridge { .. },
                        WebhookDeliveryMethod::EventBridge { .. },
                    ) => true,
                    (
                        WebhookDeliveryMethod::PubSub { .. },
                        WebhookDeliveryMethod::PubSub { .. },
                    ) => true,
                    _ => false,
                };

                if type_matches {
                    let id = node["id"]
                        .as_str()
                        .ok_or_else(|| WebhookError::ShopifyError {
                            message: "Missing webhook ID".to_string(),
                        })?
                        .to_string();

                    let include_fields = node["includeFields"].as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    });

                    let metafield_namespaces = node["metafieldNamespaces"].as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    });

                    let filter = node["filter"].as_str().map(String::from);

                    return Ok(Some((
                        id,
                        ExistingWebhookConfig {
                            delivery_method: parsed_method.clone(),
                            include_fields,
                            metafield_namespaces,
                            filter,
                        },
                    )));
                }
            }
        }

        Ok(None)
    }

    /// Compares existing webhook configuration with desired configuration.
    fn config_matches(
        &self,
        existing: &ExistingWebhookConfig,
        registration: &WebhookRegistration,
    ) -> bool {
        existing.delivery_method == registration.delivery_method
            && existing.include_fields == registration.include_fields
            && existing.metafield_namespaces == registration.metafield_namespaces
            && existing.filter == registration.filter
    }

    /// Creates a new webhook subscription in Shopify.
    async fn create_subscription(
        &self,
        client: &GraphqlClient,
        graphql_topic: &str,
        registration: &WebhookRegistration,
    ) -> Result<WebhookRegistrationResult, WebhookError> {
        let delivery_input = build_delivery_input(&registration.delivery_method);

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
                        {delivery}{include_fields}{metafield_namespaces}{filter}
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
            delivery = delivery_input,
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
        registration: &WebhookRegistration,
    ) -> Result<WebhookRegistrationResult, WebhookError> {
        let delivery_input = build_delivery_input(&registration.delivery_method);

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
                        {delivery}{include_fields}{metafield_namespaces}{filter}
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
            delivery = delivery_input,
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
#[derive(Debug, Clone)]
struct ExistingWebhookConfig {
    delivery_method: WebhookDeliveryMethod,
    include_fields: Option<Vec<String>>,
    metafield_namespaces: Option<Vec<String>>,
    filter: Option<String>,
}

/// Builds the GraphQL input for the delivery method.
///
/// Uses the unified `uri` field which accepts:
/// - HTTPS URLs for HTTP delivery
/// - ARNs for EventBridge delivery
/// - `pubsub://{project}:{topic}` URIs for Pub/Sub delivery
fn build_delivery_input(delivery_method: &WebhookDeliveryMethod) -> String {
    match delivery_method {
        WebhookDeliveryMethod::Http { uri } => {
            format!("uri: \"{}\"", uri)
        }
        WebhookDeliveryMethod::EventBridge { arn } => {
            format!("uri: \"{}\"", arn)
        }
        WebhookDeliveryMethod::PubSub {
            project_id,
            topic_id,
        } => {
            format!("uri: \"pubsub://{}:{}\"", project_id, topic_id)
        }
    }
}

/// Converts a `WebhookTopic` to GraphQL enum format.
///
/// Transforms the serde format (e.g., "orders/create") to the GraphQL
/// enum format (e.g., "ORDERS_CREATE").
///
/// # Example
///
/// ```rust,ignore
/// use shopify_sdk::rest::resources::v2025_10::common::WebhookTopic;
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
    use crate::auth::oauth::hmac::compute_signature_base64;
    use crate::config::{ApiKey, ApiSecretKey};
    use crate::webhooks::types::BoxFuture;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // Test handler implementation
    struct TestHandler {
        invoked: Arc<AtomicBool>,
    }

    impl WebhookHandler for TestHandler {
        fn handle<'a>(
            &'a self,
            _context: super::super::verification::WebhookContext,
            _payload: serde_json::Value,
        ) -> BoxFuture<'a, Result<(), WebhookError>> {
            let invoked = self.invoked.clone();
            Box::pin(async move {
                invoked.store(true, Ordering::SeqCst);
                Ok(())
            })
        }
    }

    // Error handler implementation for testing error propagation
    struct ErrorHandler {
        error_message: String,
    }

    impl WebhookHandler for ErrorHandler {
        fn handle<'a>(
            &'a self,
            _context: super::super::verification::WebhookContext,
            _payload: serde_json::Value,
        ) -> BoxFuture<'a, Result<(), WebhookError>> {
            let message = self.error_message.clone();
            Box::pin(async move { Err(WebhookError::ShopifyError { message }) })
        }
    }

    // ========================================================================
    // Task Group 4 Tests: ExistingWebhookConfig
    // ========================================================================

    #[test]
    fn test_existing_config_with_http_delivery() {
        let config = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
            include_fields: Some(vec!["id".to_string()]),
            metafield_namespaces: None,
            filter: None,
        };

        assert!(matches!(
            config.delivery_method,
            WebhookDeliveryMethod::Http { .. }
        ));
    }

    #[test]
    fn test_existing_config_with_eventbridge_delivery() {
        let config = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::EventBridge {
                arn: "arn:aws:events:us-east-1::event-source/test".to_string(),
            },
            include_fields: None,
            metafield_namespaces: None,
            filter: Some("status:active".to_string()),
        };

        assert!(matches!(
            config.delivery_method,
            WebhookDeliveryMethod::EventBridge { .. }
        ));
        assert!(config.filter.is_some());
    }

    #[test]
    fn test_existing_config_with_pubsub_delivery() {
        let config = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::PubSub {
                project_id: "my-project".to_string(),
                topic_id: "my-topic".to_string(),
            },
            include_fields: None,
            metafield_namespaces: Some(vec!["custom".to_string()]),
            filter: None,
        };

        match config.delivery_method {
            WebhookDeliveryMethod::PubSub {
                project_id,
                topic_id,
            } => {
                assert_eq!(project_id, "my-project");
                assert_eq!(topic_id, "my-topic");
            }
            _ => panic!("Expected PubSub delivery method"),
        }
    }

    // ========================================================================
    // Task Group 5 Tests: GraphQL Query Parsing
    // ========================================================================

    #[test]
    fn test_build_delivery_input_http() {
        let method = WebhookDeliveryMethod::Http {
            uri: "https://example.com/webhooks".to_string(),
        };
        let input = build_delivery_input(&method);
        // Uses unified uri field per Shopify API 2025-10+
        assert_eq!(input, "uri: \"https://example.com/webhooks\"");
    }

    #[test]
    fn test_build_delivery_input_eventbridge() {
        let method = WebhookDeliveryMethod::EventBridge {
            arn: "arn:aws:events:us-east-1::event-source/test".to_string(),
        };
        let input = build_delivery_input(&method);
        // Uses unified uri field with ARN value
        assert_eq!(
            input,
            "uri: \"arn:aws:events:us-east-1::event-source/test\""
        );
    }

    #[test]
    fn test_build_delivery_input_pubsub() {
        let method = WebhookDeliveryMethod::PubSub {
            project_id: "my-project".to_string(),
            topic_id: "my-topic".to_string(),
        };
        let input = build_delivery_input(&method);
        // Uses unified uri field with pubsub:// URI format
        assert_eq!(input, "uri: \"pubsub://my-project:my-topic\"");
    }

    // ========================================================================
    // Task Group 6 Tests: config_matches()
    // ========================================================================

    #[test]
    fn test_config_matches_http_same_url() {
        let registry = WebhookRegistry::new();

        let existing = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
            include_fields: None,
            metafield_namespaces: None,
            filter: None,
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
        )
        .build();

        assert!(registry.config_matches(&existing, &registration));
    }

    #[test]
    fn test_config_matches_http_different_url() {
        let registry = WebhookRegistry::new();

        let existing = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
            include_fields: None,
            metafield_namespaces: None,
            filter: None,
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://different.com/webhooks".to_string(),
            },
        )
        .build();

        assert!(!registry.config_matches(&existing, &registration));
    }

    #[test]
    fn test_config_matches_eventbridge_same_arn() {
        let registry = WebhookRegistry::new();

        let existing = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::EventBridge {
                arn: "arn:aws:events:us-east-1::event-source/test".to_string(),
            },
            include_fields: None,
            metafield_namespaces: None,
            filter: None,
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::EventBridge {
                arn: "arn:aws:events:us-east-1::event-source/test".to_string(),
            },
        )
        .build();

        assert!(registry.config_matches(&existing, &registration));
    }

    #[test]
    fn test_config_matches_pubsub_same_project_and_topic() {
        let registry = WebhookRegistry::new();

        let existing = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::PubSub {
                project_id: "my-project".to_string(),
                topic_id: "my-topic".to_string(),
            },
            include_fields: None,
            metafield_namespaces: None,
            filter: None,
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::PubSub {
                project_id: "my-project".to_string(),
                topic_id: "my-topic".to_string(),
            },
        )
        .build();

        assert!(registry.config_matches(&existing, &registration));
    }

    #[test]
    fn test_config_matches_different_delivery_methods_never_match() {
        let registry = WebhookRegistry::new();

        let existing = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
            include_fields: None,
            metafield_namespaces: None,
            filter: None,
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::EventBridge {
                arn: "arn:aws:events:us-east-1::event-source/test".to_string(),
            },
        )
        .build();

        assert!(!registry.config_matches(&existing, &registration));
    }

    #[test]
    fn test_config_matches_includes_other_fields() {
        let registry = WebhookRegistry::new();

        let existing = ExistingWebhookConfig {
            delivery_method: WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
            include_fields: Some(vec!["id".to_string()]),
            metafield_namespaces: Some(vec!["custom".to_string()]),
            filter: Some("status:active".to_string()),
        };

        let registration = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
        )
        .include_fields(vec!["id".to_string()])
        .metafield_namespaces(vec!["custom".to_string()])
        .filter("status:active".to_string())
        .build();

        assert!(registry.config_matches(&existing, &registration));

        // Different filter should not match
        let registration_different = WebhookRegistrationBuilder::new(
            WebhookTopic::OrdersCreate,
            WebhookDeliveryMethod::Http {
                uri: "https://example.com/webhooks".to_string(),
            },
        )
        .include_fields(vec!["id".to_string()])
        .metafield_namespaces(vec!["custom".to_string()])
        .filter("status:inactive".to_string())
        .build();

        assert!(!registry.config_matches(&existing, &registration_different));
    }

    // ========================================================================
    // Task Group 8 Tests: register() and register_all() behavior
    // ========================================================================

    #[test]
    fn test_registry_accepts_http_delivery() {
        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks".to_string(),
                },
            )
            .build(),
        );

        let registration = registry.get_registration(&WebhookTopic::OrdersCreate).unwrap();
        assert!(matches!(
            registration.delivery_method,
            WebhookDeliveryMethod::Http { .. }
        ));
    }

    #[test]
    fn test_registry_accepts_eventbridge_delivery() {
        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::EventBridge {
                    arn: "arn:aws:events:us-east-1::event-source/test".to_string(),
                },
            )
            .build(),
        );

        let registration = registry.get_registration(&WebhookTopic::OrdersCreate).unwrap();
        assert!(matches!(
            registration.delivery_method,
            WebhookDeliveryMethod::EventBridge { .. }
        ));
    }

    #[test]
    fn test_registry_accepts_pubsub_delivery() {
        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::PubSub {
                    project_id: "my-project".to_string(),
                    topic_id: "my-topic".to_string(),
                },
            )
            .build(),
        );

        let registration = registry.get_registration(&WebhookTopic::OrdersCreate).unwrap();
        assert!(matches!(
            registration.delivery_method,
            WebhookDeliveryMethod::PubSub { .. }
        ));
    }

    #[test]
    fn test_registry_allows_mixed_delivery_methods() {
        let mut registry = WebhookRegistry::new();

        registry
            .add_registration(
                WebhookRegistrationBuilder::new(
                    WebhookTopic::OrdersCreate,
                    WebhookDeliveryMethod::Http {
                        uri: "https://example.com/webhooks".to_string(),
                    },
                )
                .build(),
            )
            .add_registration(
                WebhookRegistrationBuilder::new(
                    WebhookTopic::ProductsUpdate,
                    WebhookDeliveryMethod::EventBridge {
                        arn: "arn:aws:events:us-east-1::event-source/test".to_string(),
                    },
                )
                .build(),
            )
            .add_registration(
                WebhookRegistrationBuilder::new(
                    WebhookTopic::CustomersCreate,
                    WebhookDeliveryMethod::PubSub {
                        project_id: "my-project".to_string(),
                        topic_id: "my-topic".to_string(),
                    },
                )
                .build(),
            );

        assert_eq!(registry.list_registrations().len(), 3);

        // Verify each registration has the correct delivery method type
        assert!(matches!(
            registry
                .get_registration(&WebhookTopic::OrdersCreate)
                .unwrap()
                .delivery_method,
            WebhookDeliveryMethod::Http { .. }
        ));
        assert!(matches!(
            registry
                .get_registration(&WebhookTopic::ProductsUpdate)
                .unwrap()
                .delivery_method,
            WebhookDeliveryMethod::EventBridge { .. }
        ));
        assert!(matches!(
            registry
                .get_registration(&WebhookTopic::CustomersCreate)
                .unwrap()
                .delivery_method,
            WebhookDeliveryMethod::PubSub { .. }
        ));
    }

    // ========================================================================
    // Legacy Tests (updated for new API)
    // ========================================================================

    #[test]
    fn test_webhook_registry_new_creates_empty_registry() {
        let registry = WebhookRegistry::new();
        assert!(registry.list_registrations().is_empty());
    }

    #[test]
    fn test_add_registration_stores_registration() {
        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
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
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/v1/orders".to_string(),
                },
            )
            .build(),
        );

        // Add second registration with same topic but different URL
        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/v2/orders".to_string(),
                },
            )
            .build(),
        );

        assert_eq!(registry.list_registrations().len(), 1);

        let registration = registry.get_registration(&WebhookTopic::OrdersCreate).unwrap();
        match &registration.delivery_method {
            WebhookDeliveryMethod::Http { uri } => {
                assert_eq!(uri, "https://example.com/webhooks/v2/orders");
            }
            _ => panic!("Expected Http delivery method"),
        }
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
                WebhookRegistrationBuilder::new(
                    WebhookTopic::OrdersCreate,
                    WebhookDeliveryMethod::Http {
                        uri: "https://example.com/webhooks/orders".to_string(),
                    },
                )
                .build(),
            )
            .add_registration(
                WebhookRegistrationBuilder::new(
                    WebhookTopic::ProductsCreate,
                    WebhookDeliveryMethod::Http {
                        uri: "https://example.com/webhooks/products".to_string(),
                    },
                )
                .build(),
            )
            .add_registration(
                WebhookRegistrationBuilder::new(
                    WebhookTopic::CustomersCreate,
                    WebhookDeliveryMethod::Http {
                        uri: "https://example.com/webhooks/customers".to_string(),
                    },
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
                WebhookRegistrationBuilder::new(
                    WebhookTopic::OrdersCreate,
                    WebhookDeliveryMethod::Http {
                        uri: "https://example.com/webhooks/orders".to_string(),
                    },
                )
                .build(),
            )
            .add_registration(
                WebhookRegistrationBuilder::new(
                    WebhookTopic::ProductsCreate,
                    WebhookDeliveryMethod::Http {
                        uri: "https://example.com/webhooks/products".to_string(),
                    },
                )
                .build(),
            );

        // Verify chaining worked
        assert_eq!(chain_result.list_registrations().len(), 2);
    }

    // ========================================================================
    // Handler Tests (updated for new API)
    // ========================================================================

    #[test]
    fn test_add_registration_extracts_and_stores_handler_separately() {
        let invoked = Arc::new(AtomicBool::new(false));
        let handler = TestHandler {
            invoked: invoked.clone(),
        };

        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .handler(handler)
            .build(),
        );

        // Verify registration exists
        assert!(registry.get_registration(&WebhookTopic::OrdersCreate).is_some());

        // Verify handler was stored separately in the handlers map
        assert!(registry.handlers.contains_key(&WebhookTopic::OrdersCreate));
    }

    #[test]
    fn test_handler_lookup_by_topic_succeeds_for_registered_handler() {
        let invoked = Arc::new(AtomicBool::new(false));
        let handler = TestHandler {
            invoked: invoked.clone(),
        };

        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .handler(handler)
            .build(),
        );

        // Lookup handler by topic
        let found_handler = registry.handlers.get(&WebhookTopic::OrdersCreate);
        assert!(found_handler.is_some());
    }

    #[test]
    fn test_handler_lookup_returns_none_for_topic_without_handler() {
        let mut registry = WebhookRegistry::new();

        // Add registration without handler
        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .build(),
        );

        // Lookup handler by topic
        let found_handler = registry.handlers.get(&WebhookTopic::OrdersCreate);
        assert!(found_handler.is_none());
    }

    #[tokio::test]
    async fn test_process_returns_no_handler_for_topic_error() {
        let mut registry = WebhookRegistry::new();

        // Add registration without handler
        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .build(),
        );

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = b"{}";
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("orders/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            WebhookError::NoHandlerForTopic { topic } => {
                assert_eq!(topic, "orders/create");
            }
            other => panic!("Expected NoHandlerForTopic, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_process_returns_payload_parse_error_for_invalid_json() {
        let invoked = Arc::new(AtomicBool::new(false));
        let handler = TestHandler {
            invoked: invoked.clone(),
        };

        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .handler(handler)
            .build(),
        );

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        // Invalid JSON body
        let body = b"not valid json {{{";
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("orders/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            WebhookError::PayloadParseError { message } => {
                assert!(!message.is_empty());
            }
            other => panic!("Expected PayloadParseError, got: {:?}", other),
        }

        // Ensure handler was not invoked
        assert!(!invoked.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_process_invokes_handler_with_correct_context_and_payload() {
        let invoked = Arc::new(AtomicBool::new(false));
        let handler = TestHandler {
            invoked: invoked.clone(),
        };

        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .handler(handler)
            .build(),
        );

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = br#"{"order_id": 123}"#;
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("orders/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &request).await;
        assert!(result.is_ok());

        // Verify handler was invoked
        assert!(invoked.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_handler_error_propagation_through_process() {
        let handler = ErrorHandler {
            error_message: "Handler failed intentionally".to_string(),
        };

        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .handler(handler)
            .build(),
        );

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = br#"{"order_id": 123}"#;
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("orders/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            WebhookError::ShopifyError { message } => {
                assert_eq!(message, "Handler failed intentionally");
            }
            other => panic!("Expected ShopifyError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_multiple_handlers_for_different_topics() {
        let orders_invoked = Arc::new(AtomicBool::new(false));
        let products_invoked = Arc::new(AtomicBool::new(false));

        let orders_handler = TestHandler {
            invoked: orders_invoked.clone(),
        };
        let products_handler = TestHandler {
            invoked: products_invoked.clone(),
        };

        let mut registry = WebhookRegistry::new();

        registry
            .add_registration(
                WebhookRegistrationBuilder::new(
                    WebhookTopic::OrdersCreate,
                    WebhookDeliveryMethod::Http {
                        uri: "https://example.com/webhooks/orders".to_string(),
                    },
                )
                .handler(orders_handler)
                .build(),
            )
            .add_registration(
                WebhookRegistrationBuilder::new(
                    WebhookTopic::ProductsCreate,
                    WebhookDeliveryMethod::Http {
                        uri: "https://example.com/webhooks/products".to_string(),
                    },
                )
                .handler(products_handler)
                .build(),
            );

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        // Process orders webhook
        let orders_body = br#"{"order_id": 123}"#;
        let orders_hmac = compute_signature_base64(orders_body, "secret");
        let orders_request = WebhookRequest::new(
            orders_body.to_vec(),
            orders_hmac,
            Some("orders/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &orders_request).await;
        assert!(result.is_ok());
        assert!(orders_invoked.load(Ordering::SeqCst));
        assert!(!products_invoked.load(Ordering::SeqCst));

        // Process products webhook
        let products_body = br#"{"product_id": 456}"#;
        let products_hmac = compute_signature_base64(products_body, "secret");
        let products_request = WebhookRequest::new(
            products_body.to_vec(),
            products_hmac,
            Some("products/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &products_request).await;
        assert!(result.is_ok());
        assert!(products_invoked.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_handler_replacement_when_re_registering_same_topic() {
        let first_invoked = Arc::new(AtomicBool::new(false));
        let second_invoked = Arc::new(AtomicBool::new(false));

        let first_handler = TestHandler {
            invoked: first_invoked.clone(),
        };
        let second_handler = TestHandler {
            invoked: second_invoked.clone(),
        };

        let mut registry = WebhookRegistry::new();

        // Register first handler
        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .handler(first_handler)
            .build(),
        );

        // Replace with second handler
        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders/v2".to_string(),
                },
            )
            .handler(second_handler)
            .build(),
        );

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = br#"{"order_id": 123}"#;
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("orders/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &request).await;
        assert!(result.is_ok());

        // Only second handler should be invoked
        assert!(!first_invoked.load(Ordering::SeqCst));
        assert!(second_invoked.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_process_returns_invalid_hmac_for_bad_signature() {
        let invoked = Arc::new(AtomicBool::new(false));
        let handler = TestHandler {
            invoked: invoked.clone(),
        };

        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .handler(handler)
            .build(),
        );

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = br#"{"order_id": 123}"#;
        // Use wrong secret for HMAC
        let hmac = compute_signature_base64(body, "wrong-secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("orders/create".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &request).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WebhookError::InvalidHmac));

        // Handler should not be invoked
        assert!(!invoked.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_process_handles_unknown_topic() {
        let invoked = Arc::new(AtomicBool::new(false));
        let handler = TestHandler {
            invoked: invoked.clone(),
        };

        let mut registry = WebhookRegistry::new();

        registry.add_registration(
            WebhookRegistrationBuilder::new(
                WebhookTopic::OrdersCreate,
                WebhookDeliveryMethod::Http {
                    uri: "https://example.com/webhooks/orders".to_string(),
                },
            )
            .handler(handler)
            .build(),
        );

        let config = ShopifyConfig::builder()
            .api_key(ApiKey::new("key").unwrap())
            .api_secret_key(ApiSecretKey::new("secret").unwrap())
            .build()
            .unwrap();

        let body = br#"{"data": "test"}"#;
        let hmac = compute_signature_base64(body, "secret");
        let request = WebhookRequest::new(
            body.to_vec(),
            hmac,
            Some("custom/unknown_topic".to_string()),
            Some("shop.myshopify.com".to_string()),
            None,
            None,
        );

        let result = registry.process(&config, &request).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            WebhookError::NoHandlerForTopic { topic } => {
                assert_eq!(topic, "custom/unknown_topic");
            }
            other => panic!("Expected NoHandlerForTopic, got: {:?}", other),
        }

        // Handler should not be invoked
        assert!(!invoked.load(Ordering::SeqCst));
    }
}

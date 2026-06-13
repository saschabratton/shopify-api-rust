# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0](https://github.com/saschabratton/shopify-api-rust/compare/v1.0.0...v2.0.0) - 2026-06-13

### Added

- [**breaking**] make ApiVersion non-exhaustive and require versioned resource imports
- *(version)* add API version 2026-04 support
- *(version)* add API version 2026-01 support
- *(version)* add API version lifecycle management and deprecation handling
- *(webhooks)* add EventBridge and Pub/Sub delivery method support
- *(webhooks)* implement webhook handler infrastructure for processing incoming webhooks
- *(webhooks)* implement HMAC-based webhook signature verification
- *(webhooks)* implement webhook registry for managing Shopify webhook subscriptions
- *(rest)* implement extended REST resources for specialized Shopify use cases
- *(rest)* implement remaining REST resources
- *(rest)* implement additional REST resources for Shopify API
- *(rest)* implement core REST resources for Shopify API
- *(rest)* implement REST resource base infrastructure
- *(clients)* implement GraphQL Storefront API client
- *(clients)* implement GraphQL Admin API client with query execution
- *(clients)* implement REST API client with HTTP convenience methods
- *(oauth)* implement token refresh and migration to expiring tokens
- *(oauth)* implement client credentials grant for private apps
- *(oauth)* implement token exchange for embedded apps
- *(oauth)* implement authorization code grant flow
- *(auth)* add session management with serialization support
- *(http)* add async HTTP client foundation
- initial commit, fundamental context configuration

### Fixed

- rename shopify_api to shopify_sdk in tests and doc examples
- remove unused WebhookRegistrationBuilder import
- *(webhooks)* use unified uri field for webhook delivery methods
- *(oauth)* add missing refresh token fields to test AccessTokenResponse structs

### Other

- update stale API version examples
- add release-plz automation and CI workflows
- add readme and usage documentation

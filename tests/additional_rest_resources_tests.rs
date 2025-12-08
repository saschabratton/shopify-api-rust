//! Integration tests for Additional REST Resources.
//!
//! These tests verify the complete integration of the REST resource types:
//! Shop, Metafield, CustomCollection, SmartCollection, Webhook, Page, Blog, Article, Theme, Asset.
//!
//! Tests cover:
//! - Shop singleton resource behavior
//! - Metafield polymorphic path resolution
//! - Collection trait polymorphism for both collection types
//! - Webhook topic enum handling
//! - Article nested under Blog path patterns
//! - Theme role enum handling
//! - Asset binary content round-trip (base64 encoding/decoding)
//! - TrackedResource works with new resource types
//! - All resources implement required traits

use std::collections::HashMap;

use shopify_api::rest::resources::{
    // Article types
    Article,
    ArticleCountParams,
    ArticleFindParams,
    ArticleImage,
    ArticleListParams,
    // Asset types
    Asset,
    AssetListParams,
    // Blog types
    Blog,
    BlogCommentable,
    BlogCountParams,
    BlogFindParams,
    BlogListParams,
    // Collection types
    Collection,
    CollectionImage,
    CustomCollection,
    CustomCollectionCountParams,
    CustomCollectionFindParams,
    CustomCollectionListParams,
    // Metafield types
    Metafield,
    MetafieldCountParams,
    MetafieldFindParams,
    MetafieldListParams,
    MetafieldOwner,
    // Page types
    Page,
    PageCountParams,
    PageFindParams,
    PageListParams,
    // Shop type
    Shop,
    // SmartCollection types
    SmartCollection,
    SmartCollectionCountParams,
    SmartCollectionFindParams,
    SmartCollectionListParams,
    SmartCollectionRule,
    // Theme types
    Theme,
    ThemeFindParams,
    ThemeListParams,
    ThemeRole,
    // Webhook types
    Webhook,
    WebhookCountParams,
    WebhookFindParams,
    WebhookFormat,
    WebhookListParams,
    WebhookTopic,
};
use shopify_api::rest::{build_path, get_path, ResourceOperation, RestResource, TrackedResource};
use shopify_api::HttpMethod;

// ============================================================================
// Test 1: Shop singleton resource behavior
// ============================================================================

#[test]
fn test_shop_singleton_resource_behavior() {
    // Shop is a singleton resource - it uses Find operation without an ID parameter
    // It only has the special current() method for fetching

    // Verify Shop resource constants
    assert_eq!(Shop::NAME, "Shop");
    assert_eq!(Shop::PLURAL, "shop");

    // Verify Shop get_id returns None (singleton has no ID)
    let shop = Shop {
        id: Some(548380009),
        name: Some("My Test Shop".to_string()),
        email: Some("owner@example.com".to_string()),
        ..Default::default()
    };
    assert!(shop.get_id().is_none()); // Always returns None for singleton

    // Verify Shop deserialization from API response
    let json_str = r#"{
        "id": 548380009,
        "name": "My Test Shop",
        "email": "owner@example.com",
        "domain": "my-test-shop.myshopify.com",
        "myshopify_domain": "my-test-shop.myshopify.com",
        "currency": "USD",
        "money_format": "${{amount}}",
        "money_with_currency_format": "${{amount}} USD",
        "timezone": "America/New_York",
        "iana_timezone": "America/New_York",
        "plan_name": "basic",
        "plan_display_name": "Basic Shopify",
        "shop_owner": "John Doe",
        "country": "US",
        "country_code": "US",
        "country_name": "United States",
        "province": "New York",
        "province_code": "NY",
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-06-20T15:45:00Z",
        "weight_unit": "lb",
        "primary_locale": "en",
        "admin_graphql_api_id": "gid://shopify/Shop/548380009"
    }"#;

    let shop: Shop = serde_json::from_str(json_str).unwrap();

    assert_eq!(shop.id, Some(548380009));
    assert_eq!(shop.name, Some("My Test Shop".to_string()));
    assert_eq!(shop.email, Some("owner@example.com".to_string()));
    assert_eq!(shop.currency, Some("USD".to_string()));
    assert_eq!(shop.timezone, Some("America/New_York".to_string()));
    assert_eq!(shop.plan_name, Some("basic".to_string()));
    assert_eq!(shop.shop_owner, Some("John Doe".to_string()));
    assert_eq!(shop.country_code, Some("US".to_string()));
    assert_eq!(shop.province_code, Some("NY".to_string()));
    assert_eq!(shop.weight_unit, Some("lb".to_string()));
    assert!(shop.created_at.is_some());
    assert!(shop.updated_at.is_some());

    // Verify Shop HAS a Find path but with no ID parameter (singleton pattern)
    // It uses empty params &[] instead of &["id"]
    let find_path = get_path(Shop::PATHS, ResourceOperation::Find, &[]);
    assert!(find_path.is_some());
    assert_eq!(find_path.unwrap().template, "shop");

    // Verify Shop does not have Create/Update/Delete/All/Count paths
    let create_path = get_path(Shop::PATHS, ResourceOperation::Create, &[]);
    let update_path = get_path(Shop::PATHS, ResourceOperation::Update, &["id"]);
    let delete_path = get_path(Shop::PATHS, ResourceOperation::Delete, &["id"]);
    let all_path = get_path(Shop::PATHS, ResourceOperation::All, &[]);
    let count_path = get_path(Shop::PATHS, ResourceOperation::Count, &[]);
    assert!(create_path.is_none());
    assert!(update_path.is_none());
    assert!(delete_path.is_none());
    assert!(all_path.is_none());
    assert!(count_path.is_none());
}

// ============================================================================
// Test 2: Metafield polymorphic path resolution
// ============================================================================

#[test]
fn test_metafield_polymorphic_path_resolution() {
    // Metafields support polymorphic paths based on owner type
    // All paths should have the correct structure

    // Test Product metafield paths
    let product_find = get_path(
        Metafield::PATHS,
        ResourceOperation::Find,
        &["product_id", "id"],
    );
    assert!(product_find.is_some());
    assert_eq!(
        product_find.unwrap().template,
        "products/{product_id}/metafields/{id}"
    );

    let product_all = get_path(Metafield::PATHS, ResourceOperation::All, &["product_id"]);
    assert!(product_all.is_some());
    assert_eq!(
        product_all.unwrap().template,
        "products/{product_id}/metafields"
    );

    // Test CustomCollection metafield paths (via collection_id)
    let collection_find = get_path(
        Metafield::PATHS,
        ResourceOperation::Find,
        &["collection_id", "id"],
    );
    assert!(collection_find.is_some());
    assert_eq!(
        collection_find.unwrap().template,
        "collections/{collection_id}/metafields/{id}"
    );

    let collection_all = get_path(Metafield::PATHS, ResourceOperation::All, &["collection_id"]);
    assert!(collection_all.is_some());
    assert_eq!(
        collection_all.unwrap().template,
        "collections/{collection_id}/metafields"
    );

    // Test Page metafield paths
    let page_find = get_path(
        Metafield::PATHS,
        ResourceOperation::Find,
        &["page_id", "id"],
    );
    assert!(page_find.is_some());
    assert_eq!(
        page_find.unwrap().template,
        "pages/{page_id}/metafields/{id}"
    );

    // Test Blog metafield paths
    let blog_find = get_path(
        Metafield::PATHS,
        ResourceOperation::Find,
        &["blog_id", "id"],
    );
    assert!(blog_find.is_some());
    assert_eq!(
        blog_find.unwrap().template,
        "blogs/{blog_id}/metafields/{id}"
    );

    // Test standalone (shop-level) metafield paths
    let shop_find = get_path(Metafield::PATHS, ResourceOperation::Find, &["id"]);
    assert!(shop_find.is_some());
    assert_eq!(shop_find.unwrap().template, "metafields/{id}");

    let shop_all = get_path(Metafield::PATHS, ResourceOperation::All, &[]);
    assert!(shop_all.is_some());
    assert_eq!(shop_all.unwrap().template, "metafields");

    // Test Metafield deserialization
    let json_str = r#"{
        "id": 721389482,
        "namespace": "inventory",
        "key": "warehouse_location",
        "value": "US-WEST-1",
        "type": "single_line_text_field",
        "owner_id": 632910392,
        "owner_resource": "product",
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-06-20T15:45:00Z"
    }"#;

    let metafield: Metafield = serde_json::from_str(json_str).unwrap();

    assert_eq!(metafield.id, Some(721389482));
    assert_eq!(metafield.namespace, Some("inventory".to_string()));
    assert_eq!(metafield.key, Some("warehouse_location".to_string()));
    assert_eq!(metafield.value, Some("US-WEST-1".to_string()));
    assert_eq!(
        metafield.metafield_type,
        Some("single_line_text_field".to_string())
    );
    assert_eq!(metafield.owner_id, Some(632910392));
    assert_eq!(metafield.owner_resource, Some("product".to_string()));
}

// ============================================================================
// Test 3: Collection trait polymorphism
// ============================================================================

#[test]
fn test_collection_trait_polymorphism() {
    // Both CustomCollection and SmartCollection implement the Collection trait

    // Verify both types implement Collection trait
    fn assert_collection<T: Collection>() {}
    assert_collection::<CustomCollection>();
    assert_collection::<SmartCollection>();

    // Test CustomCollection
    let custom_collection = CustomCollection {
        id: Some(841564295),
        title: Some("Summer Collection".to_string()),
        body_html: Some("<p>Best summer products</p>".to_string()),
        handle: Some("summer-collection".to_string()),
        published_scope: Some("web".to_string()),
        sort_order: Some("best-selling".to_string()),
        image: Some(CollectionImage {
            src: Some("https://cdn.shopify.com/collection.jpg".to_string()),
            alt: Some("Summer".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Verify Collection trait get_collection_id method works
    assert_eq!(custom_collection.get_collection_id(), Some(841564295));

    // Verify struct fields are accessible
    assert_eq!(
        custom_collection.title,
        Some("Summer Collection".to_string())
    );
    assert_eq!(
        custom_collection.handle,
        Some("summer-collection".to_string())
    );

    // Test SmartCollection with rules
    let smart_collection = SmartCollection {
        id: Some(1063001322),
        title: Some("Nike Products".to_string()),
        body_html: Some("<p>All Nike products</p>".to_string()),
        handle: Some("nike-products".to_string()),
        published_scope: Some("global".to_string()),
        sort_order: Some("price-asc".to_string()),
        disjunctive: Some(false),
        rules: Some(vec![
            SmartCollectionRule {
                column: "vendor".to_string(),
                relation: "equals".to_string(),
                condition: "Nike".to_string(),
            },
            SmartCollectionRule {
                column: "variant_price".to_string(),
                relation: "greater_than".to_string(),
                condition: "100".to_string(),
            },
        ]),
        ..Default::default()
    };

    // Verify Collection trait get_collection_id method works
    assert_eq!(smart_collection.get_collection_id(), Some(1063001322));

    // Verify struct fields are accessible
    assert_eq!(smart_collection.title, Some("Nike Products".to_string()));
    assert_eq!(smart_collection.handle, Some("nike-products".to_string()));

    // Verify SmartCollection rules
    let rules = smart_collection.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].column, "vendor");
    assert_eq!(rules[0].relation, "equals");
    assert_eq!(rules[0].condition, "Nike");
    assert_eq!(rules[1].column, "variant_price");
    assert_eq!(rules[1].relation, "greater_than");
    assert_eq!(rules[1].condition, "100");

    // Verify disjunctive field (AND logic when false)
    assert_eq!(smart_collection.disjunctive, Some(false));

    // Verify polymorphic usage with generic function
    fn get_id_from_collection<C: Collection>(collection: &C) -> Option<u64> {
        collection.get_collection_id()
    }

    assert_eq!(get_id_from_collection(&custom_collection), Some(841564295));
    assert_eq!(get_id_from_collection(&smart_collection), Some(1063001322));
}

// ============================================================================
// Test 4: Asset binary content round-trip
// ============================================================================

#[test]
fn test_asset_binary_content_round_trip() {
    // Asset supports both text (value) and binary (attachment) content
    // Binary content is base64 encoded

    // Test text asset
    let text_asset = Asset {
        key: "templates/index.liquid".to_string(),
        value: Some("{{ content_for_header }}".to_string()),
        attachment: None,
        public_url: None,
        content_type: Some("text/x-liquid".to_string()),
        size: Some(24),
        checksum: None,
        theme_id: Some(123456789),
        created_at: None,
        updated_at: None,
    };

    assert!(!text_asset.is_binary());
    assert_eq!(text_asset.key, "templates/index.liquid");
    assert_eq!(
        text_asset.value.as_deref(),
        Some("{{ content_for_header }}")
    );

    // Test binary asset creation
    let binary_data: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    let binary_asset = Asset::upload_from_bytes("assets/logo.png", &binary_data);

    assert!(binary_asset.is_binary());
    assert_eq!(binary_asset.key, "assets/logo.png");
    assert!(binary_asset.attachment.is_some());
    assert!(binary_asset.value.is_none());

    // Verify base64 encoding
    let expected_base64 = "iVBORw0KGgo=";
    assert_eq!(binary_asset.attachment.as_deref(), Some(expected_base64));

    // Test download_content for binary asset
    let downloaded = binary_asset.download_content().unwrap();
    assert_eq!(downloaded, binary_data);

    // Test download_content for text asset
    let text_downloaded = text_asset.download_content().unwrap();
    assert_eq!(
        text_downloaded,
        "{{ content_for_header }}".as_bytes().to_vec()
    );

    // Test Asset deserialization from API response (text)
    let text_json = r#"{
        "key": "snippets/footer.liquid",
        "public_url": "https://cdn.shopify.com/s/assets/footer.liquid",
        "value": "<footer>Store Footer</footer>",
        "content_type": "text/x-liquid",
        "size": 30,
        "theme_id": 828155753,
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-06-20T15:45:00Z"
    }"#;

    let text_asset_api: Asset = serde_json::from_str(text_json).unwrap();
    assert_eq!(text_asset_api.key, "snippets/footer.liquid");
    assert_eq!(
        text_asset_api.value.as_deref(),
        Some("<footer>Store Footer</footer>")
    );
    assert!(!text_asset_api.is_binary());

    // Test Asset deserialization from API response (binary)
    let binary_json = r#"{
        "key": "assets/background.png",
        "public_url": "https://cdn.shopify.com/s/assets/background.png",
        "attachment": "iVBORw0KGgo=",
        "content_type": "image/png",
        "size": 8,
        "theme_id": 828155753,
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-06-20T15:45:00Z"
    }"#;

    let binary_asset_api: Asset = serde_json::from_str(binary_json).unwrap();
    assert_eq!(binary_asset_api.key, "assets/background.png");
    assert!(binary_asset_api.is_binary());
    assert_eq!(binary_asset_api.attachment.as_deref(), Some("iVBORw0KGgo="));
}

// ============================================================================
// Test 5: Article nested under Blog path patterns
// ============================================================================

#[test]
fn test_article_nested_under_blog_path_patterns() {
    // Articles are nested under blogs, similar to Variants under Products

    // Test Find path requires both blog_id and id
    let find_path = get_path(Article::PATHS, ResourceOperation::Find, &["blog_id", "id"]);
    assert!(find_path.is_some());
    assert_eq!(find_path.unwrap().template, "blogs/{blog_id}/articles/{id}");

    // Test Find without blog_id fails
    let find_without_blog = get_path(Article::PATHS, ResourceOperation::Find, &["id"]);
    assert!(find_without_blog.is_none());

    // Test All path requires blog_id
    let all_path = get_path(Article::PATHS, ResourceOperation::All, &["blog_id"]);
    assert!(all_path.is_some());
    assert_eq!(all_path.unwrap().template, "blogs/{blog_id}/articles");

    // Test All without blog_id fails
    let all_without_blog = get_path(Article::PATHS, ResourceOperation::All, &[]);
    assert!(all_without_blog.is_none());

    // Test Count path requires blog_id
    let count_path = get_path(Article::PATHS, ResourceOperation::Count, &["blog_id"]);
    assert!(count_path.is_some());
    assert_eq!(
        count_path.unwrap().template,
        "blogs/{blog_id}/articles/count"
    );

    // Test Create path requires blog_id
    let create_path = get_path(Article::PATHS, ResourceOperation::Create, &["blog_id"]);
    assert!(create_path.is_some());
    assert_eq!(create_path.unwrap().template, "blogs/{blog_id}/articles");

    // Test Update path requires both blog_id and id
    let update_path = get_path(
        Article::PATHS,
        ResourceOperation::Update,
        &["blog_id", "id"],
    );
    assert!(update_path.is_some());
    assert_eq!(
        update_path.unwrap().template,
        "blogs/{blog_id}/articles/{id}"
    );

    // Test Delete path requires both blog_id and id
    let delete_path = get_path(
        Article::PATHS,
        ResourceOperation::Delete,
        &["blog_id", "id"],
    );
    assert!(delete_path.is_some());
    assert_eq!(
        delete_path.unwrap().template,
        "blogs/{blog_id}/articles/{id}"
    );

    // Test path building with IDs
    let mut ids: HashMap<&str, u64> = HashMap::new();
    ids.insert("blog_id", 241253187);
    ids.insert("id", 134645308);

    let path = get_path(Article::PATHS, ResourceOperation::Find, &["blog_id", "id"]).unwrap();
    let url = build_path(path.template, &ids);
    assert_eq!(url, "blogs/241253187/articles/134645308");

    // Test Article deserialization
    let json_str = r#"{
        "id": 134645308,
        "blog_id": 241253187,
        "title": "My New Blog Post",
        "handle": "my-new-blog-post",
        "body_html": "<p>This is the content.</p>",
        "author": "John Smith",
        "summary_html": "<p>Summary here.</p>",
        "tags": "tech, news",
        "image": {
            "src": "https://cdn.shopify.com/s/files/article.jpg",
            "alt": "Article image",
            "width": 1200,
            "height": 800,
            "created_at": "2024-01-15T10:30:00Z"
        },
        "published_at": "2024-01-15T10:30:00Z",
        "user_id": 799407056,
        "created_at": "2024-01-10T08:00:00Z",
        "updated_at": "2024-06-20T15:45:00Z",
        "admin_graphql_api_id": "gid://shopify/OnlineStoreArticle/134645308"
    }"#;

    let article: Article = serde_json::from_str(json_str).unwrap();

    assert_eq!(article.id, Some(134645308));
    assert_eq!(article.blog_id, Some(241253187));
    assert_eq!(article.title, Some("My New Blog Post".to_string()));
    assert_eq!(article.author, Some("John Smith".to_string()));
    assert_eq!(article.tags, Some("tech, news".to_string()));
    assert!(article.image.is_some());
    let image = article.image.unwrap();
    assert_eq!(image.alt, Some("Article image".to_string()));
    assert_eq!(image.width, Some(1200));
}

// ============================================================================
// Test 6: Webhook topic enum handling
// ============================================================================

#[test]
fn test_webhook_topic_enum_handling() {
    // Test all WebhookTopic variants serialize correctly
    let topics = vec![
        (WebhookTopic::OrdersCreate, "orders/create"),
        (WebhookTopic::OrdersUpdated, "orders/updated"),
        (WebhookTopic::OrdersPaid, "orders/paid"),
        (WebhookTopic::OrdersCancelled, "orders/cancelled"),
        (WebhookTopic::OrdersFulfilled, "orders/fulfilled"),
        (
            WebhookTopic::OrdersPartiallyFulfilled,
            "orders/partially_fulfilled",
        ),
        (WebhookTopic::ProductsCreate, "products/create"),
        (WebhookTopic::ProductsUpdate, "products/update"),
        (WebhookTopic::ProductsDelete, "products/delete"),
        (WebhookTopic::CustomersCreate, "customers/create"),
        (WebhookTopic::CustomersUpdate, "customers/update"),
        (WebhookTopic::CustomersDelete, "customers/delete"),
        (WebhookTopic::FulfillmentsCreate, "fulfillments/create"),
        (WebhookTopic::FulfillmentsUpdate, "fulfillments/update"),
        (WebhookTopic::AppUninstalled, "app/uninstalled"),
        (WebhookTopic::ShopUpdate, "shop/update"),
        (WebhookTopic::ThemesCreate, "themes/create"),
        (WebhookTopic::ThemesUpdate, "themes/update"),
        (WebhookTopic::ThemesDelete, "themes/delete"),
        (WebhookTopic::ThemesPublish, "themes/publish"),
    ];

    for (topic, expected_str) in topics {
        let webhook = Webhook {
            topic: Some(topic),
            address: Some("https://example.com/webhooks".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&webhook).unwrap();
        assert_eq!(json["topic"], expected_str);

        // Test deserialization
        let json_str = format!(
            r#"{{"topic":"{}","address":"https://example.com/webhooks"}}"#,
            expected_str
        );
        let deserialized: Webhook = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.topic, webhook.topic);
    }

    // Test WebhookFormat enum
    assert_eq!(WebhookFormat::default(), WebhookFormat::Json);

    let webhook_json = Webhook {
        format: Some(WebhookFormat::Json),
        address: Some("https://example.com".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_value(&webhook_json).unwrap();
    assert_eq!(json["format"], "json");

    let webhook_xml = Webhook {
        format: Some(WebhookFormat::Xml),
        address: Some("https://example.com".to_string()),
        ..Default::default()
    };
    let json = serde_json::to_value(&webhook_xml).unwrap();
    assert_eq!(json["format"], "xml");

    // Test Webhook list params with topic filter
    let params = WebhookListParams {
        topic: Some("orders/create".to_string()),
        address: Some("https://example.com".to_string()),
        limit: Some(50),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["topic"], "orders/create");
    assert_eq!(json["address"], "https://example.com");
    assert_eq!(json["limit"], 50);
}

// ============================================================================
// Test 7: Theme role handling and transitions
// ============================================================================

#[test]
fn test_theme_role_handling() {
    // Test ThemeRole enum variants
    let roles = vec![
        (ThemeRole::Main, "main"),
        (ThemeRole::Unpublished, "unpublished"),
        (ThemeRole::Demo, "demo"),
        (ThemeRole::Development, "development"),
    ];

    for (role, expected_str) in roles {
        let theme = Theme {
            role: Some(role.clone()),
            name: Some("Test Theme".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&theme).unwrap();
        assert_eq!(json["role"], expected_str);

        // Test deserialization
        let json_str = format!(r#"{{"role":"{}","name":"Test Theme"}}"#, expected_str);
        let deserialized: Theme = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.role, Some(role));
    }

    // Test Theme deserialization from API response
    let json_str = r#"{
        "id": 828155753,
        "name": "My New Theme",
        "role": "main",
        "theme_store_id": null,
        "previewable": true,
        "processing": false,
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-06-20T15:45:00Z",
        "admin_graphql_api_id": "gid://shopify/OnlineStoreTheme/828155753"
    }"#;

    let theme: Theme = serde_json::from_str(json_str).unwrap();

    assert_eq!(theme.id, Some(828155753));
    assert_eq!(theme.name, Some("My New Theme".to_string()));
    assert_eq!(theme.role, Some(ThemeRole::Main));
    assert_eq!(theme.previewable, Some(true));
    assert_eq!(theme.processing, Some(false));

    // Test Theme list params with role filter
    let params = ThemeListParams {
        role: Some(ThemeRole::Main),
        ..Default::default()
    };

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["role"], "main");

    // Verify Theme paths
    let find_path = get_path(Theme::PATHS, ResourceOperation::Find, &["id"]);
    assert!(find_path.is_some());
    assert_eq!(find_path.unwrap().template, "themes/{id}");

    let all_path = get_path(Theme::PATHS, ResourceOperation::All, &[]);
    assert!(all_path.is_some());
    assert_eq!(all_path.unwrap().template, "themes");

    // Verify Theme does not have Count path
    let count_path = get_path(Theme::PATHS, ResourceOperation::Count, &[]);
    assert!(count_path.is_none());

    // Verify Theme has Create, Update, Delete
    let create_path = get_path(Theme::PATHS, ResourceOperation::Create, &[]);
    assert!(create_path.is_some());
    assert_eq!(create_path.unwrap().http_method, HttpMethod::Post);

    let update_path = get_path(Theme::PATHS, ResourceOperation::Update, &["id"]);
    assert!(update_path.is_some());
    assert_eq!(update_path.unwrap().http_method, HttpMethod::Put);

    let delete_path = get_path(Theme::PATHS, ResourceOperation::Delete, &["id"]);
    assert!(delete_path.is_some());
    assert_eq!(delete_path.unwrap().http_method, HttpMethod::Delete);
}

// ============================================================================
// Test 8: TrackedResource works with additional resource types
// ============================================================================

#[test]
fn test_tracked_resource_with_additional_resources() {
    // Test with CustomCollection
    let collection = CustomCollection {
        id: Some(841564295),
        title: Some("Summer Collection".to_string()),
        body_html: Some("<p>Summer products</p>".to_string()),
        ..Default::default()
    };

    let mut tracked_collection = TrackedResource::from_existing(collection);
    assert!(!tracked_collection.is_dirty());

    tracked_collection.title = Some("Summer Sale Collection".to_string());
    assert!(tracked_collection.is_dirty());

    let changes = tracked_collection.changed_fields();
    assert!(changes.get("title").is_some());
    assert_eq!(changes.get("title").unwrap(), "Summer Sale Collection");

    // Test with Page
    let page = Page {
        id: Some(131092082),
        title: Some("About Us".to_string()),
        body_html: Some("<p>About our store</p>".to_string()),
        ..Default::default()
    };

    let mut tracked_page = TrackedResource::from_existing(page);
    assert!(!tracked_page.is_dirty());

    tracked_page.body_html = Some("<p>Updated about section</p>".to_string());
    assert!(tracked_page.is_dirty());

    // Test with Blog
    let blog = Blog {
        id: Some(241253187),
        title: Some("News".to_string()),
        commentable: Some(BlogCommentable::Moderate),
        ..Default::default()
    };

    let mut tracked_blog = TrackedResource::from_existing(blog);
    assert!(!tracked_blog.is_dirty());

    tracked_blog.title = Some("Store News".to_string());
    assert!(tracked_blog.is_dirty());

    // Test with Theme
    let theme = Theme {
        id: Some(828155753),
        name: Some("My Theme".to_string()),
        role: Some(ThemeRole::Unpublished),
        ..Default::default()
    };

    let mut tracked_theme = TrackedResource::from_existing(theme);
    assert!(!tracked_theme.is_dirty());

    tracked_theme.name = Some("My Custom Theme".to_string());
    assert!(tracked_theme.is_dirty());

    // Test new resources (not from API)
    let new_webhook = Webhook {
        topic: Some(WebhookTopic::OrdersCreate),
        address: Some("https://example.com/webhooks".to_string()),
        ..Default::default()
    };

    let tracked_new = TrackedResource::new(new_webhook);
    assert!(tracked_new.is_dirty()); // New resources are always dirty
    assert!(tracked_new.is_new());
}

// ============================================================================
// Test 9: All additional resources implement RestResource trait
// ============================================================================

#[test]
fn test_all_additional_resources_implement_rest_resource_trait() {
    fn assert_rest_resource<T: RestResource>() {}

    assert_rest_resource::<Shop>();
    assert_rest_resource::<Metafield>();
    assert_rest_resource::<CustomCollection>();
    assert_rest_resource::<SmartCollection>();
    assert_rest_resource::<Webhook>();
    assert_rest_resource::<Page>();
    assert_rest_resource::<Blog>();
    assert_rest_resource::<Article>();
    assert_rest_resource::<Theme>();
}

// ============================================================================
// Test 10: All additional types are Send + Sync
// ============================================================================

#[test]
fn test_additional_types_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    // Resources
    assert_send_sync::<Shop>();
    assert_send_sync::<Metafield>();
    assert_send_sync::<CustomCollection>();
    assert_send_sync::<SmartCollection>();
    assert_send_sync::<Webhook>();
    assert_send_sync::<Page>();
    assert_send_sync::<Blog>();
    assert_send_sync::<Article>();
    assert_send_sync::<Theme>();
    assert_send_sync::<Asset>();

    // Params
    assert_send_sync::<MetafieldListParams>();
    assert_send_sync::<MetafieldFindParams>();
    assert_send_sync::<MetafieldCountParams>();
    assert_send_sync::<CustomCollectionListParams>();
    assert_send_sync::<CustomCollectionFindParams>();
    assert_send_sync::<CustomCollectionCountParams>();
    assert_send_sync::<SmartCollectionListParams>();
    assert_send_sync::<SmartCollectionFindParams>();
    assert_send_sync::<SmartCollectionCountParams>();
    assert_send_sync::<WebhookListParams>();
    assert_send_sync::<WebhookFindParams>();
    assert_send_sync::<WebhookCountParams>();
    assert_send_sync::<PageListParams>();
    assert_send_sync::<PageFindParams>();
    assert_send_sync::<PageCountParams>();
    assert_send_sync::<BlogListParams>();
    assert_send_sync::<BlogFindParams>();
    assert_send_sync::<BlogCountParams>();
    assert_send_sync::<ArticleListParams>();
    assert_send_sync::<ArticleFindParams>();
    assert_send_sync::<ArticleCountParams>();
    assert_send_sync::<ThemeListParams>();
    assert_send_sync::<ThemeFindParams>();
    assert_send_sync::<AssetListParams>();

    // Common types
    assert_send_sync::<CollectionImage>();
    assert_send_sync::<SmartCollectionRule>();
    assert_send_sync::<ArticleImage>();
    assert_send_sync::<MetafieldOwner>();
    assert_send_sync::<WebhookTopic>();
    assert_send_sync::<WebhookFormat>();
    assert_send_sync::<ThemeRole>();
    assert_send_sync::<BlogCommentable>();

    // TrackedResource wrapping additional types
    assert_send_sync::<TrackedResource<Shop>>();
    assert_send_sync::<TrackedResource<Metafield>>();
    assert_send_sync::<TrackedResource<CustomCollection>>();
    assert_send_sync::<TrackedResource<SmartCollection>>();
    assert_send_sync::<TrackedResource<Webhook>>();
    assert_send_sync::<TrackedResource<Page>>();
    assert_send_sync::<TrackedResource<Blog>>();
    assert_send_sync::<TrackedResource<Article>>();
    assert_send_sync::<TrackedResource<Theme>>();
}

// ============================================================================
// Test 11: All additional types exported correctly
// ============================================================================

#[test]
fn test_additional_types_exported_correctly() {
    // Verify all types can be instantiated from the exports
    let _: Shop = Shop::default();
    let _: Metafield = Metafield::default();
    let _: MetafieldListParams = MetafieldListParams::default();
    let _: MetafieldFindParams = MetafieldFindParams::default();
    let _: MetafieldCountParams = MetafieldCountParams::default();

    let _: CustomCollection = CustomCollection::default();
    let _: CustomCollectionListParams = CustomCollectionListParams::default();
    let _: CustomCollectionFindParams = CustomCollectionFindParams::default();
    let _: CustomCollectionCountParams = CustomCollectionCountParams::default();

    let _: SmartCollection = SmartCollection::default();
    let _: SmartCollectionListParams = SmartCollectionListParams::default();
    let _: SmartCollectionFindParams = SmartCollectionFindParams::default();
    let _: SmartCollectionCountParams = SmartCollectionCountParams::default();
    let _: SmartCollectionRule = SmartCollectionRule::default();

    let _: Webhook = Webhook::default();
    let _: WebhookListParams = WebhookListParams::default();
    let _: WebhookFindParams = WebhookFindParams::default();
    let _: WebhookCountParams = WebhookCountParams::default();
    let _: WebhookTopic = WebhookTopic::OrdersCreate;
    let _: WebhookFormat = WebhookFormat::Json;

    let _: Page = Page::default();
    let _: PageListParams = PageListParams::default();
    let _: PageFindParams = PageFindParams::default();
    let _: PageCountParams = PageCountParams::default();

    let _: Blog = Blog::default();
    let _: BlogListParams = BlogListParams::default();
    let _: BlogFindParams = BlogFindParams::default();
    let _: BlogCountParams = BlogCountParams::default();
    let _: BlogCommentable = BlogCommentable::Moderate;

    let _: Article = Article::default();
    let _: ArticleListParams = ArticleListParams::default();
    let _: ArticleFindParams = ArticleFindParams::default();
    let _: ArticleCountParams = ArticleCountParams::default();
    let _: ArticleImage = ArticleImage::default();

    let _: Theme = Theme::default();
    let _: ThemeListParams = ThemeListParams::default();
    let _: ThemeFindParams = ThemeFindParams::default();
    let _: ThemeRole = ThemeRole::Main;

    let _: Asset = Asset::default();
    let _: AssetListParams = AssetListParams::default();

    let _: CollectionImage = CollectionImage::default();
    let _: MetafieldOwner = MetafieldOwner::Product;
}

// ============================================================================
// Test 12: Blog commentable enum handling
// ============================================================================

#[test]
fn test_blog_commentable_enum_handling() {
    // Test BlogCommentable variants
    let commentable_options = vec![
        (BlogCommentable::No, "no"),
        (BlogCommentable::Moderate, "moderate"),
        (BlogCommentable::Yes, "yes"),
    ];

    for (commentable, expected_str) in commentable_options {
        let blog = Blog {
            commentable: Some(commentable.clone()),
            title: Some("Test Blog".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&blog).unwrap();
        assert_eq!(json["commentable"], expected_str);

        // Test deserialization
        let json_str = format!(
            r#"{{"commentable":"{}","title":"Test Blog"}}"#,
            expected_str
        );
        let deserialized: Blog = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.commentable, Some(commentable));
    }

    // Test Blog deserialization from API response
    let json_str = r#"{
        "id": 241253187,
        "handle": "news",
        "title": "Company News",
        "commentable": "moderate",
        "feedburner": null,
        "feedburner_location": null,
        "tags": "tech, updates",
        "created_at": "2024-01-15T10:30:00Z",
        "updated_at": "2024-06-20T15:45:00Z",
        "admin_graphql_api_id": "gid://shopify/OnlineStoreBlog/241253187"
    }"#;

    let blog: Blog = serde_json::from_str(json_str).unwrap();

    assert_eq!(blog.id, Some(241253187));
    assert_eq!(blog.handle, Some("news".to_string()));
    assert_eq!(blog.title, Some("Company News".to_string()));
    assert_eq!(blog.commentable, Some(BlogCommentable::Moderate));
    assert_eq!(blog.tags, Some("tech, updates".to_string()));
}

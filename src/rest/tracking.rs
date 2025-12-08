//! Dirty tracking for efficient partial updates.
//!
//! This module provides [`TrackedResource<T>`], a wrapper that tracks
//! changes to a resource for efficient partial updates. Only modified
//! fields are sent in PUT requests, reducing bandwidth and avoiding
//! overwriting concurrent changes.
//!
//! # How It Works
//!
//! When a resource is loaded from the API or after a successful save,
//! its state is captured as JSON. On subsequent saves, the current
//! state is compared to the original, and only changed fields are
//! serialized for the update request.
//!
//! # Example
//!
//! ```rust
//! use shopify_api::rest::TrackedResource;
//! use serde::{Serialize, Deserialize};
//! use serde_json::json;
//!
//! #[derive(Debug, Clone, Serialize, Deserialize)]
//! struct Product {
//!     id: u64,
//!     title: String,
//!     vendor: String,
//! }
//!
//! // Create a tracked resource (simulating load from API)
//! let product = Product {
//!     id: 123,
//!     title: "Original Title".to_string(),
//!     vendor: "Original Vendor".to_string(),
//! };
//! let mut tracked = TrackedResource::from_existing(product);
//!
//! // Resource is not dirty initially
//! assert!(!tracked.is_dirty());
//!
//! // Modify via DerefMut
//! tracked.title = "New Title".to_string();
//!
//! // Resource is now dirty
//! assert!(tracked.is_dirty());
//!
//! // Get only changed fields for partial update
//! let changes = tracked.changed_fields();
//! assert!(changes.get("title").is_some());
//! assert!(changes.get("vendor").is_none()); // Unchanged fields excluded
//!
//! // After successful save, mark clean
//! tracked.mark_clean();
//! assert!(!tracked.is_dirty());
//! ```

use std::ops::{Deref, DerefMut};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

/// A wrapper that tracks changes to a resource.
///
/// `TrackedResource<T>` stores both the current resource data and its
/// original state (as JSON). This allows detecting which fields have
/// changed since the resource was loaded or last saved.
///
/// # Type Parameters
///
/// * `T` - The resource type. Must implement `Serialize`, `DeserializeOwned`,
///   and `Clone` for state tracking to work.
///
/// # Deref Pattern
///
/// Implements `Deref<Target = T>` and `DerefMut`, so you can access
/// and modify the resource transparently:
///
/// ```rust
/// use shopify_api::rest::TrackedResource;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// struct Product { title: String }
///
/// let mut tracked = TrackedResource::new(Product { title: "Test".to_string() });
///
/// // Read via Deref
/// println!("{}", tracked.title);
///
/// // Write via DerefMut
/// tracked.title = "Modified".to_string();
/// ```
#[derive(Debug, Clone)]
pub struct TrackedResource<T> {
    /// The actual resource data.
    resource: T,
    /// The original state captured when loaded or after save.
    /// `None` for new resources that haven't been saved yet.
    original_state: Option<Value>,
}

impl<T: Serialize + DeserializeOwned + Clone> TrackedResource<T> {
    /// Creates a new tracked resource for a resource that doesn't exist yet.
    ///
    /// New resources have no original state, so `is_dirty()` returns `true`
    /// and `changed_fields()` returns all fields.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::rest::TrackedResource;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct Product { title: String }
    ///
    /// let tracked = TrackedResource::new(Product { title: "New".to_string() });
    /// assert!(tracked.is_dirty()); // New resources are always dirty
    /// ```
    #[must_use]
    pub const fn new(resource: T) -> Self {
        Self {
            resource,
            original_state: None,
        }
    }

    /// Creates a tracked resource from an existing resource.
    ///
    /// The current state is captured as the original state, so `is_dirty()`
    /// returns `false` until the resource is modified.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::rest::TrackedResource;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct Product { title: String }
    ///
    /// let tracked = TrackedResource::from_existing(Product { title: "Loaded".to_string() });
    /// assert!(!tracked.is_dirty()); // Existing resources start clean
    /// ```
    #[must_use]
    pub fn from_existing(resource: T) -> Self {
        let original_state = serde_json::to_value(&resource).ok();
        Self {
            resource,
            original_state,
        }
    }

    /// Returns `true` if the resource has been modified since loading or last save.
    ///
    /// For new resources (no original state), always returns `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::rest::TrackedResource;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct Product { title: String }
    ///
    /// let mut tracked = TrackedResource::from_existing(Product { title: "Test".to_string() });
    /// assert!(!tracked.is_dirty());
    ///
    /// tracked.title = "Changed".to_string();
    /// assert!(tracked.is_dirty());
    /// ```
    #[must_use]
    #[allow(clippy::option_if_let_else)]
    pub fn is_dirty(&self) -> bool {
        match &self.original_state {
            None => true, // New resource, always dirty
            Some(original) => {
                let current = serde_json::to_value(&self.resource).ok();
                current.as_ref() != Some(original)
            }
        }
    }

    /// Returns only the fields that have changed since loading or last save.
    ///
    /// For new resources, returns all fields (since there's no original state
    /// to compare against).
    ///
    /// For existing resources, returns only the fields whose values differ
    /// from the original state. Nested objects are handled recursively.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::rest::TrackedResource;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct Product { title: String, vendor: String }
    ///
    /// let mut tracked = TrackedResource::from_existing(Product {
    ///     title: "Original".to_string(),
    ///     vendor: "Vendor".to_string(),
    /// });
    ///
    /// tracked.title = "Changed".to_string();
    ///
    /// let changes = tracked.changed_fields();
    /// assert!(changes.get("title").is_some());
    /// assert!(changes.get("vendor").is_none()); // Unchanged
    /// ```
    #[must_use]
    pub fn changed_fields(&self) -> Value {
        let current = serde_json::to_value(&self.resource).unwrap_or(Value::Null);

        match &self.original_state {
            None => current, // New resource, return all fields
            Some(original) => diff_json_objects(original, &current),
        }
    }

    /// Marks the resource as clean by capturing the current state as original.
    ///
    /// Call this after a successful save operation to reset dirty tracking.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::rest::TrackedResource;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Clone, Serialize, Deserialize)]
    /// struct Product { title: String }
    ///
    /// let mut tracked = TrackedResource::from_existing(Product { title: "Test".to_string() });
    /// tracked.title = "Changed".to_string();
    /// assert!(tracked.is_dirty());
    ///
    /// tracked.mark_clean();
    /// assert!(!tracked.is_dirty());
    /// ```
    pub fn mark_clean(&mut self) {
        self.original_state = serde_json::to_value(&self.resource).ok();
    }

    /// Returns a reference to the inner resource.
    ///
    /// In most cases, you can use Deref coercion instead.
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.resource
    }

    /// Returns a mutable reference to the inner resource.
    ///
    /// In most cases, you can use `DerefMut` coercion instead.
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.resource
    }

    /// Consumes the wrapper and returns the inner resource.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.resource
    }

    /// Returns `true` if this is a new resource (no original state).
    #[must_use]
    pub const fn is_new(&self) -> bool {
        self.original_state.is_none()
    }
}

/// Computes the difference between two JSON objects.
///
/// Returns a JSON object containing only the fields from `current` that
/// differ from `original`. Handles nested objects recursively.
fn diff_json_objects(original: &Value, current: &Value) -> Value {
    match (original, current) {
        (Value::Object(orig_map), Value::Object(curr_map)) => {
            let mut diff = serde_json::Map::new();

            for (key, curr_value) in curr_map {
                match orig_map.get(key) {
                    Some(orig_value) => {
                        // Key exists in both - check if changed
                        if orig_value != curr_value {
                            // For nested objects, recursively diff
                            if orig_value.is_object() && curr_value.is_object() {
                                let nested_diff = diff_json_objects(orig_value, curr_value);
                                if !nested_diff.is_null()
                                    && nested_diff.as_object().is_some_and(|m| !m.is_empty())
                                {
                                    diff.insert(key.clone(), nested_diff);
                                }
                            } else {
                                // Primitive value changed
                                diff.insert(key.clone(), curr_value.clone());
                            }
                        }
                    }
                    None => {
                        // New field - include it
                        diff.insert(key.clone(), curr_value.clone());
                    }
                }
            }

            // Note: We don't include deleted fields (fields in original but not in current)
            // because REST APIs typically don't support field deletion via partial updates

            Value::Object(diff)
        }
        // For non-objects, if they differ, return current
        _ => {
            if original == current {
                Value::Null
            } else {
                current.clone()
            }
        }
    }
}

/// Provides transparent read access to the inner resource.
impl<T> Deref for TrackedResource<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

/// Provides transparent mutable access to the inner resource.
///
/// Modifications via `DerefMut` will be detected by `is_dirty()`.
impl<T> DerefMut for TrackedResource<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}

// Verify TrackedResource is Send + Sync when T is Send + Sync
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<TrackedResource<String>>();
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestProduct {
        id: Option<u64>,
        title: String,
        vendor: String,
        tags: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestProductWithNested {
        id: u64,
        title: String,
        options: TestOptions,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestOptions {
        name: String,
        values: Vec<String>,
    }

    #[test]
    fn test_tracked_resource_new_captures_no_initial_state() {
        let product = TestProduct {
            id: None,
            title: "New Product".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let tracked = TrackedResource::new(product);

        assert!(tracked.is_new());
        assert!(tracked.original_state.is_none());
    }

    #[test]
    fn test_is_dirty_returns_false_for_unchanged_resource() {
        let product = TestProduct {
            id: Some(123),
            title: "Test".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec!["tag1".to_string()],
        };

        let tracked = TrackedResource::from_existing(product);

        assert!(!tracked.is_dirty());
    }

    #[test]
    fn test_is_dirty_returns_true_after_field_modification() {
        let product = TestProduct {
            id: Some(123),
            title: "Original".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let mut tracked = TrackedResource::from_existing(product);
        assert!(!tracked.is_dirty());

        tracked.title = "Modified".to_string();
        assert!(tracked.is_dirty());
    }

    #[test]
    fn test_changed_fields_returns_empty_for_unchanged_resource() {
        let product = TestProduct {
            id: Some(123),
            title: "Test".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let tracked = TrackedResource::from_existing(product);
        let changes = tracked.changed_fields();

        assert!(changes.is_object());
        assert!(changes.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_changed_fields_returns_only_modified_fields() {
        let product = TestProduct {
            id: Some(123),
            title: "Original Title".to_string(),
            vendor: "Original Vendor".to_string(),
            tags: vec!["tag1".to_string()],
        };

        let mut tracked = TrackedResource::from_existing(product);
        tracked.title = "New Title".to_string();

        let changes = tracked.changed_fields();

        assert_eq!(changes.get("title"), Some(&json!("New Title")));
        assert!(changes.get("vendor").is_none()); // Unchanged
        assert!(changes.get("id").is_none()); // Unchanged
        assert!(changes.get("tags").is_none()); // Unchanged
    }

    #[test]
    fn test_changed_fields_handles_nested_object_changes() {
        let product = TestProductWithNested {
            id: 123,
            title: "Test".to_string(),
            options: TestOptions {
                name: "Color".to_string(),
                values: vec!["Red".to_string(), "Blue".to_string()],
            },
        };

        let mut tracked = TrackedResource::from_existing(product);

        // Modify nested field
        tracked.options.name = "Size".to_string();

        let changes = tracked.changed_fields();

        // The options object should be in changes with nested diff
        assert!(changes.get("options").is_some());
        let options_changes = changes.get("options").unwrap();
        assert_eq!(options_changes.get("name"), Some(&json!("Size")));
        // values should not be in changes since it wasn't modified
        assert!(options_changes.get("values").is_none());
    }

    #[test]
    fn test_mark_clean_resets_dirty_state() {
        let product = TestProduct {
            id: Some(123),
            title: "Original".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let mut tracked = TrackedResource::from_existing(product);
        tracked.title = "Modified".to_string();
        assert!(tracked.is_dirty());

        tracked.mark_clean();
        assert!(!tracked.is_dirty());

        // Changes should now be empty
        let changes = tracked.changed_fields();
        assert!(changes.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_new_resources_serialize_all_fields() {
        let product = TestProduct {
            id: None,
            title: "New Product".to_string(),
            vendor: "New Vendor".to_string(),
            tags: vec!["tag1".to_string()],
        };

        let tracked = TrackedResource::new(product);
        let changes = tracked.changed_fields();

        // All fields should be present
        assert!(changes.get("id").is_some());
        assert!(changes.get("title").is_some());
        assert!(changes.get("vendor").is_some());
        assert!(changes.get("tags").is_some());
    }

    #[test]
    fn test_deref_allows_field_access() {
        let product = TestProduct {
            id: Some(123),
            title: "Test".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let tracked = TrackedResource::from_existing(product);

        // Access fields via Deref
        assert_eq!(tracked.title, "Test");
        assert_eq!(tracked.vendor, "Vendor");
    }

    #[test]
    fn test_deref_mut_allows_field_modification() {
        let product = TestProduct {
            id: Some(123),
            title: "Original".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let mut tracked = TrackedResource::from_existing(product);

        // Modify via DerefMut
        tracked.title = "Modified".to_string();
        tracked.tags.push("new_tag".to_string());

        assert_eq!(tracked.title, "Modified");
        assert_eq!(tracked.tags, vec!["new_tag".to_string()]);
    }

    #[test]
    fn test_into_inner_returns_resource() {
        let product = TestProduct {
            id: Some(123),
            title: "Test".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let tracked = TrackedResource::from_existing(product.clone());
        let inner = tracked.into_inner();

        assert_eq!(inner, product);
    }

    #[test]
    fn test_is_new_differentiates_new_and_existing() {
        let new_product = TestProduct {
            id: None,
            title: "New".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let existing_product = TestProduct {
            id: Some(123),
            title: "Existing".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec![],
        };

        let new_tracked = TrackedResource::new(new_product);
        assert!(new_tracked.is_new());

        let existing_tracked = TrackedResource::from_existing(existing_product);
        assert!(!existing_tracked.is_new());
    }

    #[test]
    fn test_changed_fields_detects_array_modifications() {
        let product = TestProduct {
            id: Some(123),
            title: "Test".to_string(),
            vendor: "Vendor".to_string(),
            tags: vec!["original".to_string()],
        };

        let mut tracked = TrackedResource::from_existing(product);
        tracked.tags.push("new_tag".to_string());

        let changes = tracked.changed_fields();
        assert!(changes.get("tags").is_some());
    }

    #[test]
    fn test_diff_json_objects_handles_added_fields() {
        let original = json!({"a": 1});
        let current = json!({"a": 1, "b": 2});

        let diff = diff_json_objects(&original, &current);

        assert_eq!(diff.get("b"), Some(&json!(2)));
        assert!(diff.get("a").is_none()); // Unchanged
    }

    #[test]
    fn test_multiple_modifications_and_mark_clean() {
        let product = TestProduct {
            id: Some(123),
            title: "Original".to_string(),
            vendor: "Original Vendor".to_string(),
            tags: vec![],
        };

        let mut tracked = TrackedResource::from_existing(product);

        // First modification
        tracked.title = "First Change".to_string();
        assert!(tracked.is_dirty());

        // Mark clean (simulating save)
        tracked.mark_clean();
        assert!(!tracked.is_dirty());

        // Second modification
        tracked.vendor = "New Vendor".to_string();
        assert!(tracked.is_dirty());

        let changes = tracked.changed_fields();
        assert!(changes.get("title").is_none()); // Was cleaned
        assert_eq!(changes.get("vendor"), Some(&json!("New Vendor")));
    }
}

//! State parameter handling for OAuth CSRF protection.
//!
//! This module provides the [`StateParam`] type for generating and parsing
//! OAuth state parameters used for CSRF protection during the authorization flow.
//!
//! # Overview
//!
//! The state parameter serves two purposes in OAuth:
//! 1. **CSRF Protection**: Prevents cross-site request forgery attacks by ensuring
//!    the callback was initiated by a legitimate authorization request.
//! 2. **Data Preservation**: Optionally carries custom data through the OAuth flow.
//!
//! # Formats
//!
//! `StateParam` supports three usage patterns:
//!
//! - **Simple nonce**: Generated via [`StateParam::new()`], a 15-character
//!   alphanumeric string for basic CSRF protection.
//! - **Structured with data**: Generated via [`StateParam::with_data()`], embeds
//!   a nonce and custom JSON data in a base64-encoded string.
//! - **Raw string**: Created via [`StateParam::from_raw()`], wraps an arbitrary
//!   string for advanced use cases.
//!
//! # Example
//!
//! ```rust
//! use shopify_sdk::auth::oauth::StateParam;
//! use serde::{Serialize, Deserialize};
//!
//! // Simple CSRF protection
//! let state = StateParam::new();
//! assert_eq!(state.nonce().len(), 15);
//!
//! // Embed custom data through the flow
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct FlowData {
//!     return_url: String,
//! }
//!
//! let data = FlowData { return_url: "/dashboard".to_string() };
//! let state = StateParam::with_data(&data);
//! let extracted: Option<FlowData> = state.extract_data();
//! assert_eq!(extracted.unwrap().return_url, "/dashboard");
//! ```

use base64::prelude::*;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;

/// OAuth state parameter for CSRF protection and data preservation.
///
/// This type represents the state parameter used in OAuth authorization flows.
/// It provides cryptographically secure nonce generation and optional data embedding.
///
/// # Thread Safety
///
/// `StateParam` is `Send + Sync`, making it safe to share across threads.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::auth::oauth::StateParam;
///
/// // Generate a simple state for CSRF protection
/// let state = StateParam::new();
/// println!("State: {}", state);
/// println!("Nonce: {}", state.nonce());
///
/// // Use as_ref for URL encoding
/// let encoded = urlencoding::encode(state.as_ref());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateParam {
    /// The full state string value.
    value: String,
    /// Whether this is a structured state (base64 JSON with nonce and data).
    is_structured: bool,
}

/// Internal structure for structured state parameters.
#[derive(Serialize, Deserialize)]
struct StructuredState<T> {
    nonce: String,
    data: T,
}

/// Internal structure for extracting just the nonce.
#[derive(Deserialize)]
struct NonceOnly {
    nonce: String,
}

// Verify StateParam is Send + Sync at compile time
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<StateParam>();
};

impl StateParam {
    /// The length of generated nonces (matches Ruby SDK's SecureRandom.alphanumeric(15)).
    const NONCE_LENGTH: usize = 15;

    /// Creates a new state parameter with a cryptographically secure random nonce.
    ///
    /// The nonce is a 15-character alphanumeric string generated using a
    /// cryptographically secure random number generator, matching the Ruby SDK's
    /// `SecureRandom.alphanumeric(15)` behavior.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::auth::oauth::StateParam;
    ///
    /// let state = StateParam::new();
    /// assert_eq!(state.nonce().len(), 15);
    /// assert!(state.nonce().chars().all(|c| c.is_ascii_alphanumeric()));
    /// ```
    #[must_use]
    pub fn new() -> Self {
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(Self::NONCE_LENGTH)
            .map(char::from)
            .collect();

        Self {
            value: nonce,
            is_structured: false,
        }
    }

    /// Creates a state parameter with embedded custom data.
    ///
    /// The state is created as a base64-encoded JSON object containing both
    /// a secure random nonce and the provided data. This allows passing custom
    /// information through the OAuth flow (e.g., a return URL).
    ///
    /// # Arguments
    ///
    /// * `data` - Any serializable data to embed in the state
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::auth::oauth::StateParam;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct FlowContext {
    ///     return_url: String,
    ///     user_id: u64,
    /// }
    ///
    /// let context = FlowContext {
    ///     return_url: "/dashboard".to_string(),
    ///     user_id: 12345,
    /// };
    ///
    /// let state = StateParam::with_data(&context);
    /// // State is base64-encoded, can be safely used in URLs
    /// println!("State for OAuth: {}", state);
    /// ```
    #[must_use]
    pub fn with_data<T: Serialize>(data: &T) -> Self {
        let nonce: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(Self::NONCE_LENGTH)
            .map(char::from)
            .collect();

        let structured = StructuredState { nonce, data };
        let json = serde_json::to_string(&structured).unwrap_or_default();
        let encoded = BASE64_STANDARD.encode(json.as_bytes());

        Self {
            value: encoded,
            is_structured: true,
        }
    }

    /// Creates a state parameter from a raw string.
    ///
    /// This allows advanced users to provide their own state value. The string
    /// is used as-is without any processing or validation.
    ///
    /// # Arguments
    ///
    /// * `raw` - The raw state string to use
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::auth::oauth::StateParam;
    ///
    /// let state = StateParam::from_raw("custom-state-value");
    /// assert_eq!(state.as_ref(), "custom-state-value");
    /// ```
    #[must_use]
    pub fn from_raw(raw: impl Into<String>) -> Self {
        Self {
            value: raw.into(),
            is_structured: false,
        }
    }

    /// Returns the raw state value.
    ///
    /// For simple states (created with `new()` or `from_raw()`), this returns
    /// the nonce or raw value directly. For structured states (created with
    /// `with_data()`), this returns the full base64-encoded value.
    ///
    /// To extract the actual nonce from a structured state, use [`extract_nonce()`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::auth::oauth::StateParam;
    ///
    /// // Simple state - nonce() returns the 15-char nonce
    /// let simple = StateParam::new();
    /// assert_eq!(simple.nonce().len(), 15);
    ///
    /// // Structured state - nonce() returns the full encoded value
    /// // Use extract_nonce() to get the actual nonce
    /// let structured = StateParam::with_data(&"test");
    /// let actual_nonce = structured.extract_nonce();
    /// assert_eq!(actual_nonce.len(), 15);
    /// ```
    ///
    /// [`extract_nonce()`]: Self::extract_nonce
    #[must_use]
    pub fn nonce(&self) -> &str {
        &self.value
    }

    /// Extracts the embedded data from a structured state.
    ///
    /// Attempts to base64-decode the state, parse it as JSON, and deserialize
    /// the `data` field to the specified type.
    ///
    /// # Returns
    ///
    /// - `Some(T)` if the state was structured and the data could be deserialized
    /// - `None` if the state is not structured or deserialization fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::auth::oauth::StateParam;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug, PartialEq)]
    /// struct UserData {
    ///     name: String,
    /// }
    ///
    /// let data = UserData { name: "Alice".to_string() };
    /// let state = StateParam::with_data(&data);
    ///
    /// let extracted: Option<UserData> = state.extract_data();
    /// assert_eq!(extracted, Some(data));
    ///
    /// // Simple states don't have embedded data
    /// let simple = StateParam::new();
    /// let extracted: Option<UserData> = simple.extract_data();
    /// assert!(extracted.is_none());
    /// ```
    #[must_use]
    pub fn extract_data<T: DeserializeOwned>(&self) -> Option<T> {
        // Attempt to decode base64
        let decoded = BASE64_STANDARD.decode(self.value.as_bytes()).ok()?;
        let json_str = String::from_utf8(decoded).ok()?;

        // Parse as structured state
        let structured: StructuredState<T> = serde_json::from_str(&json_str).ok()?;
        Some(structured.data)
    }

    /// Extracts the nonce from a potentially structured state.
    ///
    /// For simple states, returns the full value (which is the nonce).
    /// For structured states, decodes and extracts the actual nonce.
    ///
    /// # Returns
    ///
    /// The 15-character nonce string, or the full value if parsing fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_sdk::auth::oauth::StateParam;
    ///
    /// // Simple state
    /// let simple = StateParam::new();
    /// assert_eq!(simple.extract_nonce().len(), 15);
    ///
    /// // Structured state
    /// let structured = StateParam::with_data(&42);
    /// assert_eq!(structured.extract_nonce().len(), 15);
    /// ```
    #[must_use]
    pub fn extract_nonce(&self) -> String {
        if !self.is_structured {
            return self.value.clone();
        }

        // Try to decode and extract nonce
        if let Ok(decoded) = BASE64_STANDARD.decode(self.value.as_bytes()) {
            if let Ok(json_str) = String::from_utf8(decoded) {
                if let Ok(nonce_only) = serde_json::from_str::<NonceOnly>(&json_str) {
                    return nonce_only.nonce;
                }
            }
        }

        // Fallback to full value if parsing fails
        self.value.clone()
    }
}

impl Default for StateParam {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StateParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl AsRef<str> for StateParam {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_new_generates_15_char_alphanumeric_nonce() {
        let state = StateParam::new();
        let nonce = state.nonce();

        assert_eq!(nonce.len(), 15);
        assert!(nonce.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_new_generates_unique_nonces() {
        let state1 = StateParam::new();
        let state2 = StateParam::new();

        // Extremely unlikely to generate the same nonce twice
        assert_ne!(state1.nonce(), state2.nonce());
    }

    #[test]
    fn test_with_data_embeds_json_in_base64() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestData {
            return_url: String,
        }

        let data = TestData {
            return_url: "/dashboard".to_string(),
        };
        let state = StateParam::with_data(&data);

        // Should be valid base64
        let decoded = BASE64_STANDARD.decode(state.as_ref().as_bytes());
        assert!(decoded.is_ok());

        // Should contain valid JSON
        let json_str = String::from_utf8(decoded.unwrap()).unwrap();
        assert!(json_str.contains("nonce"));
        assert!(json_str.contains("data"));
        assert!(json_str.contains("/dashboard"));
    }

    #[test]
    fn test_from_raw_wraps_string_correctly() {
        let state = StateParam::from_raw("custom-state-123");
        assert_eq!(state.as_ref(), "custom-state-123");
        assert_eq!(state.nonce(), "custom-state-123");
    }

    #[test]
    fn test_nonce_returns_value_for_simple_state() {
        let state = StateParam::new();
        assert_eq!(state.nonce().len(), 15);
    }

    #[test]
    fn test_nonce_returns_full_value_for_structured_state() {
        let state = StateParam::with_data(&"test");
        // For structured state, nonce() returns the full base64 value
        // Use extract_nonce() to get the actual nonce
        assert!(state.nonce().len() > 15);
        assert_eq!(state.extract_nonce().len(), 15);
    }

    #[test]
    fn test_extract_data_returns_embedded_data() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct FlowData {
            user_id: u64,
            redirect_to: String,
        }

        let original = FlowData {
            user_id: 12345,
            redirect_to: "/admin/orders".to_string(),
        };
        let state = StateParam::with_data(&original);

        let extracted: Option<FlowData> = state.extract_data();
        assert_eq!(extracted, Some(original));
    }

    #[test]
    fn test_extract_data_returns_none_for_simple_state() {
        #[derive(Deserialize)]
        struct SomeData {
            #[allow(dead_code)]
            field: String,
        }

        let state = StateParam::new();
        let extracted: Option<SomeData> = state.extract_data();
        assert!(extracted.is_none());
    }

    #[test]
    fn test_extract_data_returns_none_for_type_mismatch() {
        #[derive(Serialize)]
        struct DataA {
            field_a: String,
        }

        #[derive(Deserialize)]
        struct DataB {
            #[allow(dead_code)]
            field_b: i32,
        }

        let data = DataA {
            field_a: "test".to_string(),
        };
        let state = StateParam::with_data(&data);

        let extracted: Option<DataB> = state.extract_data();
        assert!(extracted.is_none());
    }

    #[test]
    fn test_display_returns_full_state_string() {
        let state = StateParam::from_raw("display-test");
        assert_eq!(format!("{}", state), "display-test");

        let state = StateParam::new();
        assert_eq!(format!("{}", state), state.as_ref());
    }

    #[test]
    fn test_as_ref_provides_string_slice() {
        let state = StateParam::from_raw("ref-test");
        let s: &str = state.as_ref();
        assert_eq!(s, "ref-test");
    }

    #[test]
    fn test_with_data_handles_various_types() {
        // String
        let state = StateParam::with_data(&"simple string");
        let extracted: Option<String> = state.extract_data();
        assert_eq!(extracted, Some("simple string".to_string()));

        // Number
        let state = StateParam::with_data(&42i32);
        let extracted: Option<i32> = state.extract_data();
        assert_eq!(extracted, Some(42));

        // Vec
        let state = StateParam::with_data(&vec![1, 2, 3]);
        let extracted: Option<Vec<i32>> = state.extract_data();
        assert_eq!(extracted, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_extract_nonce_from_structured_state() {
        #[derive(Serialize)]
        struct Data {
            value: i32,
        }

        let state = StateParam::with_data(&Data { value: 42 });
        let nonce = state.extract_nonce();

        // Nonce should be 15 alphanumeric characters
        assert_eq!(nonce.len(), 15);
        assert!(nonce.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_state_param_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<StateParam>();
    }

    #[test]
    fn test_state_param_clone() {
        let state = StateParam::new();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_state_param_eq() {
        let state1 = StateParam::from_raw("same");
        let state2 = StateParam::from_raw("same");
        let state3 = StateParam::from_raw("different");

        assert_eq!(state1, state2);
        assert_ne!(state1, state3);
    }

    #[test]
    fn test_state_param_default() {
        let state = StateParam::default();
        assert_eq!(state.nonce().len(), 15);
    }
}

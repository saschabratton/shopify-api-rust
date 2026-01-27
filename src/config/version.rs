//! Shopify API version definitions.
//!
//! This module provides the [`ApiVersion`] enum for specifying which version
//! of the Shopify API to use.

use crate::error::ConfigError;
use std::fmt;
use std::str::FromStr;

/// Shopify API version.
///
/// Shopify releases new API versions quarterly (January, April, July, October).
/// This enum provides variants for known stable versions, plus an `Unstable`
/// variant for development and a `Custom` variant for future versions.
///
/// # Example
///
/// ```rust
/// use shopify_sdk::ApiVersion;
///
/// // Use the latest stable version
/// let version = ApiVersion::latest();
/// assert!(version.is_stable());
///
/// // Parse from string
/// let version: ApiVersion = "2024-10".parse().unwrap();
/// assert_eq!(version, ApiVersion::V2024_10);
///
/// // Display as string
/// assert_eq!(format!("{}", ApiVersion::V2024_10), "2024-10");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ApiVersion {
    /// API version 2024-01 (January 2024)
    V2024_01,
    /// API version 2024-04 (April 2024)
    V2024_04,
    /// API version 2024-07 (July 2024)
    V2024_07,
    /// API version 2024-10 (October 2024)
    V2024_10,
    /// API version 2025-01 (January 2025)
    V2025_01,
    /// API version 2025-04 (April 2025)
    V2025_04,
    /// API version 2025-07 (July 2025)
    V2025_07,
    /// API version 2025-10 (October 2025)
    V2025_10,
    /// Unstable API version for development and testing.
    Unstable,
    /// Custom version string for future or unrecognized versions.
    Custom(String),
}

impl ApiVersion {
    /// Returns the latest stable API version.
    ///
    /// This should be updated when new stable versions are released.
    #[must_use]
    pub const fn latest() -> Self {
        Self::V2025_10
    }

    /// Returns `true` if this is a known stable API version.
    ///
    /// Returns `false` for `Unstable` and `Custom` variants.
    #[must_use]
    pub const fn is_stable(&self) -> bool {
        !matches!(self, Self::Unstable | Self::Custom(_))
    }

    /// Returns all supported stable versions in chronological order.
    ///
    /// This includes versions within Shopify's approximately 12-month support window.
    /// Versions are ordered from oldest to newest.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::ApiVersion;
    ///
    /// let versions = ApiVersion::supported_versions();
    /// assert!(!versions.is_empty());
    /// assert!(versions.contains(&ApiVersion::latest()));
    /// ```
    #[must_use]
    pub fn supported_versions() -> Vec<Self> {
        vec![
            Self::V2025_01,
            Self::V2025_04,
            Self::V2025_07,
            Self::V2025_10,
        ]
    }

    /// Returns the oldest supported API version.
    ///
    /// This represents the minimum version within Shopify's support window
    /// (approximately 12 months). Versions older than this are considered
    /// deprecated and may stop working at any time.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::ApiVersion;
    ///
    /// let minimum = ApiVersion::minimum_supported();
    /// assert!(minimum.is_supported());
    /// ```
    #[must_use]
    pub const fn minimum_supported() -> Self {
        Self::V2025_01
    }

    /// Returns `true` if this version is within Shopify's support window.
    ///
    /// Supported versions include:
    /// - All stable versions from [`minimum_supported()`] onwards
    /// - The `Unstable` version (always supported for development)
    /// - `Custom` versions (assumed supported as they may be newer versions)
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::ApiVersion;
    ///
    /// assert!(ApiVersion::V2025_10.is_supported());
    /// assert!(ApiVersion::Unstable.is_supported());
    /// assert!(!ApiVersion::V2024_01.is_supported());
    /// ```
    #[must_use]
    pub fn is_supported(&self) -> bool {
        match self {
            Self::Unstable => true,
            Self::Custom(_) => true, // Custom versions are assumed to be newer/valid
            _ => *self >= Self::minimum_supported(),
        }
    }

    /// Returns `true` if this version is past Shopify's support window.
    ///
    /// Deprecated versions are older than [`minimum_supported()`] and may
    /// stop working at any time. You should upgrade to a supported version.
    ///
    /// Note: `Unstable` and `Custom` versions are never considered deprecated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use shopify_api::ApiVersion;
    ///
    /// assert!(ApiVersion::V2024_01.is_deprecated());
    /// assert!(!ApiVersion::V2025_10.is_deprecated());
    /// assert!(!ApiVersion::Unstable.is_deprecated());
    /// ```
    #[must_use]
    pub fn is_deprecated(&self) -> bool {
        match self {
            Self::Unstable => false,
            Self::Custom(_) => false,
            _ => *self < Self::minimum_supported(),
        }
    }

    /// Returns a numeric ordering value for version comparison.
    ///
    /// This is used internally for implementing `Ord`.
    const fn ordinal(&self) -> u32 {
        match self {
            Self::V2024_01 => 1,
            Self::V2024_04 => 2,
            Self::V2024_07 => 3,
            Self::V2024_10 => 4,
            Self::V2025_01 => 5,
            Self::V2025_04 => 6,
            Self::V2025_07 => 7,
            Self::V2025_10 => 8,
            Self::Unstable => 100, // Always sorts after stable versions
            Self::Custom(_) => 101, // Custom sorts after unstable
        }
    }
}

impl PartialOrd for ApiVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ApiVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            // Custom versions compare lexicographically with each other
            (Self::Custom(a), Self::Custom(b)) => a.cmp(b),
            // Otherwise use ordinal comparison
            _ => self.ordinal().cmp(&other.ordinal()),
        }
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version_str = match self {
            Self::V2024_01 => "2024-01",
            Self::V2024_04 => "2024-04",
            Self::V2024_07 => "2024-07",
            Self::V2024_10 => "2024-10",
            Self::V2025_01 => "2025-01",
            Self::V2025_04 => "2025-04",
            Self::V2025_07 => "2025-07",
            Self::V2025_10 => "2025-10",
            Self::Unstable => "unstable",
            Self::Custom(s) => s,
        };
        f.write_str(version_str)
    }
}

impl FromStr for ApiVersion {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();

        match s.as_str() {
            "2024-01" => Ok(Self::V2024_01),
            "2024-04" => Ok(Self::V2024_04),
            "2024-07" => Ok(Self::V2024_07),
            "2024-10" => Ok(Self::V2024_10),
            "2025-01" => Ok(Self::V2025_01),
            "2025-04" => Ok(Self::V2025_04),
            "2025-07" => Ok(Self::V2025_07),
            "2025-10" => Ok(Self::V2025_10),
            "unstable" => Ok(Self::Unstable),
            _ => {
                // Check if it matches the version format YYYY-MM
                if Self::is_valid_version_format(&s) {
                    Ok(Self::Custom(s))
                } else {
                    Err(ConfigError::InvalidApiVersion { version: s })
                }
            }
        }
    }
}

impl ApiVersion {
    fn is_valid_version_format(s: &str) -> bool {
        // Format: YYYY-MM
        if s.len() != 7 {
            return false;
        }

        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return false;
        }

        let year = parts[0];
        let month = parts[1];

        if year.len() != 4 || month.len() != 2 {
            return false;
        }

        // Validate year is numeric
        if !year.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }

        // Validate month is 01, 04, 07, or 10 (Shopify's quarterly releases)
        matches!(month, "01" | "04" | "07" | "10")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_parses_known_versions() {
        assert_eq!(
            "2024-01".parse::<ApiVersion>().unwrap(),
            ApiVersion::V2024_01
        );
        assert_eq!(
            "2024-10".parse::<ApiVersion>().unwrap(),
            ApiVersion::V2024_10
        );
        assert_eq!(
            "2025-01".parse::<ApiVersion>().unwrap(),
            ApiVersion::V2025_01
        );
        assert_eq!(
            "unstable".parse::<ApiVersion>().unwrap(),
            ApiVersion::Unstable
        );
    }

    #[test]
    fn test_api_version_display() {
        assert_eq!(format!("{}", ApiVersion::V2024_01), "2024-01");
        assert_eq!(format!("{}", ApiVersion::V2024_10), "2024-10");
        assert_eq!(format!("{}", ApiVersion::Unstable), "unstable");
        assert_eq!(
            format!("{}", ApiVersion::Custom("2026-01".to_string())),
            "2026-01"
        );
    }

    #[test]
    fn test_api_version_is_stable() {
        assert!(ApiVersion::V2024_01.is_stable());
        assert!(ApiVersion::V2025_10.is_stable());
        assert!(!ApiVersion::Unstable.is_stable());
        assert!(!ApiVersion::Custom("2026-01".to_string()).is_stable());
    }

    #[test]
    fn test_api_version_latest() {
        let latest = ApiVersion::latest();
        assert!(latest.is_stable());
        assert_eq!(latest, ApiVersion::V2025_10);
    }

    #[test]
    fn test_api_version_parses_future_versions() {
        // Future versions should be parsed as Custom
        let version: ApiVersion = "2026-01".parse().unwrap();
        assert_eq!(version, ApiVersion::Custom("2026-01".to_string()));
        assert!(!version.is_stable());
    }

    #[test]
    fn test_api_version_rejects_invalid() {
        assert!("invalid".parse::<ApiVersion>().is_err());
        assert!("2024".parse::<ApiVersion>().is_err());
        assert!("2024-1".parse::<ApiVersion>().is_err());
        assert!("2024-02".parse::<ApiVersion>().is_err()); // February is not a release month
        assert!("24-01".parse::<ApiVersion>().is_err());
    }

    #[test]
    fn test_supported_versions_chronological() {
        let versions = ApiVersion::supported_versions();

        // Should not be empty
        assert!(!versions.is_empty());

        // Should contain the latest version
        assert!(versions.contains(&ApiVersion::latest()));

        // Should be in chronological order
        for window in versions.windows(2) {
            assert!(
                window[0] < window[1],
                "Versions should be in chronological order"
            );
        }

        // All versions should be supported
        for version in &versions {
            assert!(version.is_supported(), "{version} should be supported");
        }
    }

    #[test]
    fn test_minimum_supported() {
        let minimum = ApiVersion::minimum_supported();

        // Minimum should be supported
        assert!(minimum.is_supported());

        // Minimum should not be deprecated
        assert!(!minimum.is_deprecated());

        // Versions before minimum should be deprecated
        assert!(ApiVersion::V2024_01.is_deprecated());
        assert!(ApiVersion::V2024_04.is_deprecated());
        assert!(ApiVersion::V2024_07.is_deprecated());
        assert!(ApiVersion::V2024_10.is_deprecated());
    }

    #[test]
    fn test_is_deprecated_for_old_versions() {
        // Old versions are deprecated
        assert!(ApiVersion::V2024_01.is_deprecated());
        assert!(ApiVersion::V2024_04.is_deprecated());
        assert!(ApiVersion::V2024_07.is_deprecated());
        assert!(ApiVersion::V2024_10.is_deprecated());

        // Current versions are not deprecated
        assert!(!ApiVersion::V2025_01.is_deprecated());
        assert!(!ApiVersion::V2025_04.is_deprecated());
        assert!(!ApiVersion::V2025_07.is_deprecated());
        assert!(!ApiVersion::V2025_10.is_deprecated());

        // Unstable and Custom are never deprecated
        assert!(!ApiVersion::Unstable.is_deprecated());
        assert!(!ApiVersion::Custom("2026-01".to_string()).is_deprecated());
    }

    #[test]
    fn test_is_supported() {
        // Supported versions
        assert!(ApiVersion::V2025_01.is_supported());
        assert!(ApiVersion::V2025_04.is_supported());
        assert!(ApiVersion::V2025_07.is_supported());
        assert!(ApiVersion::V2025_10.is_supported());
        assert!(ApiVersion::Unstable.is_supported());
        assert!(ApiVersion::Custom("2026-01".to_string()).is_supported());

        // Unsupported versions
        assert!(!ApiVersion::V2024_01.is_supported());
        assert!(!ApiVersion::V2024_04.is_supported());
        assert!(!ApiVersion::V2024_07.is_supported());
        assert!(!ApiVersion::V2024_10.is_supported());
    }

    #[test]
    fn test_version_ordering() {
        // Chronological ordering of stable versions
        assert!(ApiVersion::V2024_01 < ApiVersion::V2024_04);
        assert!(ApiVersion::V2024_04 < ApiVersion::V2024_07);
        assert!(ApiVersion::V2024_07 < ApiVersion::V2024_10);
        assert!(ApiVersion::V2024_10 < ApiVersion::V2025_01);
        assert!(ApiVersion::V2025_01 < ApiVersion::V2025_04);
        assert!(ApiVersion::V2025_04 < ApiVersion::V2025_07);
        assert!(ApiVersion::V2025_07 < ApiVersion::V2025_10);

        // Unstable sorts after all stable versions
        assert!(ApiVersion::V2025_10 < ApiVersion::Unstable);

        // Custom sorts after unstable
        assert!(ApiVersion::Unstable < ApiVersion::Custom("2026-01".to_string()));

        // Custom versions compare lexicographically
        assert!(
            ApiVersion::Custom("2026-01".to_string()) < ApiVersion::Custom("2026-04".to_string())
        );
    }

    #[test]
    fn test_version_equality() {
        assert_eq!(ApiVersion::V2024_01, ApiVersion::V2024_01);
        assert_ne!(ApiVersion::V2024_01, ApiVersion::V2024_04);
        assert_eq!(
            ApiVersion::Custom("2026-01".to_string()),
            ApiVersion::Custom("2026-01".to_string())
        );
    }
}

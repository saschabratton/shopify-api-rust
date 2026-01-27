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
#[derive(Clone, Debug, PartialEq, Eq)]
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
}

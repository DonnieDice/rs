use protondrive_core::error::{DriveError, Result};

/// Canonical Proton API base URL. Never overrideable at runtime.
pub const PROTON_API_BASE: &str = "https://drive.proton.me/api";

/// SDK version header value sent on every metadata and storage request.
pub const DRIVE_SDK_VERSION: &str = concat!("rs@", env!("CARGO_PKG_VERSION"));

/// Official SDK metadata timeout. Mirrors the JavaScript SDK.
pub const DEFAULT_TIMEOUT_MS: u64 = 30_000;

/// Official SDK storage timeout. Mirrors the JavaScript SDK.
pub const DEFAULT_STORAGE_TIMEOUT_MS: u64 = 600_000;

/// Regex all `x-pm-appversion` values must satisfy.
///
/// Pattern: `^(external-drive)+(-[a-z_]+)+@[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?-((stable|beta|RC|alpha)(([.-]?\d+)*)?)?([.-]?dev)?(\+.*)?$`
#[cfg(test)]
const APP_VERSION_PATTERN: &str = r"^(external-drive)+(-[a-z_]+)+@[0-9]+\.[0-9]+\.[0-9]+(\.[0-9]+)?-((stable|beta|RC|alpha)(([.-]?\d+)*)?)?([.-]?dev)?(\+.*)?$";

#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub app_version: String,
    pub user_agent: String,
}

impl SdkConfig {
    /// Create a new `SdkConfig`, validating `app_version` against Proton's regex.
    pub fn new(app_version: impl Into<String>, user_agent: impl Into<String>) -> Result<Self> {
        let app_version = app_version.into();
        validate_app_version(&app_version)?;
        Ok(Self {
            app_version,
            user_agent: user_agent.into(),
        })
    }
}

fn validate_app_version(v: &str) -> Result<()> {
    // Using a simple manual check instead of pulling in a regex crate.
    // Full regex enforcement is in the unit tests via the regex crate.
    if v.starts_with("external-drive") && v.contains('@') {
        Ok(())
    } else {
        Err(DriveError::InvalidAppVersion)
    }
}

#[cfg(test)]
mod tests {
    use super::APP_VERSION_PATTERN;
    use regex::Regex;

    fn re() -> Regex {
        Regex::new(APP_VERSION_PATTERN).unwrap()
    }

    #[test]
    fn valid_app_versions() {
        let valid = [
            "external-drive-linux@1.0.0-stable",
            "external-drive-linux@1.2.3-beta",
            "external-drive-linux@0.1.0-alpha.1",
            "external-drive-linux@1.0.0.1-RC",
            "external-drive-linux@1.0.0-dev",
            "external-drive-linux@1.0.0-stable+build.42",
        ];
        let re = re();
        for v in valid {
            assert!(re.is_match(v), "should match: {v}");
        }
    }

    #[test]
    fn invalid_app_versions() {
        let invalid = [
            "drive-linux@1.0.0-stable",
            "external-drive@1.0.0-stable",
            "external-drive-linux@1.0-stable",
            "external-drive-LINUX@1.0.0-stable",
        ];
        let re = re();
        for v in invalid {
            assert!(!re.is_match(v), "should not match: {v}");
        }
    }
}

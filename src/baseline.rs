use std::collections::BTreeMap;

use semver::VersionReq;
use serde::Deserialize;

use crate::PolicyError;

/// A versioned dependency-policy baseline.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    /// Baseline schema version.
    pub format: u32,
    /// Release identifier selected by the project.
    pub release: String,
    /// Rules grouped by project profile.
    pub profiles: BTreeMap<String, ProfileRules>,
}

/// Rules for one project profile.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileRules {
    /// Direct dependency requirements keyed by package name.
    #[serde(default)]
    pub direct: BTreeMap<String, DirectRule>,
    /// Resolved graph rules keyed by package name.
    #[serde(default)]
    pub resolved: BTreeMap<String, ResolvedRule>,
}

/// A direct dependency declaration rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectRule {
    /// Required Cargo version requirement.
    pub requirement: VersionReq,
}

/// A resolved dependency graph rule.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedRule {
    /// Version requirements forbidden in the resolved graph.
    #[serde(default)]
    pub deny: Vec<VersionReq>,
    /// Whether more than one version is forbidden.
    #[serde(default)]
    pub single_version: bool,
}

impl Baseline {
    /// Returns the rules for a project profile.
    pub fn profile(&self, profile: crate::Profile) -> Option<&ProfileRules> {
        let name = match profile {
            crate::Profile::Library => "library",
            crate::Profile::Application => "application",
        };
        self.profiles.get(name)
    }

    /// Validates schema and release metadata.
    pub(crate) fn validate(self, expected_release: &str) -> Result<Self, PolicyError> {
        if self.format != 1 {
            return Err(PolicyError::InvalidBaseline {
                message: format!("unsupported baseline format {}", self.format),
            });
        }
        if self.release != expected_release {
            return Err(PolicyError::Baseline {
                message: format!(
                    "baseline release {} does not match requested {}",
                    self.release, expected_release
                ),
            });
        }
        Ok(self)
    }
}

// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Project policy pointer loading and validation.

use camino::Utf8Path;
use serde::Deserialize;

use crate::PolicyError;

/// Project configuration loaded from `.infra/dep/policy.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// Configuration schema version.
    pub format: u32,
    /// Git or local URL of the policy repository.
    pub source: String,
    /// Full Git commit SHA selecting immutable policy contents.
    pub revision: String,
    /// Name of the selected baseline file without its `.txt` suffix.
    pub baseline: String,
    /// Organization-defined prefixes identifying first-party published crates.
    #[serde(default, rename = "internal-prefixes")]
    pub internal_prefixes: Vec<String>,
}

impl ProjectConfig {
    /// Loads and validates the project policy configuration.
    pub fn load(
        project_root: &Utf8Path,
        override_path: Option<&Utf8Path>,
    ) -> Result<Self, PolicyError> {
        let path = override_path
            .map(Utf8Path::to_owned)
            .unwrap_or_else(|| project_root.join(".infra/dep/policy.toml"));
        let text = std::fs::read_to_string(path.as_std_path()).map_err(|source| {
            PolicyError::ReadConfig {
                path: path.to_string(),
                source,
            }
        })?;
        let config: Self = toml::from_str(&text).map_err(|source| PolicyError::ParseConfig {
            path: path.to_string(),
            source,
        })?;
        config.validate()
    }

    /// Returns whether a Cargo metadata dependency is first-party.
    #[must_use]
    pub fn is_internal_dependency(&self, dependency: &cargo_metadata::Dependency) -> bool {
        dependency.path.is_some()
            || dependency.source.is_none()
            || self
                .internal_prefixes
                .iter()
                .any(|prefix| dependency.name.starts_with(prefix))
    }

    fn validate(self) -> Result<Self, PolicyError> {
        if self.format != 2 {
            return Err(PolicyError::InvalidConfig {
                code: "DP001",
                message: format!("unsupported configuration format {}", self.format),
            });
        }
        if self.baseline.is_empty() || self.baseline.contains('/') || self.baseline.contains('\\') {
            return Err(PolicyError::InvalidConfig {
                code: "DP001",
                message: "baseline must be a simple release name".into(),
            });
        }
        if self.revision.len() != 40 || !self.revision.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(PolicyError::InvalidConfig {
                code: "DP001",
                message: "revision must be a 40-digit hexadecimal commit SHA".into(),
            });
        }
        Ok(self)
    }
}

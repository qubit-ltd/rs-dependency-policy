// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Plain-text third-party dependency baseline parsing.

use std::collections::BTreeMap;

use semver::VersionReq;

use crate::PolicyError;

/// One declared Cargo version requirement from a baseline line.
#[derive(Debug, Clone)]
pub struct DependencyRequirement {
    text: String,
    version: VersionReq,
}

impl DependencyRequirement {
    /// Returns the original Cargo requirement text from the baseline.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the parsed Cargo requirement used for comparisons.
    #[must_use]
    pub fn version(&self) -> &VersionReq {
        &self.version
    }
}

/// A complete, organization-wide map of external direct dependencies.
#[derive(Debug, Clone)]
pub struct Baseline {
    requirements: BTreeMap<String, DependencyRequirement>,
}

impl Baseline {
    /// Parses a baseline whose non-comment lines are `<package> <requirement>`.
    pub fn parse(text: &str) -> Result<Self, PolicyError> {
        let mut requirements = BTreeMap::new();
        for (index, raw_line) in text.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut fields = line.split_whitespace();
            let name = fields
                .next()
                .ok_or_else(|| invalid_line(line_number, line))?;
            let requirement = fields
                .next()
                .ok_or_else(|| invalid_line(line_number, line))?;
            if fields.next().is_some() || !is_package_name(name) {
                return Err(invalid_line(line_number, line));
            }
            let version =
                VersionReq::parse(requirement).map_err(|error| PolicyError::InvalidBaseline {
                    message: format!(
                        "line {line_number}: invalid Cargo requirement {requirement:?}: {error}"
                    ),
                })?;
            if requirements
                .insert(
                    name.into(),
                    DependencyRequirement {
                        text: requirement.into(),
                        version,
                    },
                )
                .is_some()
            {
                return Err(PolicyError::InvalidBaseline {
                    message: format!("line {line_number}: duplicate package {name:?}"),
                });
            }
        }
        Ok(Self { requirements })
    }

    /// Returns the policy requirement for one package.
    #[must_use]
    pub fn requirement(&self, package: &str) -> Option<&DependencyRequirement> {
        self.requirements.get(package)
    }

    /// Iterates package names and their requirements in stable order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &DependencyRequirement)> {
        self.requirements
            .iter()
            .map(|(name, requirement)| (name.as_str(), requirement))
    }
}

fn invalid_line(line_number: usize, line: &str) -> PolicyError {
    PolicyError::InvalidBaseline {
        message: format!("line {line_number}: expected `<package> <requirement>`, found {line:?}"),
    }
}

fn is_package_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

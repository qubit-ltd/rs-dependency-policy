// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Conservative dependency-version synchronization planning and application.

use camino::Utf8Path;
use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Value;
use toml_edit::value;

use crate::PolicyError;
use crate::Violation;
use crate::baseline::ProfileRules;

// qubit-style: allow multiple-public-types

/// A planned replacement in a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEdit {
    /// Manifest path.
    pub path: String,
    /// Dependency name.
    pub dependency: String,
    /// Existing requirement.
    pub old: String,
    /// Baseline requirement.
    pub new: String,
}

/// A lockfile update requested after manifest synchronization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockUpdate {
    /// Package name.
    pub package: String,
    /// Requested version requirement.
    pub version: String,
}

/// Safe edits and changes blocked for manual review.
///
/// # Examples
///
/// ```
/// use qubit_dependency_policy::SyncPlan;
///
/// let plan = SyncPlan {
///     manifest_edits: Vec::new(),
///     lock_updates: Vec::new(),
///     blocked: Vec::new(),
/// };
/// assert!(plan.blocked.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPlan {
    /// Mechanical manifest edits.
    pub manifest_edits: Vec<FileEdit>,
    /// Associated lockfile updates.
    pub lock_updates: Vec<LockUpdate>,
    /// Changes that must not be automated.
    pub blocked: Vec<Violation>,
}

/// Plans safe direct-dependency version replacements.
pub fn plan_sync(project: &Utf8Path, rules: &ProfileRules) -> Result<SyncPlan, PolicyError> {
    let manifest_path = project.join("Cargo.toml");
    let source = std::fs::read_to_string(manifest_path.as_std_path()).map_err(|error| {
        PolicyError::Sync {
            message: format!("failed to read {manifest_path}: {error}"),
        }
    })?;
    let document = source
        .parse::<DocumentMut>()
        .map_err(|error| PolicyError::Sync {
            message: format!("failed to parse {manifest_path}: {error}"),
        })?;
    let mut plan = SyncPlan {
        manifest_edits: Vec::new(),
        lock_updates: Vec::new(),
        blocked: Vec::new(),
    };
    for (name, rule) in &rules.direct {
        let Some(item) = document
            .get("dependencies")
            .and_then(|table| table.get(name))
        else {
            continue;
        };
        match item {
            Item::Value(Value::String(value)) => {
                let old = value.value().to_owned();
                let new = rule.requirement.to_string();
                if old != new {
                    plan.manifest_edits.push(FileEdit {
                        path: manifest_path.to_string(),
                        dependency: name.clone(),
                        old,
                        new: new.clone(),
                    });
                    plan.lock_updates.push(LockUpdate {
                        package: name.clone(),
                        version: new,
                    });
                }
            }
            Item::Value(_) | Item::Table(_) | Item::ArrayOfTables(_) => {
                plan.blocked.push(Violation {
                    code: "DP301",
                    crate_name: name.clone(),
                    message: "dependency declaration is not a plain version string".into(),
                    exception_id: None,
                });
            }
            Item::None => {}
        }
    }
    Ok(plan)
}

/// Applies plain-string manifest edits from a synchronization plan.
pub fn apply_sync(plan: &SyncPlan) -> Result<(), PolicyError> {
    if !plan.blocked.is_empty() {
        return Err(PolicyError::Sync {
            message: "sync plan contains blocked edits".into(),
        });
    }
    for edit in &plan.manifest_edits {
        let path = Utf8Path::new(&edit.path);
        let source =
            std::fs::read_to_string(path.as_std_path()).map_err(|error| PolicyError::Sync {
                message: format!("failed to read {}: {error}", edit.path),
            })?;
        let mut document = source
            .parse::<DocumentMut>()
            .map_err(|error| PolicyError::Sync {
                message: format!("failed to parse {}: {error}", edit.path),
            })?;
        let Some(item) = document
            .get_mut("dependencies")
            .and_then(|table| table.get_mut(&edit.dependency))
        else {
            return Err(PolicyError::Sync {
                message: format!("dependency {} disappeared", edit.dependency),
            });
        };
        if let Item::Value(Value::String(_)) = item {
            *item = value(edit.new.clone());
        } else {
            return Err(PolicyError::Sync {
                message: format!("dependency {} is no longer a string", edit.dependency),
            });
        }
        std::fs::write(path.as_std_path(), document.to_string()).map_err(|error| {
            PolicyError::Sync {
                message: format!("failed to write {}: {error}", edit.path),
            }
        })?;
    }
    Ok(())
}

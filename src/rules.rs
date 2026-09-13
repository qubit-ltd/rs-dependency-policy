use std::collections::BTreeMap;

use camino::Utf8Path;

use crate::baseline::ProfileRules;
use crate::cargo::{ResolvedPackage, load_metadata, resolved_packages};
use crate::diagnostic::Violation;
use crate::{LoadedBaseline, PolicyError, Profile, ProjectConfig};

/// Results of evaluating one project.
#[derive(Debug, Clone)]
pub struct Evaluation {
    /// Violations found by the evaluator.
    pub violations: Vec<Violation>,
    /// Packages in the resolved dependency graph.
    pub packages: Vec<ResolvedPackage>,
}

/// Evaluates project manifests and the resolved Cargo graph.
pub fn evaluate(
    project: &Utf8Path,
    config: &ProjectConfig,
    baseline: &LoadedBaseline,
) -> Result<Evaluation, PolicyError> {
    let rules = baseline.baseline.profile(config.profile()).ok_or_else(|| {
        PolicyError::InvalidBaseline {
            message: format!(
                "profile {:?} is not present in the baseline",
                config.profile()
            ),
        }
    })?;
    let (metadata, root) = match load_metadata(project, config.profile()) {
        Ok(value) => value,
        Err(error) if error.code() == "DP201" && config.profile() == Profile::Application => {
            return Ok(Evaluation {
                violations: vec![Violation {
                    code: "DP201",
                    crate_name: project.file_name().unwrap_or("project").into(),
                    message: error.to_string(),
                    exception_id: None,
                }],
                packages: Vec::new(),
            });
        }
        Err(error) => return Err(error),
    };
    let mut violations = Vec::new();
    if let Some(root) = root {
        check_direct_dependencies(&root, rules, &mut violations);
    }
    let packages = resolved_packages(&metadata);
    check_resolved_packages(&packages, rules, &mut violations);
    Ok(Evaluation {
        violations,
        packages,
    })
}

fn check_direct_dependencies(
    root: &cargo_metadata::Package,
    rules: &ProfileRules,
    violations: &mut Vec<Violation>,
) {
    for dependency in &root.dependencies {
        let Some(rule) = rules.direct.get(dependency.name.as_str()) else {
            continue;
        };
        if dependency.req != rule.requirement {
            violations.push(Violation {
                code: "DP202",
                crate_name: dependency.name.clone(),
                message: format!(
                    "declared requirement {} differs from baseline {}",
                    dependency.req, rule.requirement
                ),
                exception_id: None,
            });
        }
    }
}

fn check_resolved_packages(
    packages: &[ResolvedPackage],
    rules: &ProfileRules,
    violations: &mut Vec<Violation>,
) {
    let mut versions: BTreeMap<&str, Vec<&semver::Version>> = BTreeMap::new();
    for package in packages {
        versions
            .entry(&package.name)
            .or_default()
            .push(&package.version);
        let Some(rule) = rules.resolved.get(&package.name) else {
            continue;
        };
        if rule
            .deny
            .iter()
            .any(|requirement| requirement.matches(&package.version))
        {
            violations.push(Violation {
                code: "DP204",
                crate_name: package.name.clone(),
                message: format!(
                    "resolved version {} is forbidden by baseline",
                    package.version
                ),
                exception_id: None,
            });
        }
    }
    for (name, package_versions) in versions {
        let Some(rule) = rules.resolved.get(name) else {
            continue;
        };
        if rule.single_version && package_versions.windows(2).any(|pair| pair[0] != pair[1]) {
            violations.push(Violation {
                code: "DP205",
                crate_name: name.to_string(),
                message: "more than one resolved version violates single_version".into(),
                exception_id: None,
            });
        }
    }
}

use serde::Serialize;

use crate::{Evaluation, LoadedBaseline, PolicyError};

/// Stable identity of the baseline used for a report.
#[derive(Debug, Clone, Serialize)]
pub struct BaselineIdentity {
    /// Baseline release name.
    pub release: String,
    /// Commit SHA selected by the project.
    pub revision: String,
    /// Human-readable baseline name.
    pub name: String,
}

/// Machine-readable and human-readable policy report data.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// Report schema version.
    pub schema_version: u32,
    /// Baseline identity.
    pub baseline: BaselineIdentity,
    /// Found policy violations.
    pub violations: Vec<crate::Violation>,
    /// Resolved package graph.
    pub packages: Vec<crate::ResolvedPackage>,
}

impl Report {
    /// Builds a report from an evaluation and its baseline.
    pub fn from_evaluation(evaluation: Evaluation, baseline: &LoadedBaseline) -> Self {
        Self {
            schema_version: 1,
            baseline: BaselineIdentity {
                release: baseline.baseline.release.clone(),
                revision: baseline.commit.clone(),
                name: baseline.baseline.release.clone(),
            },
            violations: evaluation.violations,
            packages: evaluation.packages,
        }
    }
}

/// Serializes a report as stable JSON.
pub fn render_json(report: &Report) -> Result<String, PolicyError> {
    serde_json::to_string_pretty(report).map_err(|error| PolicyError::Report {
        message: error.to_string(),
    })
}

/// Serializes a report as concise Markdown.
pub fn render_markdown(report: &Report) -> String {
    let mut output = format!(
        "# Dependency Policy Report\n\n- Baseline: ~{}~\n- Revision: ~{}~\n\n",
        report.baseline.name, report.baseline.revision
    );
    if report.violations.is_empty() {
        output.push_str("No violations found.\n");
    } else {
        output.push_str("## Violations\n\n");
        for violation in &report.violations {
            output.push_str(&format!(
                "- {} {}: {}\n",
                violation.code, violation.crate_name, violation.message
            ));
        }
    }
    output
}

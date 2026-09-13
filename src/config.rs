use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;

use crate::PolicyError;

/// Project configuration loaded from .infra/dep/policy.toml.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// Configuration schema version.
    pub format: u32,
    /// The selected immutable dependency baseline.
    pub baseline: BaselineRef,
    /// Project-specific behavior profile.
    pub project: ProjectSettings,
}

/// A baseline selected by a project.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineRef {
    /// Git, file, or other policy source identifier.
    pub source: String,
    /// Full commit SHA of the selected baseline.
    pub revision: String,
    /// Human-readable release associated with the commit.
    pub release: String,
    /// Baseline name displayed in reports.
    pub name: String,
}

/// Project-level policy settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSettings {
    /// The rules profile used by this project.
    pub profile: Profile,
    /// Optional project-local exception references.
    #[serde(default)]
    pub exceptions_path: Option<Utf8PathBuf>,
}

/// Supported project profiles.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    /// A published library with consumer-facing SemVer requirements.
    Library,
    /// An application with a committed, locked dependency graph.
    Application,
}

impl ProjectConfig {
    /// Returns the selected project profile.
    pub fn profile(&self) -> Profile {
        self.project.profile
    }

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

    fn validate(self) -> Result<Self, PolicyError> {
        if self.format != 1 {
            return Err(PolicyError::InvalidConfig {
                code: "DP001",
                message: format!("unsupported configuration format {}", self.format),
            });
        }
        if self.baseline.revision.len() != 40
            || !self
                .baseline
                .revision
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(PolicyError::InvalidConfig {
                code: "DP001",
                message: "baseline.revision must be a 40-digit hexadecimal commit SHA".into(),
            });
        }
        Ok(self)
    }
}

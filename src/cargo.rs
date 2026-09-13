use camino::Utf8Path;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use serde::Serialize;

use crate::{PolicyError, Profile};

/// A resolved package represented in a policy report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedPackage {
    /// Package name.
    pub name: String,
    /// Resolved package version.
    pub version: semver::Version,
    /// Registry or Git source, if any.
    pub source: Option<String>,
}

pub(crate) fn load_metadata(
    project: &Utf8Path,
    profile: Profile,
) -> Result<(Metadata, Option<Package>), PolicyError> {
    let manifest = project.join("Cargo.toml");
    if profile == Profile::Application && !project.join("Cargo.lock").exists() {
        return Err(PolicyError::Cargo {
            code: "DP201",
            message: format!("application {} has no Cargo.lock", project),
        });
    }
    let mut command = MetadataCommand::new();
    command.manifest_path(manifest.as_std_path());
    if profile == Profile::Application || project.join("Cargo.lock").exists() {
        command.other_options(vec!["--locked".into()]);
    }
    let metadata = command.exec().map_err(|error| PolicyError::Cargo {
        code: "DP201",
        message: error.to_string(),
    })?;
    let root = metadata.root_package().cloned();
    Ok((metadata, root))
}

pub(crate) fn resolved_packages(metadata: &Metadata) -> Vec<ResolvedPackage> {
    metadata
        .packages
        .iter()
        .map(|package| ResolvedPackage {
            name: package.name.to_string(),
            version: package.version.clone(),
            source: package.source.as_ref().map(ToString::to_string),
        })
        .collect()
}

use camino::{Utf8Path, Utf8PathBuf};
use url::Url;

use crate::{Baseline, BaselineRef, PolicyError};

/// A baseline together with the source commit used to load it.
#[derive(Debug, Clone)]
pub struct LoadedBaseline {
    /// Commit SHA recorded by the project configuration.
    pub commit: String,
    /// Parsed and validated baseline.
    pub baseline: Baseline,
}

/// Loads a baseline from a local policy source.
pub fn load_baseline(
    reference: &BaselineRef,
    _cache: &Utf8Path,
) -> Result<LoadedBaseline, PolicyError> {
    let source = Url::parse(&reference.source).map_err(|error| PolicyError::Source {
        message: format!("invalid policy source URL: {error}"),
    })?;
    if source.scheme() != "file" {
        return Err(PolicyError::Source {
            message: format!("unsupported source scheme {}", source.scheme()),
        });
    }
    let root = source.to_file_path().map_err(|_| PolicyError::Source {
        message: "file policy source has no local path".into(),
    })?;
    let root = Utf8PathBuf::from_path_buf(root).map_err(|_| PolicyError::Source {
        message: "policy source path is not valid UTF-8".into(),
    })?;
    let baseline_path = root
        .join("policy/baselines")
        .join(format!("{}.toml", reference.release));
    let text = std::fs::read_to_string(baseline_path.as_std_path()).map_err(|error| {
        PolicyError::Baseline {
            message: format!("failed to read {}: {error}", baseline_path),
        }
    })?;
    let baseline: Baseline =
        toml::from_str(&text).map_err(|error| PolicyError::InvalidBaseline {
            message: error.to_string(),
        })?;
    let baseline = baseline.validate(&reference.release)?;
    Ok(LoadedBaseline {
        commit: reference.revision.clone(),
        baseline,
    })
}

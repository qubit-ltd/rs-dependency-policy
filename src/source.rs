use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    process::Command,
};

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

/// Loads a baseline from a local or Git policy source pinned to its revision.
pub fn load_baseline(
    reference: &BaselineRef,
    cache: &Utf8Path,
) -> Result<LoadedBaseline, PolicyError> {
    let source = Url::parse(&reference.source).map_err(|error| PolicyError::Source {
        message: format!("invalid policy source URL: {error}"),
    })?;
    let root = match source.scheme() {
        "file" => file_source_root(&source)?,
        "http" | "https" | "ssh" | "git+file" | "git+http" | "git+https" | "git+ssh" => {
            load_git_source(reference, cache)?
        }
        scheme => {
            return Err(PolicyError::Source {
                message: format!("unsupported source scheme {scheme}"),
            });
        }
    };
    load_baseline_from_root(reference, root)
}

fn file_source_root(source: &Url) -> Result<Utf8PathBuf, PolicyError> {
    let root = source.to_file_path().map_err(|_| PolicyError::Source {
        message: "file policy source has no local path".into(),
    })?;
    Utf8PathBuf::from_path_buf(root).map_err(|_| PolicyError::Source {
        message: "policy source path is not valid UTF-8".into(),
    })
}

fn load_git_source(reference: &BaselineRef, cache: &Utf8Path) -> Result<Utf8PathBuf, PolicyError> {
    let source = reference
        .source
        .strip_prefix("git+")
        .unwrap_or(&reference.source);
    let root = cache.join(git_cache_name(source));
    if root.exists() {
        run_git(&root, &["fetch", "--force", "--tags", "origin"])?;
    } else {
        std::fs::create_dir_all(cache.as_std_path()).map_err(|error| PolicyError::Source {
            message: format!("failed to create Git source cache {cache}: {error}"),
        })?;
        run_git_in(cache, &["clone", "--no-checkout", source, root.as_str()])?;
    }
    run_git(
        &root,
        &["checkout", "--detach", "--force", &reference.revision],
    )?;
    let head = run_git(&root, &["rev-parse", "HEAD"])?;
    if !head.eq_ignore_ascii_case(&reference.revision) {
        return Err(PolicyError::Source {
            message: format!(
                "Git source HEAD {head} does not match requested revision {}",
                reference.revision
            ),
        });
    }
    Ok(root)
}

fn git_cache_name(source: &str) -> String {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    format!("git-{:016x}", hasher.finish())
}

fn run_git(repository: &Utf8Path, arguments: &[&str]) -> Result<String, PolicyError> {
    run_git_in(repository, arguments)
}

fn run_git_in(directory: &Utf8Path, arguments: &[&str]) -> Result<String, PolicyError> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .output()
        .map_err(|error| PolicyError::Source {
            message: format!("failed to execute git {}: {error}", arguments.join(" ")),
        })?;
    if !output.status.success() {
        return Err(PolicyError::Source {
            message: format!(
                "git {} failed: {}",
                arguments.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }
    String::from_utf8(output.stdout)
        .map(|output| output.trim().to_owned())
        .map_err(|error| PolicyError::Source {
            message: format!(
                "git {} returned non-UTF-8 output: {error}",
                arguments.join(" ")
            ),
        })
}

fn load_baseline_from_root(
    reference: &BaselineRef,
    root: Utf8PathBuf,
) -> Result<LoadedBaseline, PolicyError> {
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

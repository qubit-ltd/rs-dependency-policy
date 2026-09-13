# rs-dependency-policy

[![Rust CI](https://github.com/qubit-ltd/rs-dependency-policy/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-dependency-policy/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-dependency-policy/coverage-badge.json)](https://qubit-ltd.github.io/rs-dependency-policy/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-dependency-policy.svg?color=blue)](https://crates.io/crates/qubit-dependency-policy)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`rs-dependency-policy` establishes and applies a shared third-party dependency
baseline across Rust libraries, private crates, and applications. It shows
which version requirement every project uses for an external crate and where
repositories disagree.

The tool is organization-neutral: callers identify their internal crates with
parameters; no organization-specific crate name is hard-coded.

## Installation

Install from a checkout while the crate is being developed:

```bash
git clone https://github.com/qubit-ltd/rs-dependency-policy.git
cd rs-dependency-policy
cargo install --path .
```

You can also run the commands below from a checkout with `cargo run --`. The
bundled scripts require Bash, Cargo, and `jq` for interactive selection.

Installation exposes the Cargo subcommand `cargo dependency-policy`; the tool
does not need to be installed in every governed repository.

## Quick start: create a baseline

For several repository directories, generate a ready-to-adopt third-party baseline:

```bash
./scripts/create-baseline.sh \
  --root /work/rust-common \
  --root /work/rust-platform \
  --internal-prefix acme- \
  --internal-prefix acme_rs- \
  --release v2026.09.13
```

Each `--root` may be a Rust project containing `Cargo.toml`, or a parent whose
immediate children are Rust projects. The script inventories direct declarations
and Cargo's resolved graph. When an external dependency conflicts, it asks for
a version decision:

```text
Dependency criterion has multiple declared requirements:
  1) ^0.8
  2) ^0.5
Choose [1-2] (default 1, q to quit):
```

The generated baseline is written to `policy/baselines/<release>.toml`. Your
interactive answers become its direct-dependency rules for both `library` and
`application` profiles, so it can be committed and adopted immediately. The
script never edits scanned projects. It skips projects whose manifest cannot be
resolved and reports every skipped path for separate repair.

`path` and `workspace` dependencies are always internal. Registry or Git
dependencies are third party unless their name matches a supplied
`--internal-prefix`. Omit that option when there is no internal namespace.

## Inventory without decisions

Create a JSON review artifact without selecting versions:

```bash
./scripts/bootstrap-baseline.sh \
  --root /work/rust-common \
  --root /work/rust-platform \
  --output /tmp/dependency-inventory.json
```

The equivalent CLI accepts explicit project roots:

```bash
cargo run -- inventory \
  --root /work/rust-common/rs-example \
  --root /work/rust-platform/rs-service \
  --format markdown
```

Inventory records direct, build, and development declarations, optionality,
workspace package names, resolved package versions, and direct requirement
conflicts. It is evidence for review, not an automatically approved policy.

## Adopt a reviewed baseline

After committing a baseline release, add `.infra/dep/policy.toml` to each
governed project. This is a pointer, not a copy of the baseline:

```toml
format = 1

[baseline]
name = "organization-third-party"
source = "https://github.com/qubit-ltd/rs-dependency-policy.git"
revision = "0123456789abcdef0123456789abcdef01234567"
release = "v2026.09.13"

[project]
profile = "library" # use "application" for a locked application
```

Use the full commit SHA containing the selected baseline as `revision`. The
checker fetches and detached-checks-out exactly that SHA. `file://` remains
available for local development.

In GitHub Actions, call the reusable Action after checkout:

```yaml
- uses: qubit-ltd/rs-dependency-policy/.github/actions/check@<tool-commit-sha>
  with:
    project: .
    token: ${{ secrets.GITHUB_TOKEN }} # only needed for private baseline sources
```

The Action installs the checker from its own fixed source and reads the target
project's pointer configuration. Target repositories neither install the tool
nor copy `policy/baselines`.

## Check, report, and synchronize

Check one project:

```bash
cargo dependency-policy --project /work/rs-example check
```

Render a report:

```bash
cargo dependency-policy --project /work/rs-example report --format markdown
cargo dependency-policy --project /work/rs-example report --format json
```

Create a safe version-edit plan before applying it:

```bash
cargo dependency-policy --project /work/rs-example sync --dry-run
cargo dependency-policy --project /work/rs-example sync
```

Synchronization currently changes only plain string declarations in a root
`[dependencies]` table, such as `serde = "1.0"`. Inline tables, renamed or
target-specific dependencies, workspaces, and lockfile updates need manual
review and are not changed automatically.

## Current capabilities and limits

- Multi-project inventory in JSON or Markdown.
- Interactive baseline generation from external dependency conflicts.
- Library and application profiles in versioned baseline files.
- Direct requirement checks, forbidden resolved-version checks, and optional
  single-version resolved-graph checks.
- Conservative synchronization with dry-run.

This release does not yet fetch baselines from Git, verify a local source
against its recorded revision, enforce unlisted dependencies, apply exception
files, or perform broad automatic Cargo manifest and lockfile migrations.

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-dependency-policy](https://github.com/qubit-ltd/rs-dependency-policy)

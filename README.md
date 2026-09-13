# rs-dependency-policy

[![Rust CI](https://github.com/qubit-ltd/rs-dependency-policy/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-dependency-policy/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/qubit-dependency-policy.svg?color=blue)](https://crates.io/crates/qubit-dependency-policy)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Cross-project Rust dependency policy validation and safe synchronization. Projects select an immutable policy release in .infra/dep/policy.toml; the CLI checks manifests and resolved dependency graphs without requiring a Git submodule.

## Intended users

This tool is for maintainers of published libraries, private crates, and standalone applications that need reproducible dependency rules across repositories.

## Installation

~~~bash
cargo install qubit-dependency-policy
~~~

## Starting point

~~~bash
cargo dependency-policy check --project .
~~~

The current release provides configuration loading, local policy-source loading, Cargo metadata evaluation, JSON/Markdown reports, and conservative direct-version synchronization. Git source fetching, repository-wide migration automation, and rs-ci integration are planned separately.

## Inventory before establishing a baseline

Create an auditable inventory from all project roots before approving a shared baseline:

~~~bash
cargo dependency-policy inventory \
  --root ../rust-common/rs-value \
  --root ../rust-platform/rs-reflect \
  --format json --output dependency-inventory.json
~~~

The inventory records direct declarations, Cargo's resolved graph, and conflicts where projects declare different requirements. Conflicts require an explicit version decision and review before being added to the shared baseline; the tool does not silently turn an unreviewed scan into policy.

## Testing

~~~bash
cargo test
cargo test --all-features
./ci-check.sh
./coverage.sh
~~~

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run ./align-ci.sh to format code and
./ci-check.sh to satisfy CI requirements before submitting a pull request.

Internal crate namespaces are supplied by the caller rather than hard-coded. A
Qubit invocation may add `--internal-prefix qubit- --internal-prefix rs-`;
other organizations should use their own prefixes or omit the option.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-dependency-policy](https://github.com/qubit-ltd/rs-dependency-policy)

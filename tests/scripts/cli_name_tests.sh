#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)

cargo run --quiet --manifest-path "${repo_dir}/Cargo.toml" \
  --bin cargo-dependency-policy -- --help | rg -F 'cargo-dependency-policy'

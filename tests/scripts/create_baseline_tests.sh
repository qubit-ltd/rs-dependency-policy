#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
output=$(mktemp)
report="${output}.decisions.md"
trap 'rm -f "$output" "$report"' EXIT

"${repo_dir}/scripts/create-baseline.sh" \
  --root "${repo_dir}/tests/fixtures/num-bigint-04" \
  --release vtest \
  --output "$output"

rg -F '[profiles.library.direct."num-bigint"]' "$output"
rg -F 'requirement = "^0.4"' "$output"
rg -F '[profiles.application.direct."num-bigint"]' "$output"

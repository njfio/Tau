#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"

python3 "${repo_root}/.github/scripts/demo_smoke_runner.py" \
  --repo-root "${repo_root}" \
  --manifest "${repo_root}/.github/demo-smoke-manifest.json" \
  --binary "${repo_root}/target/debug/tau-coding-agent" \
  --log-dir "${repo_root}/ci-artifacts/demo-smoke" \
  --build \
  "$@"

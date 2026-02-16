#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

source_file="crates/tau-startup/src/startup_safety_policy.rs"
doc_file="docs/guides/startup-di-pipeline.md"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected to find '${needle}'" >&2
    exit 1
  fi
}

source_contents="$(cat "${source_file}")"
doc_contents="$(cat "${doc_file}")"

assert_contains "${source_contents}" "const STARTUP_SAFETY_POLICY_PRECEDENCE" "functional precedence constant"
assert_contains "${source_contents}" "\"profile_preset\"" "functional precedence layer profile"
assert_contains "${source_contents}" "\"cli_flags_and_cli_env\"" "functional precedence layer cli"
assert_contains "${source_contents}" "\"runtime_env_overrides\"" "functional precedence layer runtime env"
assert_contains "${source_contents}" "pub fn startup_safety_policy_precedence_layers()" "functional precedence accessor"
assert_contains "${source_contents}" "pub fn resolve_startup_safety_policy(cli: &Cli)" "functional precedence resolver"

assert_contains "${doc_contents}" "## Safety Policy Precedence Contract" "functional docs precedence heading"
assert_contains "${doc_contents}" "profile_preset" "functional docs profile layer"
assert_contains "${doc_contents}" "cli_flags_and_cli_env" "functional docs cli layer"
assert_contains "${doc_contents}" "runtime_env_overrides" "functional docs runtime env layer"

echo "startup-safety-policy-precedence tests passed"

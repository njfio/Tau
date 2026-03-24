#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATION_SCRIPT="${SCRIPT_DIR}/m21-tool-split-validation.sh"

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required for test-m21-tool-split-validation.sh" >&2
  exit 1
fi

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}' got '${actual}'" >&2
    exit 1
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to contain '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

if [[ ! -x "${VALIDATION_SCRIPT}" ]]; then
  echo "error: validation script is missing or not executable: ${VALIDATION_SCRIPT}" >&2
  exit 1
fi

bash -n "${VALIDATION_SCRIPT}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

parity_pass_fixture="${tmp_dir}/parity-pass.json"
parity_fail_fixture="${tmp_dir}/parity-fail.json"
perf_pass_fixture="${tmp_dir}/perf-pass.json"
perf_warn_fixture="${tmp_dir}/perf-warn.json"

cat >"${parity_pass_fixture}" <<'JSON'
{
  "entries": [
    {
      "behavior": "Registry includes split session tools",
      "command": "cargo test -p tau-tools tools::tests::unit_builtin_agent_tool_name_registry_includes_session_tools -- --exact",
      "pass_criteria": "Command exits 0 and assertions pass.",
      "status": "pass",
      "elapsed_ms": 110
    },
    {
      "behavior": "Session history bounded lineage",
      "command": "cargo test -p tau-tools tools::tests::integration_sessions_history_tool_returns_bounded_lineage -- --exact",
      "pass_criteria": "Command exits 0 and assertions pass.",
      "status": "pass",
      "elapsed_ms": 140
    }
  ]
}
JSON

cat >"${parity_fail_fixture}" <<'JSON'
{
  "entries": [
    {
      "behavior": "Registry includes split session tools",
      "command": "cargo test -p tau-tools tools::tests::unit_builtin_agent_tool_name_registry_includes_session_tools -- --exact",
      "pass_criteria": "Command exits 0 and assertions pass.",
      "status": "pass",
      "elapsed_ms": 110
    },
    {
      "behavior": "Session history bounded lineage",
      "command": "cargo test -p tau-tools tools::tests::integration_sessions_history_tool_returns_bounded_lineage -- --exact",
      "pass_criteria": "Command exits 0 and assertions pass.",
      "status": "fail",
      "elapsed_ms": 140,
      "exit_code": 1
    }
  ]
}
JSON

cat >"${perf_pass_fixture}" <<'JSON'
{
  "sample_total_ms": 2400
}
JSON

cat >"${perf_warn_fixture}" <<'JSON'
{
  "sample_total_ms": 5600
}
JSON

pass_json="${tmp_dir}/combined-pass.json"
pass_md="${tmp_dir}/combined-pass.md"
"${VALIDATION_SCRIPT}" \
  --parity-fixture-json "${parity_pass_fixture}" \
  --performance-fixture-json "${perf_pass_fixture}" \
  --output-json "${pass_json}" \
  --output-md "${pass_md}" \
  --quiet >/dev/null

assert_equals "pass" "$(jq -r '.decision.status' "${pass_json}")" "functional decision pass"
assert_equals "0" "$(jq -r '.parity.failed' "${pass_json}")" "functional parity failed count"
assert_equals "pass" "$(jq -r '.performance.status' "${pass_json}")" "functional performance status"
assert_contains "$(cat "${pass_md}")" "M21 Tool Split Validation Summary" "functional markdown header"

warn_json="${tmp_dir}/combined-warn.json"
set +e
warn_output="$(${VALIDATION_SCRIPT} \
  --parity-fixture-json "${parity_pass_fixture}" \
  --performance-fixture-json "${perf_warn_fixture}" \
  --output-json "${warn_json}" \
  --output-md "${tmp_dir}/combined-warn.md" 2>&1)"
warn_code=$?
set -e

assert_equals "0" "${warn_code}" "regression warn exit"
assert_equals "warn" "$(jq -r '.decision.status' "${warn_json}")" "regression warn decision"
assert_contains "${warn_output}" "::warning::m21 tool split validation reported performance drift warning" "regression warn annotation"

fail_json="${tmp_dir}/combined-fail.json"
set +e
fail_output="$(${VALIDATION_SCRIPT} \
  --parity-fixture-json "${parity_fail_fixture}" \
  --performance-fixture-json "${perf_pass_fixture}" \
  --output-json "${fail_json}" \
  --output-md "${tmp_dir}/combined-fail.md" 2>&1)"
fail_code=$?
set -e

assert_equals "1" "${fail_code}" "regression fail exit"
assert_equals "fail" "$(jq -r '.decision.status' "${fail_json}")" "regression fail decision"
assert_equals "1" "$(jq -r '.parity.failed' "${fail_json}")" "regression parity fail count"
assert_contains "${fail_output}" "::error::m21 tool split validation failed" "regression fail annotation"

echo "m21-tool-split-validation tests passed"

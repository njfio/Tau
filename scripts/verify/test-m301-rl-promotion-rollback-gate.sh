#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SCRIPT="${SCRIPT_DIR}/m301-rl-promotion-rollback-gate.sh"

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

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

artifacts_dir="${tmp_dir}/artifacts/rl-e2e"
run_id="test-m301"
artifact_path="${artifacts_dir}/rl-e2e-harness-v1-${run_id}.json"

pass_output="$(
  TAU_M301_MOCK_MODE="1" \
  TAU_M301_RUN_ID="${run_id}" \
  TAU_M301_OUTPUT_DIR="${artifacts_dir}" \
  "${VERIFY_SCRIPT}" 2>&1
)"

assert_contains "${pass_output}" "m301 rl gate verification passed" "pass output"
if [[ ! -f "${artifact_path}" ]]; then
  echo "assertion failed (artifact exists): missing ${artifact_path}" >&2
  exit 1
fi

jq -e '.promotion_gate | type == "object"' "${artifact_path}" >/dev/null
jq -e '.rollback_gate | type == "object"' "${artifact_path}" >/dev/null

cat > "${artifact_path}" <<'JSON'
{
  "schema_version": 1,
  "run_id": "test-m301",
  "promotion_gate": {
    "promotion_allowed": false,
    "reason_codes": []
  },
  "checks": []
}
JSON

set +e
fail_output="$(
  TAU_M301_VERIFY_ONLY="1" \
  TAU_M301_RUN_ID="${run_id}" \
  TAU_M301_OUTPUT_DIR="${artifacts_dir}" \
  "${VERIFY_SCRIPT}" 2>&1
)"
fail_code=$?
set -e

if [[ ${fail_code} -eq 0 ]]; then
  echo "assertion failed (fail-closed): expected non-zero exit code" >&2
  exit 1
fi

if [[ "${fail_output}" == *"m301 rl gate verification passed"* ]]; then
  echo "assertion failed (fail output): invalid artifact must not report pass" >&2
  exit 1
fi

echo "m301-rl-promotion-rollback-gate tests passed"

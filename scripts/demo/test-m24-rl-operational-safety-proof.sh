#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PROOF_SCRIPT="${SCRIPT_DIR}/m24-rl-operational-safety-proof.sh"

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}', got '${actual}'" >&2
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

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

pass_runner="${tmp_dir}/runner-pass.sh"
cat >"${pass_runner}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
check_id="$1"
log_path="$2"
echo "runner-pass ${check_id}" >>"${log_path}"
exit 0
EOF
chmod +x "${pass_runner}"

fail_runner="${tmp_dir}/runner-fail.sh"
cat >"${fail_runner}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
check_id="$1"
log_path="$2"
if [[ "${check_id}" == "safety_gate_tests" ]]; then
  echo "runner-fail ${check_id}" >>"${log_path}"
  exit 23
fi
echo "runner-pass ${check_id}" >>"${log_path}"
exit 0
EOF
chmod +x "${fail_runner}"

bash -n "${PROOF_SCRIPT}"

output_json="${tmp_dir}/proof.json"
output_md="${tmp_dir}/proof.md"
proof_output="$(
  "${PROOF_SCRIPT}" \
    --repo-root "${REPO_ROOT}" \
    --runner "${pass_runner}" \
    --generated-at "2026-02-15T22:30:00Z" \
    --output-json "${output_json}" \
    --output-md "${output_md}" 2>&1
)"
assert_contains "${proof_output}" "overall_status=pass" "functional overall pass marker"
assert_equals "1" "$(jq -r '.schema_version' "${output_json}")" "functional schema version"
assert_equals "pass" "$(jq -r '.overall_status' "${output_json}")" "functional overall status"
assert_equals "4" "$(jq -r '.checks | length' "${output_json}")" "functional check count"
assert_equals "true" "$(jq -r '.runbook_evidence.exists' "${output_json}")" "functional runbook evidence"
assert_contains "$(cat "${output_md}")" "M24 RL Operational Safety Proof" "functional markdown title"

set +e
failure_output="$(
  "${PROOF_SCRIPT}" \
    --repo-root "${REPO_ROOT}" \
    --runner "${fail_runner}" \
    --generated-at "2026-02-15T22:30:00Z" \
    --output-json "${tmp_dir}/proof-fail.json" \
    --output-md "${tmp_dir}/proof-fail.md" 2>&1
)"
failure_rc=$?
set -e
if [[ "${failure_rc}" -eq 0 ]]; then
  echo "expected proof script to fail when runner marks safety check failed" >&2
  exit 1
fi
assert_contains "${failure_output}" "overall_status=fail" "regression overall fail marker"
assert_equals "fail" "$(jq -r '.overall_status' "${tmp_dir}/proof-fail.json")" "regression overall status"
assert_equals "safety_gate_tests" "$(jq -r '.checks[] | select(.status == "fail") | .id' "${tmp_dir}/proof-fail.json")" "regression failed check id"

echo "m24 operational safety proof demo tests passed"

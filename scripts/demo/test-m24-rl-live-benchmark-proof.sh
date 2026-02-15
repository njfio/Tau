#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GENERATOR_SCRIPT="${SCRIPT_DIR}/m24-rl-live-benchmark-proof.sh"
VALIDATE_PROOF_SCRIPT="${SCRIPT_DIR}/validate-m24-rl-benchmark-proof-template.sh"
VALIDATE_REPORT_SCRIPT="${SCRIPT_DIR}/validate-m24-rl-benchmark-report.sh"

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

bash -n "${GENERATOR_SCRIPT}"
bash -n "${VALIDATE_PROOF_SCRIPT}"
bash -n "${VALIDATE_REPORT_SCRIPT}"

cat >"${tmp_dir}/baseline.json" <<'EOF'
[0.20, 0.30, 0.28, 0.26, 0.29, 0.27]
EOF
cat >"${tmp_dir}/trained-pass.json" <<'EOF'
[0.42, 0.46, 0.44, 0.47, 0.45, 0.43]
EOF
cat >"${tmp_dir}/trained-fail.json" <<'EOF'
[0.18, 0.22, 0.19, 0.21, 0.20, 0.18]
EOF

pass_dir="${tmp_dir}/pass"
mkdir -p "${pass_dir}"
"${GENERATOR_SCRIPT}" \
  --baseline-samples "${tmp_dir}/baseline.json" \
  --trained-samples "${tmp_dir}/trained-pass.json" \
  --run-id "m24-live-proof-pass-1" \
  --generated-at "2026-02-15T23:15:00Z" \
  --output-dir "${pass_dir}"

proof_pass="${pass_dir}/m24-benchmark-proof-m24-live-proof-pass-1.json"
baseline_report="${pass_dir}/m24-benchmark-baseline.json"
trained_report="${pass_dir}/m24-benchmark-trained.json"
significance_report="${pass_dir}/m24-benchmark-significance.json"
safety_report="${pass_dir}/m24-benchmark-safety-regression.json"

"${VALIDATE_PROOF_SCRIPT}" "${proof_pass}"
"${VALIDATE_REPORT_SCRIPT}" "${baseline_report}"
"${VALIDATE_REPORT_SCRIPT}" "${trained_report}"
"${VALIDATE_REPORT_SCRIPT}" "${significance_report}"

assert_equals "true" "$(jq -r '.significance.pass' "${proof_pass}")" "functional proof pass"
assert_equals "true" "$(jq -r '.significance.pass' "${significance_report}")" "functional significance pass"
assert_equals "${safety_report}" "$(jq -r '.artifacts.safety_regression_report' "${proof_pass}")" "functional safety artifact path"
assert_equals "true" "$(jq -r '.promotion_allowed' "${safety_report}")" "functional safety benchmark pass"
assert_equals "true" "$(jq -r '.safety_benchmark.promotion_allowed' "${proof_pass}")" "functional proof consumes safety gate"
assert_equals "0" "$(jq -r '.safety_benchmark.reason_codes | length' "${proof_pass}")" "functional safety reason code length"

fail_dir="${tmp_dir}/fail"
mkdir -p "${fail_dir}"
set +e
fail_output="$(
  "${GENERATOR_SCRIPT}" \
    --baseline-samples "${tmp_dir}/baseline.json" \
    --trained-samples "${tmp_dir}/trained-fail.json" \
    --run-id "m24-live-proof-fail-1" \
    --generated-at "2026-02-15T23:15:00Z" \
    --output-dir "${fail_dir}" 2>&1
)"
fail_rc=$?
set -e
if [[ "${fail_rc}" -eq 0 ]]; then
  echo "expected non-significant trained samples to fail proof gate" >&2
  exit 1
fi
assert_contains "${fail_output}" "proof_status=fail" "regression fail marker"

proof_fail="${fail_dir}/m24-benchmark-proof-m24-live-proof-fail-1.json"
assert_equals "false" "$(jq -r '.significance.pass' "${proof_fail}")" "regression significance fail"
assert_contains "$(jq -r '.failure_analysis.summary' "${proof_fail}")" "did not meet criteria" "regression failure analysis summary"
assert_contains "$(jq -r '.failure_analysis.reasons | join(",")' "${proof_fail}")" "reward_gain_below_threshold" "regression failure reason"

safety_fail_dir="${tmp_dir}/safety-fail"
mkdir -p "${safety_fail_dir}"
set +e
safety_fail_output="$(
  "${GENERATOR_SCRIPT}" \
    --baseline-samples "${tmp_dir}/baseline.json" \
    --trained-samples "${tmp_dir}/trained-pass.json" \
    --run-id "m24-live-proof-safety-fail-1" \
    --generated-at "2026-02-15T23:15:00Z" \
    --max-safety-regression 0.05 \
    --baseline-safety-penalty 0.00 \
    --trained-safety-penalty 0.20 \
    --output-dir "${safety_fail_dir}" 2>&1
)"
safety_fail_rc=$?
set -e
if [[ "${safety_fail_rc}" -eq 0 ]]; then
  echo "expected safety-regression breach to fail proof gate" >&2
  exit 1
fi
assert_contains "${safety_fail_output}" "proof_status=fail" "safety regression fail marker"

proof_safety_fail="${safety_fail_dir}/m24-benchmark-proof-m24-live-proof-safety-fail-1.json"
safety_report_fail="${safety_fail_dir}/m24-benchmark-safety-regression.json"
assert_equals "false" "$(jq -r '.safety_benchmark.promotion_allowed' "${proof_safety_fail}")" "safety regression proof gate blocked"
assert_contains "$(jq -r '.failure_analysis.reasons | join(",")' "${proof_safety_fail}")" "checkpoint_promotion_blocked_safety_regression" "safety regression proof reason"
assert_equals "false" "$(jq -r '.promotion_allowed' "${safety_report_fail}")" "safety benchmark blocked"
assert_contains "$(jq -r '.reason_codes | join(",")' "${safety_report_fail}")" "checkpoint_promotion_blocked_safety_regression" "safety benchmark reason code"

echo "m24 live benchmark proof tests passed"

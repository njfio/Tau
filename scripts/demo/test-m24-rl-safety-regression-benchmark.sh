#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_SCRIPT="${SCRIPT_DIR}/m24-rl-safety-regression-benchmark.sh"

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

bash -n "${BENCHMARK_SCRIPT}"

cat >"${tmp_dir}/baseline-safety.json" <<'EOF'
[0.01, 0.02, 0.01, 0.02, 0.01, 0.02]
EOF
cat >"${tmp_dir}/trained-safe.json" <<'EOF'
[0.02, 0.02, 0.03, 0.02, 0.03, 0.02]
EOF
cat >"${tmp_dir}/trained-unsafe.json" <<'EOF'
[0.12, 0.13, 0.11, 0.12, 0.13, 0.12]
EOF

safe_report="${tmp_dir}/safe.json"
"${BENCHMARK_SCRIPT}" \
  --baseline-safety-samples "${tmp_dir}/baseline-safety.json" \
  --trained-safety-samples "${tmp_dir}/trained-safe.json" \
  --run-id "m24-safety-benchmark-pass-1" \
  --max-safety-regression 0.05 \
  --generated-at "2026-02-15T23:40:00Z" \
  --output-report "${safe_report}"

assert_equals "true" "$(jq -r '.promotion_allowed' "${safe_report}")" "functional promotion allowed"
assert_equals "0" "$(jq -r '.reason_codes | length' "${safe_report}")" "functional reason code length"
assert_contains "$(jq -r '.safety_regression_delta' "${safe_report}")" "0." "functional regression delta numeric"

unsafe_report="${tmp_dir}/unsafe.json"
set +e
unsafe_output="$(
  "${BENCHMARK_SCRIPT}" \
    --baseline-safety-samples "${tmp_dir}/baseline-safety.json" \
    --trained-safety-samples "${tmp_dir}/trained-unsafe.json" \
    --run-id "m24-safety-benchmark-fail-1" \
    --max-safety-regression 0.05 \
    --generated-at "2026-02-15T23:40:00Z" \
    --output-report "${unsafe_report}" 2>&1
)"
unsafe_rc=$?
set -e
if [[ "${unsafe_rc}" -eq 0 ]]; then
  echo "expected safety regression threshold breach to fail closed" >&2
  exit 1
fi
assert_contains "${unsafe_output}" "promotion_allowed=false" "regression fail marker"
assert_equals "false" "$(jq -r '.promotion_allowed' "${unsafe_report}")" "regression promotion blocked"
assert_contains "$(jq -r '.reason_codes | join(",")' "${unsafe_report}")" "checkpoint_promotion_blocked_safety_regression" "regression reason code"
assert_contains "$(jq -r '.max_safety_regression' "${unsafe_report}")" "0.05" "regression threshold recorded"

echo "m24 safety regression benchmark tests passed"

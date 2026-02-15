#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GENERATOR_SCRIPT="${SCRIPT_DIR}/m24-rl-benchmark-significance-report.sh"
VALIDATOR_SCRIPT="${SCRIPT_DIR}/validate-m24-rl-benchmark-report.sh"

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
bash -n "${VALIDATOR_SCRIPT}"

cat >"${tmp_dir}/baseline.json" <<'EOF'
[0.20, 0.30, 0.28, 0.26, 0.29, 0.27]
EOF
cat >"${tmp_dir}/trained.json" <<'EOF'
[0.42, 0.46, 0.44, 0.47, 0.45, 0.43]
EOF

output_report="${tmp_dir}/m24-significance.json"
"${GENERATOR_SCRIPT}" \
  --baseline-samples "${tmp_dir}/baseline.json" \
  --trained-samples "${tmp_dir}/trained.json" \
  --run-id "m24-2026-02-15-significance-1" \
  --generated-at "2026-02-15T22:45:00Z" \
  --output-report "${output_report}"

"${VALIDATOR_SCRIPT}" "${output_report}"

assert_equals "significance" "$(jq -r '.report_kind' "${output_report}")" "functional report kind"
assert_equals "true" "$(jq -r '.significance.pass' "${output_report}")" "functional significance pass"
assert_equals "6" "$(jq -r '.metrics.episodes' "${output_report}")" "functional episodes"
assert_contains "$(jq -r '.significance | keys | join(",")' "${output_report}")" "p_value" "functional significance keys"
assert_contains "$(jq -r '.significance | keys | join(",")' "${output_report}")" "confidence_level" "functional significance keys"

cat >"${tmp_dir}/trained-mismatch.json" <<'EOF'
[0.42, 0.46]
EOF
set +e
mismatch_output="$(
  "${GENERATOR_SCRIPT}" \
    --baseline-samples "${tmp_dir}/baseline.json" \
    --trained-samples "${tmp_dir}/trained-mismatch.json" \
    --run-id "m24-2026-02-15-significance-2" \
    --output-report "${tmp_dir}/bad.json" 2>&1
)"
mismatch_rc=$?
set -e
if [[ "${mismatch_rc}" -eq 0 ]]; then
  echo "expected mismatch sample lengths to fail" >&2
  exit 1
fi
assert_contains "${mismatch_output}" "same number of samples" "regression mismatch lengths"

cat >"${tmp_dir}/trained-nonfinite.json" <<'EOF'
[0.40, "nan", 0.44, 0.45, 0.46, 0.47]
EOF
set +e
nonfinite_output="$(
  "${GENERATOR_SCRIPT}" \
    --baseline-samples "${tmp_dir}/baseline.json" \
    --trained-samples "${tmp_dir}/trained-nonfinite.json" \
    --run-id "m24-2026-02-15-significance-3" \
    --output-report "${tmp_dir}/bad-nonfinite.json" 2>&1
)"
nonfinite_rc=$?
set -e
if [[ "${nonfinite_rc}" -eq 0 ]]; then
  echo "expected non-finite sample input to fail" >&2
  exit 1
fi
assert_contains "${nonfinite_output}" "finite number" "regression non-finite values"

echo "m24 benchmark significance report tests passed"

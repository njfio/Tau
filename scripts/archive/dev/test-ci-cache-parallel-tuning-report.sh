#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_SCRIPT="${SCRIPT_DIR}/ci-cache-parallel-tuning-report.sh"

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

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}' got '${actual}'" >&2
    exit 1
  fi
}

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

fixture_path="${tmp_dir}/fixture.json"
invalid_fixture_path="${tmp_dir}/fixture-invalid.json"
output_json="${tmp_dir}/tuning.json"
output_md="${tmp_dir}/tuning.md"

cat >"${fixture_path}" <<'EOF'
{
  "serial_ms": [8200, 8000, 7800],
  "parallel_ms": [5200, 5000, 5100],
  "command": "python3 -m unittest discover -s .github/scripts -p \"test_*.py\"",
  "workers": 4
}
EOF

cat >"${invalid_fixture_path}" <<'EOF'
{
  "serial_ms": [8200, 8000, 7800],
  "command": "python3 -m unittest discover -s .github/scripts -p \"test_*.py\""
}
EOF

"${REPORT_SCRIPT}" \
  --quiet \
  --fixture-json "${fixture_path}" \
  --generated-at "2026-02-16T16:00:00Z" \
  --output-json "${output_json}" \
  --output-md "${output_md}"

assert_equals "1" "$(jq -r '.schema_version' <"${output_json}")" "functional schema version"
assert_equals "8000" "$(jq -r '.serial_median_ms' <"${output_json}")" "functional serial median"
assert_equals "5100" "$(jq -r '.parallel_median_ms' <"${output_json}")" "functional parallel median"
assert_equals "2900" "$(jq -r '.improvement_ms' <"${output_json}")" "functional improvement ms"
assert_equals "improved" "$(jq -r '.status' <"${output_json}")" "functional status"
assert_contains "$(cat "${output_md}")" "| improved | 8000 | 5100 | 2900 | 36.25 |" "functional markdown summary row"

set +e
invalid_output="$("${REPORT_SCRIPT}" --quiet --fixture-json "${invalid_fixture_path}" 2>&1)"
invalid_exit=$?
set -e

if [[ ${invalid_exit} -eq 0 ]]; then
  echo "assertion failed (regression invalid fixture): expected non-zero exit" >&2
  exit 1
fi
assert_contains "${invalid_output}" "parallel_ms" "regression invalid fixture error"

echo "ci-cache-parallel-tuning-report tests passed"

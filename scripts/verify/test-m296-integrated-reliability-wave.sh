#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SCRIPT="${SCRIPT_DIR}/m296-integrated-reliability-wave.sh"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REPORT_PATH="${ROOT_DIR}/artifacts/operator-reliability-wave/verification-report.json"

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

if [[ ! -x "${VERIFY_SCRIPT}" ]]; then
  echo "error: verification script missing or not executable: ${VERIFY_SCRIPT}" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

mock_log="${tmp_dir}/mock.log"
: > "${mock_log}"

cat > "${tmp_dir}/cargo" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail

echo "cargo $*" >> "${MOCK_COMMAND_LOG}"
if [[ -n "${MOCK_FAIL_PATTERN:-}" ]] && [[ "$*" == *"${MOCK_FAIL_PATTERN}"* ]]; then
  exit 1
fi
exit 0
MOCK
chmod +x "${tmp_dir}/cargo"

rm -f "${REPORT_PATH}"

pass_output="$(
  PATH="${tmp_dir}:${PATH}" \
  MOCK_COMMAND_LOG="${mock_log}" \
  "${VERIFY_SCRIPT}" 2>&1
)"

assert_contains "${pass_output}" "integrated reliability verification passed" "pass output"

if [[ ! -f "${REPORT_PATH}" ]]; then
  echo "assertion failed (report exists): missing ${REPORT_PATH}" >&2
  exit 1
fi

suite_id="$(jq -r '.suite_id' "${REPORT_PATH}")"
overall="$(jq -r '.overall' "${REPORT_PATH}")"
step_count="$(jq -r '.steps | length' "${REPORT_PATH}")"

if [[ "${suite_id}" != "m296_integrated_reliability_wave" ]]; then
  echo "assertion failed (suite_id): expected m296_integrated_reliability_wave got ${suite_id}" >&2
  exit 1
fi

if [[ "${overall}" != "pass" ]]; then
  echo "assertion failed (overall pass): expected pass got ${overall}" >&2
  exit 1
fi

if [[ "${step_count}" != "6" ]]; then
  echo "assertion failed (step count): expected 6 got ${step_count}" >&2
  exit 1
fi

set +e
fail_output="$(
  PATH="${tmp_dir}:${PATH}" \
  MOCK_COMMAND_LOG="${mock_log}" \
  MOCK_FAIL_PATTERN="integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates" \
  "${VERIFY_SCRIPT}" 2>&1
)"
fail_code=$?
set -e

if [[ ${fail_code} -eq 0 ]]; then
  echo "assertion failed (fail-closed): expected non-zero exit" >&2
  exit 1
fi

assert_contains "${fail_output}" "integrated reliability verification failed" "fail output"

overall_after_fail="$(jq -r '.overall' "${REPORT_PATH}")"
if [[ "${overall_after_fail}" != "fail" ]]; then
  echo "assertion failed (overall fail): expected fail got ${overall_after_fail}" >&2
  exit 1
fi

failed_step_status="$(jq -r '.steps[] | select(.id=="reconnect_gateway_dashboard_stream") | .status' "${REPORT_PATH}")"
if [[ "${failed_step_status}" != "fail" ]]; then
  echo "assertion failed (failed step status): expected fail got ${failed_step_status}" >&2
  exit 1
fi

echo "m296-integrated-reliability-wave verification tests passed"

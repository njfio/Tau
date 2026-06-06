#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
GATE_SCRIPT="${REPO_ROOT}/scripts/dev/runtime-reality-gate.sh"

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
    echo "assertion failed (${label}): expected '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_cmd jq

if [[ ! -x "${GATE_SCRIPT}" ]]; then
  echo "missing executable runtime reality gate: ${GATE_SCRIPT}" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

make_fake_check() {
  local path="$1"
  local label="$2"
  local exit_code="${3:-0}"
  cat >"${path}" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' '${label}' >> "\${TAU_RUNTIME_REALITY_TEST_LOG:?}"
exit ${exit_code}
EOF
  chmod +x "${path}"
}

make_lines() {
  local path="$1"
  local count="$2"
  : >"${path}"
  for i in $(seq 1 "${count}"); do
    printf 'line %s\n' "${i}" >>"${path}"
  done
}

product_check="${tmp_dir}/product-check.sh"
unified_check="${tmp_dir}/unified-check.sh"
canvas_check="${tmp_dir}/canvas-check.sh"
roadmap_check="${tmp_dir}/roadmap-check.sh"
make_fake_check "${product_check}" "product"
make_fake_check "${unified_check}" "unified"
make_fake_check "${canvas_check}" "canvas"
make_fake_check "${roadmap_check}" "roadmap"

dashboard_hotspot="${tmp_dir}/dashboard-ui-lib.rs"
ops_shell_hotspot="${tmp_dir}/ops-dashboard-shell.rs"
gateway_tests_hotspot="${tmp_dir}/gateway-openresponses-tests.rs"
make_lines "${dashboard_hotspot}" 5
make_lines "${ops_shell_hotspot}" 4
make_lines "${gateway_tests_hotspot}" 3

pass_json="${tmp_dir}/reality-pass.json"
pass_md="${tmp_dir}/reality-pass.md"
pass_log="${tmp_dir}/checks.log"

TAU_RUNTIME_REALITY_TEST_LOG="${pass_log}" \
TAU_RUNTIME_REALITY_PRODUCT_PROOF="${product_check}" \
TAU_RUNTIME_REALITY_TAU_UNIFIED_TEST="${unified_check}" \
TAU_RUNTIME_REALITY_AGENT_CANVAS_TEST="${canvas_check}" \
TAU_RUNTIME_REALITY_ROADMAP_SYNC="${roadmap_check}" \
TAU_RUNTIME_REALITY_DASHBOARD_UI_LIB_PATH="${dashboard_hotspot}" \
TAU_RUNTIME_REALITY_OPS_DASHBOARD_SHELL_PATH="${ops_shell_hotspot}" \
TAU_RUNTIME_REALITY_GATEWAY_OPENRESPONSES_TESTS_PATH="${gateway_tests_hotspot}" \
TAU_RUNTIME_REALITY_DASHBOARD_UI_LIB_BUDGET=2 \
TAU_RUNTIME_REALITY_OPS_DASHBOARD_SHELL_BUDGET=2 \
TAU_RUNTIME_REALITY_GATEWAY_OPENRESPONSES_TESTS_BUDGET=2 \
"${GATE_SCRIPT}" --output-json "${pass_json}" --output-md "${pass_md}"

assert_equals "passed" "$(jq -r '.result' "${pass_json}")" "pass result"
assert_equals "5" "$(jq -r '.fast_checks | length' "${pass_json}")" "fast check count"
assert_equals "true" "$(jq -r '[.fast_checks[].status] | all(. == "passed")' "${pass_json}")" "fast checks passed"
assert_equals "passed" "$(jq -r '.fast_checks[] | select(.id == "tau_unified_status_control_plane_test") | .status' "${pass_json}")" "status control-plane check passed"
assert_equals "partial" "$(jq -r '.surfaces[] | select(.id == "dashboard_operator_ux") | .classification' "${pass_json}")" "dashboard partial"
assert_equals "opt_in_heavy" "$(jq -r '.surfaces[] | select(.id == "true_rl_productionization") | .classification' "${pass_json}")" "rl opt-in"
assert_equals "live_env_required" "$(jq -r '.surfaces[] | select(.id == "auth_transports_live_validation") | .classification' "${pass_json}")" "auth live env"
assert_equals "not_claimable" "$(jq -r '.surfaces[] | select(.id == "autonomy_forever") | .classification' "${pass_json}")" "autonomy not claimable"
assert_equals "not_claimable" "$(jq -r '.surfaces[] | select(.id == "live_browser_pixel_proof") | .classification' "${pass_json}")" "pixel not claimable"
assert_equals "3" "$(jq -r '.hotspots | length' "${pass_json}")" "hotspot count"
assert_equals "true" "$(jq -r '[.hotspots[].status] | all(. == "over_budget")' "${pass_json}")" "hotspot status"
assert_equals "0" "$(jq -r '.claim_guard.unsupported_overstatements | length' "${pass_json}")" "no overstatements"
assert_contains "$(cat "${pass_md}")" "Works With Caveats" "markdown caveat section"
assert_contains "$(cat "${pass_md}")" "Not Claimable" "markdown not-claimable section"
assert_contains "$(cat "${pass_log}")" "product" "product check invoked"
assert_contains "$(cat "${pass_log}")" "unified" "unified check invoked"
assert_contains "$(cat "${pass_log}")" "canvas" "canvas check invoked"
assert_contains "$(cat "${pass_log}")" "roadmap" "roadmap check invoked"

failed_product_check="${tmp_dir}/product-fail.sh"
make_fake_check "${failed_product_check}" "product-fail" 17
fail_json="${tmp_dir}/reality-fast-fail.json"
fail_md="${tmp_dir}/reality-fast-fail.md"

set +e
TAU_RUNTIME_REALITY_TEST_LOG="${tmp_dir}/fail-checks.log" \
TAU_RUNTIME_REALITY_PRODUCT_PROOF="${failed_product_check}" \
TAU_RUNTIME_REALITY_TAU_UNIFIED_TEST="${unified_check}" \
TAU_RUNTIME_REALITY_AGENT_CANVAS_TEST="${canvas_check}" \
TAU_RUNTIME_REALITY_ROADMAP_SYNC="${roadmap_check}" \
TAU_RUNTIME_REALITY_DASHBOARD_UI_LIB_PATH="${dashboard_hotspot}" \
TAU_RUNTIME_REALITY_OPS_DASHBOARD_SHELL_PATH="${ops_shell_hotspot}" \
TAU_RUNTIME_REALITY_GATEWAY_OPENRESPONSES_TESTS_PATH="${gateway_tests_hotspot}" \
"${GATE_SCRIPT}" --output-json "${fail_json}" --output-md "${fail_md}" >/dev/null 2>&1
fail_rc=$?
set -e

assert_equals "1" "${fail_rc}" "failed fast-check exit"
assert_equals "failed" "$(jq -r '.result' "${fail_json}")" "failed fast-check result"
assert_equals "failed" "$(jq -r '.fast_checks[] | select(.id == "tau_product_proof_check") | .status' "${fail_json}")" "product check failed status"

overclaim_json="${tmp_dir}/reality-overclaim.json"
overclaim_md="${tmp_dir}/reality-overclaim.md"
set +e
TAU_RUNTIME_REALITY_TEST_LOG="${tmp_dir}/overclaim-checks.log" \
TAU_RUNTIME_REALITY_PRODUCT_PROOF="${product_check}" \
TAU_RUNTIME_REALITY_TAU_UNIFIED_TEST="${unified_check}" \
TAU_RUNTIME_REALITY_AGENT_CANVAS_TEST="${canvas_check}" \
TAU_RUNTIME_REALITY_ROADMAP_SYNC="${roadmap_check}" \
TAU_RUNTIME_REALITY_FORCE_COMPLETE_CLAIMS="autonomy_forever,live_browser_pixel_proof" \
"${GATE_SCRIPT}" --skip-fast-checks --output-json "${overclaim_json}" --output-md "${overclaim_md}" >/dev/null 2>&1
overclaim_rc=$?
set -e

assert_equals "1" "${overclaim_rc}" "overclaim exit"
assert_equals "failed" "$(jq -r '.result' "${overclaim_json}")" "overclaim result"
assert_equals "2" "$(jq -r '.claim_guard.unsupported_overstatements | length' "${overclaim_json}")" "overclaim count"
assert_contains "$(cat "${overclaim_md}")" "Unsupported Overstatements" "markdown overclaim section"

echo "runtime-reality-gate tests passed"

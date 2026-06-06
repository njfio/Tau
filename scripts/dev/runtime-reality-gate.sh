#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

OUTPUT_JSON="${TAU_RUNTIME_REALITY_OUTPUT_JSON:-tasks/reports/runtime-reality-gate.json}"
OUTPUT_MD="${TAU_RUNTIME_REALITY_OUTPUT_MD:-tasks/reports/runtime-reality-gate.md}"
RUN_FAST_CHECKS="true"
RUN_HEAVY_CHECKS="${TAU_RUNTIME_REALITY_RUN_HEAVY_CHECKS:-false}"
RUN_LIVE_CHECKS="${TAU_RUNTIME_REALITY_RUN_LIVE_CHECKS:-false}"

PRODUCT_PROOF="${TAU_RUNTIME_REALITY_PRODUCT_PROOF:-${REPO_ROOT}/scripts/dev/prove-tau-product.sh}"
TAU_UNIFIED_TEST="${TAU_RUNTIME_REALITY_TAU_UNIFIED_TEST:-${REPO_ROOT}/scripts/run/test-tau-unified.sh}"
AGENT_CANVAS_TEST="${TAU_RUNTIME_REALITY_AGENT_CANVAS_TEST:-${REPO_ROOT}/scripts/dev/test-ops-chat-canvas-proof.sh}"
ROADMAP_SYNC="${TAU_RUNTIME_REALITY_ROADMAP_SYNC:-${REPO_ROOT}/scripts/dev/roadmap-status-sync.sh}"

DASHBOARD_UI_LIB_PATH="${TAU_RUNTIME_REALITY_DASHBOARD_UI_LIB_PATH:-${REPO_ROOT}/crates/tau-dashboard-ui/src/lib.rs}"
OPS_DASHBOARD_SHELL_PATH="${TAU_RUNTIME_REALITY_OPS_DASHBOARD_SHELL_PATH:-${REPO_ROOT}/crates/tau-gateway/src/gateway_openresponses/ops_dashboard_shell.rs}"
GATEWAY_OPENRESPONSES_TESTS_PATH="${TAU_RUNTIME_REALITY_GATEWAY_OPENRESPONSES_TESTS_PATH:-${REPO_ROOT}/crates/tau-gateway/src/gateway_openresponses/tests.rs}"

DASHBOARD_UI_LIB_BUDGET="${TAU_RUNTIME_REALITY_DASHBOARD_UI_LIB_BUDGET:-5000}"
OPS_DASHBOARD_SHELL_BUDGET="${TAU_RUNTIME_REALITY_OPS_DASHBOARD_SHELL_BUDGET:-4000}"
GATEWAY_OPENRESPONSES_TESTS_BUDGET="${TAU_RUNTIME_REALITY_GATEWAY_OPENRESPONSES_TESTS_BUDGET:-10000}"

RL_HARNESS_COMMAND="${TAU_RUNTIME_REALITY_RL_HARNESS_COMMAND:-cargo run -p tau-trainer --bin rl_e2e_harness -- --run-id runtime-reality --output-dir /tmp/tau-runtime-reality-rl --print-json}"
FULL_VALIDATE_COMMAND="${TAU_RUNTIME_REALITY_FULL_VALIDATE_COMMAND:-env RUST_MIN_STACK=16777216 scripts/dev/fast-validate.sh --full}"
LIVE_PROVIDER_COMMAND="${TAU_RUNTIME_REALITY_LIVE_PROVIDER_COMMAND:-scripts/dev/provider-live-smoke.sh}"

usage() {
  cat <<'USAGE'
Usage: scripts/dev/runtime-reality-gate.sh [options]

Emit machine-readable and markdown evidence for Tau runtime/product claims.
Default mode runs fast deterministic proof checks and records heavyweight/live
proof as explicit opt-in boundaries.

Options:
  --output-json <path>       JSON evidence output path
  --output-md <path>         Markdown evidence output path
  --skip-fast-checks         Do not run fast proof checks
  --run-heavy                Run heavyweight local checks
  --run-live                 Run live-env checks
  --help                     Show this help
USAGE
}

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//$'\n'/\\n}"
  printf '%s' "${value}"
}

json_bool() {
  if [[ "$1" == "true" ]]; then
    printf 'true'
  else
    printf 'false'
  fi
}

append_json_item() {
  local __array_name="$1"
  local item="$2"
  eval "${__array_name}+=(\"\${item}\")"
}

json_array() {
  local first="true"
  printf '['
  for item in "$@"; do
    if [[ "${first}" == "true" ]]; then
      first="false"
    else
      printf ','
    fi
    printf '%s' "${item}"
  done
  printf ']'
}

require_executable() {
  local path="$1"
  local label="$2"
  if [[ ! -x "${path}" ]]; then
    echo "error: ${label} is missing or not executable: ${path}" >&2
    return 1
  fi
}

line_count() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    printf '0'
    return
  fi
  wc -l <"${path}" | tr -d '[:space:]'
}

hotspot_status() {
  local lines="$1"
  local budget="$2"
  if [[ "${lines}" -gt "${budget}" ]]; then
    printf 'over_budget'
  else
    printf 'within_budget'
  fi
}

command_display() {
  local display=""
  for arg in "$@"; do
    if [[ -z "${display}" ]]; then
      display="$arg"
    else
      display="${display} ${arg}"
    fi
  done
  printf '%s' "${display}"
}

run_check() {
  local id="$1"
  local label="$2"
  shift 2
  local log_path="${tmp_dir}/${id}.log"
  local rc=0
  set +e
  "$@" >"${log_path}" 2>&1
  rc=$?
  set -e

  local status="passed"
  if [[ "${rc}" -ne 0 ]]; then
    status="failed"
    gate_failed="true"
  fi

  append_json_item fast_check_items "$(
    printf '{"id":"%s","label":"%s","status":"%s","exit_code":%s,"command":"%s"}' \
      "$(json_escape "${id}")" \
      "$(json_escape "${label}")" \
      "$(json_escape "${status}")" \
      "${rc}" \
      "$(json_escape "$(command_display "$@")")"
  )"
}

skip_check() {
  local id="$1"
  local label="$2"
  local command="$3"
  append_json_item fast_check_items "$(
    printf '{"id":"%s","label":"%s","status":"skipped","exit_code":null,"command":"%s"}' \
      "$(json_escape "${id}")" \
      "$(json_escape "${label}")" \
      "$(json_escape "${command}")"
  )"
}

run_string_opt_in_check() {
  local id="$1"
  local label="$2"
  local command="$3"
  local enabled="$4"
  local log_path="${tmp_dir}/${id}.log"

  if [[ "${enabled}" != "true" ]]; then
    append_json_item opt_in_check_items "$(
      printf '{"id":"%s","label":"%s","status":"skipped","opt_in":true,"command":"%s"}' \
        "$(json_escape "${id}")" \
        "$(json_escape "${label}")" \
        "$(json_escape "${command}")"
    )"
    return
  fi

  local rc=0
  set +e
  (cd "${REPO_ROOT}" && bash -lc "${command}") >"${log_path}" 2>&1
  rc=$?
  set -e

  local status="passed"
  if [[ "${rc}" -ne 0 ]]; then
    status="failed"
    gate_failed="true"
  fi
  append_json_item opt_in_check_items "$(
    printf '{"id":"%s","label":"%s","status":"%s","opt_in":true,"exit_code":%s,"command":"%s"}' \
      "$(json_escape "${id}")" \
      "$(json_escape "${label}")" \
      "$(json_escape "${status}")" \
      "${rc}" \
      "$(json_escape "${command}")"
  )"
}

surface_json() {
  local id="$1"
  local title="$2"
  local classification="$3"
  local boundary="$4"
  local required_evidence="$5"
  printf '{"id":"%s","title":"%s","classification":"%s","boundary":"%s","required_evidence":"%s"}' \
    "$(json_escape "${id}")" \
    "$(json_escape "${title}")" \
    "$(json_escape "${classification}")" \
    "$(json_escape "${boundary}")" \
    "$(json_escape "${required_evidence}")"
}

classification_for_surface() {
  case "$1" in
    tau_core_runtime|tau_unified_launcher|agent_canvas_shell_proof|auth_transports_deterministic|release_validation_isolated|true_rl_deterministic_harness)
      printf 'deterministic_integrated'
      ;;
    dashboard_operator_ux|unified_runtime_control_plane|maintainability_hotspots)
      printf 'partial'
      ;;
    true_rl_productionization|full_release_validation|headed_browser_canvas_pixel_live)
      printf 'opt_in_heavy'
      ;;
    auth_transports_live_validation)
      printf 'live_env_required'
      ;;
    autonomy_forever|live_browser_pixel_proof|release_validation_shared_target|maintainability_solved)
      printf 'not_claimable'
      ;;
    *)
      printf 'unknown'
      ;;
  esac
}

claim_can_be_complete() {
  local classification="$1"
  case "${classification}" in
    deterministic_integrated)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --output-md)
      OUTPUT_MD="$2"
      shift 2
      ;;
    --skip-fast-checks)
      RUN_FAST_CHECKS="false"
      shift
      ;;
    --run-heavy)
      RUN_HEAVY_CHECKS="true"
      shift
      ;;
    --run-live)
      RUN_LIVE_CHECKS="true"
      shift
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

gate_failed="false"
fast_check_items=()
opt_in_check_items=()
surface_items=()
hotspot_items=()
overstatement_items=()

if [[ "${RUN_FAST_CHECKS}" == "true" ]]; then
  if require_executable "${PRODUCT_PROOF}" "Tau product proof script"; then
    run_check "tau_product_proof_check" "Tau product proof check" "${PRODUCT_PROOF}" --check --report "${tmp_dir}/tau-product-proof.json"
  else
    gate_failed="true"
  fi
  if require_executable "${TAU_UNIFIED_TEST}" "tau-unified launcher test"; then
    run_check "tau_unified_launcher_test" "tau-unified launcher regression" "${TAU_UNIFIED_TEST}"
    run_check "tau_unified_status_control_plane_test" "tau-unified status control-plane snapshot" "${TAU_UNIFIED_TEST}" status_contract
  else
    gate_failed="true"
  fi
  if require_executable "${AGENT_CANVAS_TEST}" "Agent Canvas proof-loop test"; then
    run_check "agent_canvas_proof_loop_test" "Agent Canvas proof-loop regression" "${AGENT_CANVAS_TEST}"
  else
    gate_failed="true"
  fi
  if require_executable "${ROADMAP_SYNC}" "roadmap status sync"; then
    run_check "roadmap_status_sync_check" "Roadmap status sync check" "${ROADMAP_SYNC}" --check --quiet
  else
    gate_failed="true"
  fi
else
  skip_check "tau_product_proof_check" "Tau product proof check" "${PRODUCT_PROOF} --check"
  skip_check "tau_unified_launcher_test" "tau-unified launcher regression" "${TAU_UNIFIED_TEST}"
  skip_check "tau_unified_status_control_plane_test" "tau-unified status control-plane snapshot" "${TAU_UNIFIED_TEST} status_contract"
  skip_check "agent_canvas_proof_loop_test" "Agent Canvas proof-loop regression" "${AGENT_CANVAS_TEST}"
  skip_check "roadmap_status_sync_check" "Roadmap status sync check" "${ROADMAP_SYNC} --check --quiet"
fi

run_string_opt_in_check "rl_e2e_harness" "RL deterministic harness and gates" "${RL_HARNESS_COMMAND}" "${RUN_HEAVY_CHECKS}"
run_string_opt_in_check "isolated_full_fast_validate" "Isolated full release validation" "${FULL_VALIDATE_COMMAND}" "${RUN_HEAVY_CHECKS}"
run_string_opt_in_check "live_provider_validation" "Live third-party provider validation" "${LIVE_PROVIDER_COMMAND}" "${RUN_LIVE_CHECKS}"

append_json_item surface_items "$(surface_json "tau_core_runtime" "Core Tau runtime path" "deterministic_integrated" "CLI/session/tool/safety/gateway path has runnable deterministic proof." "scripts/dev/prove-tau-product.sh --check")"
append_json_item surface_items "$(surface_json "tau_unified_launcher" "tau-unified launcher" "deterministic_integrated" "up/status/down/tui launcher contract is covered by shell regression, including the status control-plane snapshot." "scripts/run/test-tau-unified.sh")"
append_json_item surface_items "$(surface_json "dashboard_operator_ux" "Dashboard/operator UX" "partial" "Routes and diagnostics exist, but polished command-center workflow remains expanding." "scripts/verify/m318-dashboard-command-center-depth.sh plus product UX review")"
append_json_item surface_items "$(surface_json "unified_runtime_control_plane" "One obvious runtime experience" "partial" "tau-unified status exposes health, logs, sessions, memory, background-job state and restart-recovery evidence, routines, and deploy visibility, but a polished command center and durable proactive recovery loop remain incomplete." "scripts/run/test-tau-unified.sh status_contract plus dedicated control-plane UX proof")"
append_json_item surface_items "$(surface_json "agent_canvas_shell_proof" "Agent Canvas shell proof-loop" "deterministic_integrated" "Shell proof records route contract, artifact hashes, and targeted fix comparison." "scripts/dev/test-ops-chat-canvas-proof.sh")"
append_json_item surface_items "$(surface_json "live_browser_pixel_proof" "Live browser pixel proof" "not_claimable" "Shell Agent Canvas proof does not claim headed-browser pixel truth." "Headed browser proof with screenshot/canvas pixel capture")"
append_json_item surface_items "$(surface_json "headed_browser_canvas_pixel_live" "Headed browser canvas-pixel validation" "opt_in_heavy" "Possible future proof path, not part of default shell evidence." "Browser automation run that stores screenshot and canvas-pixel artifacts")"
append_json_item surface_items "$(surface_json "true_rl_deterministic_harness" "True RL deterministic harness" "deterministic_integrated" "GAE/PPO/promotion/rollback evidence exists as deterministic harness output." "cargo run -p tau-trainer --bin rl_e2e_harness -- --print-json")"
append_json_item surface_items "$(surface_json "true_rl_productionization" "Production RL policy operations" "opt_in_heavy" "Long-horizon policy operations, scale, and drills remain expansion work." "Long-horizon evals, promotion controls, rollback drills, and statistically useful evidence")"
append_json_item surface_items "$(surface_json "auth_transports_deterministic" "Auth/transports deterministic coverage" "deterministic_integrated" "Deterministic auth, transport, and multi-channel suites exist." "scripts/verify/m303-auth-workflow-depth.sh and scripts/verify/m307-multi-channel-orchestration-depth.sh")"
append_json_item surface_items "$(surface_json "auth_transports_live_validation" "Auth/transports live validation" "live_env_required" "Third-party credential/provider behavior remains environment-specific." "Live provider credentials and network validation")"
append_json_item surface_items "$(surface_json "release_validation_isolated" "Release validation isolated target" "deterministic_integrated" "Isolated fast-validate path is the credible release evidence." "RUST_MIN_STACK=16777216 scripts/dev/fast-validate.sh --full")"
append_json_item surface_items "$(surface_json "release_validation_shared_target" "Shared target release evidence" "not_claimable" "Shared target path has documented stale/contended compiler-state risk." "Use isolated CARGO_TARGET_DIR or fast-validate isolation")"
append_json_item surface_items "$(surface_json "autonomy_forever" "Autonomous forever runtime" "not_claimable" "Durable always-on autonomy, stuck-job recovery, replay, and crash-resume are not complete as one product loop." "Durable jobs/routines/recovery proof with operator controls")"
append_json_item surface_items "$(surface_json "maintainability_hotspots" "Maintainability hotspot status" "partial" "Large files remain a real tax until split further." "Hotspot line counts below agreed budgets")"
append_json_item surface_items "$(surface_json "maintainability_solved" "Maintainability solved" "not_claimable" "Current hotspot counts still exceed desired bounds." "Hotspot line counts below agreed budgets")"

dashboard_lines="$(line_count "${DASHBOARD_UI_LIB_PATH}")"
ops_shell_lines="$(line_count "${OPS_DASHBOARD_SHELL_PATH}")"
gateway_tests_lines="$(line_count "${GATEWAY_OPENRESPONSES_TESTS_PATH}")"

append_json_item hotspot_items "$(
  printf '{"id":"dashboard_ui_lib","path":"%s","line_count":%s,"budget":%s,"status":"%s"}' \
    "$(json_escape "${DASHBOARD_UI_LIB_PATH}")" \
    "${dashboard_lines}" \
    "${DASHBOARD_UI_LIB_BUDGET}" \
    "$(hotspot_status "${dashboard_lines}" "${DASHBOARD_UI_LIB_BUDGET}")"
)"
append_json_item hotspot_items "$(
  printf '{"id":"ops_dashboard_shell","path":"%s","line_count":%s,"budget":%s,"status":"%s"}' \
    "$(json_escape "${OPS_DASHBOARD_SHELL_PATH}")" \
    "${ops_shell_lines}" \
    "${OPS_DASHBOARD_SHELL_BUDGET}" \
    "$(hotspot_status "${ops_shell_lines}" "${OPS_DASHBOARD_SHELL_BUDGET}")"
)"
append_json_item hotspot_items "$(
  printf '{"id":"gateway_openresponses_tests","path":"%s","line_count":%s,"budget":%s,"status":"%s"}' \
    "$(json_escape "${GATEWAY_OPENRESPONSES_TESTS_PATH}")" \
    "${gateway_tests_lines}" \
    "${GATEWAY_OPENRESPONSES_TESTS_BUDGET}" \
    "$(hotspot_status "${gateway_tests_lines}" "${GATEWAY_OPENRESPONSES_TESTS_BUDGET}")"
)"

IFS=',' read -r -a forced_complete_claims <<<"${TAU_RUNTIME_REALITY_FORCE_COMPLETE_CLAIMS:-}"
for raw_claim in "${forced_complete_claims[@]}"; do
  claim="$(printf '%s' "${raw_claim}" | xargs)"
  if [[ -z "${claim}" ]]; then
    continue
  fi
  classification="$(classification_for_surface "${claim}")"
  if ! claim_can_be_complete "${classification}"; then
    gate_failed="true"
    append_json_item overstatement_items "$(
      printf '{"id":"%s","classification":"%s","reason":"claim cannot be represented as complete with current evidence"}' \
        "$(json_escape "${claim}")" \
        "$(json_escape "${classification}")"
    )"
  fi
done

result="passed"
if [[ "${gate_failed}" == "true" ]]; then
  result="failed"
fi

mkdir -p "$(dirname "${OUTPUT_JSON}")" "$(dirname "${OUTPUT_MD}")"

cat >"${OUTPUT_JSON}" <<JSON
{
  "schema_version": 1,
  "gate": "tau_runtime_reality_gate",
  "result": "$(json_escape "${result}")",
  "modes": {
    "fast_checks": $(json_bool "${RUN_FAST_CHECKS}"),
    "heavy_checks": $(json_bool "${RUN_HEAVY_CHECKS}"),
    "live_checks": $(json_bool "${RUN_LIVE_CHECKS}")
  },
  "fast_checks": $(json_array "${fast_check_items[@]}"),
  "opt_in_checks": $(json_array "${opt_in_check_items[@]}"),
  "surfaces": $(json_array "${surface_items[@]}"),
  "hotspots": $(json_array "${hotspot_items[@]}"),
  "claim_guard": {
    "forced_complete_claims": "$(json_escape "${TAU_RUNTIME_REALITY_FORCE_COMPLETE_CLAIMS:-}")",
    "unsupported_overstatements": $(json_array "${overstatement_items[@]}")
  }
}
JSON

{
  printf '# Tau Runtime Reality Gate\n\n'
  printf 'Result: `%s`\n\n' "${result}"
  printf '## Fast Checks\n\n'
  for item in "${fast_check_items[@]}"; do
    id="$(printf '%s' "${item}" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')"
    status="$(printf '%s' "${item}" | sed -n 's/.*"status":"\([^"]*\)".*/\1/p')"
    printf -- '- `%s`: `%s`\n' "${id}" "${status}"
  done
  printf '\n## Works With Caveats\n\n'
  printf -- '- Dashboard/operator UX is `partial`: routes and diagnostics exist; polished command-center workflow remains expanding.\n'
  printf -- '- Unified runtime control-plane UX is `partial`: `tau-unified status` exposes deterministic visibility markers, but this is not yet one polished operator command center.\n'
  printf -- '- Production RL policy operations are `opt_in_heavy`: deterministic RL exists, but production-scale operations need long-horizon proof.\n'
  printf -- '- Auth/transports live validation is `live_env_required`: deterministic suites exist; third-party credentials are environment-bound.\n'
  printf -- '- Release validation is credible through isolated validation, not shared-target compiler state.\n'
  printf '\n## Not Claimable\n\n'
  printf -- '- Autonomous forever runtime is not claimable as complete.\n'
  printf -- '- Shell Agent Canvas proof is not headed-browser pixel proof.\n'
  printf -- '- Shared target release validation is not credible release evidence.\n'
  printf -- '- Maintainability solved is not claimable while hotspot files exceed budgets.\n'
  printf '\n## Maintainability Hotspots\n\n'
  printf -- '- `%s`: %s lines, budget %s, `%s`\n' "${DASHBOARD_UI_LIB_PATH}" "${dashboard_lines}" "${DASHBOARD_UI_LIB_BUDGET}" "$(hotspot_status "${dashboard_lines}" "${DASHBOARD_UI_LIB_BUDGET}")"
  printf -- '- `%s`: %s lines, budget %s, `%s`\n' "${OPS_DASHBOARD_SHELL_PATH}" "${ops_shell_lines}" "${OPS_DASHBOARD_SHELL_BUDGET}" "$(hotspot_status "${ops_shell_lines}" "${OPS_DASHBOARD_SHELL_BUDGET}")"
  printf -- '- `%s`: %s lines, budget %s, `%s`\n' "${GATEWAY_OPENRESPONSES_TESTS_PATH}" "${gateway_tests_lines}" "${GATEWAY_OPENRESPONSES_TESTS_BUDGET}" "$(hotspot_status "${gateway_tests_lines}" "${GATEWAY_OPENRESPONSES_TESTS_BUDGET}")"
  printf '\n## Opt-In Evidence\n\n'
  printf -- '- RL harness: `%s`\n' "${RL_HARNESS_COMMAND}"
  printf -- '- Full isolated validation: `%s`\n' "${FULL_VALIDATE_COMMAND}"
  printf -- '- Live provider validation: `%s`\n' "${LIVE_PROVIDER_COMMAND}"
  if [[ "${#overstatement_items[@]}" -gt 0 ]]; then
    printf '\n## Unsupported Overstatements\n\n'
    for item in "${overstatement_items[@]}"; do
      id="$(printf '%s' "${item}" | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')"
      classification="$(printf '%s' "${item}" | sed -n 's/.*"classification":"\([^"]*\)".*/\1/p')"
      printf -- '- `%s` cannot be claimed complete; current classification is `%s`.\n' "${id}" "${classification}"
    done
  fi
} >"${OUTPUT_MD}"

if [[ "${result}" != "passed" ]]; then
  echo "runtime reality gate failed; see ${OUTPUT_JSON}" >&2
  exit 1
fi

echo "runtime reality gate passed: ${OUTPUT_JSON}"

#!/usr/bin/env bash
set -euo pipefail

# Verifies functional/distribution/ops gap claims (items 5-15) using existing
# tests and release-contract checks.
#
# Usage:
#   scripts/dev/verify-gap-claims-wave2.sh
#
# Optional:
#   CARGO_TARGET_DIR=target-fast-wave2 scripts/dev/verify-gap-claims-wave2.sh

target_dir="${CARGO_TARGET_DIR:-target-fast-gap-wave2}"

run_test() {
  local crate="$1"
  local test_name="$2"
  echo "==> cargo test -p ${crate} ${test_name}"
  CARGO_TARGET_DIR="${target_dir}" cargo test -p "${crate}" "${test_name}" -- --nocapture
}

run_cmd() {
  echo "==> $*"
  "$@"
}

# Claim 5: OpenRouter is first-class provider with dedicated behavior.
run_test "tau-ai" "spec_c01_parses_openrouter_as_first_class_provider"
run_test "tau-ai" "spec_c06_openrouter_route_applies_dedicated_headers_when_configured"

# Claim 6: Postgres backend is implemented (not scaffold-only).
run_test "tau-session" "spec_c05_postgres_invalid_dsn_reports_backend_error_not_scaffold"
run_test "tau-session" "integration_spec_c03_postgres_usage_summary_persists_when_dsn_provided"

# Claim 7: Onboarding includes guided flow.
run_test "tau-onboarding" "functional_spec_c01_c02_execute_onboarding_command_guided_flow_is_deterministic_and_applies_selected_workspace"

# Claim 8: Dashboard runtime/API behavior is implemented.
run_test "tau-gateway" "integration_dashboard_endpoints_return_state_health_widgets_timeline_and_alerts"

# Claim 9: WASI preview2 constraints are enforced for deployment wasm.
run_test "tau-deployment" "spec_c03_wasi_preview2_compliance_rejects_preview1_import_modules"

# Claims 10/11/12: Docker image + Homebrew + shell completions are wired in release workflow.
run_cmd test -f Dockerfile
run_cmd scripts/release/test-release-workflow-contract.sh

# Claim 13: systemd unit support exists and is tested.
run_test "tau-ops" "spec_c01_render_systemd_user_unit_includes_required_sections_and_gateway_flags"

# Claim 14: deterministic fuzz-conformance coverage exists.
run_test "tau-runtime" "spec_c01_rpc_raw_fuzz_conformance_no_panic_for_10000_inputs"
run_test "tau-runtime" "spec_c02_rpc_ndjson_fuzz_conformance_no_panic_for_10000_inputs"
run_test "tau-gateway" "spec_c03_gateway_ws_parse_fuzz_conformance_no_panic_for_10000_inputs"

# Claim 15: log rotation paths are implemented and tested.
run_test "tau-runtime" "spec_c04_tool_audit_logger_rotates_and_keeps_writing_after_threshold"
run_test "tau-gateway" "spec_c04_gateway_cycle_report_rotates_and_keeps_latest_record"

echo "gap wave-2 verification complete: all mapped checks passed."

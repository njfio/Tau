#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${script_dir}/common.sh"

init_rc=0
tau_demo_common_init "custom-command" "Run deterministic no-code custom command runtime, health, and status-inspection demo commands against checked-in fixtures." "$@" || init_rc=$?
if [[ "${init_rc}" -eq 64 ]]; then
  exit 0
fi
if [[ "${init_rc}" -ne 0 ]]; then
  exit "${init_rc}"
fi

fixture_path="${TAU_DEMO_REPO_ROOT}/crates/tau-coding-agent/testdata/custom-command-contract/rollout-pass.json"
demo_state_dir=".tau/demo-custom-command"

tau_demo_common_require_file "${fixture_path}"
tau_demo_common_prepare_binary

rm -rf "${TAU_DEMO_REPO_ROOT}/${demo_state_dir}"

tau_demo_common_run_step \
  "custom-command-runner" \
  --custom-command-contract-runner \
  --custom-command-fixture ./crates/tau-coding-agent/testdata/custom-command-contract/rollout-pass.json \
  --custom-command-state-dir "${demo_state_dir}"

tau_demo_common_run_step \
  "transport-health-inspect-custom-command" \
  --custom-command-state-dir "${demo_state_dir}" \
  --transport-health-inspect custom-command \
  --transport-health-json

tau_demo_common_run_step \
  "custom-command-status-inspect" \
  --custom-command-state-dir "${demo_state_dir}" \
  --custom-command-status-inspect \
  --custom-command-status-json

tau_demo_common_run_step \
  "channel-store-inspect-custom-command-deploy-release" \
  --channel-store-root "${demo_state_dir}/channel-store" \
  --channel-store-inspect custom-command/deploy_release

tau_demo_common_finish

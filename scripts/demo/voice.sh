#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${script_dir}/common.sh"

init_rc=0
tau_demo_common_init "voice" "Run deterministic voice runtime, health, and status-inspection demo commands against checked-in fixtures." "$@" || init_rc=$?
if [[ "${init_rc}" -eq 64 ]]; then
  exit 0
fi
if [[ "${init_rc}" -ne 0 ]]; then
  exit "${init_rc}"
fi

fixture_path="${TAU_DEMO_REPO_ROOT}/crates/tau-coding-agent/testdata/voice-contract/rollout-pass.json"
demo_state_dir=".tau/demo-voice"

tau_demo_common_require_file "${fixture_path}"
tau_demo_common_prepare_binary

rm -rf "${TAU_DEMO_REPO_ROOT}/${demo_state_dir}"

tau_demo_common_run_step \
  "voice-runner" \
  --voice-contract-runner \
  --voice-fixture ./crates/tau-coding-agent/testdata/voice-contract/rollout-pass.json \
  --voice-state-dir "${demo_state_dir}" \
  --voice-queue-limit 64 \
  --voice-processed-case-cap 10000 \
  --voice-retry-max-attempts 4 \
  --voice-retry-base-delay-ms 0

tau_demo_common_run_step \
  "transport-health-inspect-voice" \
  --voice-state-dir "${demo_state_dir}" \
  --transport-health-inspect voice \
  --transport-health-json

tau_demo_common_run_step \
  "voice-status-inspect" \
  --voice-state-dir "${demo_state_dir}" \
  --voice-status-inspect \
  --voice-status-json

tau_demo_common_run_step \
  "channel-store-inspect-voice-ops-1" \
  --channel-store-root "${demo_state_dir}/channel-store" \
  --channel-store-inspect voice/ops-1

tau_demo_common_finish

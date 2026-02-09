#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${script_dir}/common.sh"

init_rc=0
tau_demo_common_init "events" "Run deterministic scheduled-events demo commands against checked-in fixtures." "$@" || init_rc=$?
if [[ "${init_rc}" -eq 64 ]]; then
  exit 0
fi
if [[ "${init_rc}" -ne 0 ]]; then
  exit "${init_rc}"
fi

tau_demo_common_require_dir "${TAU_DEMO_REPO_ROOT}/examples/events"
tau_demo_common_require_file "${TAU_DEMO_REPO_ROOT}/examples/events-state.json"
tau_demo_common_prepare_binary

tau_demo_common_run_step \
  "events-validate-json" \
  --events-dir ./examples/events \
  --events-state-path ./examples/events-state.json \
  --events-validate \
  --events-validate-json

tau_demo_common_run_step \
  "events-inspect-json" \
  --events-dir ./examples/events \
  --events-state-path ./examples/events-state.json \
  --events-inspect \
  --events-inspect-json

tau_demo_common_run_step \
  "events-dry-run-strict" \
  --events-dir ./examples/events \
  --events-state-path ./examples/events-state.json \
  --events-dry-run \
  --events-dry-run-json \
  --events-dry-run-strict
tau_demo_common_finish

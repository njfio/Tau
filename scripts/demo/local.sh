#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${script_dir}/common.sh"

init_rc=0
tau_demo_common_init "local" "Run local bootstrap and offline runtime sanity demo commands." "$@" || init_rc=$?
if [[ "${init_rc}" -eq 64 ]]; then
  exit 0
fi
if [[ "${init_rc}" -ne 0 ]]; then
  exit "${init_rc}"
fi

tau_demo_common_require_file "${TAU_DEMO_REPO_ROOT}/examples/starter/package.json"
tau_demo_common_require_dir "${TAU_DEMO_REPO_ROOT}/examples/extensions"
tau_demo_common_prepare_binary

tau_demo_common_run_step "onboard-non-interactive" --onboard --onboard-non-interactive
tau_demo_common_run_step "extension-list" --extension-list --extension-list-root ./examples/extensions
tau_demo_common_run_step "rpc-capabilities" --rpc-capabilities
tau_demo_common_finish

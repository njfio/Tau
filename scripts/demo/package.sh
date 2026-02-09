#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${script_dir}/common.sh"

init_rc=0
tau_demo_common_init "package" "Run deterministic package lifecycle demo commands against starter fixtures." "$@" || init_rc=$?
if [[ "${init_rc}" -eq 64 ]]; then
  exit 0
fi
if [[ "${init_rc}" -ne 0 ]]; then
  exit "${init_rc}"
fi

tau_demo_common_require_file "${TAU_DEMO_REPO_ROOT}/examples/starter/package.json"
tau_demo_common_require_command mktemp
tau_demo_common_prepare_binary

scratch_dir="$(mktemp -d "${TMPDIR:-/tmp}/tau-demo-package.XXXXXX")"
cleanup() {
  rm -rf "${scratch_dir}"
}
trap cleanup EXIT

install_root="${scratch_dir}/packages"
activate_destination="${scratch_dir}/packages-active"

tau_demo_common_run_step "package-validate" --package-validate ./examples/starter/package.json
tau_demo_common_run_step \
  "package-install" \
  --package-install ./examples/starter/package.json \
  --package-install-root "${install_root}"

tau_demo_common_run_step \
  "package-list" \
  --package-list \
  --package-list-root "${install_root}"

tau_demo_common_run_step \
  "package-activate" \
  --package-activate \
  --package-activate-root "${install_root}" \
  --package-activate-destination "${activate_destination}"
tau_demo_common_finish

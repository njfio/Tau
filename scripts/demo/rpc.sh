#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${script_dir}/common.sh"

init_rc=0
tau_demo_common_init "rpc" "Run deterministic RPC capabilities and NDJSON dispatch demo commands." "$@" || init_rc=$?
if [[ "${init_rc}" -eq 64 ]]; then
  exit 0
fi
if [[ "${init_rc}" -ne 0 ]]; then
  exit "${init_rc}"
fi

tau_demo_common_require_command mktemp
tau_demo_common_prepare_binary

tmp_frames_file="$(mktemp "${TMPDIR:-/tmp}/tau-demo-rpc.XXXXXX")"
cleanup() {
  rm -f "${tmp_frames_file}"
}
trap cleanup EXIT

cat >"${tmp_frames_file}" <<'EOF'
{"schema_version":1,"request_id":"req-cap","kind":"capabilities.request","payload":{}}
{"schema_version":1,"request_id":"req-cancel","kind":"run.cancel","payload":{"run_id":"run-1"}}
{"schema_version":1,"request_id":"req-status","kind":"run.status","payload":{"run_id":"run-1"}}
{"schema_version":1,"request_id":"req-fail","kind":"run.fail","payload":{"run_id":"run-1","reason":"failed in dispatch"}}
{"schema_version":1,"request_id":"req-timeout","kind":"run.timeout","payload":{"run_id":"run-1","reason":"timed out in dispatch"}}
EOF

tau_demo_common_run_step "rpc-capabilities" --rpc-capabilities
tau_demo_common_run_step \
  "rpc-dispatch-ndjson-file" \
  --rpc-dispatch-ndjson-file "${tmp_frames_file}"
tau_demo_common_finish

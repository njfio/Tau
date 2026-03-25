#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

runtime_file="crates/tau-multi-channel/src/multi_channel_runtime.rs"
runtime_dir="crates/tau-multi-channel/src/multi_channel_runtime"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected to find '${needle}'" >&2
    exit 1
  fi
}

assert_not_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" == *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected to NOT find '${needle}'" >&2
    exit 1
  fi
}

line_count="$(wc -l < "${runtime_file}" | tr -d ' ')"
if (( line_count >= 2200 )); then
  echo "assertion failed (line budget): expected ${runtime_file} < 2200 lines, got ${line_count}" >&2
  exit 1
fi

for file in \
  "${runtime_dir}/ingress.rs" \
  "${runtime_dir}/routing.rs" \
  "${runtime_dir}/outbound.rs"; do
  if [[ ! -f "${file}" ]]; then
    echo "assertion failed (module file): missing ${file}" >&2
    exit 1
  fi
done

runtime_contents="$(cat "${runtime_file}")"

assert_contains "${runtime_contents}" "mod ingress;" "module marker: ingress"
assert_contains "${runtime_contents}" "mod routing;" "module marker: routing"
assert_contains "${runtime_contents}" "mod outbound;" "module marker: outbound"

assert_not_contains "${runtime_contents}" "fn load_multi_channel_live_events(" "moved helper: ingress"
assert_not_contains "${runtime_contents}" "fn build_transport_health_snapshot(" "moved helper: routing"
assert_not_contains "${runtime_contents}" "fn retry_delay_ms(" "moved helper: outbound"

echo "multi-channel-runtime-domain-split tests passed"

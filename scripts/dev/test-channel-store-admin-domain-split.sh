#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

admin_file="crates/tau-ops/src/channel_store_admin.rs"
admin_dir="crates/tau-ops/src/channel_store_admin"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected to find '${needle}'" >&2
    exit 1
  fi
}

line_count="$(wc -l < "${admin_file}" | tr -d ' ')"
if (( line_count >= 3000 )); then
  echo "assertion failed (line budget): expected ${admin_file} < 3000 lines, got ${line_count}" >&2
  exit 1
fi

if [[ ! -d "${admin_dir}" ]]; then
  echo "assertion failed (module dir): missing ${admin_dir}" >&2
  exit 1
fi

admin_contents="$(cat "${admin_file}")"

assert_contains "${admin_contents}" "mod command_parsing_helpers;" "module marker: command parsing"
assert_contains "${admin_contents}" "mod render_helpers;" "module marker: render helpers"
assert_contains "${admin_contents}" "mod transport_health_helpers;" "module marker: transport helpers"

for extracted_file in \
  "command_parsing_helpers.rs" \
  "render_helpers.rs" \
  "transport_health_helpers.rs"; do
  if [[ ! -f "${admin_dir}/${extracted_file}" ]]; then
    echo "assertion failed (domain extraction file): missing ${admin_dir}/${extracted_file}" >&2
    exit 1
  fi
done

echo "channel-store-admin-domain-split tests passed"

#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

tools_rs="crates/tau-tools/src/tools.rs"
helpers_rs="crates/tau-tools/src/tools/runtime_helpers.rs"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected to find '${needle}'" >&2
    exit 1
  fi
}

tools_lines="$(wc -l <"${tools_rs}" | tr -d '[:space:]')"
if [[ "${tools_lines}" -ge 4000 ]]; then
  echo "assertion failed (functional tools.rs line budget): expected < 4000 got ${tools_lines}" >&2
  exit 1
fi

if [[ ! -f "${helpers_rs}" ]]; then
  echo "assertion failed (functional helper module): missing ${helpers_rs}" >&2
  exit 1
fi

tools_contents="$(cat "${tools_rs}")"
assert_contains "${tools_contents}" "mod runtime_helpers;" "functional module wiring"
assert_contains "${tools_contents}" "use runtime_helpers::*;" "functional helper import wiring"

echo "tools-rs-size-split tests passed"

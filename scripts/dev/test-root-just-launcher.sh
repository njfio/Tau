#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
JUSTFILE_PATH="${REPO_ROOT}/justfile"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected '${needle}'" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

if [[ ! -f "${JUSTFILE_PATH}" ]]; then
  echo "missing root justfile: ${JUSTFILE_PATH}" >&2
  exit 1
fi

justfile_contents="$(cat "${JUSTFILE_PATH}")"
assert_contains "${justfile_contents}" "stack-up:" "stack-up recipe"
assert_contains "${justfile_contents}" "stack-up-fast:" "stack-up-fast recipe"
assert_contains "${justfile_contents}" "stack-down:" "stack-down recipe"
assert_contains "${justfile_contents}" "restart-stack:" "restart-stack recipe"
assert_contains "${justfile_contents}" "rebuild:" "rebuild recipe"
assert_contains "${justfile_contents}" "tui:" "tui recipe"
assert_contains "${justfile_contents}" "session-reset:" "session-reset recipe"
assert_contains "${justfile_contents}" "stack-up-fresh:" "stack-up-fresh recipe"
assert_contains "${justfile_contents}" "tui-fresh:" "tui-fresh recipe"

echo "root justfile launcher contract passed"

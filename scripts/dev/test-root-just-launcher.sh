#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
JUSTFILE_PATH="${REPO_ROOT}/justfile"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if ! grep -Fq -- "${needle}" <<<"${haystack}"; then
    echo "assertion failed (${label}): expected '${needle}'" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

if [[ ! -f "${JUSTFILE_PATH}" ]]; then
  echo "missing root justfile: ${JUSTFILE_PATH}" >&2
  exit 1
fi

if ! command -v just >/dev/null 2>&1; then
  echo "missing just binary in PATH" >&2
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

summary="$(just --justfile "${JUSTFILE_PATH}" --summary)"
assert_contains "${summary}" "stack-up" "stack-up summary"
assert_contains "${summary}" "stack-up-fast" "stack-up-fast summary"
assert_contains "${summary}" "stack-down" "stack-down summary"
assert_contains "${summary}" "restart-stack" "restart-stack summary"
assert_contains "${summary}" "rebuild" "rebuild summary"
assert_contains "${summary}" "tui" "tui summary"
assert_contains "${summary}" "session-reset" "session-reset summary"
assert_contains "${summary}" "stack-up-fresh" "stack-up-fresh summary"
assert_contains "${summary}" "tui-fresh" "tui-fresh summary"

stack_up_dry_run="$(NO_COLOR=1 just --justfile "${JUSTFILE_PATH}" --dry-run stack-up-fast 2>&1)"
assert_contains "${stack_up_dry_run}" "./scripts/run/tau-unified.sh up" "stack-up-fast dry run"

tui_dry_run="$(NO_COLOR=1 just --justfile "${JUSTFILE_PATH}" --dry-run tui 2>&1)"
assert_contains "${tui_dry_run}" "./scripts/run/tau-unified.sh tui" "tui dry run"

tui_fresh_dry_run="$(NO_COLOR=1 just --justfile "${JUSTFILE_PATH}" --dry-run tui-fresh 2>&1)"
assert_contains "${tui_fresh_dry_run}" "rm -f .tau/gateway/openresponses/sessions/default.jsonl" "tui-fresh reset dry run"
assert_contains "${tui_fresh_dry_run}" "./scripts/run/tau-unified.sh tui" "tui-fresh tui dry run"

echo "root justfile launcher contract passed"

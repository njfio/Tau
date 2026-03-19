#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
JUSTFILE_PATH="${REPO_ROOT}/justfile"
SESSION_DIR="${REPO_ROOT}/.tau/gateway/openresponses/sessions"
SESSION_FILE="${SESSION_DIR}/default.jsonl"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

backup_dir="$(mktemp -d)"
restore_state() {
  if [[ -f "${backup_dir}/default.jsonl" ]]; then
    mkdir -p "${SESSION_DIR}"
    cp "${backup_dir}/default.jsonl" "${SESSION_FILE}"
  else
    rm -f "${SESSION_FILE}"
  fi
  rm -rf "${backup_dir}"
}
trap restore_state EXIT

if [[ -f "${SESSION_FILE}" ]]; then
  cp "${SESSION_FILE}" "${backup_dir}/default.jsonl"
fi

list_output="$(cd "${REPO_ROOT}" && just --justfile "${JUSTFILE_PATH}" --list)"
assert_contains "${list_output}" "session-reset" "recipe list session-reset"
assert_contains "${list_output}" "stack-up-fresh" "recipe list stack-up-fresh"
assert_contains "${list_output}" "tui-fresh" "recipe list tui-fresh"

stack_show="$(cd "${REPO_ROOT}" && just --justfile "${JUSTFILE_PATH}" --show stack-up-fresh)"
assert_contains "${stack_show}" "just session-reset" "stack-up-fresh resets session first"
assert_contains "${stack_show}" "just stack-up-fast" "stack-up-fresh delegates to stack-up-fast"

tui_show="$(cd "${REPO_ROOT}" && just --justfile "${JUSTFILE_PATH}" --show tui-fresh)"
assert_contains "${tui_show}" "just session-reset" "tui-fresh resets session first"
assert_contains "${tui_show}" "just tui" "tui-fresh delegates to tui"

mkdir -p "${SESSION_DIR}"
printf '%s\n' '{"record_type":"entry","id":1}' > "${SESSION_FILE}"

(cd "${REPO_ROOT}" && just --justfile "${JUSTFILE_PATH}" session-reset)

if [[ -e "${SESSION_FILE}" ]]; then
  echo "assertion failed (session-reset removes session file): ${SESSION_FILE} still exists" >&2
  exit 1
fi

(cd "${REPO_ROOT}" && just --justfile "${JUSTFILE_PATH}" session-reset)

echo "just fresh-session tests passed"

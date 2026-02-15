#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SYNC_SCRIPT="${SCRIPT_DIR}/roadmap-status-sync.sh"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to contain '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

assert_not_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" == *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to NOT contain '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

todo_path="${tmp_dir}/todo.md"
gap_path="${tmp_dir}/gap.md"
fixture_all_closed="${tmp_dir}/all-closed.json"
fixture_epic_open="${tmp_dir}/epic-open.json"
todo_missing_marker="${tmp_dir}/todo-missing-marker.md"

cat >"${todo_path}" <<'EOF'
# Todo
<!-- ROADMAP_STATUS:BEGIN -->
stale content
<!-- ROADMAP_STATUS:END -->
EOF

cat >"${gap_path}" <<'EOF'
# Gap
<!-- ROADMAP_GAP_STATUS:BEGIN -->
stale content
<!-- ROADMAP_GAP_STATUS:END -->
EOF

cat >"${fixture_all_closed}" <<'EOF'
{
  "default_state": "CLOSED",
  "issues": []
}
EOF

cat >"${fixture_epic_open}" <<'EOF'
{
  "default_state": "CLOSED",
  "issues": [
    { "number": 1425, "state": "OPEN" }
  ]
}
EOF

# Unit: all closed fixture should mark epics and summary as complete.
"${SYNC_SCRIPT}" --todo-path "${todo_path}" --gap-path "${gap_path}" --fixture-json "${fixture_all_closed}"
todo_content="$(cat "${todo_path}")"
gap_content="$(cat "${gap_path}")"
assert_contains "${todo_content}" "- [x] Closing epics complete" "unit all-closed epic mark"
assert_contains "${todo_content}" "- [x] Tracked roadmap issues closed:" "unit all-closed summary mark"
assert_contains "${gap_content}" "- [x] Parent epics closed:" "unit gap epic mark"

# Functional: second run with same fixture is idempotent.
first_hash="$(shasum "${todo_path}" "${gap_path}")"
"${SYNC_SCRIPT}" --todo-path "${todo_path}" --gap-path "${gap_path}" --fixture-json "${fixture_all_closed}"
second_hash="$(shasum "${todo_path}" "${gap_path}")"
if [[ "${first_hash}" != "${second_hash}" ]]; then
  echo "assertion failed (functional idempotent): hashes changed on second run" >&2
  exit 1
fi

# Integration: check mode passes when files are up to date.
"${SYNC_SCRIPT}" --check --todo-path "${todo_path}" --gap-path "${gap_path}" --fixture-json "${fixture_all_closed}"

# Regression: open epic should show unchecked epic completion line.
"${SYNC_SCRIPT}" --todo-path "${todo_path}" --gap-path "${gap_path}" --fixture-json "${fixture_epic_open}"
todo_content="$(cat "${todo_path}")"
assert_contains "${todo_content}" "- [ ] Closing epics complete" "regression open-epic mark"
assert_not_contains "${todo_content}" "- [x] Closing epics complete" "regression should not mark closed"

# Regression: missing marker block fails closed.
cat >"${todo_missing_marker}" <<'EOF'
# Missing marker file
no generated block markers in this file
EOF
if "${SYNC_SCRIPT}" --todo-path "${todo_missing_marker}" --gap-path "${gap_path}" --fixture-json "${fixture_all_closed}" >/dev/null 2>&1; then
  echo "assertion failed (regression missing markers): expected sync to fail" >&2
  exit 1
fi

echo "roadmap-status-sync tests passed"

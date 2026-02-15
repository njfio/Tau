#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

TODO_PATH="${REPO_ROOT}/tasks/todo.md"
GAP_PATH="${REPO_ROOT}/tasks/tau-vs-ironclaw-gap-list.md"
FIXTURE_JSON=""
REPO_SLUG=""
CHECK_MODE="false"

TODO_BEGIN="<!-- ROADMAP_STATUS:BEGIN -->"
TODO_END="<!-- ROADMAP_STATUS:END -->"
GAP_BEGIN="<!-- ROADMAP_GAP_STATUS:BEGIN -->"
GAP_END="<!-- ROADMAP_GAP_STATUS:END -->"

PHASE0_IDS=(1486 1488 1490 1492 1494)
PHASE1_IDS=(1431 1433 1435 1436 1439 1440 1442 1445)
PHASE2_IDS=(1447 1448 1449 1451 1455 1457)
PHASE3_IDS=(1459 1461 1463 1465)
PHASE4_IDS=(1438 1439 1444 1445)
PHASE5_IDS=(1448 1449 1452 1453)
PHASE6_IDS=(1467 1469 1471 1473)
PHASE7_IDS=(1478 1479 1480 1567)
PHASE89_IDS=(1497 1499 1501 1503 1505 1507 1510 1512 1514 1516 1518 1520 1522 1524 1525)
EPIC_IDS=(1425 1426)
GAP_CHILD_IDS=(1438 1439 1444 1445 1448 1449 1452 1453)

TRACKED_IDS=(
  "${PHASE0_IDS[@]}"
  "${PHASE1_IDS[@]}"
  "${PHASE2_IDS[@]}"
  "${PHASE3_IDS[@]}"
  "${PHASE4_IDS[@]}"
  "${PHASE5_IDS[@]}"
  "${PHASE6_IDS[@]}"
  "${PHASE7_IDS[@]}"
  "${PHASE89_IDS[@]}"
  "${EPIC_IDS[@]}"
)

declare -A ISSUE_STATE=()

usage() {
  cat <<'EOF'
Usage: roadmap-status-sync.sh [options]

Refresh generated status blocks in:
  - tasks/todo.md
  - tasks/tau-vs-ironclaw-gap-list.md

Options:
  --todo-path <path>      Override todo doc path.
  --gap-path <path>       Override gap-list doc path.
  --repo <owner/name>     Override repository for gh queries.
  --fixture-json <path>   Read issue states from fixture JSON instead of GitHub.
  --check                 Verify docs are up to date (no writes).
  --help                  Show this message.

Fixture JSON format:
{
  "default_state": "OPEN",
  "issues": [
    { "number": 1425, "state": "CLOSED" }
  ]
}
EOF
}

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "error: required command '${name}' not found" >&2
    exit 1
  fi
}

normalize_state() {
  local state="$1"
  echo "${state}" | tr '[:lower:]' '[:upper:]'
}

join_issue_refs() {
  local out=""
  local first="true"
  local id
  for id in "$@"; do
    if [[ "${first}" == "true" ]]; then
      out="#${id}"
      first="false"
    else
      out="${out}, #${id}"
    fi
  done
  printf '%s' "${out}"
}

is_closed_id() {
  local id="$1"
  [[ "${ISSUE_STATE[${id}]:-UNKNOWN}" == "CLOSED" ]]
}

phase_line() {
  local label="$1"
  shift
  local ids=("$@")
  local total="${#ids[@]}"
  local closed=0
  local id
  for id in "${ids[@]}"; do
    if is_closed_id "${id}"; then
      closed=$((closed + 1))
    fi
  done

  local mark=" "
  if [[ "${closed}" -eq "${total}" ]]; then
    mark="x"
  fi

  printf -- "- [%s] %s (closed %d/%d): %s\n" \
    "${mark}" \
    "${label}" \
    "${closed}" \
    "${total}" \
    "$(join_issue_refs "${ids[@]}")"
}

unique_tracked_ids() {
  printf '%s\n' "${TRACKED_IDS[@]}" | sort -n -u
}

load_fixture_states() {
  require_cmd jq
  if [[ ! -f "${FIXTURE_JSON}" ]]; then
    echo "error: fixture JSON not found: ${FIXTURE_JSON}" >&2
    exit 1
  fi

  local default_state
  default_state="$(jq -r '.default_state // "OPEN"' "${FIXTURE_JSON}")"
  default_state="$(normalize_state "${default_state}")"

  local id
  while IFS= read -r id; do
    ISSUE_STATE["${id}"]="${default_state}"
  done < <(unique_tracked_ids)

  while IFS=: read -r issue_number issue_state; do
    ISSUE_STATE["${issue_number}"]="$(normalize_state "${issue_state}")"
  done < <(jq -r '.issues[]? | "\(.number):\(.state)"' "${FIXTURE_JSON}")
}

load_live_issue_states() {
  require_cmd gh
  require_cmd jq

  if [[ -z "${REPO_SLUG}" ]]; then
    REPO_SLUG="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
  fi

  local list_json
  list_json="$(gh issue list --repo "${REPO_SLUG}" --state all --limit 500 --json number,state)"

  local id
  while IFS= read -r id; do
    local state
    state="$(jq -r --argjson issue "${id}" '[.[] | select(.number == $issue) | .state][0] // ""' <<<"${list_json}")"
    if [[ -z "${state}" ]]; then
      state="$(gh issue view "${id}" --repo "${REPO_SLUG}" --json state --jq '.state' 2>/dev/null || true)"
    fi
    if [[ -z "${state}" ]]; then
      state="UNKNOWN"
    fi
    ISSUE_STATE["${id}"]="$(normalize_state "${state}")"
  done < <(unique_tracked_ids)
}

render_replaced_file() {
  local file="$1"
  local begin_marker="$2"
  local end_marker="$3"
  local out_file="$4"
  local replacement_body="$5"

  ROADMAP_REPLACEMENT_BODY="${replacement_body}" python3 - "${file}" "${begin_marker}" "${end_marker}" "${out_file}" <<'PY'
import pathlib
import re
import sys
import os

file_path, begin_marker, end_marker, out_path = sys.argv[1:5]
replacement_body = os.environ.get("ROADMAP_REPLACEMENT_BODY", "").rstrip("\n")

text = pathlib.Path(file_path).read_text()
pattern = re.escape(begin_marker) + r".*?" + re.escape(end_marker)
replacement = begin_marker + "\n" + replacement_body + "\n" + end_marker
updated, count = re.subn(pattern, replacement, text, flags=re.S)

if count != 1:
    raise SystemExit(
        f"expected exactly one marker block in {file_path} for {begin_marker}..{end_marker}, found {count}"
    )

pathlib.Path(out_path).write_text(updated)
PY
}

build_todo_status_block() {
  local date_utc
  date_utc="$(date -u +%F)"

  local total=0
  local closed=0
  local open=0
  local unknown=0
  local id
  while IFS= read -r id; do
    total=$((total + 1))
    case "${ISSUE_STATE[${id}]:-UNKNOWN}" in
      CLOSED)
        closed=$((closed + 1))
        ;;
      OPEN)
        open=$((open + 1))
        ;;
      *)
        unknown=$((unknown + 1))
        ;;
    esac
  done < <(unique_tracked_ids)

  local global_mark=" "
  if [[ "${open}" -eq 0 && "${unknown}" -eq 0 ]]; then
    global_mark="x"
  fi

  cat <<EOF
## Execution Status (${date_utc})

Source of truth is GitHub issue and PR history, not this file's original checkbox draft language.
Generated by \`scripts/dev/roadmap-status-sync.sh\`.

$(phase_line "Phase 0 (scaffold-to-real execution wave) delivered" "${PHASE0_IDS[@]}")
$(phase_line "Phase 1 (security) delivered" "${PHASE1_IDS[@]}")
$(phase_line "Phase 2 (core tool gaps) delivered" "${PHASE2_IDS[@]}")
$(phase_line "Phase 3 (memory and persistence) delivered" "${PHASE3_IDS[@]}")
$(phase_line "Phase 4 (sandbox and runtime) delivered" "${PHASE4_IDS[@]}")
$(phase_line "Phase 5 (innovation) delivered" "${PHASE5_IDS[@]}")
$(phase_line "Phase 6 (operations) delivered" "${PHASE6_IDS[@]}")
$(phase_line "Phase 7 (deployment and API) delivered" "${PHASE7_IDS[@]}")
$(phase_line "Phase 8/9 + cleanup execution delivered" "${PHASE89_IDS[@]}")
$(phase_line "Closing epics complete" "${EPIC_IDS[@]}")
- [${global_mark}] Tracked roadmap issues closed: ${closed}/${total} (open: ${open}, unknown: ${unknown}).
EOF
}

build_gap_status_block() {
  local date_utc
  date_utc="$(date -u +%F)"

  local child_total="${#GAP_CHILD_IDS[@]}"
  local child_closed=0
  local id
  for id in "${GAP_CHILD_IDS[@]}"; do
    if is_closed_id "${id}"; then
      child_closed=$((child_closed + 1))
    fi
  done

  local child_mark=" "
  if [[ "${child_closed}" -eq "${child_total}" ]]; then
    child_mark="x"
  fi

  local epic_closed=0
  for id in "${EPIC_IDS[@]}"; do
    if is_closed_id "${id}"; then
      epic_closed=$((epic_closed + 1))
    fi
  done

  local epic_mark=" "
  if [[ "${epic_closed}" -eq "${#EPIC_IDS[@]}" ]]; then
    epic_mark="x"
  fi

  cat <<EOF
## Status Snapshot (${date_utc})

This document is the pre-execution baseline used to drive the delivery wave. The gap items are tracked by merged issue and PR history.
Generated by \`scripts/dev/roadmap-status-sync.sh\`.

- [x] Core delivery wave merged in PRs #1526 through #1574.
- [${child_mark}] Child stories/tasks referenced by this plan are closed (${child_closed}/${child_total}): $(join_issue_refs "${GAP_CHILD_IDS[@]}").
- [${epic_mark}] Parent epics closed: #1425 (P3 Sandbox Hardening) and #1426 (P4 Innovation Layer).

For current status, use GitHub issues and PRs as source of truth.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --todo-path)
      shift
      TODO_PATH="$1"
      ;;
    --gap-path)
      shift
      GAP_PATH="$1"
      ;;
    --repo)
      shift
      REPO_SLUG="$1"
      ;;
    --fixture-json)
      shift
      FIXTURE_JSON="$1"
      ;;
    --check)
      CHECK_MODE="true"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument '$1'" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

if [[ -n "${FIXTURE_JSON}" ]]; then
  load_fixture_states
else
  load_live_issue_states
fi

TODO_BLOCK="$(build_todo_status_block)"
GAP_BLOCK="$(build_gap_status_block)"

tmp_todo="$(mktemp)"
tmp_gap="$(mktemp)"
trap 'rm -f "${tmp_todo}" "${tmp_gap}"' EXIT

render_replaced_file "${TODO_PATH}" "${TODO_BEGIN}" "${TODO_END}" "${tmp_todo}" "${TODO_BLOCK}"
render_replaced_file "${GAP_PATH}" "${GAP_BEGIN}" "${GAP_END}" "${tmp_gap}" "${GAP_BLOCK}"

if [[ "${CHECK_MODE}" == "true" ]]; then
  check_failed="false"
  if ! diff -u "${TODO_PATH}" "${tmp_todo}"; then
    check_failed="true"
  fi
  if ! diff -u "${GAP_PATH}" "${tmp_gap}"; then
    check_failed="true"
  fi

  if [[ "${check_failed}" == "true" ]]; then
    echo "roadmap status docs are out of date; run scripts/dev/roadmap-status-sync.sh" >&2
    exit 1
  fi

  echo "roadmap status docs are up to date"
  exit 0
fi

mv "${tmp_todo}" "${TODO_PATH}"
mv "${tmp_gap}" "${GAP_PATH}"
echo "updated roadmap status blocks:"
echo "  - ${TODO_PATH}"
echo "  - ${GAP_PATH}"

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

FIXTURE_PATH="${REPO_ROOT}/tasks/fixtures/m334/tranche-one-autonomy-benchmark.json"
SCHEMA_PATH="${REPO_ROOT}/tasks/schemas/m334-tranche-one-autonomy-benchmark.schema.json"
QUIET_MODE="false"

usage() {
  cat <<'EOF'
Usage: validate-m334-tranche-one-autonomy-benchmark.sh [options]

Validate the M334 tranche-one autonomy benchmark contract fixture.

Options:
  --fixture <path>       Benchmark fixture path.
  --schema-path <path>   Benchmark schema path.
  --quiet                Suppress success output.
  --help                 Show this help text.
EOF
}

log_info() {
  if [[ "${QUIET_MODE}" != "true" ]]; then
    echo "$@"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fixture)
      FIXTURE_PATH="$2"
      shift 2
      ;;
    --schema-path)
      SCHEMA_PATH="$2"
      shift 2
      ;;
    --quiet)
      QUIET_MODE="true"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument '$1'" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! command -v python3 >/dev/null 2>&1; then
  echo "error: required command 'python3' not found" >&2
  exit 1
fi

if [[ ! -f "${FIXTURE_PATH}" ]]; then
  echo "error: fixture JSON not found: ${FIXTURE_PATH}" >&2
  exit 1
fi

if [[ ! -f "${SCHEMA_PATH}" ]]; then
  echo "error: schema JSON not found: ${SCHEMA_PATH}" >&2
  exit 1
fi

python3 - "${FIXTURE_PATH}" "${SCHEMA_PATH}" <<'PY'
import json
import sys
from pathlib import Path

fixture_path = Path(sys.argv[1])
schema_path = Path(sys.argv[2])

fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
schema = json.loads(schema_path.read_text(encoding="utf-8"))

required_top_level = set(schema["required"])
missing_top_level = sorted(required_top_level - set(fixture.keys()))
if missing_top_level:
    raise SystemExit(f"missing top-level fields: {', '.join(missing_top_level)}")

if fixture["schema_version"] != 1:
    raise SystemExit("schema_version must be 1")

expected_priority_order = [
    "agent_autonomy",
    "product_usefulness",
    "self_improvement_later",
]
if fixture["priority_order"] != expected_priority_order:
    raise SystemExit("priority_order does not match tranche-one contract")

allowed_interventions = fixture["suite_policy"]["allowed_operator_interventions"]
if allowed_interventions != ["provider_auth", "major_direction_choice"]:
    raise SystemExit("allowed_operator_interventions must be exactly provider_auth and major_direction_choice")

disallowed = set(fixture["suite_policy"]["disallowed_operator_interventions"])
required_disallowed = {
    "routine_next_step_guidance",
    "manual_subtask_decomposition",
    "manual_content_authoring",
    "manual_code_authoring",
}
if not required_disallowed.issubset(disallowed):
    raise SystemExit("disallowed_operator_interventions is missing required routine-steering guardrails")

tasks = fixture["tasks"]
if not (3 <= len(tasks) <= 5):
    raise SystemExit("tasks must contain between 3 and 5 benchmark entries")

if fixture["success_bar"]["benchmark_task_count"] != len(tasks):
    raise SystemExit("success_bar.benchmark_task_count must equal the number of tasks")

expected_categories = {
    "repo_build",
    "greenfield_build",
    "research_design",
    "data_to_deliverable",
}
actual_categories = {task["category"] for task in tasks}
if actual_categories != expected_categories:
    raise SystemExit(
        "task categories must match the tranche-one suite: "
        + ", ".join(sorted(expected_categories))
    )

task_required = set(schema["properties"]["tasks"]["items"]["required"])
for task in tasks:
    missing_task_fields = sorted(task_required - set(task.keys()))
    if missing_task_fields:
        raise SystemExit(
            f"task '{task.get('id', '<unknown>')}' is missing fields: {', '.join(missing_task_fields)}"
        )
    if not task["required_deliverables"]:
        raise SystemExit(f"task '{task['id']}' must declare required_deliverables")
    if not task["pass_requirements"]:
        raise SystemExit(f"task '{task['id']}' must declare pass_requirements")
    if not set(task["allowed_checkpoints"]).issubset(set(allowed_interventions)):
        raise SystemExit(f"task '{task['id']}' declares unsupported allowed_checkpoints")

if fixture["success_bar"]["required_terminal_state"] != "completed":
    raise SystemExit("success_bar.required_terminal_state must be 'completed'")

print("m334 tranche-one autonomy benchmark contract validated")
PY

log_info "validated ${FIXTURE_PATH}"

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

VALIDATOR_SCRIPT="${REPO_ROOT}/scripts/dev/validate-m334-tranche-one-autonomy-benchmark.sh"
FIXTURE_PATH="${REPO_ROOT}/tasks/fixtures/m334/tranche-one-autonomy-benchmark.json"
SCHEMA_PATH="${REPO_ROOT}/tasks/schemas/m334-tranche-one-autonomy-benchmark.schema.json"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local description="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${description}): expected output to contain '${needle}'" >&2
    printf '--- output ---\n%s\n-------------\n' "${haystack}" >&2
    exit 1
  fi
}

assert_equals() {
  local expected="$1"
  local actual="$2"
  local description="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${description}): expected '${expected}', got '${actual}'" >&2
    exit 1
  fi
}

if [[ ! -x "${VALIDATOR_SCRIPT}" ]]; then
  echo "error: validator script missing or not executable: ${VALIDATOR_SCRIPT}" >&2
  exit 1
fi

schema_check_output="$(python3 - "${SCHEMA_PATH}" <<'PY'
import json
import sys
from pathlib import Path

schema = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))

required_top_level = set(schema["required"])
expected_top_level = {
    "schema_version",
    "benchmark_id",
    "version",
    "origin",
    "priority_order",
    "suite_policy",
    "success_bar",
    "tasks",
}
if required_top_level != expected_top_level:
    raise SystemExit(f"unexpected top-level required fields: {sorted(required_top_level)}")

task_required = set(schema["properties"]["tasks"]["items"]["required"])
expected_task_required = {
    "id",
    "category",
    "goal",
    "required_deliverables",
    "allowed_checkpoints",
    "pass_requirements",
}
if task_required != expected_task_required:
    raise SystemExit(f"unexpected task required fields: {sorted(task_required)}")

print("schema contract ok")
PY
)"
assert_equals "schema contract ok" "${schema_check_output}" "schema required fields contract"

validator_success_output="$("${VALIDATOR_SCRIPT}")"
assert_contains "${validator_success_output}" "m334 tranche-one autonomy benchmark contract validated" "validator success banner"
assert_contains "${validator_success_output}" "validated ${FIXTURE_PATH}" "validator success path"

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

missing_top_level_fixture="${tmpdir}/missing-top-level.json"
python3 - "${FIXTURE_PATH}" "${missing_top_level_fixture}" <<'PY'
import json
import sys
from pathlib import Path

fixture = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
fixture.pop("success_bar", None)
Path(sys.argv[2]).write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")
PY
set +e
missing_top_level_output="$("${VALIDATOR_SCRIPT}" --fixture "${missing_top_level_fixture}" 2>&1)"
missing_top_level_rc=$?
set -e
if [[ "${missing_top_level_rc}" -eq 0 ]]; then
  echo "assertion failed (missing top-level field rejection): validator unexpectedly succeeded" >&2
  exit 1
fi
assert_contains "${missing_top_level_output}" "missing top-level fields: success_bar" "missing top-level field message"

invalid_category_fixture="${tmpdir}/invalid-category.json"
python3 - "${FIXTURE_PATH}" "${invalid_category_fixture}" <<'PY'
import json
import sys
from pathlib import Path

fixture = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
fixture["tasks"][0]["category"] = "unsupported_category"
Path(sys.argv[2]).write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")
PY
set +e
invalid_category_output="$("${VALIDATOR_SCRIPT}" --fixture "${invalid_category_fixture}" 2>&1)"
invalid_category_rc=$?
set -e
if [[ "${invalid_category_rc}" -eq 0 ]]; then
  echo "assertion failed (invalid category rejection): validator unexpectedly succeeded" >&2
  exit 1
fi
assert_contains "${invalid_category_output}" "task categories must match the tranche-one suite" "invalid category message"

invalid_checkpoint_fixture="${tmpdir}/invalid-checkpoint.json"
python3 - "${FIXTURE_PATH}" "${invalid_checkpoint_fixture}" <<'PY'
import json
import sys
from pathlib import Path

fixture = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
fixture["tasks"][0]["allowed_checkpoints"] = ["routine_next_step_guidance"]
Path(sys.argv[2]).write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")
PY
set +e
invalid_checkpoint_output="$("${VALIDATOR_SCRIPT}" --fixture "${invalid_checkpoint_fixture}" 2>&1)"
invalid_checkpoint_rc=$?
set -e
if [[ "${invalid_checkpoint_rc}" -eq 0 ]]; then
  echo "assertion failed (invalid checkpoint rejection): validator unexpectedly succeeded" >&2
  exit 1
fi
assert_contains "${invalid_checkpoint_output}" "declares unsupported allowed_checkpoints" "invalid checkpoint message"

echo "m334 tranche-one autonomy benchmark contract tests passed"

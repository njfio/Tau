#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DRIFT_SCRIPT="${SCRIPT_DIR}/dependency-drift-check.sh"

if ! command -v jq >/dev/null 2>&1; then
  echo "error: jq is required for test-dependency-drift-check.sh" >&2
  exit 1
fi

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}' got '${actual}'" >&2
    exit 1
  fi
}

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

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

matrix_with_drift="${tmp_dir}/matrix-with-drift.json"
matrix_clean="${tmp_dir}/matrix-clean.json"
fixture_parents="${tmp_dir}/fixture-parents.json"
dry_run_report="${tmp_dir}/dry-run-report.json"
summary_file="${tmp_dir}/summary.md"

cat >"${matrix_with_drift}" <<'EOF'
{
  "schema_version": 1,
  "repository": "fixture/repository",
  "milestone": { "number": 21 },
  "issues": [
    {
      "number": 200,
      "title": "Epic parent",
      "state": "open",
      "labels": ["epic", "roadmap", "testing-matrix"],
      "type": "epic",
      "parent_issue_url": "https://api.github.com/repos/njfio/Tau/issues/1678"
    },
    {
      "number": 300,
      "title": "Story missing testing-matrix label",
      "state": "open",
      "labels": ["story", "roadmap"],
      "type": "story",
      "parent_issue_url": "https://api.github.com/repos/njfio/Tau/issues/200"
    },
    {
      "number": 301,
      "title": "Task missing parent link",
      "state": "open",
      "labels": ["task", "roadmap", "testing-matrix"],
      "type": "task",
      "parent_issue_url": null
    },
    {
      "number": 302,
      "title": "Task with incompatible parent",
      "state": "open",
      "labels": ["task", "roadmap", "testing-matrix"],
      "type": "task",
      "parent_issue_url": "https://api.github.com/repos/njfio/Tau/issues/999"
    }
  ]
}
EOF

cat >"${matrix_clean}" <<'EOF'
{
  "schema_version": 1,
  "repository": "fixture/repository",
  "milestone": { "number": 21 },
  "issues": [
    {
      "number": 200,
      "title": "Epic parent",
      "state": "open",
      "labels": ["epic", "roadmap", "testing-matrix"],
      "type": "epic",
      "parent_issue_url": "https://api.github.com/repos/njfio/Tau/issues/1678"
    },
    {
      "number": 300,
      "title": "Story child",
      "state": "open",
      "labels": ["story", "roadmap", "testing-matrix"],
      "type": "story",
      "parent_issue_url": "https://api.github.com/repos/njfio/Tau/issues/200"
    },
    {
      "number": 301,
      "title": "Task child",
      "state": "open",
      "labels": ["task", "roadmap", "testing-matrix"],
      "type": "task",
      "parent_issue_url": "https://api.github.com/repos/njfio/Tau/issues/300"
    }
  ]
}
EOF

cat >"${fixture_parents}" <<'EOF'
{
  "issues": [
    {
      "number": 1678,
      "title": "Milestone root",
      "state": "open",
      "labels": ["epic", "roadmap", "testing-matrix"],
      "milestone_number": 21
    },
    {
      "number": 999,
      "title": "Closed incompatible parent",
      "state": "closed",
      "labels": ["task", "roadmap", "testing-matrix"],
      "milestone_number": 22
    }
  ]
}
EOF

bash -n "${DRIFT_SCRIPT}"

dry_output="$(
  "${DRIFT_SCRIPT}" \
    --matrix-json "${matrix_with_drift}" \
    --policy-json "${SCRIPT_DIR}/../../tasks/policies/issue-hierarchy-drift-rules.json" \
    --fixture-parent-issues-json "${fixture_parents}" \
    --mode dry-run \
    --output-json "${dry_run_report}" \
    --summary-file "${summary_file}" 2>&1
)"
assert_contains "${dry_output}" "DRY-RUN-WOULD-FAIL orphan.missing_parent_link" "functional dry-run missing parent marker"
assert_contains "${dry_output}" "DRY-RUN would fail strict mode due to 2 error finding(s)." "functional dry-run summary marker"
assert_equals "2" "$(jq -r '.summary.errors' "${dry_run_report}")" "functional dry-run error count"
assert_equals "3" "$(jq -r '.summary.warnings' "${dry_run_report}")" "functional dry-run warning count"
assert_equals "true" "$(jq -r '.summary.would_fail' "${dry_run_report}")" "functional dry-run would_fail"
assert_contains "$(cat "${summary_file}")" "Mode: dry-run" "functional summary mode marker"

set +e
strict_output="$(
  "${DRIFT_SCRIPT}" \
    --matrix-json "${matrix_with_drift}" \
    --policy-json "${SCRIPT_DIR}/../../tasks/policies/issue-hierarchy-drift-rules.json" \
    --fixture-parent-issues-json "${fixture_parents}" \
    --mode strict 2>&1
)"
strict_code=$?
set -e
assert_equals "1" "${strict_code}" "regression strict fail exit"
assert_contains "${strict_output}" "ERROR orphan.missing_parent_link" "regression strict error marker"

clean_output="$(
  "${DRIFT_SCRIPT}" \
    --matrix-json "${matrix_clean}" \
    --policy-json "${SCRIPT_DIR}/../../tasks/policies/issue-hierarchy-drift-rules.json" \
    --fixture-parent-issues-json "${fixture_parents}" \
    --mode strict 2>&1
)"
assert_contains "${clean_output}" "no findings" "functional strict clean marker"

set +e
skip_fetch_output="$(
  "${DRIFT_SCRIPT}" \
    --matrix-json "${matrix_clean}" \
    --policy-json "${SCRIPT_DIR}/../../tasks/policies/issue-hierarchy-drift-rules.json" \
    --skip-parent-fetch \
    --mode strict 2>&1
)"
skip_fetch_code=$?
set -e
assert_equals "1" "${skip_fetch_code}" "regression strict skip-fetch fail exit"
assert_contains "${skip_fetch_output}" "orphan.parent_issue_not_found" "regression skip-fetch condition id"

set +e
invalid_mode_output="$("${DRIFT_SCRIPT}" --mode invalid 2>&1)"
invalid_mode_code=$?
set -e
assert_equals "1" "${invalid_mode_code}" "regression invalid mode exit"
assert_contains "${invalid_mode_output}" "--mode must be one of: strict, dry-run" "regression invalid mode message"

echo "dependency-drift-check tests passed"

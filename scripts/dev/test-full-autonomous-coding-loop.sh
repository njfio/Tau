#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
FIXTURE_PATH="${REPO_ROOT}/tasks/fixtures/m334/tranche-one-autonomy-benchmark.json"
TASK_ID="repo_spec_to_pr_feature_delivery"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local description="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${description}): expected output to contain '${needle}'" >&2
    printf '%s\n' "${haystack}" >&2
    exit 1
  fi
}

run_case() {
  local mode="$1"
  local tmpdir="$2"
  local report_path="${tmpdir}/${mode}-report.json"
  local state_root="${tmpdir}/${mode}-state"
  local repo_root="${tmpdir}/${mode}-repo"

  cargo run -p tau-coding-agent --quiet --bin tau_live_coding_loop_harness -- \
    --fixture "${FIXTURE_PATH}" \
    --task-id "${TASK_ID}" \
    --mode "${mode}" \
    --state-root "${state_root}" \
    --repo-root "${repo_root}" \
    --output "${report_path}" \
    --run-id "m334-live-${mode}" \
    --started-unix-ms 1800000100000 >/dev/null

  python3 - "${mode}" "${report_path}" <<'PY'
import json
import sys
from pathlib import Path

mode = sys.argv[1]
report_path = Path(sys.argv[2])
report = json.loads(report_path.read_text(encoding="utf-8"))

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["schema_version"] == 1, "schema version mismatch")
require(report["benchmark_id"] == "m334-tranche-one-autonomy", "benchmark id mismatch")
require(report["task_id"] == "repo_spec_to_pr_feature_delivery", "task id mismatch")
require(report["mode"] == mode, "mode mismatch")
require(report["passed"] is True, f"case did not pass: {report.get('failure_reasons')}")
require(report["operator_interventions_used"] == [], "unexpected operator intervention")
require(report["no_routine_human_steering_used"] is True, "routine steering should be false")
require(report["mission_id"], "mission id missing")
require(report["repo_path"], "repo path missing")

if mode in {"success", "resume"}:
    require(report["phase"] == "pr_ready", f"expected pr_ready, got {report['phase']}")
    require(report["branch"].startswith("codex/issue-3654-live-loop"), "branch missing codex prefix")
    require(len(report["commit_hash"]) == 40, "commit hash missing")
    require("status.txt" in report["changed_files"], "status.txt change missing")
    statuses = {item["status"] for item in report["verifier_transcript"]}
    require({"failed", "succeeded"} <= statuses, f"expected red+green verifier, got {statuses}")
    require(report["pr_ready"]["status"] == "manual_ready", "manual pr-ready state missing")
    require(report["pr_ready"]["manual_gh_pr_create_command"].startswith("gh pr create --draft"), "manual gh command missing")
    require(Path(report["pr_ready"]["body_path"]).is_file(), "PR body artifact missing")

if mode == "resume":
    resume = report["resume"]
    require(resume["crash_injected"] is True, "crash injection flag missing")
    require(resume["resume_action"] == "apply_edit", f"unexpected resume action {resume}")
    require(resume["restored_branch"].startswith("codex/issue-3654-live-loop"), "restored branch missing")

if mode == "blocked":
    require(report["phase"] == "blocked", f"expected blocked, got {report['phase']}")
    require(report["blocked_reason"] == "verifier_command_failed_to_start", "blocked reason mismatch")
    require(report["commit_hash"] is None, "blocked case should not commit")
    require(report["pr_ready"] is None, "blocked case should not prepare PR")
    statuses = {item["status"] for item in report["verifier_transcript"]}
    require("failed" in statuses, "blocked verifier failure missing")

print(f"{mode} report ok")
PY
}

if [[ ! -f "${FIXTURE_PATH}" ]]; then
  echo "missing benchmark fixture: ${FIXTURE_PATH}" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

success_output="$(run_case success "${tmpdir}")"
assert_contains "${success_output}" "success report ok" "success case"

resume_output="$(run_case resume "${tmpdir}")"
assert_contains "${resume_output}" "resume report ok" "resume case"

blocked_output="$(run_case blocked "${tmpdir}")"
assert_contains "${blocked_output}" "blocked report ok" "blocked case"

echo "full autonomous coding loop benchmark tests passed"

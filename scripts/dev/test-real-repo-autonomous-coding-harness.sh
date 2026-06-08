#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
target_root="${TAU_REAL_REPO_HARNESS_TMP:-/tmp/tau-real-repo-autonomous-coding-harness}"
worktree="${target_root}/worktree"
state_root="${target_root}/state"
report="${target_root}/real-repo-report.json"
fixture_dir="tasks/tau-real-repo-harness-fixture"
run_id="m334-real-repo-coding-loop-$$"
base_branch="tau-real-repo-harness-base-$$"
mission_branch="codex/issue-3792-real-repo-harness${run_id}-real_repo"

rm -rf "${target_root}"
mkdir -p "${target_root}"

cleanup() {
  git -C "${repo_root}" worktree remove --force "${worktree}" >/dev/null 2>&1 || true
  git -C "${repo_root}" branch -D "${mission_branch}" >/dev/null 2>&1 || true
  git -C "${repo_root}" branch -D "${base_branch}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

git -C "${repo_root}" worktree add --detach "${worktree}" HEAD >/dev/null
git -C "${worktree}" config user.email "tau-real-repo-harness@example.test"
git -C "${worktree}" config user.name "Tau Real Repo Harness"
git -C "${worktree}" switch -c "${base_branch}" >/dev/null

mkdir -p "${worktree}/${fixture_dir}"
printf 'fail\n' >"${worktree}/${fixture_dir}/status.txt"
printf 'missing\n' >"${worktree}/${fixture_dir}/notes.txt"
git -C "${worktree}" add "${fixture_dir}/status.txt" "${fixture_dir}/notes.txt"
git -C "${worktree}" commit -m "seed real repo harness fixture" >/dev/null

mock_provider_response='{"edits":[{"relative_path":"tasks/tau-real-repo-harness-fixture/status.txt","contents":"pass\n","reason_code":"provider_real_repo_status"},{"relative_path":"tasks/tau-real-repo-harness-fixture/notes.txt","contents":"helper\n","reason_code":"provider_real_repo_notes"}]}'

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust_pi-3792-real-repo-target}" \
  cargo run -p tau-coding-agent --quiet --bin tau_live_coding_loop_harness -- \
  --mode real-repo \
  --repo-root "${worktree}" \
  --state-root "${state_root}" \
  --output "${report}" \
  --run-id "${run_id}" \
  --started-unix-ms 1800000090000 \
  --verifier-command "grep -q pass ${fixture_dir}/status.txt" \
  --verifier-command "grep -q helper ${fixture_dir}/notes.txt" \
  --mock-provider-response "${mock_provider_response}" >/dev/null

python3 - "${report}" "${fixture_dir}" <<'PY'
import json
import sys

report_path = sys.argv[1]
fixture_dir = sys.argv[2]
with open(report_path, "r", encoding="utf-8") as handle:
    report = json.load(handle)

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(report["passed"] is True, "real-repo report did not pass")
require(report["mode"] == "real_repo", "mode mismatch")
require(report["phase"] == "pr_ready", "mission did not reach pr_ready")
require(report["pr_ready"]["status"] == "manual_ready", "PR status should be manual_ready")
require(report["commit_hash"] and len(report["commit_hash"]) == 40, "missing commit hash")
require(
    report["changed_files"] == [
        f"{fixture_dir}/notes.txt",
        f"{fixture_dir}/status.txt",
    ],
    f"changed files mismatch: {report['changed_files']}",
)
provider = report["provider"]
require(provider["parse_status"] == "parsed", "provider parse status mismatch")
paths = set(provider["edit_relative_path"].split(","))
require(
    paths == {
        f"{fixture_dir}/status.txt",
        f"{fixture_dir}/notes.txt",
    },
    f"provider paths mismatch: {paths}",
)
statuses = [item["status"] for item in report["verifier_transcript"]]
require("failed" in statuses, "missing red verifier evidence")
require("succeeded" in statuses, "missing green verifier evidence")
require(report["no_routine_human_steering_used"] is True, "unexpected human steering")
PY

echo "real-repo autonomous coding harness passed: ${report}"

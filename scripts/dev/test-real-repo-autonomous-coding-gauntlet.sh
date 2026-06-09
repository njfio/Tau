#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust_pi-3805-target}"
WORK_ROOT="${TAU_REAL_REPO_GAUNTLET_TMP:-/tmp/tau-real-repo-autonomous-coding-gauntlet}"
WORKTREE="${WORK_ROOT}/worktree"
STATE_ROOT="${WORK_ROOT}/state"
JOBS_ROOT="${WORK_ROOT}/jobs"
RESULTS_JSONL="${WORK_ROOT}/results.jsonl"
REPORT_JSON="${WORK_ROOT}/real-repo-gauntlet-report.json"
FIXTURE_DIR="tasks/tau-real-repo-gauntlet"
RUN_ID="real-repo-gauntlet-$$"
BASE_BRANCH="tau-real-repo-gauntlet-base-$$"

export CARGO_TARGET_DIR="${TARGET_DIR}"

run_cli() {
  cargo run -q -p tau-coding-agent --bin tau_autonomous_coding_job -- "$@"
}

record_case() {
  local case_name="$1"
  local status="$2"
  local detail="$3"
  python3 - "${RESULTS_JSONL}" "${case_name}" "${status}" "${detail}" <<'PY'
import json
import sys

path, case_name, status, detail = sys.argv[1:]
with open(path, "a", encoding="utf-8") as handle:
    handle.write(json.dumps({
        "case": case_name,
        "passed": status == "pass",
        "status": status,
        "detail": detail,
    }, sort_keys=True) + "\n")
PY
}

write_provider_payload() {
  local case_name="$1"
  local payload="$2"
  local provider="${WORK_ROOT}/provider-${case_name}.sh"
  {
    printf '#!/usr/bin/env bash\n'
    printf 'set -euo pipefail\n'
    printf "cat <<'PAYLOAD'\n"
    printf '%s\n' "${payload}"
    printf 'PAYLOAD\n'
  } >"${provider}"
  chmod +x "${provider}"
  printf '%s' "${provider}"
}

reset_worktree() {
  git -C "${WORKTREE}" switch -f "${BASE_BRANCH}" >/dev/null
  git -C "${WORKTREE}" reset --hard "${BASE_BRANCH}" >/dev/null
  git -C "${WORKTREE}" clean -fd >/dev/null
}

assert_issue_to_merge_pr_ready() {
  local case_name="$1"
  local output_json="$2"
  python3 - "${output_json}" "${case_name}" <<'PY'
import json
import sys

path, case_name = sys.argv[1:]
with open(path, "r", encoding="utf-8") as handle:
    payload = json.load(handle)
assert payload["status"] == "pr_ready", (case_name, payload)
assert payload["run"]["status"]["status"] == "pr_ready", (case_name, payload)
assert payload["run"]["status"]["operator_state"] == "complete", (case_name, payload)
assert payload["run"]["status"]["pr_state"] == "manual_ready", (case_name, payload)
PY
}

run_issue_case() {
  local case_name="$1"
  shift
  local output_json="${WORK_ROOT}/${case_name}.json"
  reset_worktree
  run_cli issue-to-merge \
    --state-dir "${STATE_ROOT}/${case_name}" \
    --jobs-state-dir "${JOBS_ROOT}/${case_name}" \
    --repo-path "${WORKTREE}" \
    --intake-id "issue-3805-${case_name}" \
    --mission-id "mission-3805-${case_name}" \
    --session-key "real-repo-gauntlet" \
    --issue-url "https://github.com/njfio/Tau/issues/3805" \
    --issue-title "Real repo gauntlet ${case_name}" \
    --issue-body "Run ${case_name} through the Tau autonomous coding gauntlet." \
    --goal "Make ${case_name} verifier pass in the Tau worktree" \
    --base-branch "${BASE_BRANCH}" \
    --branch-prefix "codex/tmp-real-repo-gauntlet-${RUN_ID}-${case_name}-" \
    --commit-message "Pass real repo gauntlet ${case_name}" \
    --pr-mode pr-ready \
    "$@" \
    >"${output_json}"
  assert_issue_to_merge_pr_ready "${case_name}" "${output_json}"
  record_case "${case_name}" "pass" "${output_json}"
}

cleanup() {
  git -C "${REPO_ROOT}" worktree remove --force "${WORKTREE}" >/dev/null 2>&1 || true
  git -C "${REPO_ROOT}" branch -D "${BASE_BRANCH}" >/dev/null 2>&1 || true
  while IFS= read -r branch_name; do
    [[ -z "${branch_name}" ]] && continue
    git -C "${REPO_ROOT}" branch -D "${branch_name}" >/dev/null 2>&1 || true
  done < <(git -C "${REPO_ROOT}" branch --list "codex/tmp-real-repo-gauntlet-${RUN_ID}-*")
}
trap cleanup EXIT

rm -rf "${WORK_ROOT}"
mkdir -p "${WORK_ROOT}"
cd "${REPO_ROOT}"

git -C "${REPO_ROOT}" worktree add --detach "${WORKTREE}" HEAD >/dev/null
git -C "${WORKTREE}" config user.email "tau-real-repo-gauntlet@example.test"
git -C "${WORKTREE}" config user.name "Tau Real Repo Gauntlet"
git -C "${WORKTREE}" switch -c "${BASE_BRANCH}" >/dev/null

mkdir -p "${WORKTREE}/${FIXTURE_DIR}"
printf 'draft\n' >"${WORKTREE}/${FIXTURE_DIR}/docs-only.md"
printf 'answer=0\n' >"${WORKTREE}/${FIXTURE_DIR}/single-file.txt"
printf 'left=bad\n' >"${WORKTREE}/${FIXTURE_DIR}/multi-left.txt"
printf 'right=bad\n' >"${WORKTREE}/${FIXTURE_DIR}/multi-right.txt"
printf 'broken\n' >"${WORKTREE}/${FIXTURE_DIR}/test-data.txt"
cat >"${WORKTREE}/${FIXTURE_DIR}/test-repair.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
grep -q '^test=pass$' tasks/tau-real-repo-gauntlet/test-data.txt
SH
chmod +x "${WORKTREE}/${FIXTURE_DIR}/test-repair.sh"
cat >"${WORKTREE}/${FIXTURE_DIR}/cli-fixture.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  --old-flag) printf 'old\n' ;;
  *) printf 'missing\n'; exit 2 ;;
esac
SH
chmod +x "${WORKTREE}/${FIXTURE_DIR}/cli-fixture.sh"
cat >"${WORKTREE}/${FIXTURE_DIR}/flaky-verifier.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
counter="${TAU_FLAKY_COUNTER:?TAU_FLAKY_COUNTER required}"
count=0
if [[ -f "${counter}" ]]; then
  count="$(cat "${counter}")"
fi
count=$((count + 1))
printf '%s\n' "${count}" >"${counter}"
if (( count < 2 )); then
  echo "intentional first verifier failure" >&2
  exit 1
fi
grep -q '^flaky=pass$' tasks/tau-real-repo-gauntlet/flaky.txt
SH
chmod +x "${WORKTREE}/${FIXTURE_DIR}/flaky-verifier.sh"
printf 'flaky=fail\n' >"${WORKTREE}/${FIXTURE_DIR}/flaky.txt"

git -C "${WORKTREE}" add "${FIXTURE_DIR}"
git -C "${WORKTREE}" commit -m "seed real repo gauntlet fixtures" >/dev/null

DOCS_PROVIDER="$(write_provider_payload "docs-only" '{"edits":[{"relative_path":"tasks/tau-real-repo-gauntlet/docs-only.md","contents":"docs=pass\n","reason_code":"real_repo_docs_only"}]}')"
run_issue_case "docs-only" \
  --verifier-command "grep -q ^docs=pass$ ${FIXTURE_DIR}/docs-only.md" \
  --provider-repair-command "${DOCS_PROVIDER}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model openrouter/docs-only

SINGLE_PROVIDER="$(write_provider_payload "single-file-bug" '{"edits":[{"relative_path":"tasks/tau-real-repo-gauntlet/single-file.txt","contents":"answer=42\n","reason_code":"real_repo_single_file"}]}')"
run_issue_case "single-file-bug" \
  --verifier-command "grep -q ^answer=42$ ${FIXTURE_DIR}/single-file.txt" \
  --provider-repair-command "${SINGLE_PROVIDER}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model openrouter/single-file

MULTI_PROVIDER="$(write_provider_payload "multi-file-bug" '{"edits":[{"relative_path":"tasks/tau-real-repo-gauntlet/multi-left.txt","contents":"left=pass\n","reason_code":"real_repo_multi_left"},{"relative_path":"tasks/tau-real-repo-gauntlet/multi-right.txt","contents":"right=pass\n","reason_code":"real_repo_multi_right"}]}')"
run_issue_case "multi-file-bug" \
  --verifier-command "grep -q ^left=pass$ ${FIXTURE_DIR}/multi-left.txt" \
  --verifier-command "grep -q ^right=pass$ ${FIXTURE_DIR}/multi-right.txt" \
  --provider-repair-command "${MULTI_PROVIDER}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model openrouter/multi-file

PROVIDER_FIX="${WORK_ROOT}/provider-fix.sh"
cat >"${PROVIDER_FIX}" <<JSON
#!/usr/bin/env bash
set -euo pipefail
cat <<'PAYLOAD'
{"edits":[{"relative_path":"${FIXTURE_DIR}/test-data.txt","contents":"test=pass\\n","reason_code":"real_repo_test_repair"}]}
PAYLOAD
JSON
chmod +x "${PROVIDER_FIX}"
run_issue_case "failing-test-repair" \
  --verifier-command "bash ${FIXTURE_DIR}/test-repair.sh" \
  --provider-repair-command "${PROVIDER_FIX}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model openrouter/test-repair

CLI_PROVIDER="$(write_provider_payload "cli-flag-change" '{"edits":[{"relative_path":"tasks/tau-real-repo-gauntlet/cli-fixture.sh","contents":"#!/usr/bin/env bash\nset -euo pipefail\ncase \"${1:-}\" in\n  --old-flag) printf \"old\\n\" ;;\n  --new-flag) printf \"enabled\\n\" ;;\n  *) printf \"missing\\n\"; exit 2 ;;\nesac\n","reason_code":"real_repo_cli_flag"}]}')"
run_issue_case "cli-flag-change" \
  --verifier-command "bash ${FIXTURE_DIR}/cli-fixture.sh --new-flag" \
  --provider-repair-command "${CLI_PROVIDER}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model openrouter/cli-flag

FLAKY_PROVIDER="$(write_provider_payload "flaky-verifier" '{"edits":[{"relative_path":"tasks/tau-real-repo-gauntlet/flaky.txt","contents":"flaky=pass\n","reason_code":"real_repo_flaky"}]}')"
run_issue_case "flaky-verifier" \
  --verifier-command "env TAU_FLAKY_COUNTER=${WORK_ROOT}/flaky-counter.txt bash ${FIXTURE_DIR}/flaky-verifier.sh" \
  --provider-repair-command "${FLAKY_PROVIDER}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model openrouter/flaky

MALFORMED_PROVIDER="${WORK_ROOT}/malformed-provider.sh"
cat >"${MALFORMED_PROVIDER}" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf '{"diff":"--- a/tasks/tau-real-repo-gauntlet/missing.txt\n+++ b/tasks/tau-real-repo-gauntlet/missing.txt\n@@ -1 +1 @@\n-old\n+new\n"}\n'
SH
chmod +x "${MALFORMED_PROVIDER}"
reset_worktree
MALFORMED_JSON="${WORK_ROOT}/malformed-provider-patch.json"
run_cli issue-to-merge \
  --state-dir "${STATE_ROOT}/malformed-provider-patch" \
  --jobs-state-dir "${JOBS_ROOT}/malformed-provider-patch" \
  --repo-path "${WORKTREE}" \
  --intake-id "issue-3805-malformed-provider-patch" \
  --mission-id "mission-3805-malformed-provider-patch" \
  --session-key "real-repo-gauntlet" \
  --issue-url "https://github.com/njfio/Tau/issues/3805" \
  --issue-title "Real repo gauntlet malformed provider patch" \
  --issue-body "Malformed provider patch should fail closed." \
  --goal "Reject malformed provider patch" \
  --base-branch "${BASE_BRANCH}" \
  --branch-prefix "codex/tmp-real-repo-gauntlet-${RUN_ID}-malformed-" \
  --verifier-command "grep -q ^malformed=pass$ ${FIXTURE_DIR}/single-file.txt" \
  --provider-repair-command "${MALFORMED_PROVIDER}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model openrouter/malformed \
  --commit-message "Should not commit malformed provider patch" \
  --pr-mode pr-ready \
  >"${MALFORMED_JSON}"
python3 - "${MALFORMED_JSON}" <<'PY'
import json
import sys
with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)
assert payload["status"] == "blocked", payload
assert payload["reason_code"] == "autonomous_coding_job_provider_repair_exhausted", payload
status = payload["run"]["status"]
assert status["provider_repair_status"] == "rejected", status
PY
record_case "malformed-provider-patch" "pass" "${MALFORMED_JSON}"

reset_worktree
CRASH_SUBMIT_JSON="${WORK_ROOT}/crash-mid-run-submit.json"
run_cli submit \
  --state-dir "${STATE_ROOT}/crash-mid-run" \
  --jobs-state-dir "${JOBS_ROOT}/crash-mid-run" \
  --repo-path "${WORKTREE}" \
  --mission-id "mission-3805-crash-mid-run" \
  --session-key "real-repo-gauntlet" \
  --issue-url "https://github.com/njfio/Tau/issues/3805" \
  --goal "Classify a crashed mid-run job" \
  --base-branch "${BASE_BRANCH}" \
  --branch-prefix "codex/tmp-real-repo-gauntlet-${RUN_ID}-crash-" \
  --verifier-command "grep -q ^answer=42$ ${FIXTURE_DIR}/single-file.txt" \
  --edit "${FIXTURE_DIR}/single-file.txt=answer=42" \
  --commit-message "Crash case should not commit" \
  --pr-mode pr-ready \
  >"${CRASH_SUBMIT_JSON}"
CRASH_JOB_ID="$(python3 - "${CRASH_SUBMIT_JSON}" <<'PY'
import json
import sys
with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)
print(payload["record"]["job_id"])
PY
)"
python3 - "${STATE_ROOT}/crash-mid-run" "${CRASH_JOB_ID}" <<'PY'
import json
import os
import sys
state_root, job_id = sys.argv[1:]
path = os.path.join(state_root, "autonomous-coding-jobs", f"{job_id}.json")
with open(path, "r", encoding="utf-8") as handle:
    record = json.load(handle)
record["status"] = "running"
record["reason_code"] = "autonomous_coding_job_running"
record["lease_expires_unix_ms"] = 1
with open(path, "w", encoding="utf-8") as handle:
    json.dump(record, handle, indent=2)
PY
CRASH_STATUS_JSON="${WORK_ROOT}/crash-mid-run-status.json"
run_cli status \
  --state-dir "${STATE_ROOT}/crash-mid-run" \
  --jobs-state-dir "${JOBS_ROOT}/crash-mid-run" \
  --job-id "${CRASH_JOB_ID}" \
  >"${CRASH_STATUS_JSON}"
python3 - "${CRASH_STATUS_JSON}" <<'PY'
import json
import sys
with open(sys.argv[1], "r", encoding="utf-8") as handle:
    status = json.load(handle)
assert status["operator_state"] == "stale_lease", status
assert status["recoverable"] is True, status
assert "recover" in status["operator_next_command"], status
PY
run_cli mark-blocked \
  --state-dir "${STATE_ROOT}/crash-mid-run" \
  --jobs-state-dir "${JOBS_ROOT}/crash-mid-run" \
  --job-id "${CRASH_JOB_ID}" \
  --reason-code real_repo_gauntlet_crash_blocked \
  --detail "gauntlet marked simulated crash blocked" \
  >"${WORK_ROOT}/crash-mid-run-blocked.json"
record_case "crash-mid-run" "pass" "${CRASH_STATUS_JSON}"

python3 - "${RESULTS_JSONL}" "${REPORT_JSON}" <<'PY'
import json
import sys

results_path, report_path = sys.argv[1:]
with open(results_path, "r", encoding="utf-8") as handle:
    cases = [json.loads(line) for line in handle if line.strip()]
passed = sum(1 for item in cases if item["passed"])
report = {
    "suite": "real_repo_autonomous_coding_gauntlet",
    "passed": passed,
    "failed": len(cases) - passed,
    "cases": cases,
}
with open(report_path, "w", encoding="utf-8") as handle:
    json.dump(report, handle, indent=2, sort_keys=True)
assert report["failed"] == 0, report
assert len(cases) == 8, report
print(json.dumps({
    "suite": report["suite"],
    "passed": report["passed"],
    "failed": report["failed"],
    "report": report_path,
}, sort_keys=True))
PY

printf 'real_repo_autonomous_coding_gauntlet=pass\n'

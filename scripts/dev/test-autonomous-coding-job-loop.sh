#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust_pi-3794-target}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tau-autonomous-coding-job.XXXXXX")"
trap 'rm -rf "${WORK_DIR}"' EXIT

export CARGO_TARGET_DIR="${TARGET_DIR}"

run_cli() {
  cargo run -q -p tau-coding-agent --bin tau_autonomous_coding_job -- "$@"
}

init_fixture_repo() {
  local repo="$1"
  mkdir -p "${repo}/docs"
  git -C "${repo}" init -b master >/dev/null
  git -C "${repo}" config user.email tau@example.test
  git -C "${repo}" config user.name "Tau Test"
  printf 'fail\n' >"${repo}/status.txt"
  printf 'draft\n' >"${repo}/docs/notes.txt"
  git -C "${repo}" add .
  git -C "${repo}" commit -m "Initial fixture" >/dev/null
}

extract_json_field() {
  local path="$1"
  local expr="$2"
  python3 - "$path" "$expr" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

value = payload
for part in sys.argv[2].split("."):
    value = value[part]
print(value)
PY
}

assert_status_pr_ready() {
  local status_path="$1"
  python3 - "$status_path" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    status = json.load(handle)

assert status["status"] == "pr_ready", status
assert status["phase"] == "pr_ready", status
assert status["pr_state"] == "manual_ready", status
assert "gh pr create --draft" in status.get("pr_ready_command", ""), status
assert "status.txt" in status["changed_files"], status
assert "docs/notes.txt" in status["changed_files"], status
assert status["verifier_summary"].startswith("succeeded:"), status
PY
}

cd "${REPO_ROOT}"

DIRECT_REPO="${WORK_DIR}/direct-repo"
DIRECT_STATE="${WORK_DIR}/direct-state"
DIRECT_JOBS="${WORK_DIR}/direct-jobs"
mkdir -p "${DIRECT_REPO}"
init_fixture_repo "${DIRECT_REPO}"

run_cli submit \
  --state-dir "${DIRECT_STATE}" \
  --jobs-state-dir "${DIRECT_JOBS}" \
  --repo-path "${DIRECT_REPO}" \
  --mission-id direct-mission \
  --session-key script-direct \
  --issue-url https://github.com/njfio/Tau/issues/3794 \
  --goal "Make direct autonomous coding verifier pass" \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q proof docs/notes.txt" \
  --edit "status.txt=pass" \
  --edit "docs/notes.txt=proof" \
  --commit-message "Make direct autonomous verifier green" \
  >"${WORK_DIR}/direct-submit.json"

DIRECT_JOB_ID="$(extract_json_field "${WORK_DIR}/direct-submit.json" "record.job_id")"
run_cli run \
  --state-dir "${DIRECT_STATE}" \
  --jobs-state-dir "${DIRECT_JOBS}" \
  --job-id "${DIRECT_JOB_ID}" \
  >"${WORK_DIR}/direct-run.json"
run_cli status \
  --state-dir "${DIRECT_STATE}" \
  --jobs-state-dir "${DIRECT_JOBS}" \
  --job-id "${DIRECT_JOB_ID}" \
  >"${WORK_DIR}/direct-status.json"
assert_status_pr_ready "${WORK_DIR}/direct-status.json"

REPLAY_REPO="${WORK_DIR}/replay-repo"
REPLAY_STATE="${WORK_DIR}/replay-state"
REPLAY_JOBS="${WORK_DIR}/replay-jobs"
mkdir -p "${REPLAY_REPO}"
init_fixture_repo "${REPLAY_REPO}"

run_cli submit \
  --state-dir "${REPLAY_STATE}" \
  --jobs-state-dir "${REPLAY_JOBS}" \
  --repo-path "${REPLAY_REPO}" \
  --mission-id replay-mission \
  --session-key script-replay \
  --issue-url https://github.com/njfio/Tau/issues/3794 \
  --goal "Replay autonomous coding checkpoint to PR ready" \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q proof docs/notes.txt" \
  --commit-message "Prepare replay checkpoint" \
  >"${WORK_DIR}/replay-submit.json"

REPLAY_JOB_ID="$(extract_json_field "${WORK_DIR}/replay-submit.json" "record.job_id")"
run_cli run \
  --state-dir "${REPLAY_STATE}" \
  --jobs-state-dir "${REPLAY_JOBS}" \
  --job-id "${REPLAY_JOB_ID}" \
  >"${WORK_DIR}/replay-waiting.json"
python3 - "${WORK_DIR}/replay-waiting.json" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

status = payload["status"]
assert status["phase"] == "executing", status
assert status["status"] == "running", status
assert status["reason_code"] == "autonomous_coding_job_waiting_for_controlled_edit", status
assert "mission resume" in status["resume_command"], status
PY

run_cli replay \
  --state-dir "${REPLAY_STATE}" \
  --jobs-state-dir "${REPLAY_JOBS}" \
  --job-id "${REPLAY_JOB_ID}" \
  --edit "status.txt=pass" \
  --edit "docs/notes.txt=proof" \
  --commit-message "Replay autonomous verifier green" \
  >"${WORK_DIR}/replay-run.json"
run_cli status \
  --state-dir "${REPLAY_STATE}" \
  --jobs-state-dir "${REPLAY_JOBS}" \
  --job-id "${REPLAY_JOB_ID}" \
  >"${WORK_DIR}/replay-status.json"
assert_status_pr_ready "${WORK_DIR}/replay-status.json"

printf 'autonomous_coding_job_loop=pass\n'
printf 'direct_job_id=%s\n' "${DIRECT_JOB_ID}"
printf 'replay_job_id=%s\n' "${REPLAY_JOB_ID}"

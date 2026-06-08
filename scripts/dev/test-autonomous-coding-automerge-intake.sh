#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust_pi-3796-target}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tau-automerge-intake.XXXXXX")"
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

cd "${REPO_ROOT}"

STATE="${WORK_DIR}/state"
JOBS="${WORK_DIR}/jobs"
REPO="${WORK_DIR}/repo"
mkdir -p "${REPO}"
init_fixture_repo "${REPO}"

run_cli submit \
  --state-dir "${STATE}" \
  --jobs-state-dir "${JOBS}" \
  --repo-path "${REPO}" \
  --mission-id automerge-mission \
  --session-key script-automerge \
  --issue-url https://github.com/njfio/Tau/issues/3796 \
  --goal "Make guarded auto-merge fixture pass" \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q proof docs/notes.txt" \
  --edit "status.txt=pass" \
  --edit "docs/notes.txt=proof" \
  --commit-message "Make guarded automerge verifier green" \
  >"${WORK_DIR}/submit.json"

JOB_ID="$(extract_json_field "${WORK_DIR}/submit.json" "record.job_id")"
run_cli run \
  --state-dir "${STATE}" \
  --jobs-state-dir "${JOBS}" \
  --job-id "${JOB_ID}" \
  >"${WORK_DIR}/run.json"

python3 - "${STATE}/coding-missions/automerge-mission.json" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as handle:
    state = json.load(handle)

assert state["phase"] == "pr_ready", state["phase"]
state["pr_ready_bundle"]["pr_url"] = "https://github.com/njfio/Tau/pull/3796"

with open(path, "w", encoding="utf-8") as handle:
    json.dump(state, handle, indent=2)
    handle.write("\n")
PY

FAKE_GH="${WORK_DIR}/fake-gh.sh"
FAKE_GH_ARGV="${WORK_DIR}/fake-gh-argv.txt"
cat >"${FAKE_GH}" <<SH
#!/usr/bin/env bash
printf '%s\n' "\$@" > "${FAKE_GH_ARGV}"
printf 'auto-merge-enabled\n'
SH
chmod +x "${FAKE_GH}"

GH_TOKEN=test-token run_cli auto-merge \
  --state-dir "${STATE}" \
  --jobs-state-dir "${JOBS}" \
  --job-id "${JOB_ID}" \
  --allow-auto-merge \
  --merge-method squash \
  --delete-branch \
  --gh-binary "${FAKE_GH}" \
  >"${WORK_DIR}/auto-merge.json"

python3 - "${WORK_DIR}/auto-merge.json" "${FAKE_GH_ARGV}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

assert payload["evidence"]["status"] == "requested", payload
assert payload["evidence"]["reason_code"] == "auto_merge_requested", payload
assert payload["status"]["auto_merge_status"] == "requested", payload
assert payload["status"]["auto_merge_pr_url"] == "https://github.com/njfio/Tau/pull/3796", payload

with open(sys.argv[2], "r", encoding="utf-8") as handle:
    argv = handle.read().splitlines()

assert argv[:3] == ["pr", "merge", "https://github.com/njfio/Tau/pull/3796"], argv
assert "--auto" in argv, argv
assert "--squash" in argv, argv
assert "--delete-branch" in argv, argv
assert "--admin" not in argv, argv
PY

rm -f "${FAKE_GH_ARGV}"
GH_TOKEN=test-token run_cli auto-merge \
  --state-dir "${STATE}" \
  --jobs-state-dir "${JOBS}" \
  --job-id "${JOB_ID}" \
  --merge-method squash \
  --gh-binary "${FAKE_GH}" \
  >"${WORK_DIR}/auto-merge-blocked.json"

python3 - "${WORK_DIR}/auto-merge-blocked.json" "${FAKE_GH_ARGV}" <<'PY'
import json
import os
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

assert payload["evidence"]["status"] == "blocked", payload
assert payload["evidence"]["reason_code"] == "auto_merge_policy_disabled", payload
assert not os.path.exists(sys.argv[2]), "blocked auto-merge invoked gh"
PY

BEFORE_STATUS="$(cat "${REPO}/status.txt")"
run_cli intake-issue \
  --state-dir "${STATE}" \
  --jobs-state-dir "${JOBS}" \
  --intake-id issue-3796-intake \
  --issue-url https://github.com/njfio/Tau/issues/3796 \
  --issue-title "Arbitrary issue without authority" \
  --issue-body "Solve this without verifier or edit authority." \
  --repo-path "${REPO}" \
  --base-branch master \
  >"${WORK_DIR}/intake.json"

run_cli intake-status \
  --state-dir "${STATE}" \
  --jobs-state-dir "${JOBS}" \
  --intake-id issue-3796-intake \
  >"${WORK_DIR}/intake-status.json"

python3 - "${WORK_DIR}/intake-status.json" "${REPO}/status.txt" "${BEFORE_STATUS}" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    intake = json.load(handle)

assert intake["status"] == "blocked", intake
reasons = {item["reason_code"] for item in intake["required_authority"]}
assert "verifier_authority_required" in reasons, intake
assert "edit_authority_required" in reasons, intake

with open(sys.argv[2], "r", encoding="utf-8") as handle:
    after = handle.read().strip()
assert after == sys.argv[3], (after, sys.argv[3])
PY

printf 'autonomous_coding_automerge_intake=pass\n'
printf 'job_id=%s\n' "${JOB_ID}"

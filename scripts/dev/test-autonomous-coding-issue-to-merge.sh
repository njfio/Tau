#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust_pi-3801-target}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tau-issue-to-merge.XXXXXX")"
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

cd "${REPO_ROOT}"

STATE="${WORK_DIR}/state"
JOBS="${WORK_DIR}/jobs"
REPO="${WORK_DIR}/repo"
mkdir -p "${REPO}"
init_fixture_repo "${REPO}"

FAKE_GH="${WORK_DIR}/fake-gh.sh"
FAKE_GH_ARGV="${WORK_DIR}/fake-gh-argv.txt"
cat >"${FAKE_GH}" <<SH
#!/usr/bin/env bash
{
  printf 'CALL\\n'
  printf '%s\\n' "\$@"
} >> "${FAKE_GH_ARGV}"
if [ "\${1:-}" = "pr" ] && [ "\${2:-}" = "create" ]; then
  printf 'https://github.com/njfio/Tau/pull/3801\\n'
else
  printf 'auto-merge-enabled\\n'
fi
SH
chmod +x "${FAKE_GH}"

GH_TOKEN=test-token run_cli issue-to-merge \
  --state-dir "${STATE}" \
  --jobs-state-dir "${JOBS}" \
  --repo-path "${REPO}" \
  --intake-id issue-3801-intake \
  --mission-id issue-3801-mission \
  --session-key script-issue-to-merge \
  --issue-url https://github.com/njfio/Tau/issues/3801 \
  --issue-title "Hands-off issue to merge" \
  --issue-body "Run the authorized coding loop all the way to auto-merge request." \
  --goal "Make the issue-to-merge verifier pass" \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q proof docs/notes.txt" \
  --edit "status.txt=pass" \
  --edit "docs/notes.txt=proof" \
  --commit-message "Make issue-to-merge verifier green" \
  --pr-mode draft \
  --allow-auto-merge \
  --merge-method squash \
  --delete-branch \
  --gh-binary "${FAKE_GH}" \
  >"${WORK_DIR}/issue-to-merge.json"

python3 - "${WORK_DIR}/issue-to-merge.json" "${FAKE_GH_ARGV}" "${REPO}/status.txt" "${REPO}/docs/notes.txt" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

assert payload["status"] == "auto_merge_requested", payload
assert payload["reason_code"] == "auto_merge_requested", payload
assert payload["intake"] is None, payload
assert payload["submit"]["status"]["status"] == "queued", payload
assert payload["run"]["status"]["status"] == "pr_ready", payload
assert payload["run"]["status"]["pr_state"] == "draft_created", payload
assert payload["run"]["status"]["pr_url"] == "https://github.com/njfio/Tau/pull/3801", payload
assert payload["auto_merge"]["evidence"]["status"] == "requested", payload
assert payload["auto_merge"]["status"]["auto_merge_status"] == "requested", payload

with open(sys.argv[2], "r", encoding="utf-8") as handle:
    argv = handle.read().splitlines()

assert argv.count("CALL") == 2, argv
assert "create" in argv, argv
assert "merge" in argv, argv
assert "--draft" in argv, argv
assert "--auto" in argv, argv
assert "--squash" in argv, argv
assert "--delete-branch" in argv, argv
assert "--admin" not in argv, argv

with open(sys.argv[3], "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "pass"
with open(sys.argv[4], "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "proof"
PY

PROVIDER_STATE="${WORK_DIR}/provider-state"
PROVIDER_JOBS="${WORK_DIR}/provider-jobs"
PROVIDER_REPO="${WORK_DIR}/provider-repo"
mkdir -p "${PROVIDER_REPO}"
init_fixture_repo "${PROVIDER_REPO}"

FAKE_PROVIDER="${WORK_DIR}/fake-provider-repair.sh"
FAKE_PROVIDER_CONTEXT="${WORK_DIR}/fake-provider-context-path.txt"
cat >"${FAKE_PROVIDER}" <<SH
#!/usr/bin/env bash
set -euo pipefail
printf '%s\\n' "\${TAU_AUTONOMOUS_CODING_REPAIR_CONTEXT}" > "${FAKE_PROVIDER_CONTEXT}"
cat <<'JSON'
{
  "edits": [
    {
      "relative_path": "status.txt",
      "contents": "pass\\n",
      "reason_code": "script_provider_repair_status"
    },
    {
      "relative_path": "docs/notes.txt",
      "contents": "proof\\n",
      "reason_code": "script_provider_repair_notes"
    }
  ]
}
JSON
SH
chmod +x "${FAKE_PROVIDER}"

run_cli issue-to-merge \
  --state-dir "${PROVIDER_STATE}" \
  --jobs-state-dir "${PROVIDER_JOBS}" \
  --repo-path "${PROVIDER_REPO}" \
  --intake-id issue-3802-provider \
  --mission-id issue-3802-provider-mission \
  --session-key script-provider-repair \
  --issue-url https://github.com/njfio/Tau/issues/3802 \
  --issue-title "Provider repair issue to merge" \
  --issue-body "Use provider repair authority to make the verifier pass." \
  --goal "Make the provider-repair verifier pass" \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q proof docs/notes.txt" \
  --provider-repair-command "${FAKE_PROVIDER}" \
  --provider-repair-attempts 2 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model qwen/qwen3-235b-a22b \
  --commit-message "Make provider-repair verifier green" \
  --pr-mode pr-ready \
  >"${WORK_DIR}/issue-to-merge-provider.json"

python3 - "${WORK_DIR}/issue-to-merge-provider.json" "${PROVIDER_REPO}/status.txt" "${PROVIDER_REPO}/docs/notes.txt" "${FAKE_PROVIDER_CONTEXT}" <<'PY'
import json
import os
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

assert payload["status"] == "pr_ready", payload
assert payload["reason_code"] == "autonomous_coding_job_pr_ready", payload
assert payload["intake"] is None, payload
assert payload["submit"]["status"]["status"] == "queued", payload
run = payload["run"]["status"]
assert run["status"] == "pr_ready", run
assert run["provider_repair_status"] == "applied", run
assert run["provider_repair_reason_code"] == "provider_repair_edit_parsed", run
assert run["provider_repair_attempts"] == 1, run
assert run["provider_repair_provider"] == "fake-openrouter", run
assert run["provider_repair_model"] == "qwen/qwen3-235b-a22b", run
assert run["event_log_path"], run

with open(sys.argv[2], "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "pass"
with open(sys.argv[3], "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "proof"
with open(sys.argv[4], "r", encoding="utf-8") as handle:
    context_path = handle.read().strip()
assert context_path and os.path.exists(context_path), context_path
with open(context_path, "r", encoding="utf-8") as handle:
    context = json.load(handle)
assert len(context["failed_verifiers"]) == 2, context
PY

BLOCKED_STATE="${WORK_DIR}/blocked-state"
BLOCKED_JOBS="${WORK_DIR}/blocked-jobs"
BLOCKED_REPO="${WORK_DIR}/blocked-repo"
mkdir -p "${BLOCKED_REPO}"
init_fixture_repo "${BLOCKED_REPO}"
BEFORE_STATUS="$(cat "${BLOCKED_REPO}/status.txt")"

run_cli issue-to-merge \
  --state-dir "${BLOCKED_STATE}" \
  --jobs-state-dir "${BLOCKED_JOBS}" \
  --repo-path "${BLOCKED_REPO}" \
  --intake-id issue-3801-blocked \
  --mission-id issue-3801-blocked-mission \
  --issue-url https://github.com/njfio/Tau/issues/3801 \
  --issue-title "Missing edit authority" \
  --issue-body "No edit plan is allowed." \
  --verifier-command "grep -q pass status.txt" \
  --allow-auto-merge \
  >"${WORK_DIR}/issue-to-merge-blocked.json"

python3 - "${WORK_DIR}/issue-to-merge-blocked.json" "${BLOCKED_REPO}/status.txt" "${BEFORE_STATUS}" "${BLOCKED_STATE}" <<'PY'
import json
import os
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

assert payload["status"] == "blocked", payload
assert payload["reason_code"] == "issue_intake_authority_required", payload
assert payload["intake"]["status"] == "blocked", payload
assert payload["submit"] is None, payload
assert payload["run"] is None, payload
assert payload["auto_merge"] is None, payload

with open(sys.argv[2], "r", encoding="utf-8") as handle:
    assert handle.read().strip() == sys.argv[3], payload

jobs_dir = os.path.join(sys.argv[4], "autonomous-coding-jobs")
assert not os.path.isdir(jobs_dir) or not os.listdir(jobs_dir), os.listdir(jobs_dir)
intake_path = os.path.join(sys.argv[4], "issue-intake", "issue-3801-blocked.json")
assert os.path.exists(intake_path), intake_path
PY

printf 'autonomous_coding_issue_to_merge=pass\n'

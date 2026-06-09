#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust_pi-3802-target}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tau-autonomous-coding-gauntlet.XXXXXX")"
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

FULL_REPO="${WORK_DIR}/full-repair-repo"
FULL_STATE="${WORK_DIR}/full-repair-state"
FULL_JOBS="${WORK_DIR}/full-repair-jobs"
mkdir -p "${FULL_REPO}"
init_fixture_repo "${FULL_REPO}"

FULL_PROVIDER="${WORK_DIR}/full-provider.sh"
cat >"${FULL_PROVIDER}" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
test -f "${TAU_AUTONOMOUS_CODING_REPAIR_CONTEXT}"
cat <<'JSON'
{
  "files": {
    "status.txt": "pass\n",
    "docs/notes.txt": "proof\n"
  },
  "reason_code": "gauntlet_provider_full_file"
}
JSON
SH
chmod +x "${FULL_PROVIDER}"

run_cli issue-to-merge \
  --state-dir "${FULL_STATE}" \
  --jobs-state-dir "${FULL_JOBS}" \
  --repo-path "${FULL_REPO}" \
  --intake-id gauntlet-full-intake \
  --mission-id gauntlet-full-mission \
  --issue-url https://github.com/njfio/Tau/issues/3802 \
  --issue-title "Gauntlet full-file provider repair" \
  --issue-body "Provider must repair two files." \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q proof docs/notes.txt" \
  --provider-repair-command "${FULL_PROVIDER}" \
  --provider-repair-attempts 2 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model qwen/qwen3-235b-a22b \
  --commit-message "Gauntlet full provider repair" \
  >"${WORK_DIR}/full.json"

DIFF_REPO="${WORK_DIR}/diff-repair-repo"
DIFF_STATE="${WORK_DIR}/diff-repair-state"
DIFF_JOBS="${WORK_DIR}/diff-repair-jobs"
mkdir -p "${DIFF_REPO}"
init_fixture_repo "${DIFF_REPO}"

DIFF_PROVIDER="${WORK_DIR}/diff-provider.sh"
cat >"${DIFF_PROVIDER}" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
test -f "${TAU_AUTONOMOUS_CODING_REPAIR_CONTEXT}"
cat <<'JSON'
{
  "diff": "--- a/status.txt\n+++ b/status.txt\n@@ -1 +1 @@\n-fail\n+pass\n",
  "reason_code": "gauntlet_provider_unified_diff"
}
JSON
SH
chmod +x "${DIFF_PROVIDER}"

run_cli issue-to-merge \
  --state-dir "${DIFF_STATE}" \
  --jobs-state-dir "${DIFF_JOBS}" \
  --repo-path "${DIFF_REPO}" \
  --intake-id gauntlet-diff-intake \
  --mission-id gauntlet-diff-mission \
  --issue-url https://github.com/njfio/Tau/issues/3802 \
  --issue-title "Gauntlet unified diff provider repair" \
  --issue-body "Provider must repair one file with a unified diff." \
  --verifier-command "grep -q pass status.txt" \
  --provider-repair-command "${DIFF_PROVIDER}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model qwen/qwen3-235b-a22b \
  --commit-message "Gauntlet diff provider repair" \
  >"${WORK_DIR}/diff.json"

MALFORMED_REPO="${WORK_DIR}/malformed-repo"
MALFORMED_STATE="${WORK_DIR}/malformed-state"
MALFORMED_JOBS="${WORK_DIR}/malformed-jobs"
mkdir -p "${MALFORMED_REPO}"
init_fixture_repo "${MALFORMED_REPO}"
MALFORMED_HEAD_BEFORE="$(git -C "${MALFORMED_REPO}" rev-parse HEAD)"

MALFORMED_PROVIDER="${WORK_DIR}/malformed-provider.sh"
cat >"${MALFORMED_PROVIDER}" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'not-json\n'
SH
chmod +x "${MALFORMED_PROVIDER}"

run_cli issue-to-merge \
  --state-dir "${MALFORMED_STATE}" \
  --jobs-state-dir "${MALFORMED_JOBS}" \
  --repo-path "${MALFORMED_REPO}" \
  --intake-id gauntlet-malformed-intake \
  --mission-id gauntlet-malformed-mission \
  --issue-url https://github.com/njfio/Tau/issues/3802 \
  --issue-title "Gauntlet malformed provider repair" \
  --issue-body "Provider emits malformed output." \
  --verifier-command "grep -q pass status.txt" \
  --provider-repair-command "${MALFORMED_PROVIDER}" \
  --provider-repair-attempts 1 \
  --provider-repair-provider fake-openrouter \
  --provider-repair-model bad-model \
  --commit-message "Gauntlet malformed provider repair" \
  >"${WORK_DIR}/malformed.json"
MALFORMED_HEAD_AFTER="$(git -C "${MALFORMED_REPO}" rev-parse HEAD)"

INTAKE_REPO="${WORK_DIR}/intake-repo"
INTAKE_STATE="${WORK_DIR}/intake-state"
INTAKE_JOBS="${WORK_DIR}/intake-jobs"
mkdir -p "${INTAKE_REPO}"
init_fixture_repo "${INTAKE_REPO}"

run_cli issue-to-merge \
  --state-dir "${INTAKE_STATE}" \
  --jobs-state-dir "${INTAKE_JOBS}" \
  --repo-path "${INTAKE_REPO}" \
  --intake-id gauntlet-intake-only \
  --mission-id gauntlet-intake-mission \
  --issue-url https://github.com/njfio/Tau/issues/3802 \
  --issue-title "Gauntlet missing verifier" \
  --issue-body "This request has no verifier plan." \
  >"${WORK_DIR}/intake.json"

MERGE_REPO="${WORK_DIR}/merge-repo"
MERGE_STATE="${WORK_DIR}/merge-state"
MERGE_JOBS="${WORK_DIR}/merge-jobs"
mkdir -p "${MERGE_REPO}"
init_fixture_repo "${MERGE_REPO}"

FAKE_GH="${WORK_DIR}/fake-gh.sh"
FAKE_GH_ARGV="${WORK_DIR}/fake-gh-argv.txt"
cat >"${FAKE_GH}" <<SH
#!/usr/bin/env bash
{
  printf 'CALL\\n'
  printf '%s\\n' "\$@"
} >> "${FAKE_GH_ARGV}"
if [ "\${1:-}" = "pr" ] && [ "\${2:-}" = "create" ]; then
  printf 'https://github.com/njfio/Tau/pull/3802\\n'
else
  printf 'auto-merge-enabled\\n'
fi
SH
chmod +x "${FAKE_GH}"

GH_TOKEN=test-token run_cli issue-to-merge \
  --state-dir "${MERGE_STATE}" \
  --jobs-state-dir "${MERGE_JOBS}" \
  --repo-path "${MERGE_REPO}" \
  --intake-id gauntlet-merge-intake \
  --mission-id gauntlet-merge-mission \
  --issue-url https://github.com/njfio/Tau/issues/3802 \
  --issue-title "Gauntlet guarded auto merge" \
  --issue-body "Request draft PR plus safe auto-merge." \
  --verifier-command "grep -q pass status.txt" \
  --edit "status.txt=pass" \
  --commit-message "Gauntlet guarded auto merge" \
  --pr-mode draft \
  --allow-auto-merge \
  --merge-method squash \
  --delete-branch \
  --gh-binary "${FAKE_GH}" \
  >"${WORK_DIR}/merge.json"

python3 - \
  "${WORK_DIR}/full.json" \
  "${FULL_REPO}/status.txt" \
  "${FULL_REPO}/docs/notes.txt" \
  "${WORK_DIR}/diff.json" \
  "${DIFF_REPO}/status.txt" \
  "${WORK_DIR}/malformed.json" \
  "${MALFORMED_REPO}/status.txt" \
  "${MALFORMED_HEAD_BEFORE}" \
  "${MALFORMED_HEAD_AFTER}" \
  "${WORK_DIR}/intake.json" \
  "${WORK_DIR}/merge.json" \
  "${FAKE_GH_ARGV}" <<'PY'
import json
import sys

(
    full_path,
    full_status_path,
    full_notes_path,
    diff_path,
    diff_status_path,
    malformed_path,
    malformed_status_path,
    malformed_head_before,
    malformed_head_after,
    intake_path,
    merge_path,
    fake_gh_argv_path,
) = sys.argv[1:]

def load(path):
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)

full = load(full_path)
assert full["status"] == "pr_ready", full
full_status = full["run"]["status"]
assert full_status["provider_repair_status"] == "applied", full_status
assert full_status["provider_repair_attempts"] == 1, full_status
with open(full_status_path, "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "pass"
with open(full_notes_path, "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "proof"

diff = load(diff_path)
assert diff["status"] == "pr_ready", diff
diff_status = diff["run"]["status"]
assert diff_status["provider_repair_status"] == "applied", diff_status
assert diff_status["provider_repair_reason_code"] == "provider_repair_edit_parsed", diff_status
with open(diff_status_path, "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "pass"

malformed = load(malformed_path)
assert malformed["status"] == "blocked", malformed
assert malformed["reason_code"] == "autonomous_coding_job_provider_repair_exhausted", malformed
malformed_status = malformed["run"]["status"]
assert malformed_status["provider_repair_status"] == "rejected", malformed_status
assert malformed_status["provider_repair_reason_code"] == "provider_repair_output_invalid", malformed_status
assert malformed_head_before == malformed_head_after, (malformed_head_before, malformed_head_after)
with open(malformed_status_path, "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "fail"

intake = load(intake_path)
assert intake["status"] == "blocked", intake
assert intake["reason_code"] == "issue_intake_authority_required", intake
assert intake["intake"]["status"] == "blocked", intake
assert intake["run"] is None, intake

merge = load(merge_path)
assert merge["status"] == "auto_merge_requested", merge
assert merge["auto_merge"]["evidence"]["status"] == "requested", merge
with open(fake_gh_argv_path, "r", encoding="utf-8") as handle:
    argv = handle.read().splitlines()
assert argv.count("CALL") == 2, argv
assert "--auto" in argv, argv
assert "--squash" in argv, argv
assert "--delete-branch" in argv, argv
assert "--admin" not in argv, argv

print('{"suite":"autonomous_coding_gauntlet","passed":5,"failed":0}')
PY

"${REPO_ROOT}/scripts/dev/test-openrouter-repair-adapter.sh"

printf 'autonomous_coding_gauntlet=pass\n'

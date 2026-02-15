#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VALIDATOR="${SCRIPT_DIR}/validate-m24-rl-resume-after-crash-playbook.sh"
TEMPLATE="${SCRIPT_DIR}/m24-rl-resume-after-crash-playbook-template.json"

if [[ ! -x "${VALIDATOR}" ]]; then
  echo "missing validator: ${VALIDATOR}" >&2
  exit 1
fi

if [[ ! -f "${TEMPLATE}" ]]; then
  echo "missing template: ${TEMPLATE}" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

VALID_ARTIFACT="${TMP_DIR}/valid-playbook.json"
INVALID_ARTIFACT="${TMP_DIR}/invalid-playbook.json"

cat > "${VALID_ARTIFACT}" <<'JSON'
{
  "schema_version": 1,
  "artifact_kind": "m24_rl_resume_after_crash_playbook",
  "run_id": "m24-recovery-2026-02-15-0001",
  "crash_drill": {
    "state_dir": ".tau/runtime/background-jobs",
    "running_manifest_path": ".tau/runtime/background-jobs/jobs/job-recover-1.json",
    "crash_simulation_command": "pkill -9 tau-coding-agent"
  },
  "resume_drill": {
    "restart_command": "cargo run -p tau-coding-agent -- --prompt-optimization-config .tau/prompt-optimization.json",
    "resumed_job_id": "job-recover-1",
    "checkpoint_path": ".tau/training/store.sqlite"
  },
  "evidence": {
    "events_log_path": ".tau/runtime/background-jobs/events.jsonl",
    "health_snapshot_path": ".tau/runtime/background-jobs/state.json",
    "operator_log_path": "tasks/reports/m24-recovery-operator.log"
  },
  "outcome": {
    "resume_status": "succeeded",
    "recovery_reason_code": "job_recovered_after_restart"
  }
}
JSON

"${VALIDATOR}" "${VALID_ARTIFACT}" >/dev/null

cat > "${INVALID_ARTIFACT}" <<'JSON'
{
  "schema_version": 1,
  "artifact_kind": "m24_rl_resume_after_crash_playbook",
  "run_id": "m24-recovery-2026-02-15-0001",
  "crash_drill": {
    "state_dir": ".tau/runtime/background-jobs",
    "running_manifest_path": ".tau/runtime/background-jobs/jobs/job-recover-1.json",
    "crash_simulation_command": "pkill -9 tau-coding-agent"
  },
  "resume_drill": {
    "restart_command": "cargo run -p tau-coding-agent -- --prompt-optimization-config .tau/prompt-optimization.json",
    "resumed_job_id": "job-recover-1",
    "checkpoint_path": ".tau/training/store.sqlite"
  },
  "evidence": {
    "events_log_path": ".tau/runtime/background-jobs/events.jsonl",
    "health_snapshot_path": ".tau/runtime/background-jobs/state.json",
    "operator_log_path": "tasks/reports/m24-recovery-operator.log"
  },
  "outcome": {
    "resume_status": "failed",
    "recovery_reason_code": "job_recovered_after_restart"
  }
}
JSON

if "${VALIDATOR}" "${INVALID_ARTIFACT}" >/dev/null 2>&1; then
  echo "expected invalid artifact to fail validation" >&2
  exit 1
fi

echo "ok - m24 resume-after-crash playbook validation"

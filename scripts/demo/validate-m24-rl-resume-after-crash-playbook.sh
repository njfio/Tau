#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: validate-m24-rl-resume-after-crash-playbook.sh <artifact-json>" >&2
  exit 2
fi

artifact_path="$1"
if [[ ! -f "${artifact_path}" ]]; then
  echo "artifact not found: ${artifact_path}" >&2
  exit 2
fi

python3 - "$artifact_path" <<'PY'
import json
import re
import sys
from pathlib import Path

artifact_path = Path(sys.argv[1])

with artifact_path.open("r", encoding="utf-8") as handle:
    payload = json.load(handle)


def fail(message: str) -> None:
    print(f"invalid m24 resume-after-crash playbook: {message}", file=sys.stderr)
    raise SystemExit(1)


def require_non_empty_string(container: dict, key: str, context: str) -> str:
    value = container.get(key)
    if not isinstance(value, str) or not value.strip():
        fail(f"{context}.{key} must be a non-empty string")
    return value.strip()


if payload.get("schema_version") != 1:
    fail("schema_version must equal 1")
if payload.get("artifact_kind") != "m24_rl_resume_after_crash_playbook":
    fail("artifact_kind must equal 'm24_rl_resume_after_crash_playbook'")

run_id = payload.get("run_id")
if not isinstance(run_id, str) or not re.match(r"^m24-recovery-[0-9-]+$", run_id):
    fail("run_id must match ^m24-recovery-[0-9-]+$")

crash_drill = payload.get("crash_drill")
if not isinstance(crash_drill, dict):
    fail("crash_drill must be an object")
require_non_empty_string(crash_drill, "state_dir", "crash_drill")
running_manifest_path = require_non_empty_string(
    crash_drill, "running_manifest_path", "crash_drill"
)
if not running_manifest_path.endswith(".json"):
    fail("crash_drill.running_manifest_path must end with .json")
require_non_empty_string(crash_drill, "crash_simulation_command", "crash_drill")

resume_drill = payload.get("resume_drill")
if not isinstance(resume_drill, dict):
    fail("resume_drill must be an object")
require_non_empty_string(resume_drill, "restart_command", "resume_drill")
require_non_empty_string(resume_drill, "resumed_job_id", "resume_drill")
checkpoint_path = require_non_empty_string(resume_drill, "checkpoint_path", "resume_drill")
if not checkpoint_path.endswith(".sqlite"):
    fail("resume_drill.checkpoint_path must end with .sqlite")

evidence = payload.get("evidence")
if not isinstance(evidence, dict):
    fail("evidence must be an object")
events_log_path = require_non_empty_string(evidence, "events_log_path", "evidence")
if not events_log_path.endswith("events.jsonl"):
    fail("evidence.events_log_path must end with events.jsonl")
health_snapshot_path = require_non_empty_string(
    evidence, "health_snapshot_path", "evidence"
)
if not health_snapshot_path.endswith(".json"):
    fail("evidence.health_snapshot_path must end with .json")
operator_log_path = require_non_empty_string(evidence, "operator_log_path", "evidence")
if not operator_log_path.endswith(".log"):
    fail("evidence.operator_log_path must end with .log")

outcome = payload.get("outcome")
if not isinstance(outcome, dict):
    fail("outcome must be an object")
resume_status = require_non_empty_string(outcome, "resume_status", "outcome")
if resume_status != "succeeded":
    fail("outcome.resume_status must equal 'succeeded' for drill pass")
recovery_reason_code = require_non_empty_string(
    outcome, "recovery_reason_code", "outcome"
)
if recovery_reason_code != "job_recovered_after_restart":
    fail("outcome.recovery_reason_code must equal 'job_recovered_after_restart'")

print("ok - m24 resume-after-crash playbook artifact valid")
PY

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DIR="${TAU_M296_REPORT_DIR:-${ROOT_DIR}/artifacts/operator-ga-readiness}"
REPORT_PATH="${REPORT_DIR}/verification-report.json"
STEPS_TMP="${REPORT_DIR}/steps.tmp"
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
MOCK_MODE="${TAU_M296_MOCK_MODE:-0}"
MOCK_FAIL_PATTERN="${TAU_M296_MOCK_FAIL_PATTERN:-}"
MOCK_SKIP_PATTERN="${TAU_M296_MOCK_SKIP_PATTERN:-}"
overall="pass"

usage() {
  cat <<'USAGE'
Usage: m296-ga-readiness-gate.sh [options]

Run the M296 final GA readiness gate and emit deterministic verification output.

Options:
  --report-dir <path>       Output directory for report + step logs.
  --generated-at <iso>      Override report generated timestamp.
  --help                    Show this help text.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --report-dir)
      REPORT_DIR="$2"
      shift 2
      ;;
    --generated-at)
      GENERATED_AT="$2"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option '$1'" >&2
      usage >&2
      exit 2
      ;;
  esac
done

mkdir -p "${REPORT_DIR}"
: > "${STEPS_TMP}"

run_step() {
  local id="$1"
  shift
  local cmd="$*"
  local log_path="${REPORT_DIR}/${id}.log"
  local status="pass"
  local rc=0

  echo "==> ${id}"
  if [[ "${MOCK_MODE}" == "1" ]]; then
    if [[ -n "${MOCK_FAIL_PATTERN}" ]] && [[ "${id}" == *"${MOCK_FAIL_PATTERN}"* ]]; then
      status="fail"
    elif [[ -n "${MOCK_SKIP_PATTERN}" ]] && [[ "${id}" == *"${MOCK_SKIP_PATTERN}"* ]]; then
      status="skip"
    fi
    printf 'mock-mode command: %s\nmock-mode status: %s\n' "${cmd}" "${status}" >"${log_path}"
  else
    if (cd "${ROOT_DIR}" && bash -c "${cmd}") >"${log_path}" 2>&1; then
      status="pass"
    else
      rc=$?
      if [[ "${rc}" -eq 20 ]]; then
        status="skip"
      else
        status="fail"
      fi
    fi
  fi

  if [[ "${status}" == "fail" ]]; then
    overall="fail"
    echo "    FAIL (${log_path})"
  elif [[ "${status}" == "skip" ]]; then
    echo "    SKIP (${log_path})"
  else
    echo "    PASS (${log_path})"
  fi
  printf '%s|%s|%s|%s\n' "${id}" "${status}" "${log_path}" "${cmd}" >> "${STEPS_TMP}"
}

run_step "m295_operator_maturity_wave" \
  "bash scripts/verify/m295-operator-maturity-wave.sh"
run_step "rl_hardening_live_benchmark_proof" \
  "bash scripts/demo/test-m24-rl-live-benchmark-proof.sh"
run_step "operator_readiness_contract_tests" \
  "bash scripts/dev/test-operator-readiness-live-check.sh"
run_step "auth_live_validation_matrix" \
  "bash scripts/verify/m296-live-auth-validation.sh"
run_step "rollback_contract_checks" \
  "bash scripts/demo/test-rollback-drill-checklist.sh"
run_step "rollback_trigger_matrix_markers" \
  "rg -n \"proof-summary-missing|proof-runs-failed|proof-markers-missing|validation-matrix-missing|validation-open-issues|validation-completion-below-100\" docs/guides/consolidated-runtime-rollback-drill.md"
run_step "readme_ga_workflow_markers" \
  "rg -n \"M296 GA readiness gate|scripts/verify/m296-ga-readiness-gate.sh|Connected operator GA loop\" README.md"
run_step "docs_index_ga_entry" \
  "rg -n \"M296 GA Readiness Gate|guides/m296-ga-readiness-gate.md\" docs/README.md"
run_step "milestone_artifacts_present" \
  "test -f specs/milestones/m296/index.md && test -f specs/3430/spec.md && test -f specs/3430/plan.md && test -f specs/3430/tasks.md"

python3 - \
  "${GENERATED_AT}" \
  "${overall}" \
  "${REPORT_PATH}" \
  "${STEPS_TMP}" <<'PY'
import json
import sys
from pathlib import Path

generated_at, overall, report_path_raw, steps_tmp_raw = sys.argv[1:]
report_path = Path(report_path_raw)
steps_tmp = Path(steps_tmp_raw)

steps = []
for line in steps_tmp.read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    step_id, status, log, command = line.split("|", 3)
    steps.append({"id": step_id, "status": status, "log": log, "command": command})

status_by_id = {step["id"]: step["status"] for step in steps}

def criterion(criterion_id: str, description: str, evidence_steps, allow_skip=False):
    accepted = {"pass"}
    if allow_skip:
        accepted.add("skip")
    status = "pass" if all(status_by_id.get(step_id) in accepted for step_id in evidence_steps) else "fail"
    return {
        "id": criterion_id,
        "description": description,
        "status": status,
        "evidence_steps": evidence_steps,
    }

signoff_criteria = [
    criterion(
        "runtime_and_auth_contracts",
        "Core RL/auth/readiness regression contracts are green.",
        ["m295_operator_maturity_wave", "operator_readiness_contract_tests"],
    ),
    criterion(
        "rl_hardening_contracts",
        "RL hardening proof checks are green.",
        ["rl_hardening_live_benchmark_proof"],
    ),
    criterion(
        "auth_live_validation",
        "Live auth validation matrix is green or explicitly skipped when live env inputs are unavailable.",
        ["auth_live_validation_matrix"],
        allow_skip=True,
    ),
    criterion(
        "rollback_contract",
        "Rollback trigger matrix and drill checks are validated.",
        ["rollback_contract_checks", "rollback_trigger_matrix_markers"],
    ),
    criterion(
        "docs_connected_flow",
        "README and docs index expose a connected GA operator flow.",
        ["readme_ga_workflow_markers", "docs_index_ga_entry"],
    ),
    criterion(
        "milestone_closeout_artifacts",
        "Milestone + issue spec/plan/tasks artifacts are present.",
        ["milestone_artifacts_present"],
    ),
]

closeout_status = (
    "ready"
    if overall == "pass" and all(item["status"] == "pass" for item in signoff_criteria)
    else "blocked"
)

report = {
    "schema_version": 1,
    "suite_id": "m296_ga_readiness_gate",
    "milestone": "m296",
    "issue": 3430,
    "generated_at": generated_at,
    "overall": overall,
    "rollback_trigger_matrix": {
        "source": "docs/guides/consolidated-runtime-rollback-drill.md",
        "trigger_ids": [
            "proof-summary-missing",
            "proof-runs-failed",
            "proof-markers-missing",
            "validation-matrix-missing",
            "validation-open-issues",
            "validation-completion-below-100",
        ],
    },
    "signoff_criteria": signoff_criteria,
    "steps": steps,
    "closeout_summary": {
        "status": closeout_status,
        "milestone_index": "specs/milestones/m296/index.md",
        "spec": "specs/3430/spec.md",
        "plan": "specs/3430/plan.md",
        "tasks": "specs/3430/tasks.md",
        "reports": [
            "artifacts/operator-maturity-wave/verification-report.json",
            "artifacts/operator-ga-readiness/verification-report.json",
        ],
    },
}

report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
PY

rm -f "${STEPS_TMP}"
echo "verification report: ${REPORT_PATH}"

if [[ "${overall}" != "pass" ]]; then
  echo "ga readiness verification failed"
  exit 1
fi

echo "ga readiness verification passed"

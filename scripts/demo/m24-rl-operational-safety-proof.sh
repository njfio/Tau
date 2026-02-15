#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OUTPUT_JSON="tasks/reports/m24-rl-operational-safety-proof.json"
OUTPUT_MD="tasks/reports/m24-rl-operational-safety-proof.md"
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
RUNNER=""
FAIL_FAST="false"

usage() {
  cat <<EOF
Usage: m24-rl-operational-safety-proof.sh [options]

Run deterministic M24 operational safety proof checks and emit JSON/Markdown artifacts.

Options:
  --repo-root <path>    Repository root path (default: inferred from script location).
  --output-json <path>  Output JSON artifact path (default: ${OUTPUT_JSON}).
  --output-md <path>    Output Markdown artifact path (default: ${OUTPUT_MD}).
  --generated-at <iso>  Override generated timestamp (UTC ISO-8601 string).
  --runner <path>       Optional check runner hook for tests (called as: <runner> <check_id> <log_path>).
  --fail-fast           Stop after first failing check.
  --help                Show this usage message.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      REPO_ROOT="$2"
      shift 2
      ;;
    --output-json)
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --output-md)
      OUTPUT_MD="$2"
      shift 2
      ;;
    --generated-at)
      GENERATED_AT="$2"
      shift 2
      ;;
    --runner)
      RUNNER="$2"
      shift 2
      ;;
    --fail-fast)
      FAIL_FAST="true"
      shift
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ ! -d "${REPO_ROOT}" ]]; then
  echo "invalid --repo-root directory: ${REPO_ROOT}" >&2
  exit 2
fi
REPO_ROOT="$(cd "${REPO_ROOT}" && pwd)"

if [[ -n "${RUNNER}" && ! -x "${RUNNER}" ]]; then
  echo "runner must be executable: ${RUNNER}" >&2
  exit 2
fi

OUTPUT_JSON_ABS="${OUTPUT_JSON}"
OUTPUT_MD_ABS="${OUTPUT_MD}"
if [[ "${OUTPUT_JSON_ABS}" != /* ]]; then
  OUTPUT_JSON_ABS="${REPO_ROOT}/${OUTPUT_JSON_ABS}"
fi
if [[ "${OUTPUT_MD_ABS}" != /* ]]; then
  OUTPUT_MD_ABS="${REPO_ROOT}/${OUTPUT_MD_ABS}"
fi

REPORT_DIR="$(dirname "${OUTPUT_JSON_ABS}")"
LOG_DIR="${REPORT_DIR}/m24-rl-operational-safety-logs"
mkdir -p "${REPORT_DIR}" "${LOG_DIR}"

RUNBOOK_PATH_REL="docs/guides/prompt-optimization-recovery-runbook.md"
RUNBOOK_PATH="${REPO_ROOT}/${RUNBOOK_PATH_REL}"
RUNBOOK_EXISTS="false"
if [[ -f "${RUNBOOK_PATH}" ]]; then
  RUNBOOK_EXISTS="true"
fi

RESULTS_TSV="$(mktemp)"
trap 'rm -f "${RESULTS_TSV}"' EXIT

now_ms() {
  python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
}

check_ids=(
  "control_sequence_tests"
  "safety_gate_tests"
  "resume_playbook_validation"
  "benchmark_proof_validation"
)

check_descriptions=(
  "Prompt optimization control suite (pause/resume/rollback coverage)"
  "Checkpoint promotion safety-gate suite"
  "Resume-after-crash playbook validation suite"
  "Benchmark proof template validation suite"
)

check_commands=(
  "cargo test -p tau-coding-agent prompt_optimization_control_"
  "cargo test -p tau-trainer checkpoint_promotion_gate"
  "bash scripts/demo/test-m24-rl-resume-after-crash-playbook.sh"
  "bash scripts/demo/test-m24-rl-benchmark-proof-template.sh"
)

overall_status="pass"

for index in "${!check_ids[@]}"; do
  check_id="${check_ids[$index]}"
  check_description="${check_descriptions[$index]}"
  check_command="${check_commands[$index]}"
  log_path="${LOG_DIR}/${check_id}.log"
  start_ms="$(now_ms)"

  echo "[m24-operational-safety] running ${check_id}: ${check_command}"
  set +e
  if [[ -n "${RUNNER}" ]]; then
    "${RUNNER}" "${check_id}" "${log_path}" >"${log_path}" 2>&1
    check_rc=$?
  else
    (
      cd "${REPO_ROOT}"
      bash -lc "${check_command}"
    ) >"${log_path}" 2>&1
    check_rc=$?
  fi
  set -e

  end_ms="$(now_ms)"
  duration_ms=$((end_ms - start_ms))
  check_status="pass"
  if [[ "${check_rc}" -ne 0 ]]; then
    check_status="fail"
    overall_status="fail"
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${check_id}" \
    "${check_description}" \
    "${check_command}" \
    "${check_status}" \
    "${check_rc}" \
    "${duration_ms}" \
    "${log_path}" >>"${RESULTS_TSV}"

  if [[ "${check_status}" == "fail" && "${FAIL_FAST}" == "true" ]]; then
    break
  fi
done

if [[ "${RUNBOOK_EXISTS}" != "true" ]]; then
  overall_status="fail"
fi

python3 - "${RESULTS_TSV}" "${OUTPUT_JSON_ABS}" "${OUTPUT_MD_ABS}" "${GENERATED_AT}" "${REPO_ROOT}" "${overall_status}" "${RUNBOOK_PATH_REL}" "${RUNBOOK_EXISTS}" <<'PY'
import csv
import json
import pathlib
import sys

(
    results_path,
    output_json_path,
    output_md_path,
    generated_at,
    repo_root,
    overall_status,
    runbook_path_rel,
    runbook_exists,
) = sys.argv[1:]

checks = []
with open(results_path, "r", encoding="utf-8") as handle:
    reader = csv.reader(handle, delimiter="\t")
    for row in reader:
        if not row:
            continue
        checks.append(
            {
                "id": row[0],
                "description": row[1],
                "command": row[2],
                "status": row[3],
                "exit_code": int(row[4]),
                "duration_ms": int(row[5]),
                "log_path": row[6],
            }
        )

payload = {
    "schema_version": 1,
    "generated_at": generated_at,
    "repo_root": repo_root,
    "overall_status": overall_status,
    "checks": checks,
    "runbook_evidence": {
        "path": runbook_path_rel,
        "exists": runbook_exists == "true",
    },
}

output_json = pathlib.Path(output_json_path)
output_json.parent.mkdir(parents=True, exist_ok=True)
output_json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

lines = [
    "# M24 RL Operational Safety Proof",
    "",
    f"- Generated at: `{generated_at}`",
    f"- Repo root: `{repo_root}`",
    f"- Overall status: `{overall_status}`",
    f"- Runbook evidence: `{runbook_path_rel}` (exists={runbook_exists})",
    "",
    "| Check | Status | Duration (ms) | Exit |",
    "| --- | --- | ---: | ---: |",
]
for check in checks:
    lines.append(
        f"| `{check['id']}` | `{check['status']}` | {check['duration_ms']} | {check['exit_code']} |"
    )
lines.extend(
    [
        "",
        "## Commands",
        "",
    ]
)
for check in checks:
    lines.append(f"- `{check['id']}`: `{check['command']}`")

output_md = pathlib.Path(output_md_path)
output_md.parent.mkdir(parents=True, exist_ok=True)
output_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY

echo "m24 operational safety proof: overall_status=${overall_status} json=${OUTPUT_JSON_ABS} md=${OUTPUT_MD_ABS}"

if [[ "${overall_status}" != "pass" ]]; then
  exit 1
fi

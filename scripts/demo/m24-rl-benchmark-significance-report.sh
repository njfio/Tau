#!/usr/bin/env bash
set -euo pipefail

GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
BASELINE_SAMPLES=""
TRAINED_SAMPLES=""
RUN_ID=""
OUTPUT_REPORT="tasks/reports/m24-benchmark-significance.json"
SUITE_NAME="m24-rl-suite"
SUITE_VERSION="v1"
ALPHA="0.05"
BASELINE_SAFETY_PENALTY="0.0"
TRAINED_SAFETY_PENALTY="0.0"
RETENTION_POLICY="archive-then-purge"
RETAIN_DAYS="365"
ARCHIVE_AFTER_DAYS="30"

usage() {
  cat <<EOF
Usage: m24-rl-benchmark-significance-report.sh [options]

Generate a baseline-vs-trained M24 benchmark significance report artifact.

Required:
  --baseline-samples <path>   JSON array file with baseline reward samples.
  --trained-samples <path>    JSON array file with trained reward samples.
  --run-id <id>               Run id matching ^m24-[a-z0-9-]+$.

Optional:
  --output-report <path>             Output report path (default: ${OUTPUT_REPORT}).
  --generated-at <iso>               UTC generated-at timestamp.
  --suite-name <name>                Benchmark suite name (default: ${SUITE_NAME}).
  --suite-version <version>          Benchmark suite version (default: ${SUITE_VERSION}).
  --alpha <value>                    Supported: 0.10, 0.05, 0.01 (default: ${ALPHA}).
  --baseline-safety-penalty <value>  Baseline mean safety penalty (default: ${BASELINE_SAFETY_PENALTY}).
  --trained-safety-penalty <value>   Trained mean safety penalty (default: ${TRAINED_SAFETY_PENALTY}).
  --retention-policy <value>         archive-then-purge|retain-only (default: ${RETENTION_POLICY}).
  --retain-days <int>                Retention days (default: ${RETAIN_DAYS}).
  --archive-after-days <int>         Archive-after days (default: ${ARCHIVE_AFTER_DAYS}).
  --help                             Show this usage message.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --baseline-samples)
      BASELINE_SAMPLES="$2"
      shift 2
      ;;
    --trained-samples)
      TRAINED_SAMPLES="$2"
      shift 2
      ;;
    --run-id)
      RUN_ID="$2"
      shift 2
      ;;
    --output-report)
      OUTPUT_REPORT="$2"
      shift 2
      ;;
    --generated-at)
      GENERATED_AT="$2"
      shift 2
      ;;
    --suite-name)
      SUITE_NAME="$2"
      shift 2
      ;;
    --suite-version)
      SUITE_VERSION="$2"
      shift 2
      ;;
    --alpha)
      ALPHA="$2"
      shift 2
      ;;
    --baseline-safety-penalty)
      BASELINE_SAFETY_PENALTY="$2"
      shift 2
      ;;
    --trained-safety-penalty)
      TRAINED_SAFETY_PENALTY="$2"
      shift 2
      ;;
    --retention-policy)
      RETENTION_POLICY="$2"
      shift 2
      ;;
    --retain-days)
      RETAIN_DAYS="$2"
      shift 2
      ;;
    --archive-after-days)
      ARCHIVE_AFTER_DAYS="$2"
      shift 2
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

if [[ -z "${BASELINE_SAMPLES}" || -z "${TRAINED_SAMPLES}" || -z "${RUN_ID}" ]]; then
  echo "error: --baseline-samples, --trained-samples, and --run-id are required" >&2
  usage >&2
  exit 2
fi

if [[ ! -f "${BASELINE_SAMPLES}" ]]; then
  echo "error: baseline samples file not found: ${BASELINE_SAMPLES}" >&2
  exit 2
fi
if [[ ! -f "${TRAINED_SAMPLES}" ]]; then
  echo "error: trained samples file not found: ${TRAINED_SAMPLES}" >&2
  exit 2
fi

output_dir="$(dirname "${OUTPUT_REPORT}")"
mkdir -p "${output_dir}"

python3 - "${BASELINE_SAMPLES}" "${TRAINED_SAMPLES}" "${RUN_ID}" "${OUTPUT_REPORT}" "${GENERATED_AT}" "${SUITE_NAME}" "${SUITE_VERSION}" "${ALPHA}" "${BASELINE_SAFETY_PENALTY}" "${TRAINED_SAFETY_PENALTY}" "${RETENTION_POLICY}" "${RETAIN_DAYS}" "${ARCHIVE_AFTER_DAYS}" <<'PY'
import json
import math
import re
import statistics
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

(
    baseline_path,
    trained_path,
    run_id,
    output_path,
    generated_at,
    suite_name,
    suite_version,
    alpha_raw,
    baseline_safety_raw,
    trained_safety_raw,
    retention_policy,
    retain_days_raw,
    archive_after_days_raw,
) = sys.argv[1:]

run_id_re = re.compile(r"^m24-[a-z0-9-]+$")
if not run_id_re.match(run_id):
    raise SystemExit("run_id must match ^m24-[a-z0-9-]+$")

if retention_policy not in {"archive-then-purge", "retain-only"}:
    raise SystemExit("retention policy must be archive-then-purge or retain-only")

try:
    retain_days = int(retain_days_raw)
    archive_after_days = int(archive_after_days_raw)
except ValueError as error:
    raise SystemExit(f"retention fields must be integers: {error}")
if retain_days <= 0:
    raise SystemExit("retain_days must be > 0")
if archive_after_days < 0:
    raise SystemExit("archive_after_days must be >= 0")
if archive_after_days > retain_days:
    raise SystemExit("archive_after_days must be <= retain_days")

try:
    alpha = float(alpha_raw)
except ValueError as error:
    raise SystemExit(f"alpha must be numeric: {error}")
z_scores = {0.10: 1.645, 0.05: 1.96, 0.01: 2.576}
if alpha not in z_scores:
    raise SystemExit("alpha must be one of: 0.10, 0.05, 0.01")

def parse_samples(path: str, label: str) -> list[float]:
    payload = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(payload, list):
        raise SystemExit(f"{label} samples payload must be a JSON array")
    if len(payload) < 2:
        raise SystemExit(f"{label} samples must contain at least two values")
    values: list[float] = []
    for index, item in enumerate(payload):
        if not isinstance(item, (int, float)):
            raise SystemExit(
                f"{label} sample at index {index} must be a finite number"
            )
        value = float(item)
        if not math.isfinite(value):
            raise SystemExit(
                f"{label} sample at index {index} must be a finite number"
            )
        values.append(value)
    return values

baseline = parse_samples(baseline_path, "baseline")
trained = parse_samples(trained_path, "trained")
if len(baseline) != len(trained):
    raise SystemExit("baseline and trained samples must contain the same number of samples")

try:
    baseline_safety = float(baseline_safety_raw)
    trained_safety = float(trained_safety_raw)
except ValueError as error:
    raise SystemExit(f"safety penalty must be numeric: {error}")
if not math.isfinite(baseline_safety) or not math.isfinite(trained_safety):
    raise SystemExit("safety penalty values must be finite")

baseline_mean = statistics.fmean(baseline)
trained_mean = statistics.fmean(trained)
baseline_var = statistics.variance(baseline)
trained_var = statistics.variance(trained)
count = len(baseline)

delta = trained_mean - baseline_mean
standard_error = math.sqrt(baseline_var / count + trained_var / count)
z_score = z_scores[alpha]
delta_margin = z_score * standard_error
delta_ci_low = delta - delta_margin
delta_ci_high = delta + delta_margin

if standard_error == 0.0:
    p_value = 0.0 if delta > 0 else 1.0
else:
    z_stat = delta / standard_error
    p_value = math.erfc(abs(z_stat) / math.sqrt(2.0))
p_value = min(max(p_value, 0.0), 1.0)
confidence_level = 1.0 - p_value

significance_pass = delta_ci_low > 0.0

try:
    generated_at_dt = datetime.strptime(generated_at, "%Y-%m-%dT%H:%M:%SZ").replace(
        tzinfo=timezone.utc
    )
except ValueError as error:
    raise SystemExit(f"generated-at must be UTC RFC3339 without subseconds: {error}")

purge_after = (generated_at_dt + timedelta(days=retain_days)).strftime("%Y-%m-%dT%H:%M:%SZ")
archive_prefix = generated_at_dt.strftime("%Y/%m")

report = {
    "schema_version": 1,
    "report_kind": "significance",
    "run_id": run_id,
    "generated_at": generated_at,
    "benchmark_suite": {
        "name": suite_name,
        "version": suite_version,
    },
    "metrics": {
        "episodes": count,
        "mean_reward": trained_mean,
        "mean_safety_penalty": trained_safety,
    },
    "significance": {
        "alpha": alpha,
        "p_value": p_value,
        "confidence_level": confidence_level,
        "mean_delta": delta,
        "delta_ci_low": delta_ci_low,
        "delta_ci_high": delta_ci_high,
        "pass": significance_pass,
        "baseline_mean_reward": baseline_mean,
        "trained_mean_reward": trained_mean,
        "baseline_mean_safety_penalty": baseline_safety,
        "trained_mean_safety_penalty": trained_safety,
    },
    "publication": {
        "report_path": f".tau/reports/m24/{run_id}/m24-benchmark-report-significance.json",
        "archive_path": f".tau/reports/archive/m24/{archive_prefix}/m24-benchmark-report-{run_id}-significance.json",
    },
    "retention": {
        "policy": retention_policy,
        "retain_days": retain_days,
        "archive_after_days": archive_after_days,
        "purge_after": purge_after,
    },
}

output = Path(output_path)
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

print(
    f"m24 benchmark significance report generated: pass={str(significance_pass).lower()} p_value={p_value:.6f} output={output_path}"
)
PY

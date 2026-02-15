#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

REPO_ROOT="${DEFAULT_REPO_ROOT}"
BASELINE_JSON="${DEFAULT_REPO_ROOT}/tasks/reports/m23-rustdoc-marker-count-baseline.json"
CURRENT_JSON="${DEFAULT_REPO_ROOT}/tasks/reports/m23-rustdoc-marker-count.json"
OUTPUT_JSON="${DEFAULT_REPO_ROOT}/tasks/reports/m23-rustdoc-marker-threshold-verify.json"
OUTPUT_MD="${DEFAULT_REPO_ROOT}/tasks/reports/m23-rustdoc-marker-threshold-verify.md"
THRESHOLD=3000
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
QUIET_MODE="false"

usage() {
  cat <<'EOF'
Usage: rustdoc-marker-threshold-verify.sh [options]

Compare baseline and current rustdoc marker count artifacts and emit threshold
verification reports with per-crate deltas.

Options:
  --repo-root <path>      Repository root (default: auto-detected).
  --baseline-json <path>  Baseline marker JSON artifact path.
  --current-json <path>   Current marker JSON artifact path.
  --threshold <int>       Marker threshold target (default: 3000).
  --output-json <path>    Output JSON artifact path.
  --output-md <path>      Output Markdown artifact path.
  --generated-at <iso>    Override generated-at timestamp (UTC ISO-8601).
  --quiet                 Suppress informational stdout summary.
  --help                  Show this help text.
EOF
}

fail() {
  echo "error: $*" >&2
  exit 1
}

log_info() {
  if [[ "${QUIET_MODE}" != "true" ]]; then
    echo "$@"
  fi
}

resolve_path() {
  local base="$1"
  local path="$2"
  if [[ "${path}" = /* ]]; then
    printf '%s\n' "${path}"
  else
    printf '%s\n' "${base}/${path}"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      REPO_ROOT="$2"
      shift 2
      ;;
    --baseline-json)
      BASELINE_JSON="$2"
      shift 2
      ;;
    --current-json)
      CURRENT_JSON="$2"
      shift 2
      ;;
    --threshold)
      THRESHOLD="$2"
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
    --quiet)
      QUIET_MODE="true"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown option '$1'"
      ;;
  esac
done

if [[ ! -d "${REPO_ROOT}" ]]; then
  fail "repo root not found: ${REPO_ROOT}"
fi

if ! [[ "${THRESHOLD}" =~ ^[0-9]+$ ]]; then
  fail "threshold must be a non-negative integer"
fi

BASELINE_JSON_ABS="$(resolve_path "${REPO_ROOT}" "${BASELINE_JSON}")"
CURRENT_JSON_ABS="$(resolve_path "${REPO_ROOT}" "${CURRENT_JSON}")"
OUTPUT_JSON_ABS="$(resolve_path "${REPO_ROOT}" "${OUTPUT_JSON}")"
OUTPUT_MD_ABS="$(resolve_path "${REPO_ROOT}" "${OUTPUT_MD}")"

if [[ ! -f "${BASELINE_JSON_ABS}" ]]; then
  fail "baseline json not found: ${BASELINE_JSON_ABS}"
fi
if [[ ! -f "${CURRENT_JSON_ABS}" ]]; then
  fail "current json not found: ${CURRENT_JSON_ABS}"
fi

mkdir -p "$(dirname "${OUTPUT_JSON_ABS}")"
mkdir -p "$(dirname "${OUTPUT_MD_ABS}")"

python3 - \
  "${BASELINE_JSON_ABS}" \
  "${CURRENT_JSON_ABS}" \
  "${THRESHOLD}" \
  "${GENERATED_AT}" \
  "${OUTPUT_JSON_ABS}" \
  "${OUTPUT_MD_ABS}" <<'PY'
import json
import pathlib
import sys

(
    baseline_json_path,
    current_json_path,
    threshold_raw,
    generated_at,
    output_json_path,
    output_md_path,
) = sys.argv[1:]

baseline_payload = json.loads(pathlib.Path(baseline_json_path).read_text(encoding="utf-8"))
current_payload = json.loads(pathlib.Path(current_json_path).read_text(encoding="utf-8"))
threshold = int(threshold_raw)

baseline_by_crate = {row["crate"]: row for row in baseline_payload.get("crates", [])}
current_by_crate = {row["crate"]: row for row in current_payload.get("crates", [])}

all_crates = sorted(set(baseline_by_crate) | set(current_by_crate))
crate_deltas = []
for crate in all_crates:
    baseline_row = baseline_by_crate.get(crate, {})
    current_row = current_by_crate.get(crate, {})
    baseline_markers = int(baseline_row.get("markers", 0))
    current_markers = int(current_row.get("markers", 0))
    crate_deltas.append(
        {
            "crate": crate,
            "baseline_markers": baseline_markers,
            "current_markers": current_markers,
            "delta_markers": current_markers - baseline_markers,
            "baseline_files_scanned": int(baseline_row.get("files_scanned", 0)),
            "current_files_scanned": int(current_row.get("files_scanned", 0)),
        }
    )

crate_deltas.sort(key=lambda row: row["crate"])

baseline_total = int(baseline_payload.get("total_markers", 0))
current_total = int(current_payload.get("total_markers", 0))
delta_total = current_total - baseline_total
remaining = max(0, threshold - current_total)
meets_threshold = current_total >= threshold

top_positive = sorted(
    [row for row in crate_deltas if row["delta_markers"] > 0],
    key=lambda row: (-row["delta_markers"], row["crate"]),
)[:10]
top_negative = sorted(
    [row for row in crate_deltas if row["delta_markers"] < 0],
    key=lambda row: (row["delta_markers"], row["crate"]),
)[:10]

payload = {
    "schema_version": 1,
    "generated_at": generated_at,
    "threshold_markers": threshold,
    "baseline_artifact": baseline_json_path,
    "current_artifact": current_json_path,
    "baseline_total_markers": baseline_total,
    "current_total_markers": current_total,
    "delta_total_markers": delta_total,
    "remaining_to_threshold": remaining,
    "meets_threshold": meets_threshold,
    "crate_deltas": crate_deltas,
    "top_positive_deltas": top_positive,
    "top_negative_deltas": top_negative,
}

pathlib.Path(output_json_path).write_text(
    json.dumps(payload, indent=2) + "\n",
    encoding="utf-8",
)

status = "PASS" if meets_threshold else "FAIL"
lines = [
    "# M23 Rustdoc Marker Threshold Verification",
    "",
    f"Generated at: {generated_at}",
    "",
    "## Summary",
    "",
    f"- Threshold markers: `{threshold}`",
    f"- Baseline total markers: `{baseline_total}`",
    f"- Current total markers: `{current_total}`",
    f"- Delta markers: `{delta_total:+d}`",
    f"- Remaining to threshold: `{remaining}`",
    f"- Gate status: `{status}`",
    "",
    "## Per-Crate Delta Breakdown",
    "",
    "| Crate | Baseline | Current | Delta |",
    "| --- | ---: | ---: | ---: |",
]
for row in crate_deltas:
    lines.append(
        f"| {row['crate']} | {row['baseline_markers']} | "
        f"{row['current_markers']} | {row['delta_markers']:+d} |"
    )
lines.extend(
    [
        "",
        "## Reproduction Commands",
        "",
        "```bash",
        "scripts/dev/rustdoc-marker-count.sh \\",
        "  --repo-root . \\",
        "  --scan-root crates \\",
        "  --output-json tasks/reports/m23-rustdoc-marker-count.json \\",
        "  --output-md tasks/reports/m23-rustdoc-marker-count.md",
        "",
        "scripts/dev/rustdoc-marker-threshold-verify.sh \\",
        "  --repo-root . \\",
        "  --baseline-json tasks/reports/m23-rustdoc-marker-count-baseline.json \\",
        "  --current-json tasks/reports/m23-rustdoc-marker-count.json \\",
        "  --threshold 3000 \\",
        "  --output-json tasks/reports/m23-rustdoc-marker-threshold-verify.json \\",
        "  --output-md tasks/reports/m23-rustdoc-marker-threshold-verify.md",
        "```",
        "",
    ]
)
pathlib.Path(output_md_path).write_text("\n".join(lines), encoding="utf-8")
PY

current_total_markers="$(jq -r '.current_total_markers' "${OUTPUT_JSON_ABS}")"
remaining_to_threshold="$(jq -r '.remaining_to_threshold' "${OUTPUT_JSON_ABS}")"
meets_threshold="$(jq -r '.meets_threshold' "${OUTPUT_JSON_ABS}")"

log_info "rustdoc threshold verify: current=${current_total_markers} threshold=${THRESHOLD} meets=${meets_threshold} remaining=${remaining_to_threshold}"
log_info "json_artifact: ${OUTPUT_JSON_ABS}"
log_info "md_artifact: ${OUTPUT_MD_ABS}"

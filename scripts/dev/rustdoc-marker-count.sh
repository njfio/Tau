#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEFAULT_REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

REPO_ROOT="${DEFAULT_REPO_ROOT}"
SCAN_ROOT="crates"
OUTPUT_JSON="${DEFAULT_REPO_ROOT}/tasks/reports/m23-rustdoc-marker-count.json"
OUTPUT_MD="${DEFAULT_REPO_ROOT}/tasks/reports/m23-rustdoc-marker-count.md"
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
QUIET_MODE="false"

usage() {
  cat <<'EOF'
Usage: rustdoc-marker-count.sh [options]

Count rustdoc markers (`///`, `//!`) across crates and emit per-crate totals.

Options:
  --repo-root <path>      Repository root (default: auto-detected).
  --scan-root <path>      Scan root relative to repo root (default: crates).
  --output-json <path>    Output JSON artifact path (default: tasks/reports/m23-rustdoc-marker-count.json).
  --output-md <path>      Output Markdown artifact path (default: tasks/reports/m23-rustdoc-marker-count.md).
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
    --scan-root)
      SCAN_ROOT="$2"
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

SCAN_ROOT_ABS="$(resolve_path "${REPO_ROOT}" "${SCAN_ROOT}")"
if [[ ! -d "${SCAN_ROOT_ABS}" ]]; then
  fail "scan root not found: ${SCAN_ROOT_ABS}"
fi

OUTPUT_JSON_ABS="$(resolve_path "${REPO_ROOT}" "${OUTPUT_JSON}")"
OUTPUT_MD_ABS="$(resolve_path "${REPO_ROOT}" "${OUTPUT_MD}")"
mkdir -p "$(dirname "${OUTPUT_JSON_ABS}")"
mkdir -p "$(dirname "${OUTPUT_MD_ABS}")"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

count_json_path="${tmp_dir}/count.json"

python3 - \
  "${REPO_ROOT}" \
  "${SCAN_ROOT_ABS}" \
  "${SCAN_ROOT}" \
  "${GENERATED_AT}" \
  "${count_json_path}" \
  "${OUTPUT_MD_ABS}" <<'PY'
import json
import pathlib
import re
import sys

(
    repo_root,
    scan_root_abs,
    scan_root_rel,
    generated_at,
    count_json_path,
    output_md_path,
) = sys.argv[1:]

repo = pathlib.Path(repo_root)
scan_root = pathlib.Path(scan_root_abs)
marker_pattern = re.compile(r"^\s*(///|//!)")

crate_reports = []
for crate_dir in sorted(scan_root.iterdir()):
    if not crate_dir.is_dir():
        continue
    src_dir = crate_dir / "src"
    if not src_dir.is_dir():
        continue
    rust_files = sorted(src_dir.rglob("*.rs"))
    if not rust_files:
        continue
    marker_count = 0
    for rust_file in rust_files:
        with rust_file.open(encoding="utf-8") as handle:
            for line in handle:
                if marker_pattern.match(line):
                    marker_count += 1
    crate_reports.append(
        {
            "crate": crate_dir.name,
            "markers": marker_count,
            "files_scanned": len(rust_files),
        }
    )

crate_reports.sort(key=lambda item: item["crate"])
total_markers = sum(item["markers"] for item in crate_reports)

payload = {
    "schema_version": 1,
    "generated_at": generated_at,
    "repo_root": str(repo),
    "scan_root": scan_root_rel,
    "total_markers": total_markers,
    "crates": crate_reports,
}

count_json = pathlib.Path(count_json_path)
count_json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

md_lines = [
    "# M23 Rustdoc Marker Count",
    "",
    f"Generated at: {generated_at}",
    "",
    "## Summary",
    "",
    f"- Scan root: `{scan_root_rel}`",
    f"- Total markers: `{total_markers}`",
    f"- Crates scanned: `{len(crate_reports)}`",
    "",
    "## Per-Crate Breakdown",
    "",
    "| Crate | Markers | Files Scanned |",
    "| --- | ---: | ---: |",
]
for item in crate_reports:
    md_lines.append(f"| {item['crate']} | {item['markers']} | {item['files_scanned']} |")
md_lines.append("")
md_lines.append("## Reproduction Command")
md_lines.append("")
md_lines.append(
    "```bash\n"
    "scripts/dev/rustdoc-marker-count.sh \\\n"
    "  --repo-root . \\\n"
    "  --scan-root crates \\\n"
    "  --output-json tasks/reports/m23-rustdoc-marker-count.json \\\n"
    "  --output-md tasks/reports/m23-rustdoc-marker-count.md\n"
    "```"
)
md_lines.append("")
pathlib.Path(output_md_path).write_text("\n".join(md_lines), encoding="utf-8")
PY

cp "${count_json_path}" "${OUTPUT_JSON_ABS}"

total_markers="$(jq -r '.total_markers' "${OUTPUT_JSON_ABS}")"
crate_count="$(jq -r '.crates | length' "${OUTPUT_JSON_ABS}")"
per_crate_summary="$(jq -r '.crates | map("\(.crate)=\(.markers)") | join(", ")' "${OUTPUT_JSON_ABS}")"

log_info "rustdoc marker count: total_markers=${total_markers} crates=${crate_count} scan_root=${SCAN_ROOT}"
log_info "per_crate: ${per_crate_summary}"
log_info "json_artifact: ${OUTPUT_JSON_ABS}"
log_info "md_artifact: ${OUTPUT_MD_ABS}"

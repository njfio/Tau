#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

POLICY_JSON="${REPO_ROOT}/tasks/policies/hierarchy-graph-publication-policy.json"
GRAPH_JSON="${REPO_ROOT}/tasks/reports/issue-hierarchy-graph.json"
GRAPH_MD="${REPO_ROOT}/tasks/reports/issue-hierarchy-graph.md"
HISTORY_DIR="${REPO_ROOT}/tasks/reports/issue-hierarchy-history"
RETENTION_DAYS=""
NOW_UTC=""
QUIET_MODE="false"

usage() {
  cat <<'USAGE'
Usage: hierarchy-graph-publish.sh [options]

Publish hierarchy graph snapshots and enforce retention.

Options:
  --policy-json <path>        Publication policy JSON path
  --graph-json <path>         Input graph JSON path
  --graph-md <path>           Input graph Markdown path
  --history-dir <path>        Snapshot history directory
  --retention-days <days>     Retention window in days (overrides policy)
  --now-utc <ISO8601>         Deterministic current UTC timestamp for tests
  --quiet                     Suppress informational output
  --help                      Show this help
USAGE
}

log_info() {
  if [[ "${QUIET_MODE}" != "true" ]]; then
    echo "$@"
  fi
}

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "error: required command '${name}' not found" >&2
    exit 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --policy-json)
      POLICY_JSON="$2"
      shift 2
      ;;
    --graph-json)
      GRAPH_JSON="$2"
      shift 2
      ;;
    --graph-md)
      GRAPH_MD="$2"
      shift 2
      ;;
    --history-dir)
      HISTORY_DIR="$2"
      shift 2
      ;;
    --retention-days)
      RETENTION_DAYS="$2"
      shift 2
      ;;
    --now-utc)
      NOW_UTC="$2"
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
      echo "error: unknown option '$1'" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_cmd python3
require_cmd jq

if [[ ! -f "${POLICY_JSON}" ]]; then
  echo "error: publication policy not found: ${POLICY_JSON}" >&2
  exit 1
fi

if [[ ! -f "${GRAPH_JSON}" ]]; then
  echo "error: graph JSON input not found: ${GRAPH_JSON}" >&2
  exit 1
fi

if [[ ! -f "${GRAPH_MD}" ]]; then
  echo "error: graph Markdown input not found: ${GRAPH_MD}" >&2
  exit 1
fi

if [[ -z "${RETENTION_DAYS}" ]]; then
  RETENTION_DAYS="$(jq -r '.retention_days // empty' "${POLICY_JSON}")"
fi

if ! [[ "${RETENTION_DAYS}" =~ ^[0-9]+$ ]] || [[ "${RETENTION_DAYS}" -lt 1 ]]; then
  echo "error: retention days must be an integer >= 1" >&2
  exit 1
fi

mkdir -p "${HISTORY_DIR}"

python3 - \
  "${POLICY_JSON}" \
  "${GRAPH_JSON}" \
  "${GRAPH_MD}" \
  "${HISTORY_DIR}" \
  "${RETENTION_DAYS}" \
  "${NOW_UTC}" \
  "${QUIET_MODE}" <<'PY'
from __future__ import annotations

import json
import shutil
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

(
    policy_path,
    graph_json_path,
    graph_md_path,
    history_dir_path,
    retention_days_raw,
    now_utc_raw,
    quiet_mode,
) = sys.argv[1:]


def log(message: str) -> None:
    if quiet_mode == "true":
        return
    print(message)


def parse_utc(value: str) -> datetime:
    candidate = value.strip()
    if candidate.endswith("Z"):
        candidate = candidate[:-1] + "+00:00"
    dt = datetime.fromisoformat(candidate)
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(timezone.utc)


policy = json.loads(Path(policy_path).read_text(encoding="utf-8"))
graph = json.loads(Path(graph_json_path).read_text(encoding="utf-8"))

retention_days = int(retention_days_raw)

history_dir = Path(history_dir_path)
history_dir.mkdir(parents=True, exist_ok=True)

policy_id = str(policy.get("policy_id", "hierarchy-graph-publication-policy"))
artifacts = policy.get("artifacts", {}) if isinstance(policy.get("artifacts"), dict) else {}
json_filename = str(artifacts.get("json_filename", "issue-hierarchy-graph.json"))
markdown_filename = str(artifacts.get("markdown_filename", "issue-hierarchy-graph.md"))

history = policy.get("history", {}) if isinstance(policy.get("history"), dict) else {}
index_json_name = str(history.get("index_json", "index.json"))
index_md_name = str(history.get("index_markdown", "index.md"))

root_issue = graph.get("root_issue_number")
if not isinstance(root_issue, int) or root_issue <= 0:
    raise SystemExit("error: graph JSON must include a positive integer root_issue_number")

generated_at_raw = graph.get("generated_at")
if isinstance(generated_at_raw, str) and generated_at_raw.strip():
    generated_at = parse_utc(generated_at_raw)
else:
    generated_at = datetime.now(timezone.utc)

if now_utc_raw.strip():
    now_utc = parse_utc(now_utc_raw)
else:
    now_utc = datetime.now(timezone.utc)

timestamp = generated_at.strftime("%Y%m%dT%H%M%SZ")
snapshot_id = f"{timestamp}-root{root_issue}"
snapshot_dir = history_dir / snapshot_id
snapshot_dir.mkdir(parents=True, exist_ok=True)

snapshot_json_path = snapshot_dir / json_filename
snapshot_md_path = snapshot_dir / markdown_filename
shutil.copy2(graph_json_path, snapshot_json_path)
shutil.copy2(graph_md_path, snapshot_md_path)

index_json_path = history_dir / index_json_name
if index_json_path.exists():
    existing_index = json.loads(index_json_path.read_text(encoding="utf-8"))
else:
    existing_index = {
        "schema_version": 1,
        "policy_id": policy_id,
        "retention_days": retention_days,
        "snapshots": [],
    }

existing_snapshots = existing_index.get("snapshots")
if not isinstance(existing_snapshots, list):
    existing_snapshots = []

entries_by_id: dict[str, dict[str, Any]] = {}
for entry in existing_snapshots:
    if not isinstance(entry, dict):
        continue
    sid = entry.get("snapshot_id")
    if isinstance(sid, str) and sid:
        entries_by_id[sid] = entry

current_entry = {
    "snapshot_id": snapshot_id,
    "generated_at": generated_at.replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "root_issue_number": root_issue,
    "json_path": str(snapshot_json_path.relative_to(history_dir)),
    "markdown_path": str(snapshot_md_path.relative_to(history_dir)),
}
entries_by_id[snapshot_id] = current_entry

cutoff = now_utc - timedelta(days=retention_days)
pruned_snapshot_ids: list[str] = []
kept_entries: list[dict[str, Any]] = []
for sid, entry in entries_by_id.items():
    generated_value = entry.get("generated_at")
    keep_entry = sid == snapshot_id
    if not keep_entry:
        try:
            if isinstance(generated_value, str):
                keep_entry = parse_utc(generated_value) >= cutoff
            else:
                keep_entry = True
        except Exception:
            keep_entry = True
    if keep_entry:
        kept_entries.append(entry)
    else:
        pruned_snapshot_ids.append(sid)

for sid in pruned_snapshot_ids:
    candidate = history_dir / sid
    if candidate.exists() and candidate.is_dir():
        shutil.rmtree(candidate)

kept_entries.sort(key=lambda entry: (entry.get("generated_at", ""), entry.get("snapshot_id", "")), reverse=True)

index_payload = {
    "schema_version": 1,
    "policy_id": policy_id,
    "retention_days": retention_days,
    "updated_at": now_utc.replace(microsecond=0).isoformat().replace("+00:00", "Z"),
    "latest_snapshot_id": snapshot_id,
    "snapshots": kept_entries,
}
index_json_path.write_text(json.dumps(index_payload, indent=2) + "\n", encoding="utf-8")

index_lines = [
    "# Hierarchy Graph Snapshot Index",
    "",
    f"- Policy: `{policy_id}`",
    f"- Retention days: `{retention_days}`",
    f"- Updated at (UTC): `{index_payload['updated_at']}`",
    f"- Latest snapshot: `{snapshot_id}`",
    "",
    "## Snapshots",
    "",
]

if kept_entries:
    for entry in kept_entries:
        index_lines.append(
            "- "
            f"`{entry['snapshot_id']}` | generated_at=`{entry.get('generated_at', 'unknown')}` "
            f"| root_issue=`#{entry.get('root_issue_number', 'unknown')}` "
            f"| json=`{entry.get('json_path', '')}` "
            f"| markdown=`{entry.get('markdown_path', '')}`"
        )
else:
    index_lines.append("- none")

(history_dir / index_md_name).write_text("\n".join(index_lines) + "\n", encoding="utf-8")

log(
    "[hierarchy-graph-publish] "
    f"snapshot={snapshot_id} "
    f"retention_days={retention_days} "
    f"kept={len(kept_entries)} "
    f"pruned={len(pruned_snapshot_ids)}"
)
PY

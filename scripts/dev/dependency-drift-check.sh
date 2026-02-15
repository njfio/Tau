#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

MATRIX_JSON="${REPO_ROOT}/tasks/reports/m21-validation-matrix.json"
POLICY_JSON="${REPO_ROOT}/tasks/policies/issue-hierarchy-drift-rules.json"
REPO_SLUG=""
MODE="strict"
SKIP_PARENT_FETCH="false"
FIXTURE_PARENT_ISSUES_JSON=""
OUTPUT_JSON=""
SUMMARY_FILE=""
QUIET_MODE="false"

usage() {
  cat <<'EOF'
Usage: dependency-drift-check.sh [options]

Detect hierarchy/orphan dependency drift across roadmap issues using the
validation matrix plus hierarchy rule policy.

Options:
  --matrix-json <path>                 Validation matrix JSON input.
  --policy-json <path>                 Hierarchy drift policy JSON input.
  --repo <owner/name>                  Repository slug for parent issue lookups.
  --mode <strict|dry-run>              Enforcement mode (default: strict).
  --skip-parent-fetch                  Do not call GitHub API for missing parent issue records.
  --fixture-parent-issues-json <path>  Fixture parent issue metadata for deterministic local tests.
  --output-json <path>                 Optional JSON report output.
  --summary-file <path>                Optional markdown summary append output.
  --quiet                              Suppress informational output.
  --help                               Show this help text.
EOF
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
    --matrix-json)
      MATRIX_JSON="$2"
      shift 2
      ;;
    --policy-json)
      POLICY_JSON="$2"
      shift 2
      ;;
    --repo)
      REPO_SLUG="$2"
      shift 2
      ;;
    --mode)
      MODE="$2"
      shift 2
      ;;
    --skip-parent-fetch)
      SKIP_PARENT_FETCH="true"
      shift
      ;;
    --fixture-parent-issues-json)
      FIXTURE_PARENT_ISSUES_JSON="$2"
      shift 2
      ;;
    --output-json)
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --summary-file)
      SUMMARY_FILE="$2"
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

if [[ "${MODE}" != "strict" && "${MODE}" != "dry-run" ]]; then
  echo "error: --mode must be one of: strict, dry-run" >&2
  exit 1
fi

if [[ ! -f "${MATRIX_JSON}" ]]; then
  echo "error: matrix JSON not found: ${MATRIX_JSON}" >&2
  exit 1
fi

if [[ ! -f "${POLICY_JSON}" ]]; then
  echo "error: policy JSON not found: ${POLICY_JSON}" >&2
  exit 1
fi

if [[ -n "${FIXTURE_PARENT_ISSUES_JSON}" && ! -f "${FIXTURE_PARENT_ISSUES_JSON}" ]]; then
  echo "error: fixture parent issues JSON not found: ${FIXTURE_PARENT_ISSUES_JSON}" >&2
  exit 1
fi

require_cmd python3

if [[ "${SKIP_PARENT_FETCH}" != "true" && -z "${FIXTURE_PARENT_ISSUES_JSON}" ]]; then
  require_cmd gh
fi

if [[ -n "${OUTPUT_JSON}" ]]; then
  mkdir -p "$(dirname "${OUTPUT_JSON}")"
fi

if [[ -n "${SUMMARY_FILE}" ]]; then
  mkdir -p "$(dirname "${SUMMARY_FILE}")"
fi

python3 - \
  "${MATRIX_JSON}" \
  "${POLICY_JSON}" \
  "${REPO_SLUG}" \
  "${MODE}" \
  "${SKIP_PARENT_FETCH}" \
  "${FIXTURE_PARENT_ISSUES_JSON}" \
  "${OUTPUT_JSON}" \
  "${SUMMARY_FILE}" \
  "${QUIET_MODE}" <<'PY'
from __future__ import annotations

import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

(
    matrix_json,
    policy_json,
    repo_slug_arg,
    mode,
    skip_parent_fetch,
    fixture_parent_issues_json,
    output_json,
    summary_file,
    quiet_mode,
) = sys.argv[1:]


def load_json(path: str) -> Any:
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def normalize_labels(value: Any) -> list[str]:
    labels: list[str] = []
    if not isinstance(value, list):
        return labels
    for entry in value:
        if isinstance(entry, str):
            candidate = entry.strip()
            if candidate:
                labels.append(candidate)
        elif isinstance(entry, dict):
            name = entry.get("name")
            if isinstance(name, str) and name.strip():
                labels.append(name.strip())
    return labels


def parse_issue_number_from_url(url: str) -> int | None:
    match = re.search(r"/issues/([0-9]+)$", url)
    if not match:
        return None
    return int(match.group(1))


def issue_record(raw: dict[str, Any]) -> dict[str, Any]:
    milestone_number = None
    milestone = raw.get("milestone")
    if isinstance(milestone, dict):
        number = milestone.get("number")
        if isinstance(number, int):
            milestone_number = number
    elif isinstance(raw.get("milestone_number"), int):
        milestone_number = raw["milestone_number"]
    return {
        "number": raw.get("number"),
        "title": raw.get("title", ""),
        "state": raw.get("state", "unknown"),
        "labels": normalize_labels(raw.get("labels", [])),
        "milestone_number": milestone_number,
        "url": raw.get("url") or raw.get("html_url", ""),
        "parent_issue_url": raw.get("parent_issue_url"),
    }


def first_hierarchy_label(labels: set[str], declared_type: Any, hierarchy_labels: list[str]) -> str | None:
    if isinstance(declared_type, str) and declared_type in hierarchy_labels:
        return declared_type
    for candidate in hierarchy_labels:
        if candidate in labels:
            return candidate
    return None


def log(message: str) -> None:
    if quiet_mode == "true":
        return
    print(message)


matrix = load_json(matrix_json)
if not isinstance(matrix, dict):
    raise SystemExit("error: matrix JSON must decode to an object")
policy = load_json(policy_json)
if not isinstance(policy, dict):
    raise SystemExit("error: policy JSON must decode to an object")

issues_raw = matrix.get("issues", [])
if not isinstance(issues_raw, list):
    raise SystemExit("error: matrix JSON must include array field 'issues'")

required_metadata = policy.get("required_metadata", {})
if not isinstance(required_metadata, dict):
    raise SystemExit("error: policy missing object field 'required_metadata'")

required_labels = required_metadata.get("required_labels", [])
hierarchy_labels = required_metadata.get("hierarchy_labels", [])
parent_rules_raw = required_metadata.get("parent_rules", [])
milestone_required = bool(required_metadata.get("milestone_required_for_hierarchy_labels", True))

if not isinstance(required_labels, list) or not all(isinstance(label, str) for label in required_labels):
    raise SystemExit("error: policy required_metadata.required_labels must be array[string]")
if not isinstance(hierarchy_labels, list) or not all(
    isinstance(label, str) for label in hierarchy_labels
):
    raise SystemExit("error: policy required_metadata.hierarchy_labels must be array[string]")
if not isinstance(parent_rules_raw, list):
    raise SystemExit("error: policy required_metadata.parent_rules must be an array")

severity_by_condition: dict[str, str] = {}
for entry in policy.get("orphan_conditions", []) + policy.get("drift_conditions", []):
    if not isinstance(entry, dict):
        continue
    condition_id = entry.get("id")
    severity = entry.get("severity")
    if isinstance(condition_id, str) and isinstance(severity, str):
        severity_by_condition[condition_id] = severity

remediation_by_condition: dict[str, list[str]] = {}
for entry in policy.get("remediation", []):
    if not isinstance(entry, dict):
        continue
    condition_id = entry.get("condition_id")
    steps = entry.get("steps", [])
    if not isinstance(condition_id, str) or not isinstance(steps, list):
        continue
    remediation_steps = [step for step in steps if isinstance(step, str) and step.strip()]
    remediation_by_condition[condition_id] = remediation_steps

parent_rules: dict[str, set[str]] = {}
for rule in parent_rules_raw:
    if not isinstance(rule, dict):
        continue
    child_label = rule.get("child_label")
    allowed_parent_labels = rule.get("allowed_parent_labels", [])
    if not isinstance(child_label, str) or not isinstance(allowed_parent_labels, list):
        continue
    parent_rules[child_label] = {
        value for value in allowed_parent_labels if isinstance(value, str) and value.strip()
    }

repository = repo_slug_arg.strip()
if not repository:
    matrix_repository = matrix.get("repository")
    if isinstance(matrix_repository, str) and matrix_repository.strip():
        repository = matrix_repository.strip()
if not repository:
    repository = "njfio/Tau"

issues: list[dict[str, Any]] = []
for raw in issues_raw:
    if not isinstance(raw, dict):
        continue
    record = issue_record(raw)
    if not isinstance(record.get("number"), int):
        continue
    issues.append(record)

issues_by_number = {issue["number"]: issue for issue in issues}

fixture_parents_by_number: dict[int, dict[str, Any]] = {}
if fixture_parent_issues_json:
    fixture_data = load_json(fixture_parent_issues_json)
    fixture_items = fixture_data
    if isinstance(fixture_data, dict):
        if isinstance(fixture_data.get("issues"), list):
            fixture_items = fixture_data["issues"]
    if not isinstance(fixture_items, list):
        raise SystemExit("error: fixture parent issues JSON must decode to array or object.issues")
    for raw in fixture_items:
        if not isinstance(raw, dict):
            continue
        record = issue_record(raw)
        if isinstance(record.get("number"), int):
            fixture_parents_by_number[record["number"]] = record

remote_parent_cache: dict[int, dict[str, Any]] = {}


def fetch_parent_issue(number: int) -> tuple[dict[str, Any] | None, str]:
    if number in issues_by_number:
        return issues_by_number[number], "matrix"
    if number in fixture_parents_by_number:
        return fixture_parents_by_number[number], "fixture"
    if skip_parent_fetch == "true":
        return None, "skipped"
    if number in remote_parent_cache:
        return remote_parent_cache[number], "remote-cache"
    completed = subprocess.run(
        ["gh", "api", f"repos/{repository}/issues/{number}"],
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        return None, "not_found"
    try:
        payload = json.loads(completed.stdout)
    except json.JSONDecodeError:
        return None, "decode_error"
    if not isinstance(payload, dict):
        return None, "decode_error"
    record = issue_record(payload)
    if not isinstance(record.get("number"), int):
        return None, "decode_error"
    remote_parent_cache[number] = record
    return record, "remote"


child_milestone_number = None
milestone = matrix.get("milestone")
if isinstance(milestone, dict):
    milestone_number = milestone.get("number")
    if isinstance(milestone_number, int):
        child_milestone_number = milestone_number

findings: list[dict[str, Any]] = []


def add_finding(
    *,
    condition_id: str,
    issue: dict[str, Any],
    message: str,
    parent_issue_number: int | None = None,
) -> None:
    severity = severity_by_condition.get(condition_id, "warning")
    findings.append(
        {
            "severity": severity,
            "condition_id": condition_id,
            "issue_number": issue.get("number"),
            "issue_title": issue.get("title", ""),
            "parent_issue_number": parent_issue_number,
            "message": message,
            "remediation_steps": remediation_by_condition.get(condition_id, []),
        }
    )


for issue in sorted(issues, key=lambda item: item["number"]):
    labels = set(issue.get("labels", []))
    hierarchy = first_hierarchy_label(labels, issue.get("type"), hierarchy_labels)
    if hierarchy is None:
        continue

    missing_required = [label for label in required_labels if label not in labels]
    if missing_required:
        add_finding(
            condition_id="drift.missing_required_labels",
            issue=issue,
            message=f"missing required labels: {', '.join(sorted(missing_required))}",
        )

    if milestone_required and child_milestone_number is None:
        add_finding(
            condition_id="drift.missing_milestone",
            issue=issue,
            message="no milestone number found in matrix source",
        )

    parent_issue_url = issue.get("parent_issue_url")
    if not isinstance(parent_issue_url, str) or not parent_issue_url.strip():
        add_finding(
            condition_id="orphan.missing_parent_link",
            issue=issue,
            message="parent_issue_url is missing for hierarchy issue",
        )
        continue

    parent_issue_number = parse_issue_number_from_url(parent_issue_url)
    if parent_issue_number is None:
        add_finding(
            condition_id="orphan.parent_issue_not_found",
            issue=issue,
            parent_issue_number=None,
            message=f"parent_issue_url does not contain parsable issue id: {parent_issue_url}",
        )
        continue

    parent_issue, parent_source = fetch_parent_issue(parent_issue_number)
    if parent_issue is None:
        reason = "parent issue not found"
        if parent_source == "skipped":
            reason = "parent issue resolution skipped (--skip-parent-fetch enabled)"
        elif parent_source == "decode_error":
            reason = "parent issue response could not be decoded"
        add_finding(
            condition_id="orphan.parent_issue_not_found",
            issue=issue,
            parent_issue_number=parent_issue_number,
            message=f"{reason} (parent #{parent_issue_number})",
        )
        continue

    parent_labels = set(parent_issue.get("labels", []))
    allowed_parent_labels = parent_rules.get(hierarchy, set())
    if allowed_parent_labels and not (parent_labels & allowed_parent_labels):
        add_finding(
            condition_id="orphan.parent_label_incompatible",
            issue=issue,
            parent_issue_number=parent_issue_number,
            message=(
                f"child type '{hierarchy}' requires parent labels "
                f"{sorted(allowed_parent_labels)}; parent labels are {sorted(parent_labels)}"
            ),
        )

    parent_milestone_number = parent_issue.get("milestone_number")
    if (
        isinstance(child_milestone_number, int)
        and isinstance(parent_milestone_number, int)
        and child_milestone_number != parent_milestone_number
    ):
        add_finding(
            condition_id="drift.parent_milestone_mismatch",
            issue=issue,
            parent_issue_number=parent_issue_number,
            message=(
                f"child milestone #{child_milestone_number} differs from parent milestone "
                f"#{parent_milestone_number}"
            ),
        )

    if issue.get("state") == "open" and parent_issue.get("state") == "closed":
        add_finding(
            condition_id="drift.parent_child_state_mismatch",
            issue=issue,
            parent_issue_number=parent_issue_number,
            message=f"child issue is open while parent issue #{parent_issue_number} is closed",
        )

findings.sort(
    key=lambda item: (
        0 if item["severity"] == "error" else 1,
        int(item.get("issue_number") or 0),
        item.get("condition_id", ""),
    )
)

errors = sum(1 for finding in findings if finding.get("severity") == "error")
warnings = sum(1 for finding in findings if finding.get("severity") == "warning")
would_fail = errors > 0

report = {
    "schema_version": 1,
    "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "mode": mode,
    "source": {
        "matrix_json": str(Path(matrix_json).resolve()),
        "policy_json": str(Path(policy_json).resolve()),
        "repository": repository,
        "skip_parent_fetch": skip_parent_fetch == "true",
        "fixture_parent_issues_json": (
            str(Path(fixture_parent_issues_json).resolve())
            if fixture_parent_issues_json
            else None
        ),
    },
    "summary": {
        "issues_scanned": len(issues),
        "findings_total": len(findings),
        "errors": errors,
        "warnings": warnings,
        "would_fail": would_fail,
    },
    "findings": findings,
}

if findings:
    log(
        f"[dependency-drift-check] findings={len(findings)} "
        f"errors={errors} warnings={warnings} mode={mode}"
    )
    for finding in findings:
        severity = finding["severity"].upper()
        if mode == "dry-run" and finding["severity"] == "error":
            severity = "DRY-RUN-WOULD-FAIL"
        parent_part = ""
        if finding.get("parent_issue_number") is not None:
            parent_part = f" parent=#{finding['parent_issue_number']}"
        log(
            f"[dependency-drift-check] {severity} {finding['condition_id']} "
            f"issue=#{finding['issue_number']}{parent_part}: {finding['message']}"
        )
        for step in finding.get("remediation_steps", []):
            log(f"[dependency-drift-check] remediation: {step}")
else:
    log(
        f"[dependency-drift-check] no findings "
        f"(issues_scanned={len(issues)} mode={mode})"
    )

if mode == "dry-run" and would_fail:
    log(
        f"[dependency-drift-check] DRY-RUN would fail strict mode due to "
        f"{errors} error finding(s)."
    )

if output_json:
    output_path = Path(output_json)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

if summary_file:
    summary_path = Path(summary_file)
    summary_path.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "### Dependency Drift Check",
        f"- Mode: {mode}",
        f"- Issues scanned: {len(issues)}",
        f"- Findings: {len(findings)}",
        f"- Errors: {errors}",
        f"- Warnings: {warnings}",
        f"- Would fail strict mode: {'yes' if would_fail else 'no'}",
    ]
    if findings:
        first = findings[0]
        lines.append(
            f"- First finding: `{first['condition_id']}` on issue #{first['issue_number']}"
        )
    with summary_path.open("a", encoding="utf-8") as handle:
        handle.write("\n".join(lines))
        handle.write("\n")

if mode == "strict" and would_fail:
    raise SystemExit(1)
raise SystemExit(0)
PY

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

POLICY_FILE="${REPO_ROOT}/tasks/policies/scaffold-merge-remove-rubric.json"
OUTPUT_JSON="${REPO_ROOT}/tasks/reports/m21-scaffold-merge-remove-decision-matrix.json"
OUTPUT_MD="${REPO_ROOT}/tasks/reports/m21-scaffold-merge-remove-decision-matrix.md"
SCHEMA_PATH="${REPO_ROOT}/tasks/schemas/m21-scaffold-merge-remove-decision-matrix.schema.json"
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
FIXTURE_POLICY_JSON=""
FIXTURE_CANDIDATES_JSON=""
QUIET_MODE="false"

usage() {
  cat <<'EOF'
Usage: scaffold-merge-remove-decision-matrix.sh [options]

Generate deterministic M21 scaffold merge/remove rubric scoring artifacts.

Options:
  --repo-root <path>                Repository root (default: detected from script location).
  --policy-file <path>              Rubric policy JSON path (default: tasks/policies/scaffold-merge-remove-rubric.json).
  --output-json <path>              JSON matrix output path.
  --output-md <path>                Markdown matrix output path.
  --schema-path <path>              Schema path reference embedded in output JSON.
  --generated-at <iso>              Override generated timestamp.
  --fixture-policy-json <path>      Optional fixture policy JSON (for tests/regression checks).
  --fixture-candidates-json <path>  Optional fixture candidate JSON (for tests/regression checks).
  --quiet                           Suppress informational logs.
  --help                            Show this help text.
EOF
}

log_info() {
  if [[ "${QUIET_MODE}" != "true" ]]; then
    echo "$@"
  fi
}

resolve_path() {
  local repo_root="$1"
  local raw="$2"
  if [[ "${raw}" = /* ]]; then
    printf '%s\n' "${raw}"
  else
    printf '%s\n' "${repo_root}/${raw}"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      REPO_ROOT="$2"
      shift 2
      ;;
    --policy-file)
      POLICY_FILE="$2"
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
    --schema-path)
      SCHEMA_PATH="$2"
      shift 2
      ;;
    --generated-at)
      GENERATED_AT="$2"
      shift 2
      ;;
    --fixture-policy-json)
      FIXTURE_POLICY_JSON="$2"
      shift 2
      ;;
    --fixture-candidates-json)
      FIXTURE_CANDIDATES_JSON="$2"
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
      echo "error: unknown argument '$1'" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! command -v python3 >/dev/null 2>&1; then
  echo "error: required command 'python3' not found" >&2
  exit 1
fi

if [[ -n "${FIXTURE_POLICY_JSON}" && ! -f "${FIXTURE_POLICY_JSON}" ]]; then
  echo "error: fixture policy JSON not found: ${FIXTURE_POLICY_JSON}" >&2
  exit 1
fi
if [[ -n "${FIXTURE_CANDIDATES_JSON}" && ! -f "${FIXTURE_CANDIDATES_JSON}" ]]; then
  echo "error: fixture candidates JSON not found: ${FIXTURE_CANDIDATES_JSON}" >&2
  exit 1
fi

POLICY_FILE_ABS="$(resolve_path "${REPO_ROOT}" "${POLICY_FILE}")"
OUTPUT_JSON_ABS="$(resolve_path "${REPO_ROOT}" "${OUTPUT_JSON}")"
OUTPUT_MD_ABS="$(resolve_path "${REPO_ROOT}" "${OUTPUT_MD}")"
SCHEMA_PATH_ABS="$(resolve_path "${REPO_ROOT}" "${SCHEMA_PATH}")"

if [[ ! -f "${POLICY_FILE_ABS}" && -z "${FIXTURE_POLICY_JSON}" ]]; then
  echo "error: rubric policy not found: ${POLICY_FILE_ABS}" >&2
  exit 1
fi

mkdir -p "$(dirname "${OUTPUT_JSON_ABS}")" "$(dirname "${OUTPUT_MD_ABS}")"

python3 - \
  "${REPO_ROOT}" \
  "${POLICY_FILE_ABS}" \
  "${OUTPUT_JSON_ABS}" \
  "${OUTPUT_MD_ABS}" \
  "${SCHEMA_PATH_ABS}" \
  "${GENERATED_AT}" \
  "${FIXTURE_POLICY_JSON}" \
  "${FIXTURE_CANDIDATES_JSON}" <<'PY'
import json
import sys
from pathlib import Path
from typing import Any

(
    repo_root_raw,
    policy_file_raw,
    output_json_raw,
    output_md_raw,
    schema_path_raw,
    generated_at,
    fixture_policy_json_raw,
    fixture_candidates_json_raw,
) = sys.argv[1:]

repo_root = Path(repo_root_raw).resolve()
policy_file = Path(policy_file_raw).resolve()
output_json = Path(output_json_raw).resolve()
output_md = Path(output_md_raw).resolve()
schema_path = Path(schema_path_raw).resolve()
fixture_policy_json = Path(fixture_policy_json_raw).resolve() if fixture_policy_json_raw else None
fixture_candidates_json = (
    Path(fixture_candidates_json_raw).resolve() if fixture_candidates_json_raw else None
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"error: {message}")


def rel_to_repo(path: Path) -> str:
    if path.is_relative_to(repo_root):
        return str(path.relative_to(repo_root))
    return str(path)


def load_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        payload = json.load(handle)
    require(isinstance(payload, dict), f"{path} must decode to a JSON object")
    return payload


def default_candidates() -> list[dict[str, Any]]:
    return [
        {
            "candidate_id": "tau-browser-automation",
            "surface": "crate:tau-browser-automation",
            "owner": "tools-runtime",
            "operator_value": 2,
            "runtime_usage": 1,
            "maintenance_cost": 4,
            "test_posture": 3,
            "merge_target": None,
            "rationale": "Contract-only browser scaffolding remains low-usage and high-maintenance without live automation hooks.",
        },
        {
            "candidate_id": "tau-custom-command",
            "surface": "crate:tau-custom-command",
            "owner": "events-runtime",
            "operator_value": 3,
            "runtime_usage": 2,
            "maintenance_cost": 3,
            "test_posture": 3,
            "merge_target": None,
            "rationale": "Custom command contracts provide operator value but are not yet high-frequency runtime paths.",
        },
        {
            "candidate_id": "tau-dashboard-widget-contracts",
            "surface": "crate:tau-dashboard (contract widgets)",
            "owner": "gateway-ui",
            "operator_value": 5,
            "runtime_usage": 4,
            "maintenance_cost": 4,
            "test_posture": 4,
            "merge_target": "tau-dashboard",
            "rationale": "Widget contracts and runtime shell should converge into one maintained dashboard ownership boundary.",
        },
        {
            "candidate_id": "tau-memory-postgres-backend",
            "surface": "crate:tau-memory (postgres fixture backend)",
            "owner": "memory-runtime",
            "operator_value": 2,
            "runtime_usage": 1,
            "maintenance_cost": 4,
            "test_posture": 1,
            "merge_target": None,
            "rationale": "Postgres path is scaffold-level and duplicates active memory ownership in retained runtime surfaces.",
        },
        {
            "candidate_id": "tau-contract-runner-remnants",
            "surface": "contract-runner remnant paths across retained crates",
            "owner": "runtime-core",
            "operator_value": 1,
            "runtime_usage": 0,
            "maintenance_cost": 4,
            "test_posture": 1,
            "merge_target": None,
            "rationale": "No operator-facing value and no active runtime usage; dead remnant maintenance burden is unjustified.",
        },
        {
            "candidate_id": "tau-voice-runtime",
            "surface": "crate:tau-voice",
            "owner": "multi-channel-runtime",
            "operator_value": 2,
            "runtime_usage": 1,
            "maintenance_cost": 4,
            "test_posture": 2,
            "merge_target": None,
            "rationale": "Voice contracts are present but live speech pipeline is not yet in retained production flows.",
        },
        {
            "candidate_id": "tau-training-types",
            "surface": "crate:tau-training-types",
            "owner": "training-runtime",
            "operator_value": 4,
            "runtime_usage": 3,
            "maintenance_cost": 2,
            "test_posture": 4,
            "merge_target": None,
            "rationale": "Shared training type boundary remains stable and keeps compile-time dependencies acyclic.",
        },
        {
            "candidate_id": "tau-training-store",
            "surface": "crate:tau-training-store",
            "owner": "training-runtime",
            "operator_value": 4,
            "runtime_usage": 3,
            "maintenance_cost": 2,
            "test_posture": 4,
            "merge_target": None,
            "rationale": "Store ownership is explicit and reused by runner/tracer/trainer orchestration flows.",
        },
        {
            "candidate_id": "tau-training-tracer",
            "surface": "crate:tau-training-tracer",
            "owner": "training-runtime",
            "operator_value": 4,
            "runtime_usage": 3,
            "maintenance_cost": 2,
            "test_posture": 3,
            "merge_target": None,
            "rationale": "Tracer boundary is actively used and independently testable from runner/store concerns.",
        },
        {
            "candidate_id": "tau-training-runner",
            "surface": "crate:tau-training-runner",
            "owner": "training-runtime",
            "operator_value": 4,
            "runtime_usage": 3,
            "maintenance_cost": 2,
            "test_posture": 4,
            "merge_target": None,
            "rationale": "Runner orchestration remains a clean runtime boundary with active test coverage.",
        },
        {
            "candidate_id": "tau-training-proxy",
            "surface": "crate:tau-training-proxy",
            "owner": "training-runtime",
            "operator_value": 3,
            "runtime_usage": 2,
            "maintenance_cost": 3,
            "test_posture": 3,
            "merge_target": None,
            "rationale": "Proxy surface is optional and should remain isolated until live usage justifies deeper consolidation.",
        },
        {
            "candidate_id": "tau-trainer",
            "surface": "crate:tau-trainer",
            "owner": "training-runtime",
            "operator_value": 4,
            "runtime_usage": 3,
            "maintenance_cost": 2,
            "test_posture": 4,
            "merge_target": None,
            "rationale": "Top-level trainer lifecycle remains stable and easier to evolve as a focused crate boundary.",
        },
        {
            "candidate_id": "tau-algorithm",
            "surface": "crate:tau-algorithm",
            "owner": "training-runtime",
            "operator_value": 4,
            "runtime_usage": 3,
            "maintenance_cost": 2,
            "test_posture": 4,
            "merge_target": None,
            "rationale": "Algorithm strategy layer is independently testable and should remain decoupled from runtime plumbing.",
        },
    ]


def parse_policy(policy: dict[str, Any]) -> dict[str, Any]:
    require(int(policy.get("schema_version", 0)) == 1, "policy schema_version must be 1")
    policy_id = policy.get("policy_id")
    require(isinstance(policy_id, str) and policy_id.strip(), "policy_id must be non-empty")

    score_scale = policy.get("score_scale")
    require(isinstance(score_scale, dict), "policy score_scale must be an object")
    score_min = score_scale.get("min")
    score_max = score_scale.get("max")
    require(
        isinstance(score_min, int) and isinstance(score_max, int) and score_min <= score_max,
        "policy score_scale min/max must be integers with min <= max",
    )

    weights = policy.get("weights")
    require(isinstance(weights, dict), "policy weights must be an object")
    required_weight_keys = ["operator_value", "runtime_usage", "maintenance_cost", "test_posture"]
    for key in required_weight_keys:
        value = weights.get(key)
        require(isinstance(value, int) and value > 0, f"policy weights.{key} must be positive integer")

    thresholds = policy.get("thresholds")
    require(isinstance(thresholds, dict), "policy thresholds must be an object")
    remove_max = thresholds.get("remove_max_score")
    keep_min = thresholds.get("keep_min_score")
    keep_max = thresholds.get("keep_max_score")
    merge_min = thresholds.get("merge_min_score")
    require(
        all(isinstance(value, int) for value in [remove_max, keep_min, keep_max, merge_min]),
        "policy thresholds values must be integers",
    )
    require(remove_max < keep_min <= keep_max < merge_min, "policy thresholds ordering is invalid")

    unresolved_allowed = policy.get("unresolved_allowed")
    require(
        isinstance(unresolved_allowed, bool),
        "policy unresolved_allowed must be boolean",
    )

    return {
        "policy_id": policy_id.strip(),
        "score_min": score_min,
        "score_max": score_max,
        "weights": {key: int(weights[key]) for key in required_weight_keys},
        "thresholds": {
            "remove_max_score": remove_max,
            "keep_min_score": keep_min,
            "keep_max_score": keep_max,
            "merge_min_score": merge_min,
        },
        "unresolved_allowed": unresolved_allowed,
    }


def load_candidates() -> list[dict[str, Any]]:
    if fixture_candidates_json is None:
        return default_candidates()
    payload = load_json(fixture_candidates_json)
    candidates = payload.get("candidates")
    require(isinstance(candidates, list) and candidates, "fixture candidates must contain non-empty candidates[]")
    return candidates


def normalize_candidate(entry: dict[str, Any], index: int, policy_cfg: dict[str, Any]) -> dict[str, Any]:
    require(isinstance(entry, dict), f"candidates[{index}] must be an object")

    candidate_id = entry.get("candidate_id")
    surface = entry.get("surface")
    owner = entry.get("owner")
    rationale = entry.get("rationale")

    require(isinstance(candidate_id, str) and candidate_id.strip(), f"candidates[{index}].candidate_id must be non-empty")
    require(isinstance(surface, str) and surface.strip(), f"candidates[{index}].surface must be non-empty")
    require(isinstance(owner, str) and owner.strip(), f"candidates[{index}].owner must be non-empty")
    require(isinstance(rationale, str) and rationale.strip(), f"candidates[{index}].rationale must be non-empty")

    score_min = policy_cfg["score_min"]
    score_max = policy_cfg["score_max"]
    scores = {}
    for key in ["operator_value", "runtime_usage", "maintenance_cost", "test_posture"]:
        value = entry.get(key)
        require(
            isinstance(value, int) and score_min <= value <= score_max,
            f"candidates[{index}].{key} must be integer in [{score_min}, {score_max}]",
        )
        scores[key] = value

    weights = policy_cfg["weights"]
    effective_maintenance = score_max - scores["maintenance_cost"]
    weighted_score = (
        scores["operator_value"] * weights["operator_value"]
        + scores["runtime_usage"] * weights["runtime_usage"]
        + scores["test_posture"] * weights["test_posture"]
        + effective_maintenance * weights["maintenance_cost"]
    )

    thresholds = policy_cfg["thresholds"]
    if weighted_score <= thresholds["remove_max_score"]:
        action = "remove"
    elif thresholds["keep_min_score"] <= weighted_score <= thresholds["keep_max_score"]:
        action = "keep"
    elif weighted_score >= thresholds["merge_min_score"]:
        action = "merge"
    else:
        action = "unresolved"

    merge_target = entry.get("merge_target")
    if action == "merge":
        require(
            isinstance(merge_target, str) and merge_target.strip(),
            f"candidates[{index}].merge_target must be non-empty for merge action",
        )
        merge_target_out: str | None = merge_target.strip()
    else:
        merge_target_out = None

    return {
        "candidate_id": candidate_id.strip(),
        "surface": surface.strip(),
        "owner": owner.strip(),
        "action": action,
        "merge_target": merge_target_out,
        "weighted_score": weighted_score,
        "scores": scores,
        "rationale": rationale.strip(),
    }


selected_policy_path = fixture_policy_json or policy_file
policy_payload = load_json(selected_policy_path)
policy_cfg = parse_policy(policy_payload)

candidates = load_candidates()
normalized = [normalize_candidate(entry, index, policy_cfg) for index, entry in enumerate(candidates)]
normalized.sort(key=lambda item: item["candidate_id"])

seen = set()
for candidate in normalized:
    candidate_id = candidate["candidate_id"]
    require(candidate_id not in seen, f"duplicate candidate_id '{candidate_id}'")
    seen.add(candidate_id)

keep_count = sum(1 for item in normalized if item["action"] == "keep")
merge_count = sum(1 for item in normalized if item["action"] == "merge")
remove_count = sum(1 for item in normalized if item["action"] == "remove")
unresolved_count = sum(1 for item in normalized if item["action"] == "unresolved")

if unresolved_count > 0 and not policy_cfg["unresolved_allowed"]:
    raise SystemExit(f"error: unresolved decisions are not allowed (count={unresolved_count})")

payload = {
    "schema_version": 1,
    "generated_at": generated_at,
    "repository_root": ".",
    "schema_path": rel_to_repo(schema_path),
    "policy_path": rel_to_repo(selected_policy_path),
    "policy_id": policy_cfg["policy_id"],
    "weights": policy_cfg["weights"],
    "thresholds": policy_cfg["thresholds"],
    "summary": {
        "total_candidates": len(normalized),
        "keep_count": keep_count,
        "merge_count": merge_count,
        "remove_count": remove_count,
        "unresolved_count": unresolved_count,
    },
    "candidates": normalized,
}

output_json.parent.mkdir(parents=True, exist_ok=True)
output_md.parent.mkdir(parents=True, exist_ok=True)
output_json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

lines = [
    "# M21 Scaffold Merge/Remove Decision Matrix",
    "",
    f"- Generated: {generated_at}",
    f"- Policy: `{payload['policy_path']}`",
    "",
    "## Summary",
    "",
    "| Metric | Value |",
    "| --- | ---: |",
    f"| Total candidates | {payload['summary']['total_candidates']} |",
    f"| Keep decisions | {payload['summary']['keep_count']} |",
    f"| Merge decisions | {payload['summary']['merge_count']} |",
    f"| Remove decisions | {payload['summary']['remove_count']} |",
    f"| Unresolved decisions | {payload['summary']['unresolved_count']} |",
    "",
    "## Rubric",
    "",
    "| Criterion | Weight | Direction |",
    "| --- | ---: | --- |",
    f"| `operator_value` | {payload['weights']['operator_value']} | higher better |",
    f"| `runtime_usage` | {payload['weights']['runtime_usage']} | higher better |",
    f"| `maintenance_cost` | {payload['weights']['maintenance_cost']} | lower better |",
    f"| `test_posture` | {payload['weights']['test_posture']} | higher better |",
    "",
    "| Action | Threshold |",
    "| --- | --- |",
    f"| remove | score <= {payload['thresholds']['remove_max_score']} |",
    (
        f"| keep | {payload['thresholds']['keep_min_score']} <= score <= "
        f"{payload['thresholds']['keep_max_score']} |"
    ),
    f"| merge | score >= {payload['thresholds']['merge_min_score']} |",
    "",
    "## Decision Matrix",
    "",
    "| Candidate | Action | Merge Target | Owner | Score | Rationale |",
    "| --- | --- | --- | --- | ---: | --- |",
]

for candidate in normalized:
    merge_target = candidate["merge_target"] if candidate["merge_target"] else "-"
    lines.append(
        "| "
        f"`{candidate['candidate_id']}` | {candidate['action']} | {merge_target} | "
        f"`{candidate['owner']}` | {candidate['weighted_score']} | {candidate['rationale']} |"
    )

output_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY

log_info "wrote scaffold decision matrix artifacts:"
log_info "  JSON: ${OUTPUT_JSON_ABS}"
log_info "  Markdown: ${OUTPUT_MD_ABS}"

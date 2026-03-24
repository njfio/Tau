#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

WRAPPER_MANIFEST_JSON='[
  {
    "id": "tools-check",
    "command": "cargo check -p tau-tools --lib --target-dir target-fast",
    "use_case": "Compile tau-tools quickly for tool-layer edits"
  },
  {
    "id": "trainer-check",
    "command": "cargo check -p tau-trainer --lib --target-dir target-fast",
    "use_case": "Compile tau-trainer library surfaces for trainer/runtime edits"
  },
  {
    "id": "trainer-smoke",
    "command": "cargo test -p tau-trainer --target-dir target-fast benchmark_artifact::tests::regression_summary_gate_report_manifest_ignores_non_json_files -- --nocapture",
    "use_case": "Run targeted trainer regression smoke check"
  }
]'

usage() {
  cat <<'USAGE'
Usage: fast-lane-dev-loop.sh <subcommand> [options]

Subcommands:
  list
      Print fast-lane wrapper catalog.

  run <wrapper-id> [--dry-run]
      Execute a wrapper command (or print the command in dry-run mode).

  benchmark [options]
      Compare fast-lane measured median loop time against baseline report.

Benchmark options:
  --baseline-json <path>  Baseline JSON report path
                          (default: tasks/reports/m25-build-test-latency-baseline.json)
  --fixture-json <path>   Deterministic wrapper measurement fixture JSON
  --output-json <path>    Output JSON report path
                          (default: tasks/reports/m25-fast-lane-loop-comparison.json)
  --output-md <path>      Output Markdown report path
                          (default: tasks/reports/m25-fast-lane-loop-comparison.md)
  --generated-at <iso>    Generated timestamp override (ISO-8601 UTC)
  --repo <owner/name>     Repository slug override
  --iterations <n>        Live mode measurement iterations per wrapper (default: 1)
  --no-warmup             Skip unmeasured warm-up execution in live mode
  --quiet                 Suppress informational output

Fixture format:
{
  "wrappers": [
    {
      "id": "tools-check",
      "command": "cargo check -p tau-tools --lib --target-dir target-fast",
      "duration_ms": 840,
      "exit_code": 0,
      "use_case": "tools compile feedback"
    }
  ]
}
USAGE
}

require_cmd() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "error: required command '${name}' not found" >&2
    exit 1
  fi
}

log_info() {
  local quiet_mode="$1"
  shift
  if [[ "${quiet_mode}" != "true" ]]; then
    echo "$@"
  fi
}

wrapper_exists() {
  local wrapper_id="$1"
  jq -e --arg id "${wrapper_id}" '.[] | select(.id == $id)' <<<"${WRAPPER_MANIFEST_JSON}" >/dev/null
}

wrapper_command() {
  local wrapper_id="$1"
  jq -r --arg id "${wrapper_id}" '.[] | select(.id == $id) | .command' <<<"${WRAPPER_MANIFEST_JSON}"
}

list_wrappers() {
  echo -e "id\tcommand\tuse_case"
  jq -r '.[] | "\(.id)\t\(.command)\t\(.use_case)"' <<<"${WRAPPER_MANIFEST_JSON}"
}

run_wrapper() {
  local wrapper_id="$1"
  local dry_run="$2"

  if ! wrapper_exists "${wrapper_id}"; then
    echo "error: unknown wrapper id '${wrapper_id}'" >&2
    exit 1
  fi

  local command
  command="$(wrapper_command "${wrapper_id}")"

  if [[ "${dry_run}" == "true" ]]; then
    echo "${command}"
    return 0
  fi

  /bin/bash -lc "${command}"
}

benchmark() {
  local baseline_json="${REPO_ROOT}/tasks/reports/m25-build-test-latency-baseline.json"
  local fixture_json=""
  local output_json="${REPO_ROOT}/tasks/reports/m25-fast-lane-loop-comparison.json"
  local output_md="${REPO_ROOT}/tasks/reports/m25-fast-lane-loop-comparison.md"
  local generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  local repo_slug=""
  local iterations="1"
  local warmup="true"
  local quiet_mode="false"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --baseline-json)
        baseline_json="$2"
        shift 2
        ;;
      --fixture-json)
        fixture_json="$2"
        shift 2
        ;;
      --output-json)
        output_json="$2"
        shift 2
        ;;
      --output-md)
        output_md="$2"
        shift 2
        ;;
      --generated-at)
        generated_at="$2"
        shift 2
        ;;
      --repo)
        repo_slug="$2"
        shift 2
        ;;
      --iterations)
        iterations="$2"
        shift 2
        ;;
      --no-warmup)
        warmup="false"
        shift
        ;;
      --quiet)
        quiet_mode="true"
        shift
        ;;
      *)
        echo "error: unknown benchmark argument '$1'" >&2
        exit 1
        ;;
    esac
  done

  if [[ ! -f "${baseline_json}" ]]; then
    echo "error: baseline report not found: ${baseline_json}" >&2
    exit 1
  fi
  if [[ -n "${fixture_json}" && ! -f "${fixture_json}" ]]; then
    echo "error: fixture JSON not found: ${fixture_json}" >&2
    exit 1
  fi
  if ! [[ "${iterations}" =~ ^[0-9]+$ ]]; then
    echo "error: --iterations must be a non-negative integer" >&2
    exit 1
  fi
  if (( iterations <= 0 )); then
    echo "error: --iterations must be greater than zero" >&2
    exit 1
  fi

  mkdir -p "$(dirname "${output_json}")"
  mkdir -p "$(dirname "${output_md}")"

  python3 - \
    "${baseline_json}" \
    "${fixture_json}" \
    "${output_json}" \
    "${output_md}" \
    "${generated_at}" \
    "${repo_slug}" \
    "${iterations}" \
    "${warmup}" \
    "${quiet_mode}" \
    "${WRAPPER_MANIFEST_JSON}" <<'PY'
from __future__ import annotations

import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

(
    baseline_path_raw,
    fixture_path_raw,
    output_json_raw,
    output_md_raw,
    generated_at_raw,
    repo_slug_raw,
    iterations_raw,
    warmup_raw,
    quiet_mode_raw,
    wrapper_manifest_raw,
) = sys.argv[1:]

baseline_path = Path(baseline_path_raw)
fixture_path = Path(fixture_path_raw) if fixture_path_raw else None
output_json_path = Path(output_json_raw)
output_md_path = Path(output_md_raw)
iterations = int(iterations_raw)
warmup_enabled = warmup_raw == "true"
quiet_mode = quiet_mode_raw == "true"


def log(message: str) -> None:
    if not quiet_mode:
        print(message)


def fail(message: str) -> None:
    raise SystemExit(f"error: {message}")


def parse_iso8601_utc(value: str) -> datetime:
    candidate = value.strip()
    if not candidate:
        fail("generated-at value must not be empty")
    if candidate.endswith("Z"):
        candidate = candidate[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(candidate)
    except ValueError as exc:
        fail(f"invalid --generated-at timestamp: {value} ({exc})")
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc).replace(microsecond=0)


def iso_utc(dt: datetime) -> str:
    return dt.astimezone(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def detect_repository_slug(explicit_repo: str, baseline_repo: str | None) -> str:
    candidate = explicit_repo.strip()
    if candidate:
        return candidate
    if baseline_repo:
        return baseline_repo
    try:
        completed = subprocess.run(
            ["gh", "repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"],
            text=True,
            capture_output=True,
            check=False,
        )
        if completed.returncode == 0:
            value = completed.stdout.strip()
            if value:
                return value
    except Exception:
        pass
    return f"local/{Path.cwd().name}"


def median(values: list[int]) -> int:
    ordered = sorted(values)
    return ordered[(len(ordered) - 1) // 2]


def parse_wrappers_from_fixture(path: Path) -> list[dict[str, Any]]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"unable to parse fixture JSON: {exc}")

    wrappers = payload.get("wrappers") if isinstance(payload, dict) else None
    if not isinstance(wrappers, list):
        fail("fixture field 'wrappers' must be an array")

    measured: list[dict[str, Any]] = []
    for index, row in enumerate(wrappers):
        if not isinstance(row, dict):
            fail(f"fixture wrappers[{index}] must be an object")
        for required_key in ["id", "command", "duration_ms", "exit_code", "use_case"]:
            if required_key not in row:
                fail(f"fixture wrappers[{index}] missing {required_key}")
        if not isinstance(row["id"], str) or not row["id"].strip():
            fail(f"fixture wrappers[{index}] id must be non-empty string")
        if not isinstance(row["command"], str) or not row["command"].strip():
            fail(f"fixture wrappers[{index}] command must be non-empty string")
        if not isinstance(row["use_case"], str) or not row["use_case"].strip():
            fail(f"fixture wrappers[{index}] use_case must be non-empty string")
        if not isinstance(row["duration_ms"], int):
            fail(f"fixture wrappers[{index}] duration_ms must be an integer")
        if not isinstance(row["exit_code"], int):
            fail(f"fixture wrappers[{index}] exit_code must be an integer")

        measured.append(
            {
                "id": row["id"].strip(),
                "command": row["command"].strip(),
                "use_case": row["use_case"].strip(),
                "duration_ms": row["duration_ms"],
                "exit_code": row["exit_code"],
            }
        )
    return measured


def parse_wrapper_manifest(raw_json: str) -> list[dict[str, str]]:
    try:
        payload = json.loads(raw_json)
    except json.JSONDecodeError as exc:
        fail(f"unable to parse wrapper manifest JSON: {exc}")
    if not isinstance(payload, list) or len(payload) == 0:
        fail("wrapper manifest must be a non-empty array")

    manifest: list[dict[str, str]] = []
    for index, item in enumerate(payload):
        if not isinstance(item, dict):
            fail(f"wrapper manifest[{index}] must be an object")
        wrapper_id = item.get("id")
        command = item.get("command")
        use_case = item.get("use_case")
        if not isinstance(wrapper_id, str) or not wrapper_id.strip():
            fail(f"wrapper manifest[{index}] id must be non-empty string")
        if not isinstance(command, str) or not command.strip():
            fail(f"wrapper manifest[{index}] command must be non-empty string")
        if not isinstance(use_case, str) or not use_case.strip():
            fail(f"wrapper manifest[{index}] use_case must be non-empty string")
        manifest.append(
            {
                "id": wrapper_id.strip(),
                "command": command.strip(),
                "use_case": use_case.strip(),
            }
        )
    return manifest


def measure_live_wrappers(manifest: list[dict[str, str]]) -> list[dict[str, Any]]:
    measured: list[dict[str, Any]] = []
    for wrapper in manifest:
        command = wrapper["command"]

        if warmup_enabled:
            subprocess.run(["/bin/bash", "-lc", command], text=True, capture_output=True, check=False)

        durations: list[int] = []
        exit_codes: list[int] = []
        for _ in range(iterations):
            start_ns = time.perf_counter_ns()
            completed = subprocess.run(
                ["/bin/bash", "-lc", command],
                text=True,
                capture_output=True,
                check=False,
            )
            duration_ms = int((time.perf_counter_ns() - start_ns) / 1_000_000)
            durations.append(duration_ms)
            exit_codes.append(int(completed.returncode))

        measured.append(
            {
                "id": wrapper["id"],
                "command": wrapper["command"],
                "use_case": wrapper["use_case"],
                "duration_ms": median(durations),
                "exit_code": 0 if all(code == 0 for code in exit_codes) else next(code for code in exit_codes if code != 0),
            }
        )
    return measured


def render_markdown(report: dict[str, Any]) -> str:
    lines: list[str] = []
    lines.append("# M25 Fast-Lane Loop Comparison")
    lines.append("")
    lines.append(f"Generated: `{report['generated_at']}`")
    lines.append(f"Repository: `{report['repository']}`")
    lines.append(f"Baseline report: `{report['baseline_report_path']}`")
    lines.append("")
    lines.append("## Summary")
    lines.append("")
    lines.append("| Status | Baseline median ms | Fast-lane median ms | Improvement ms | Improvement % |")
    lines.append("|---|---:|---:|---:|---:|")
    improvement_percent_render = f"{float(report['improvement_percent']):.2f}"
    lines.append(
        f"| {report['status']} | {report['baseline_median_ms']} | {report['fast_lane_median_ms']} | {report['improvement_ms']} | {improvement_percent_render} |"
    )
    lines.append("")
    lines.append("## Wrapper Measurements")
    lines.append("")
    lines.append("| Wrapper | Duration ms | Exit code | Use case | Command |")
    lines.append("|---|---:|---:|---|---|")
    for wrapper in report["wrappers"]:
        command_render = wrapper["command"].replace("|", "\\|")
        use_case_render = wrapper["use_case"].replace("|", "\\|")
        lines.append(
            f"| {wrapper['id']} | {wrapper['duration_ms']} | {wrapper['exit_code']} | {use_case_render} | `{command_render}` |"
        )
    return "\n".join(lines) + "\n"


def main() -> None:
    generated_at = iso_utc(parse_iso8601_utc(generated_at_raw))

    try:
        baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    except Exception as exc:
        fail(f"unable to parse baseline JSON: {exc}")

    baseline_commands = baseline.get("commands") if isinstance(baseline, dict) else None
    if not isinstance(baseline_commands, list) or len(baseline_commands) == 0:
        fail("baseline report must contain non-empty 'commands' array")

    baseline_values: list[int] = []
    for index, row in enumerate(baseline_commands):
        if not isinstance(row, dict):
            fail(f"baseline commands[{index}] must be an object")
        stats = row.get("stats")
        if not isinstance(stats, dict) or "avg_ms" not in stats or not isinstance(stats["avg_ms"], int):
            fail(f"baseline commands[{index}] missing stats.avg_ms")
        baseline_values.append(stats["avg_ms"])

    baseline_median_ms = median(baseline_values)
    if baseline_median_ms <= 0:
        fail("baseline median must be greater than zero")

    wrapper_manifest = parse_wrapper_manifest(wrapper_manifest_raw)

    if fixture_path is not None:
        wrappers = parse_wrappers_from_fixture(fixture_path)
        source_mode = "fixture"
    else:
        wrappers = measure_live_wrappers(wrapper_manifest)
        source_mode = "live"

    if len(wrappers) == 0:
        fail("wrapper measurements must be non-empty")

    fast_lane_values = [row["duration_ms"] for row in wrappers]
    fast_lane_median_ms = median(fast_lane_values)

    improvement_ms = baseline_median_ms - fast_lane_median_ms
    improvement_percent = round((improvement_ms / baseline_median_ms) * 100, 2)

    if improvement_ms > 0:
        status = "improved"
    elif improvement_ms < 0:
        status = "regressed"
    else:
        status = "no-change"

    repository = detect_repository_slug(repo_slug_raw, baseline.get("repository"))

    report = {
        "schema_version": 1,
        "generated_at": generated_at,
        "repository": repository,
        "source_mode": source_mode,
        "baseline_report_path": str(baseline_path),
        "baseline_median_ms": baseline_median_ms,
        "fast_lane_median_ms": fast_lane_median_ms,
        "improvement_ms": improvement_ms,
        "improvement_percent": improvement_percent,
        "status": status,
        "wrappers": wrappers,
    }

    output_json_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    output_md_path.write_text(render_markdown(report), encoding="utf-8")

    log(
        "[fast-lane-dev-loop] "
        f"source={source_mode} baseline_median_ms={baseline_median_ms} "
        f"fast_lane_median_ms={fast_lane_median_ms} status={status}"
    )


if __name__ == "__main__":
    main()
PY
}

main() {
  require_cmd jq
  require_cmd python3

  if [[ $# -lt 1 ]]; then
    usage >&2
    exit 1
  fi

  local subcommand="$1"
  shift

  case "${subcommand}" in
    list)
      list_wrappers
      ;;
    run)
      if [[ $# -lt 1 ]]; then
        echo "error: run requires <wrapper-id>" >&2
        exit 1
      fi
      local wrapper_id="$1"
      shift
      local dry_run="false"
      if [[ $# -gt 0 ]]; then
        if [[ "$1" == "--dry-run" ]]; then
          dry_run="true"
          shift
        else
          echo "error: unknown run argument '$1'" >&2
          exit 1
        fi
      fi
      if [[ $# -gt 0 ]]; then
        echo "error: unexpected trailing run arguments" >&2
        exit 1
      fi
      run_wrapper "${wrapper_id}" "${dry_run}"
      ;;
    benchmark)
      benchmark "$@"
      ;;
    --help|-h|help)
      usage
      ;;
    *)
      echo "error: unknown subcommand '${subcommand}'" >&2
      usage >&2
      exit 1
      ;;
  esac
}

main "$@"

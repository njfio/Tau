#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

FIXTURE_JSON=""
OUTPUT_MD="${REPO_ROOT}/tasks/reports/m25-build-test-latency-baseline.md"
OUTPUT_JSON="${REPO_ROOT}/tasks/reports/m25-build-test-latency-baseline.json"
GENERATED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
QUIET_MODE="false"
ITERATIONS=1
REPO_SLUG=""

DEFAULT_COMMAND_SPECS=(
  "check-tools::cargo check -p tau-tools --lib --target-dir target-fast"
  "test-runtime-no-run::cargo test -p tau-github-issues-runtime --target-dir target-fast --no-run"
  "test-trainer-regression::cargo test -p tau-trainer --target-dir target-fast benchmark_artifact::tests::regression_summary_gate_report_manifest_ignores_non_json_files -- --nocapture"
)

COMMAND_SPECS=()

usage() {
  cat <<'USAGE'
Usage: build-test-latency-baseline.sh [options]

Generate deterministic build/test latency baseline artifacts (JSON + Markdown)
with command-level timing stats and hotspot ranking.

Options:
  --fixture-json <path>   Use fixture JSON instead of live command execution.
  --command <id::cmd>     Command specification; repeatable in live mode.
  --iterations <n>        Iterations per command in live mode (default: 1).
  --repo <owner/name>     Repository slug override for artifact metadata.
  --output-json <path>    JSON artifact output path.
  --output-md <path>      Markdown artifact output path.
  --generated-at <iso>    Deterministic generated timestamp (ISO-8601 UTC).
  --quiet                 Suppress informational output.
  --help                  Show this help text.

Fixture JSON format:
{
  "repository": "owner/repo",
  "source_mode": "fixture",
  "environment": {
    "os": "linux",
    "arch": "x86_64",
    "shell": "bash",
    "rustc_version": "rustc 1.86.0",
    "cargo_version": "cargo 1.86.0"
  },
  "runs": [
    {
      "id": "check-tools",
      "command": "cargo check -p tau-tools --lib --target-dir target-fast",
      "iteration": 1,
      "duration_ms": 1820,
      "exit_code": 0
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
  if [[ "${QUIET_MODE}" != "true" ]]; then
    echo "$@"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --fixture-json)
      FIXTURE_JSON="$2"
      shift 2
      ;;
    --command)
      COMMAND_SPECS+=("$2")
      shift 2
      ;;
    --iterations)
      ITERATIONS="$2"
      shift 2
      ;;
    --repo)
      REPO_SLUG="$2"
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
      echo "error: unknown argument '$1'" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_cmd python3

if ! [[ "${ITERATIONS}" =~ ^[0-9]+$ ]]; then
  echo "error: --iterations must be a non-negative integer" >&2
  exit 1
fi
if (( ITERATIONS <= 0 )); then
  echo "error: --iterations must be greater than zero" >&2
  exit 1
fi

if [[ -n "${FIXTURE_JSON}" && ! -f "${FIXTURE_JSON}" ]]; then
  echo "error: fixture JSON not found: ${FIXTURE_JSON}" >&2
  exit 1
fi

if [[ -z "${FIXTURE_JSON}" && ${#COMMAND_SPECS[@]} -eq 0 ]]; then
  COMMAND_SPECS=("${DEFAULT_COMMAND_SPECS[@]}")
fi

commands_json="[]"
for spec in "${COMMAND_SPECS[@]}"; do
  if [[ "${spec}" != *"::"* ]]; then
    echo "error: invalid --command '${spec}' (expected id::command)" >&2
    exit 1
  fi
  cmd_id="${spec%%::*}"
  cmd_value="${spec#*::}"
  if [[ -z "${cmd_id}" || -z "${cmd_value}" ]]; then
    echo "error: invalid --command '${spec}' (empty id or command)" >&2
    exit 1
  fi
  commands_json="$(jq -c --arg id "${cmd_id}" --arg command "${cmd_value}" '. + [{"id": $id, "command": $command}]' <<<"${commands_json}")"
done

mkdir -p "$(dirname "${OUTPUT_JSON}")"
mkdir -p "$(dirname "${OUTPUT_MD}")"

python3 - \
  "${FIXTURE_JSON}" \
  "${REPO_SLUG}" \
  "${OUTPUT_JSON}" \
  "${OUTPUT_MD}" \
  "${GENERATED_AT}" \
  "${ITERATIONS}" \
  "${QUIET_MODE}" \
  "${commands_json}" <<'PY'
from __future__ import annotations

import json
import os
import platform
import statistics
import subprocess
import sys
import time
from collections import OrderedDict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

(
    fixture_path_raw,
    repo_slug_raw,
    output_json_raw,
    output_md_raw,
    generated_at_raw,
    iterations_raw,
    quiet_mode_raw,
    commands_json_raw,
) = sys.argv[1:]

fixture_path = Path(fixture_path_raw) if fixture_path_raw else None
output_json_path = Path(output_json_raw)
output_md_path = Path(output_md_raw)
iterations = int(iterations_raw)
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



def detect_repository_slug(explicit_repo: str | None) -> str:
    if explicit_repo:
        return explicit_repo
    try:
        completed = subprocess.run(
            ["gh", "repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"],
            text=True,
            capture_output=True,
            check=False,
        )
        if completed.returncode == 0:
            candidate = completed.stdout.strip()
            if candidate:
                return candidate
    except Exception:
        pass

    cwd_name = Path.cwd().name
    return f"local/{cwd_name}"



def safe_command_output(command: list[str]) -> str:
    try:
        completed = subprocess.run(command, text=True, capture_output=True, check=False)
        if completed.returncode != 0:
            return "unknown"
        return completed.stdout.strip() or "unknown"
    except Exception:
        return "unknown"



def detect_environment() -> dict[str, Any]:
    shell_path = os.environ.get("SHELL", "")
    shell_name = Path(shell_path).name if shell_path else "unknown"
    cpu_count = os.cpu_count() or 1
    return {
        "os": platform.system().lower() or "unknown",
        "arch": platform.machine() or "unknown",
        "shell": shell_name,
        "python_version": platform.python_version(),
        "rustc_version": safe_command_output(["rustc", "--version"]),
        "cargo_version": safe_command_output(["cargo", "--version"]),
        "cpu_count": cpu_count,
    }



def parse_int_field(row: dict[str, Any], key: str, index: int) -> int:
    if key not in row:
        fail(f"run[{index}] missing {key}")
    value = row[key]
    if not isinstance(value, int):
        fail(f"run[{index}] {key} must be an integer")
    return value



def validate_runs(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    normalized: list[dict[str, Any]] = []
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            fail(f"run[{index}] must be an object")
        run_id = row.get("id")
        command = row.get("command")
        if not isinstance(run_id, str) or not run_id.strip():
            fail(f"run[{index}] id must be a non-empty string")
        if not isinstance(command, str) or not command.strip():
            fail(f"run[{index}] command must be a non-empty string")

        iteration = parse_int_field(row, "iteration", index)
        duration_ms = parse_int_field(row, "duration_ms", index)
        exit_code = parse_int_field(row, "exit_code", index)

        if iteration <= 0:
            fail(f"run[{index}] iteration must be > 0")
        if duration_ms < 0:
            fail(f"run[{index}] duration_ms must be >= 0")

        normalized.append(
            {
                "id": run_id.strip(),
                "command": command.strip(),
                "iteration": iteration,
                "duration_ms": duration_ms,
                "exit_code": exit_code,
            }
        )
    return normalized



def collect_live_runs(command_specs: list[dict[str, str]], run_iterations: int) -> list[dict[str, Any]]:
    runs: list[dict[str, Any]] = []
    for command_spec in command_specs:
        command_id = command_spec["id"]
        command = command_spec["command"]
        for iteration in range(1, run_iterations + 1):
            start_ns = time.perf_counter_ns()
            completed = subprocess.run(
                ["/bin/bash", "-lc", command],
                text=True,
                capture_output=True,
                check=False,
            )
            end_ns = time.perf_counter_ns()
            duration_ms = int((end_ns - start_ns) / 1_000_000)
            runs.append(
                {
                    "id": command_id,
                    "command": command,
                    "iteration": iteration,
                    "duration_ms": duration_ms,
                    "exit_code": int(completed.returncode),
                }
            )
    return runs



def stats(values: list[int]) -> dict[str, int]:
    sorted_values = sorted(values)
    count = len(sorted_values)
    avg_ms = int(round(sum(sorted_values) / count))
    p50_ms = sorted_values[(count - 1) // 2]
    return {
        "count": count,
        "avg_ms": avg_ms,
        "p50_ms": p50_ms,
        "min_ms": sorted_values[0],
        "max_ms": sorted_values[-1],
    }



def build_artifact(
    repository: str,
    source_mode: str,
    environment: dict[str, Any],
    generated_at: str,
    runs: list[dict[str, Any]],
) -> dict[str, Any]:
    grouped: OrderedDict[str, dict[str, Any]] = OrderedDict()
    for row in sorted(runs, key=lambda candidate: (candidate["id"], candidate["iteration"])):
        entry = grouped.setdefault(
            row["id"],
            {
                "id": row["id"],
                "command": row["command"],
                "durations": [],
                "nonzero_exit_count": 0,
                "iterations": [],
            },
        )
        if entry["command"] != row["command"]:
            fail(
                f"command mismatch for id '{row['id']}' (expected '{entry['command']}', got '{row['command']}')"
            )
        entry["durations"].append(row["duration_ms"])
        entry["iterations"].append(row["iteration"])
        if row["exit_code"] != 0:
            entry["nonzero_exit_count"] += 1

    command_rows: list[dict[str, Any]] = []
    for command_id, entry in grouped.items():
        command_stats = stats(entry["durations"])
        command_rows.append(
            {
                "id": command_id,
                "command": entry["command"],
                "run_count": command_stats["count"],
                "stats": command_stats,
                "nonzero_exit_count": entry["nonzero_exit_count"],
                "invocation": f"/bin/bash -lc \"{entry['command']}\"",
            }
        )

    hotspots = [
        {
            "id": row["id"],
            "command": row["command"],
            "avg_ms": row["stats"]["avg_ms"],
        }
        for row in sorted(command_rows, key=lambda row: (-row["stats"]["avg_ms"], row["id"]))
    ]

    for index, row in enumerate(hotspots, start=1):
        row["rank"] = index

    failing_runs = sum(row["nonzero_exit_count"] for row in command_rows)
    slowest_command_id = hotspots[0]["id"] if hotspots else "none"

    return {
        "schema_version": 1,
        "generated_at": generated_at,
        "repository": repository,
        "source_mode": source_mode,
        "environment": environment,
        "summary": {
            "command_count": len(command_rows),
            "run_count": len(runs),
            "failing_runs": failing_runs,
            "slowest_command_id": slowest_command_id,
        },
        "commands": command_rows,
        "hotspots": hotspots,
    }



def render_markdown(artifact: dict[str, Any]) -> str:
    lines: list[str] = []
    lines.append("# M25 Build/Test Latency Baseline")
    lines.append("")
    lines.append(f"Generated: `{artifact['generated_at']}`")
    lines.append(f"Repository: `{artifact['repository']}`")
    lines.append(f"Source mode: `{artifact['source_mode']}`")
    lines.append("")
    lines.append("## Environment")
    lines.append("")
    lines.append("| Field | Value |")
    lines.append("|---|---|")
    for key in [
        "os",
        "arch",
        "shell",
        "python_version",
        "rustc_version",
        "cargo_version",
        "cpu_count",
    ]:
        lines.append(f"| {key} | {artifact['environment'].get(key, 'unknown')} |")

    lines.append("")
    lines.append("## Command Timing Matrix")
    lines.append("")
    lines.append("| Command ID | Runs | Avg ms | P50 ms | Min ms | Max ms | Failing runs |")
    lines.append("|---|---:|---:|---:|---:|---:|---:|")
    if artifact["commands"]:
        for row in artifact["commands"]:
            stats_row = row["stats"]
            lines.append(
                f"| {row['id']} | {row['run_count']} | {stats_row['avg_ms']} | {stats_row['p50_ms']} | {stats_row['min_ms']} | {stats_row['max_ms']} | {row['nonzero_exit_count']} |"
            )
    else:
        lines.append("| _none_ | - | - | - | - | - | - |")

    lines.append("")
    lines.append("## Hotspot Ranking")
    lines.append("")
    lines.append("| Rank | Command ID | Avg ms | Command |")
    lines.append("|---:|---|---:|---|")
    if artifact["hotspots"]:
        for row in artifact["hotspots"]:
            command_render = row["command"].replace("|", "\\|")
            lines.append(f"| {row['rank']} | {row['id']} | {row['avg_ms']} | `{command_render}` |")
    else:
        lines.append("| - | _none_ | - | - |")

    lines.append("")
    lines.append("## Summary")
    lines.append("")
    lines.append(f"- commands: {artifact['summary']['command_count']}")
    lines.append(f"- runs: {artifact['summary']['run_count']}")
    lines.append(f"- failing runs: {artifact['summary']['failing_runs']}")
    lines.append(f"- slowest command id: {artifact['summary']['slowest_command_id']}")

    return "\n".join(lines) + "\n"



def main() -> None:
    generated_at_iso = iso_utc(parse_iso8601_utc(generated_at_raw))
    explicit_repo = repo_slug_raw.strip() if repo_slug_raw.strip() else None
    repository = detect_repository_slug(explicit_repo)
    detected_environment = detect_environment()

    try:
        command_specs_payload = json.loads(commands_json_raw)
    except json.JSONDecodeError as exc:
        fail(f"unable to parse command spec JSON: {exc}")

    command_specs: list[dict[str, str]] = []
    if isinstance(command_specs_payload, list):
        for index, item in enumerate(command_specs_payload):
            if not isinstance(item, dict):
                fail(f"command[{index}] must be an object")
            command_id = item.get("id")
            command = item.get("command")
            if not isinstance(command_id, str) or not command_id.strip():
                fail(f"command[{index}] id must be non-empty string")
            if not isinstance(command, str) or not command.strip():
                fail(f"command[{index}] command must be non-empty string")
            command_specs.append({"id": command_id.strip(), "command": command.strip()})

    source_mode = "live"
    environment = detected_environment

    if fixture_path is not None:
        source_mode = "fixture"
        try:
            fixture_payload = json.loads(fixture_path.read_text(encoding="utf-8"))
        except Exception as exc:
            fail(f"unable to parse fixture JSON: {exc}")
        if not isinstance(fixture_payload, dict):
            fail("fixture JSON must decode to an object")

        fixture_repository = fixture_payload.get("repository")
        if isinstance(fixture_repository, str) and fixture_repository.strip() and explicit_repo is None:
            repository = fixture_repository.strip()

        fixture_environment = fixture_payload.get("environment")
        if fixture_environment is not None:
            if not isinstance(fixture_environment, dict):
                fail("fixture field 'environment' must be an object")
            merged_environment = dict(detected_environment)
            for key, value in fixture_environment.items():
                if isinstance(value, (str, int)):
                    merged_environment[key] = value
            environment = merged_environment

        runs_payload = fixture_payload.get("runs")
        if not isinstance(runs_payload, list):
            fail("fixture field 'runs' must be an array")
        runs = validate_runs(runs_payload)
    else:
        if not command_specs:
            fail("live mode requires at least one --command entry")
        runs = collect_live_runs(command_specs, iterations)

    artifact = build_artifact(repository, source_mode, environment, generated_at_iso, runs)
    markdown = render_markdown(artifact)

    output_json_path.write_text(json.dumps(artifact, indent=2) + "\n", encoding="utf-8")
    output_md_path.write_text(markdown, encoding="utf-8")

    log(
        "[build-test-latency-baseline] "
        f"source={source_mode} commands={artifact['summary']['command_count']} "
        f"runs={artifact['summary']['run_count']} "
        f"slowest={artifact['summary']['slowest_command_id']}"
    )

if __name__ == "__main__":
    main()
PY

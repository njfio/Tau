#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../.." && pwd)"
binary_path="${repo_root}/target/debug/tau-coding-agent"
harness_bin="${repo_root}/target/debug/browser_automation_live_harness"
skip_build="false"
timeout_seconds=""
playwright_cli_override=""
step_total=0
step_passed=0

print_usage() {
  cat <<EOF
Usage: dashboard-live.sh [--repo-root PATH] [--binary PATH] [--harness-bin PATH] [--skip-build] [--timeout-seconds N] [--playwright-cli PATH] [--help]

Run deterministic dashboard browser-E2E live proof flow and emit machine-readable artifacts.

Options:
  --repo-root PATH      Repository root (defaults to caller-derived root)
  --binary PATH         tau-coding-agent binary path
  --harness-bin PATH    browser_automation_live_harness binary path
  --skip-build          Skip cargo build and require binaries to exist
  --timeout-seconds N   Positive integer timeout per command step
  --playwright-cli PATH Use this Playwright CLI path instead of deterministic mock fallback
  --help                Show this usage message
EOF
}

log_info() {
  echo "[demo:dashboard-live] $1"
}

run_step() {
  local label="$1"
  shift
  step_total=$((step_total + 1))
  log_info "[${step_total}] ${label}"
  if "$@"; then
    step_passed=$((step_passed + 1))
    log_info "PASS ${label}"
  else
    local rc=$?
    log_info "FAIL ${label} exit=${rc}"
    return "${rc}"
  fi
}

run_with_timeout() {
  local -a command=("$@")
  if [[ -n "${timeout_seconds}" ]]; then
    python3 - "${timeout_seconds}" "${command[@]}" <<'PY'
import subprocess
import sys

timeout_seconds = int(sys.argv[1])
command = sys.argv[2:]
try:
    completed = subprocess.run(command, timeout=timeout_seconds)
except subprocess.TimeoutExpired:
    sys.exit(124)
sys.exit(completed.returncode)
PY
  else
    "${command[@]}"
  fi
}

run_logged_command() {
  local stdout_log="$1"
  local stderr_log="$2"
  shift 2
  set +e
  run_with_timeout "$@" >"${stdout_log}" 2>"${stderr_log}"
  local rc=$?
  set -e
  return "${rc}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --repo-root" >&2
        print_usage >&2
        exit 2
      fi
      repo_root="$2"
      shift 2
      ;;
    --binary)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --binary" >&2
        print_usage >&2
        exit 2
      fi
      binary_path="$2"
      shift 2
      ;;
    --harness-bin)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --harness-bin" >&2
        print_usage >&2
        exit 2
      fi
      harness_bin="$2"
      shift 2
      ;;
    --skip-build)
      skip_build="true"
      shift
      ;;
    --timeout-seconds)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --timeout-seconds" >&2
        print_usage >&2
        exit 2
      fi
      if [[ ! "$2" =~ ^[1-9][0-9]*$ ]]; then
        echo "invalid value for --timeout-seconds (expected positive integer): $2" >&2
        print_usage >&2
        exit 2
      fi
      timeout_seconds="$2"
      shift 2
      ;;
    --playwright-cli)
      if [[ $# -lt 2 ]]; then
        echo "missing value for --playwright-cli" >&2
        print_usage >&2
        exit 2
      fi
      playwright_cli_override="$2"
      shift 2
      ;;
    --help)
      print_usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      print_usage >&2
      exit 2
      ;;
  esac
done

if [[ ! -d "${repo_root}" ]]; then
  echo "invalid --repo-root path (directory not found): ${repo_root}" >&2
  exit 2
fi
repo_root="$(cd "${repo_root}" && pwd)"

if [[ "${binary_path}" != /* ]]; then
  binary_path="${repo_root}/${binary_path}"
fi
if [[ "${harness_bin}" != /* ]]; then
  harness_bin="${repo_root}/${harness_bin}"
fi

fixture_path="${repo_root}/crates/tau-coding-agent/testdata/dashboard-contract/snapshot-layout.json"
if [[ ! -f "${fixture_path}" ]]; then
  echo "missing required dashboard fixture: ${fixture_path}" >&2
  exit 1
fi

work_root="${repo_root}/.tau/demo-dashboard-live"
work_dir="${work_root}/work"
dashboard_state_dir="${work_root}/dashboard-state"
browser_state_dir="${work_root}/browser-state"
browser_fixture_path="${work_dir}/dashboard-live-browser-fixture.json"
webchat_page_path="${work_dir}/webchat-fallback.html"
summary_json="${work_root}/dashboard-live-summary.json"
report_json="${work_root}/dashboard-live-report.json"
transcript_log="${work_root}/dashboard-live-transcript.log"
audit_log="${work_root}/dashboard-action-audit.json"
webchat_check_json="${work_root}/webchat-fallback-check.json"
mock_cli_path="${work_dir}/mock-dashboard-playwright-cli.py"

rm -rf "${work_root}"
mkdir -p "${work_dir}" "${dashboard_state_dir}" "${browser_state_dir}"

if [[ "${skip_build}" != "true" ]]; then
  run_step "build-tau-coding-agent" \
    bash -lc "cd '${repo_root}' && cargo build -p tau-coding-agent >/dev/null"
  run_step "build-browser-live-harness" \
    bash -lc "cd '${repo_root}' && cargo build -p tau-browser-automation --bin browser_automation_live_harness >/dev/null"
fi

if [[ ! -x "${binary_path}" ]]; then
  echo "missing tau-coding-agent binary: ${binary_path}" >&2
  exit 1
fi
if [[ ! -x "${harness_bin}" ]]; then
  echo "missing harness binary: ${harness_bin}" >&2
  exit 1
fi

cat >"${webchat_page_path}" <<'EOF'
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>Tau Gateway Webchat</title>
</head>
<body>
  <h1>Tau Gateway Webchat</h1>
  <p>Fallback shell for dashboard/webchat operator access.</p>
  <textarea id="prompt"></textarea>
  <button id="send">Send</button>
  <button id="refreshStatus">Refresh status</button>
  <button id="clearOutput">Clear output</button>
  <pre id="status">multi-channel lifecycle summary</pre>
  <pre id="output">No response yet.</pre>
</body>
</html>
EOF

dom_nodes_count="$(python3 - "${webchat_page_path}" <<'PY'
import sys
from pathlib import Path

path = Path(sys.argv[1])
print(path.read_text(encoding="utf-8").count("<"))
PY
)"

cat >"${browser_fixture_path}" <<EOF
{
  "schema_version": 1,
  "name": "dashboard-live-browser-e2e",
  "description": "Deterministic browser-E2E flow for dashboard fallback webchat shell.",
  "cases": [
    {
      "schema_version": 1,
      "case_id": "dashboard-load-webchat",
      "operation": "navigate",
      "url": "file://${webchat_page_path}",
      "expected": {
        "outcome": "success",
        "status_code": 200,
        "response_body": {
          "status": "ok",
          "operation": "navigate",
          "url": "file://${webchat_page_path}",
          "title": "Tau Gateway Webchat",
          "dom_nodes": ${dom_nodes_count}
        }
      }
    },
    {
      "schema_version": 1,
      "case_id": "dashboard-live-updates-snapshot",
      "operation": "snapshot",
      "expected": {
        "outcome": "success",
        "status_code": 200,
        "response_body": {
          "status": "ok",
          "operation": "snapshot",
          "snapshot_id": "dashboard-shell",
          "elements": [
            {"id": "send", "role": "button", "name": "send"},
            {"id": "refreshStatus", "role": "button", "name": "refreshStatus"},
            {"id": "clearOutput", "role": "button", "name": "clearOutput"},
            {"id": "status", "role": "status", "name": "status"},
            {"id": "output", "role": "status", "name": "output"}
          ]
        }
      }
    },
    {
      "schema_version": 1,
      "case_id": "dashboard-control-refresh-status",
      "operation": "action",
      "action": "click",
      "selector": "#refreshStatus",
      "timeout_ms": 1000,
      "expected": {
        "outcome": "success",
        "status_code": 200,
        "response_body": {
          "status": "ok",
          "operation": "action",
          "action": "click",
          "selector": "#refreshStatus",
          "repeat_count": 1,
          "text": "",
          "timeout_ms": 1000
        }
      }
    },
    {
      "schema_version": 1,
      "case_id": "dashboard-control-clear-output",
      "operation": "action",
      "action": "click",
      "selector": "#clearOutput",
      "timeout_ms": 1000,
      "expected": {
        "outcome": "success",
        "status_code": 200,
        "response_body": {
          "status": "ok",
          "operation": "action",
          "action": "click",
          "selector": "#clearOutput",
          "repeat_count": 1,
          "text": "",
          "timeout_ms": 1000
        }
      }
    }
  ]
}
EOF

if [[ -n "${playwright_cli_override}" ]]; then
  playwright_cli="${playwright_cli_override}"
  if [[ ! -x "${playwright_cli}" ]]; then
    echo "provided --playwright-cli is not executable: ${playwright_cli}" >&2
    exit 2
  fi
else
  cat >"${mock_cli_path}" <<'EOF'
#!/usr/bin/env python3
import json
import pathlib
import re
import sys
import urllib.request

session_file = pathlib.Path(__file__).with_suffix(".session.json")
command = sys.argv[1] if len(sys.argv) > 1 else ""


def load_session() -> dict:
    if session_file.exists():
        return json.loads(session_file.read_text(encoding="utf-8"))
    return {"last_url": ""}


def save_session(payload: dict) -> None:
    session_file.write_text(json.dumps(payload), encoding="utf-8")


def load_html(url: str) -> tuple[str, int]:
    if url.startswith("file://"):
        html = pathlib.Path(url[7:]).read_text(encoding="utf-8")
        return html, 200
    with urllib.request.urlopen(url) as response:
        body = response.read().decode("utf-8")
        status = int(getattr(response, "status", 200))
    return body, status


def parse_title(html: str) -> str:
    match = re.search(r"<title>(.*?)</title>", html, re.IGNORECASE | re.DOTALL)
    if not match:
        return "Untitled"
    return match.group(1).strip()


def parse_ids(html: str) -> list[str]:
    return re.findall(r"id=['\\\"]([^'\\\"]+)['\\\"]", html)


if command == "start-session":
    save_session({"last_url": ""})
    print(json.dumps({"status": "ok"}))
    raise SystemExit(0)

if command == "shutdown-session":
    if session_file.exists():
        session_file.unlink()
    print(json.dumps({"status": "ok"}))
    raise SystemExit(0)

if command != "execute-action":
    print("unsupported command", file=sys.stderr)
    raise SystemExit(2)

payload = json.loads(sys.argv[2]) if len(sys.argv) > 2 else {}
operation = payload.get("operation", "")
session = load_session()

if operation == "navigate":
    url = payload.get("url", "")
    html, status = load_html(url)
    session["last_url"] = url
    save_session(session)
    print(
        json.dumps(
            {
                "status_code": status,
                "response_body": {
                    "status": "ok",
                    "operation": "navigate",
                    "url": url,
                    "title": parse_title(html),
                    "dom_nodes": html.count("<"),
                },
                "artifacts": {
                    "dom_snapshot_html": html,
                    "screenshot_svg": "<svg xmlns='http://www.w3.org/2000/svg'><text x='4' y='12'>dashboard-load</text></svg>",
                    "trace_json": json.dumps({"events": ["navigate"], "url": url}),
                },
            }
        )
    )
    raise SystemExit(0)

if operation == "snapshot":
    url = session.get("last_url", "")
    if not url:
        print(
            json.dumps(
                {
                    "status_code": 400,
                    "error_code": "browser_automation_snapshot_without_navigation",
                    "response_body": {"status": "rejected", "reason": "missing_navigation"},
                }
            )
        )
        raise SystemExit(0)
    html, _ = load_html(url)
    element_ids = parse_ids(html)
    elements = [
        {"id": "send", "role": "button", "name": "send"},
        {"id": "refreshStatus", "role": "button", "name": "refreshStatus"},
        {"id": "clearOutput", "role": "button", "name": "clearOutput"},
        {"id": "status", "role": "status", "name": "status"},
        {"id": "output", "role": "status", "name": "output"},
    ]
    if not all(entry["id"] in element_ids for entry in elements):
        print(
            json.dumps(
                {
                    "status_code": 500,
                    "error_code": "browser_automation_missing_dashboard_elements",
                    "response_body": {"status": "rejected", "reason": "missing_elements"},
                }
            )
        )
        raise SystemExit(0)
    print(
        json.dumps(
            {
                "status_code": 200,
                "response_body": {
                    "status": "ok",
                    "operation": "snapshot",
                    "snapshot_id": "dashboard-shell",
                    "elements": elements,
                },
                "artifacts": {
                    "dom_snapshot_html": html,
                    "screenshot_svg": "<svg xmlns='http://www.w3.org/2000/svg'><text x='4' y='12'>dashboard-snapshot</text></svg>",
                    "trace_json": json.dumps({"events": ["snapshot"], "elements": element_ids}),
                },
            }
        )
    )
    raise SystemExit(0)

if operation == "action":
    url = session.get("last_url", "")
    html, _ = load_html(url)
    selector = payload.get("selector", "")
    selector_id = selector[1:] if selector.startswith("#") else selector
    if selector_id not in parse_ids(html):
        print(
            json.dumps(
                {
                    "status_code": 404,
                    "error_code": "browser_automation_selector_not_found",
                    "response_body": {"status": "rejected", "reason": "selector_not_found"},
                }
            )
        )
        raise SystemExit(0)
    print(
        json.dumps(
            {
                "status_code": 200,
                "response_body": {
                    "status": "ok",
                    "operation": "action",
                    "action": payload.get("action", ""),
                    "selector": payload.get("selector", ""),
                    "repeat_count": payload.get("action_repeat_count", 1),
                    "text": payload.get("text", ""),
                    "timeout_ms": payload.get("timeout_ms", 0),
                },
                "artifacts": {
                    "dom_snapshot_html": "",
                    "screenshot_svg": "<svg xmlns='http://www.w3.org/2000/svg'><text x='4' y='12'>dashboard-action</text></svg>",
                    "trace_json": json.dumps({"events": ["action"], "selector": selector}),
                },
            }
        )
    )
    raise SystemExit(0)

print(
    json.dumps(
        {
            "status_code": 400,
            "error_code": "browser_automation_invalid_operation",
            "response_body": {"status": "rejected", "reason": "invalid_operation"},
        }
    )
)
EOF
  chmod +x "${mock_cli_path}"
  playwright_cli="${mock_cli_path}"
fi

run_step "dashboard-runtime-runner" \
  run_logged_command \
  "${work_root}/dashboard-runner.stdout.log" \
  "${work_root}/dashboard-runner.stderr.log" \
  "${binary_path}" \
  --dashboard-contract-runner \
  --dashboard-fixture "${fixture_path}" \
  --dashboard-state-dir "${dashboard_state_dir}" \
  --dashboard-queue-limit 64 \
  --dashboard-processed-case-cap 10000 \
  --dashboard-retry-max-attempts 4 \
  --dashboard-retry-base-delay-ms 0

run_step "dashboard-transport-health-inspect" \
  run_logged_command \
  "${work_root}/dashboard-health.stdout.log" \
  "${work_root}/dashboard-health.stderr.log" \
  "${binary_path}" \
  --dashboard-state-dir "${dashboard_state_dir}" \
  --transport-health-inspect dashboard \
  --transport-health-json

run_step "dashboard-status-inspect" \
  run_logged_command \
  "${work_root}/dashboard-status.stdout.log" \
  "${work_root}/dashboard-status.stderr.log" \
  "${binary_path}" \
  --dashboard-state-dir "${dashboard_state_dir}" \
  --dashboard-status-inspect \
  --dashboard-status-json

run_step "channel-store-inspect-dashboard-control-audit" \
  run_logged_command \
  "${work_root}/dashboard-channel-store.stdout.log" \
  "${work_root}/dashboard-channel-store.stderr.log" \
  "${binary_path}" \
  --channel-store-root "${dashboard_state_dir}/channel-store" \
  --channel-store-inspect dashboard/operator:ops-release-2

run_harness() {
  local -a command=(
    "${harness_bin}"
    "--fixture" "${browser_fixture_path}"
    "--state-dir" "${browser_state_dir}"
    "--playwright-cli" "${playwright_cli}"
    "--summary-json-out" "${summary_json}"
    "--artifact-retention-days" "7"
    "--action-timeout-ms" "4000"
    "--max-actions-per-case" "6"
  )

  set +e
  if [[ -n "${timeout_seconds}" ]]; then
    python3 - "${timeout_seconds}" "${command[@]}" <<'PY' 2>&1 | tee "${transcript_log}"
import subprocess
import sys

timeout_seconds = int(sys.argv[1])
command = sys.argv[2:]
try:
    completed = subprocess.run(command, timeout=timeout_seconds)
except subprocess.TimeoutExpired:
    sys.exit(124)
sys.exit(completed.returncode)
PY
  else
    "${command[@]}" 2>&1 | tee "${transcript_log}"
  fi
  local rc=${PIPESTATUS[0]}
  set -e
  return "${rc}"
}

run_step "run-dashboard-browser-e2e-harness" run_harness

validate_outputs() {
  python3 - \
    "${summary_json}" \
    "${report_json}" \
    "${transcript_log}" \
    "${dashboard_state_dir}" \
    "${audit_log}" \
    "${webchat_check_json}" \
    "${webchat_page_path}" <<'PY'
import json
import sys
from pathlib import Path

summary_path = Path(sys.argv[1])
report_path = Path(sys.argv[2])
transcript_path = Path(sys.argv[3])
dashboard_state_dir = Path(sys.argv[4])
audit_path = Path(sys.argv[5])
webchat_check_path = Path(sys.argv[6])
webchat_page = Path(sys.argv[7])

if not summary_path.exists():
    raise SystemExit(f"summary JSON missing: {summary_path}")

summary = json.loads(summary_path.read_text(encoding="utf-8"))
if summary.get("discovered_cases") != 4:
    raise SystemExit("unexpected discovered_cases in dashboard live summary")
if summary.get("success_cases") != 4:
    raise SystemExit("unexpected success_cases in dashboard live summary")
if summary.get("health_state") != "healthy":
    raise SystemExit("dashboard browser summary health_state is not healthy")

timeline = summary.get("timeline", [])
if len(timeline) != 4:
    raise SystemExit("dashboard browser timeline does not contain all expected cases")

state_path = dashboard_state_dir / "state.json"
if not state_path.exists():
    raise SystemExit(f"dashboard state missing: {state_path}")
state = json.loads(state_path.read_text(encoding="utf-8"))
widget_views = state.get("widget_views", [])
if not isinstance(widget_views, list) or not widget_views:
    raise SystemExit("dashboard state does not contain widget_views")

audit_events = []
for log_file in sorted((dashboard_state_dir / "channel-store" / "channels" / "dashboard").glob("*/log.jsonl")):
    for line in log_file.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        payload = json.loads(line)
        action = (
            payload.get("payload", {}).get("control_action")
            if isinstance(payload.get("payload"), dict)
            else None
        )
        if action:
            payload["channel_log"] = str(log_file)
            audit_events.append(payload)

if not audit_events:
    raise SystemExit("dashboard control action audit log entries were not captured")

audit_path.write_text(json.dumps(audit_events, indent=2), encoding="utf-8")

html = webchat_page.read_text(encoding="utf-8")
required_snippets = [
    "Tau Gateway Webchat",
    "id=\"send\"",
    "id=\"refreshStatus\"",
    "id=\"clearOutput\"",
    "id=\"status\"",
    "id=\"output\"",
]
missing = [snippet for snippet in required_snippets if snippet not in html]
if missing:
    raise SystemExit(f"webchat fallback shell missing snippets: {missing}")

webchat_check = {
    "status": "pass",
    "required_snippets_checked": required_snippets,
    "missing_snippets": missing,
    "webchat_page_path": str(webchat_page),
}
webchat_check_path.write_text(json.dumps(webchat_check, indent=2), encoding="utf-8")

report = {
    "schema_version": 1,
    "summary_path": str(summary_path),
    "transcript_path": str(transcript_path),
    "dashboard_state_path": str(state_path),
    "health_state": summary.get("health_state"),
    "reason_codes": summary.get("reason_codes", []),
    "discovered_cases": summary.get("discovered_cases"),
    "success_cases": summary.get("success_cases"),
    "artifact_records": summary.get("artifact_records"),
    "timeline": timeline,
    "dashboard_widget_view_count": len(widget_views),
    "action_audit_event_count": len(audit_events),
    "action_audit_path": str(audit_path),
    "webchat_fallback_check_path": str(webchat_check_path),
}
report_path.parent.mkdir(parents=True, exist_ok=True)
report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
print(f"report_json={report_path}")
PY
}

run_step "validate-dashboard-live-artifacts-and-write-report" validate_outputs

log_info "summary_json=${summary_json}"
log_info "report_json=${report_json}"
log_info "transcript_log=${transcript_log}"
log_info "audit_log=${audit_log}"
log_info "webchat_check_json=${webchat_check_json}"
log_info "summary: total=${step_total} passed=${step_passed} failed=$((step_total - step_passed))"

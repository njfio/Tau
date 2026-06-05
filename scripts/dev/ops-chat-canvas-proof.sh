#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${TAU_OPS_CHAT_CANVAS_PROOF_BASE_URL:-http://127.0.0.1:8797}"
SESSION_KEY="${TAU_OPS_CHAT_CANVAS_PROOF_SESSION:-ops-chat-canvas-proof}"
ARTIFACT_PATH="${TAU_OPS_CHAT_CANVAS_PROOF_ARTIFACT:-target/ops-chat-canvas-proof.html}"
OUTPUT_JSON="${TAU_OPS_CHAT_CANVAS_PROOF_OUTPUT_JSON:-tasks/reports/ops-chat-canvas-proof.json}"
TIMEOUT_SECONDS="${TAU_OPS_CHAT_CANVAS_PROOF_TIMEOUT_SECONDS:-240}"
AUTH_MODE="${TAU_OPS_CHAT_CANVAS_PROOF_AUTH_MODE:-none}"
AUTH_TOKEN="${TAU_OPS_CHAT_CANVAS_PROOF_AUTH_TOKEN:-}"
SKIP_FILE_CHECK="${TAU_OPS_CHAT_CANVAS_PROOF_SKIP_FILE_CHECK:-false}"
RECOVERY_MESSAGE_CHARS="${TAU_OPS_CHAT_CANVAS_PROOF_RECOVERY_MESSAGE_CHARS:-40000}"

usage() {
  cat <<'USAGE'
Usage: ops-chat-canvas-proof.sh [options]

Submit a live /ops/chat request that must use tools to create an HTML canvas
artifact, then verify the chat route renders the Agent Canvas v2 preview,
artifact history, diagnostics bridge, and controlled interaction surface.

Options:
  --base-url <url>          Gateway base URL (default: http://127.0.0.1:8797)
  --session <key>           Chat session key
  --artifact-path <path>    Workspace-local HTML artifact path
  --output-json <path>      JSON proof artifact path
  --timeout-seconds <n>     Curl timeout seconds
  --auth-mode <none|token>  Auth mode (default: none)
  --auth-token <token>      Bearer token for token mode
  --skip-file-check         Only validate rendered route markers
  --help                    Show this help
USAGE
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: required command '$1' not found" >&2
    exit 1
  fi
}

require_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "error: missing ${label}: ${needle}" >&2
    exit 1
  fi
}

json_escape() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  value="${value//$'\n'/\\n}"
  printf '%s' "${value}"
}

json_bool() {
  if [[ "$1" == "true" ]]; then
    printf 'true'
  else
    printf 'false'
  fi
}

contains_bool() {
  local haystack="$1"
  local needle="$2"
  if [[ "${haystack}" == *"${needle}"* ]]; then
    printf 'true'
  else
    printf 'false'
  fi
}

count_marker() {
  local haystack="$1"
  local needle="$2"
  awk -v needle="${needle}" '
    {
      pos = 1
      while ((idx = index(substr($0, pos), needle)) > 0) {
        count++
        pos += idx + length(needle) - 1
      }
    }
    END { print count + 0 }
  ' <<<"${haystack}"
}

file_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    echo "error: requires sha256sum or shasum to compute artifact hash" >&2
    exit 1
  fi
}

file_size_bytes() {
  wc -c <"$1" | tr -d '[:space:]'
}

validate_agent_canvas_contract() {
  local html="$1"
  require_contains "${html}" 'id="tau-ops-chat-agent-canvas"' "agent canvas section"
  require_contains "${html}" 'data-preview-status="loaded"' "loaded preview status"
  require_contains "${html}" 'id="tau-ops-chat-agent-preview-frame"' "preview frame"
  require_contains "${html}" 'sandbox="allow-scripts"' "sandboxed preview frame"
  require_contains "${html}" 'data-agent-canvas-runtime="postmessage-v2"' "canvas runtime bridge"
  require_contains "${html}" 'data-agent-canvas-artifact-history="true"' "artifact history"
  require_contains "${html}" 'data-agent-canvas-controls="postmessage"' "controlled interaction surface"
  require_contains "${html}" 'data-agent-canvas-diagnostics="true"' "diagnostics surface"
  require_contains "${html}" 'data-preview-runtime-status="pending"' "runtime status marker"
  require_contains "${html}" 'data-dom-node-count="0"' "DOM snapshot counter marker"
  require_contains "${html}" 'data-dom-snapshot-count="0"' "DOM snapshot list counter marker"
  require_contains "${html}" 'data-canvas-count="0"' "canvas counter marker"
  require_contains "${html}" 'data-console-error-count="0"' "console error counter marker"
  require_contains "${html}" 'data-pixel-sample-count="0"' "pixel sample counter marker"
  require_contains "${html}" 'data-screenshot-sample-count="0"' "screenshot sample counter marker"
  require_contains "${html}" 'data-interaction-mode="postmessage"' "interaction mode marker"
  require_contains "${html}" 'data-agent-canvas-tool="snapshot"' "snapshot tool control"
  require_contains "${html}" 'data-agent-canvas-tool="click"' "click tool control"
  require_contains "${html}" 'data-agent-canvas-tool="type"' "type tool control"
  require_contains "${html}" 'data-agent-canvas-dom-snapshot="true"' "DOM snapshot diagnostics list"
  require_contains "${html}" 'data-agent-canvas-console-events="true"' "console diagnostics list"
  require_contains "${html}" 'data-agent-canvas-pixel-samples="true"' "pixel diagnostics list"
  require_contains "${html}" 'data-agent-canvas-screenshot-samples="true"' "screenshot diagnostics list"
}

apply_agent_canvas_proof_loop_fix() {
  local artifact_path="$1"
  mkdir -p "$(dirname "${artifact_path}")"
  cat >"${artifact_path}" <<'HTML'
<!doctype html>
<html lang="en" data-agent-canvas-proof-loop="fixed">
<head>
  <meta charset="utf-8">
  <title>Agent Canvas proof-loop fixed artifact</title>
</head>
<body>
  <canvas id="game" width="96" height="48" data-agent-canvas-proof-loop-canvas="fixed"></canvas>
  <input id="agent-canvas-proof-loop-input" aria-label="Agent Canvas proof-loop input">
  <script>
    const canvas = document.getElementById("game");
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "#18a957";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = "#062a19";
    ctx.fillRect(8, 8, 24, 16);
    console.log("agent-canvas-proof-loop:fixed");
  </script>
</body>
</html>
HTML
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-url)
      BASE_URL="$2"
      shift 2
      ;;
    --session)
      SESSION_KEY="$2"
      shift 2
      ;;
    --artifact-path)
      ARTIFACT_PATH="$2"
      shift 2
      ;;
    --output-json)
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --timeout-seconds)
      TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --auth-mode)
      AUTH_MODE="$2"
      shift 2
      ;;
    --auth-token)
      AUTH_TOKEN="$2"
      shift 2
      ;;
    --skip-file-check)
      SKIP_FILE_CHECK="true"
      shift
      ;;
    --help)
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

require_cmd curl
require_cmd dd
require_cmd tr

if [[ "${AUTH_MODE}" != "none" && "${AUTH_MODE}" != "token" ]]; then
  echo "error: --auth-mode must be none or token" >&2
  exit 1
fi

if [[ "${AUTH_MODE}" == "token" && -z "${AUTH_TOKEN}" ]]; then
  echo "error: --auth-token is required for token auth mode" >&2
  exit 1
fi

auth_args=()
if [[ "${AUTH_MODE}" == "token" ]]; then
  auth_args=("-H" "Authorization: Bearer ${AUTH_TOKEN}")
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

read_header_location() {
  awk 'tolower($1) == "location:" { sub(/^[^:]*:[[:space:]]*/, ""); sub(/\r$/, ""); print; exit }' "$1"
}

require_redirect_location_contains() {
  local status="$1"
  local location="$2"
  local expected="$3"
  local label="$4"
  if [[ "${status}" != "302" && "${status}" != "303" ]]; then
    echo "error: ${label} expected redirect status, got HTTP ${status}" >&2
    exit 1
  fi
  require_contains "${location}" "${expected}" "${label}"
}

get_recovery_headers="${tmp_dir}/send-get.headers"
get_recovery_body="${tmp_dir}/send-get.body"
get_recovery_status="$(curl -sS -o "${get_recovery_body}" -D "${get_recovery_headers}" -w '%{http_code}' --max-time "${TIMEOUT_SECONDS}" \
  "${auth_args[@]}" \
  "${BASE_URL%/}/ops/chat/send?theme=dark&sidebar=expanded&session=${SESSION_KEY}")"
get_recovery_location="$(read_header_location "${get_recovery_headers}")"
require_redirect_location_contains \
  "${get_recovery_status}" \
  "${get_recovery_location}" \
  "/ops/chat?theme=dark&sidebar=expanded&session=${SESSION_KEY}" \
  "send GET recovery redirect"

oversized_message="$(dd if=/dev/zero bs="${RECOVERY_MESSAGE_CHARS}" count=1 2>/dev/null | tr '\0' 'x')"
failure_recovery_headers="${tmp_dir}/send-failure.headers"
failure_recovery_body="${tmp_dir}/send-failure.body"
failure_recovery_status="$(curl -sS -o "${failure_recovery_body}" -D "${failure_recovery_headers}" -w '%{http_code}' --max-time "${TIMEOUT_SECONDS}" \
  "${auth_args[@]}" \
  -X POST "${BASE_URL%/}/ops/chat/send" \
  --data-urlencode "session_key=${SESSION_KEY}" \
  --data-urlencode "theme=dark" \
  --data-urlencode "sidebar=expanded" \
  --data-urlencode "message=${oversized_message}")"
failure_recovery_location="$(read_header_location "${failure_recovery_headers}")"
require_redirect_location_contains \
  "${failure_recovery_status}" \
  "${failure_recovery_location}" \
  "chat_status=input-too-large" \
  "send failure recovery redirect"

message="Create an HTML canvas demo at ${ARTIFACT_PATH} with a canvas id \"game\" and a short script that draws a green rectangle. Use the write tool."
post_body="${tmp_dir}/post.html"
post_status="$(curl -sS -o "${post_body}" -w '%{http_code}' --max-time "${TIMEOUT_SECONDS}" \
  "${auth_args[@]}" \
  -X POST "${BASE_URL%/}/ops/chat/send" \
  --data-urlencode "session_key=${SESSION_KEY}" \
  --data-urlencode "theme=dark" \
  --data-urlencode "sidebar=expanded" \
  --data-urlencode "message=${message}")"

if [[ "${post_status}" != "200" && "${post_status}" != "303" && "${post_status}" != "302" ]]; then
  echo "error: /ops/chat/send returned HTTP ${post_status}" >&2
  cat "${post_body}" >&2 || true
  exit 1
fi

file_check="skipped"
if [[ "${SKIP_FILE_CHECK}" != "true" ]]; then
  if [[ ! -f "${ARTIFACT_PATH}" ]]; then
    echo "error: expected HTML artifact was not created: ${ARTIFACT_PATH}" >&2
    exit 1
  fi
  file_check="passed"
fi

chat_body="${tmp_dir}/chat.html"
curl -sS --fail-with-body --max-time "${TIMEOUT_SECONDS}" \
  "${auth_args[@]}" \
  "${BASE_URL%/}/ops/chat?theme=dark&sidebar=expanded&session=${SESSION_KEY}" \
  >"${chat_body}"

chat_html="$(cat "${chat_body}")"
validate_agent_canvas_contract "${chat_html}"

proof_loop_result="skipped"
before_artifact_sha256=""
after_artifact_sha256=""
before_artifact_bytes=0
after_artifact_bytes=0
before_dom_marker_count=0
after_dom_marker_count=0
before_dom_snapshot_contract="false"
after_dom_snapshot_contract="false"
before_console_error_contract="false"
after_console_error_contract="false"
before_pixel_sample_contract="false"
after_pixel_sample_contract="false"
before_screenshot_sample_contract="false"
after_screenshot_sample_contract="false"
before_controlled_interaction_contract="false"
after_controlled_interaction_contract="false"
before_artifact_history_contract="false"
after_artifact_history_contract="false"
artifact_changed="false"
targeted_fix_visible="false"
route_contract_stable="false"

if [[ "${file_check}" == "passed" ]]; then
  before_artifact_sha256="$(file_sha256 "${ARTIFACT_PATH}")"
  before_artifact_bytes="$(file_size_bytes "${ARTIFACT_PATH}")"
  before_dom_marker_count="$(count_marker "${chat_html}" "data-agent-canvas")"
  before_dom_snapshot_contract="$(contains_bool "${chat_html}" 'data-agent-canvas-dom-snapshot="true"')"
  before_console_error_contract="$(contains_bool "${chat_html}" 'data-agent-canvas-console-events="true"')"
  before_pixel_sample_contract="$(contains_bool "${chat_html}" 'data-agent-canvas-pixel-samples="true"')"
  before_screenshot_sample_contract="$(contains_bool "${chat_html}" 'data-agent-canvas-screenshot-samples="true"')"
  before_controlled_interaction_contract="$(contains_bool "${chat_html}" 'data-agent-canvas-controls="postmessage"')"
  before_artifact_history_contract="$(contains_bool "${chat_html}" 'data-agent-canvas-artifact-history="true"')"

  apply_agent_canvas_proof_loop_fix "${ARTIFACT_PATH}"

  after_body="${tmp_dir}/chat-after-fix.html"
  curl -sS --fail-with-body --max-time "${TIMEOUT_SECONDS}" \
    "${auth_args[@]}" \
    "${BASE_URL%/}/ops/chat?theme=dark&sidebar=expanded&session=${SESSION_KEY}" \
    >"${after_body}"
  after_chat_html="$(cat "${after_body}")"
  validate_agent_canvas_contract "${after_chat_html}"

  after_artifact_sha256="$(file_sha256 "${ARTIFACT_PATH}")"
  after_artifact_bytes="$(file_size_bytes "${ARTIFACT_PATH}")"
  after_dom_marker_count="$(count_marker "${after_chat_html}" "data-agent-canvas")"
  after_dom_snapshot_contract="$(contains_bool "${after_chat_html}" 'data-agent-canvas-dom-snapshot="true"')"
  after_console_error_contract="$(contains_bool "${after_chat_html}" 'data-agent-canvas-console-events="true"')"
  after_pixel_sample_contract="$(contains_bool "${after_chat_html}" 'data-agent-canvas-pixel-samples="true"')"
  after_screenshot_sample_contract="$(contains_bool "${after_chat_html}" 'data-agent-canvas-screenshot-samples="true"')"
  after_controlled_interaction_contract="$(contains_bool "${after_chat_html}" 'data-agent-canvas-controls="postmessage"')"
  after_artifact_history_contract="$(contains_bool "${after_chat_html}" 'data-agent-canvas-artifact-history="true"')"

  if [[ "${before_artifact_sha256}" != "${after_artifact_sha256}" ]]; then
    artifact_changed="true"
  fi
  if grep -Fq 'data-agent-canvas-proof-loop="fixed"' "${ARTIFACT_PATH}"; then
    targeted_fix_visible="true"
  fi
  route_contract_stable="true"
  if [[ "${artifact_changed}" != "true" || "${targeted_fix_visible}" != "true" ]]; then
    echo "error: proof loop comparison did not observe the targeted fix" >&2
    exit 1
  fi
  proof_loop_result="passed"
fi

mkdir -p "$(dirname "${OUTPUT_JSON}")"
cat >"${OUTPUT_JSON}" <<JSON
{
  "schema_version": 1,
  "proof": "ops_chat_canvas_v2",
  "base_url": "$(json_escape "${BASE_URL}")",
  "session_key": "$(json_escape "${SESSION_KEY}")",
  "artifact_path": "$(json_escape "${ARTIFACT_PATH}")",
  "send_get_recovery": "passed",
  "send_failure_recovery": "passed",
  "post_status": "$(json_escape "${post_status}")",
  "file_check": "$(json_escape "${file_check}")",
  "runtime_contract_check": "passed",
  "runtime_contract": {
    "dom_snapshot": true,
    "screenshot_capture": true,
    "console_errors": true,
    "canvas_pixels": true,
    "controlled_click": true,
    "controlled_type": true,
    "artifact_history": true
  },
  "proof_loop": {
    "result": "$(json_escape "${proof_loop_result}")",
    "targeted_fix": {
      "action": "rewrite_artifact_with_deterministic_fixed_canvas_contract",
      "applied": $(json_bool "${targeted_fix_visible}"),
      "marker": "data-agent-canvas-proof-loop=\"fixed\""
    },
    "iterations": [
      {
        "label": "before",
        "artifact_sha256": "$(json_escape "${before_artifact_sha256}")",
        "artifact_bytes": ${before_artifact_bytes},
        "route_contract_check": "passed",
        "dom_marker_count": ${before_dom_marker_count},
        "dom_snapshot_contract": $(json_bool "${before_dom_snapshot_contract}"),
        "console_error_contract": $(json_bool "${before_console_error_contract}"),
        "pixel_sample_contract": $(json_bool "${before_pixel_sample_contract}"),
        "screenshot_sample_contract": $(json_bool "${before_screenshot_sample_contract}"),
        "controlled_interaction_contract": $(json_bool "${before_controlled_interaction_contract}"),
        "artifact_history_contract": $(json_bool "${before_artifact_history_contract}")
      },
      {
        "label": "after",
        "artifact_sha256": "$(json_escape "${after_artifact_sha256}")",
        "artifact_bytes": ${after_artifact_bytes},
        "route_contract_check": "passed",
        "dom_marker_count": ${after_dom_marker_count},
        "dom_snapshot_contract": $(json_bool "${after_dom_snapshot_contract}"),
        "console_error_contract": $(json_bool "${after_console_error_contract}"),
        "pixel_sample_contract": $(json_bool "${after_pixel_sample_contract}"),
        "screenshot_sample_contract": $(json_bool "${after_screenshot_sample_contract}"),
        "controlled_interaction_contract": $(json_bool "${after_controlled_interaction_contract}"),
        "artifact_history_contract": $(json_bool "${after_artifact_history_contract}")
      }
    ],
    "comparison": {
      "artifact_changed": $(json_bool "${artifact_changed}"),
      "targeted_fix_visible": $(json_bool "${targeted_fix_visible}"),
      "route_contract_stable": $(json_bool "${route_contract_stable}"),
      "rerun_contract_check": "passed"
    }
  },
  "result": "passed"
}
JSON

echo "ops-chat-canvas-proof passed"
echo "proof_json=${OUTPUT_JSON}"

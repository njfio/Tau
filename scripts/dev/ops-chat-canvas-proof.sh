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

message="Create an HTML canvas demo at ${ARTIFACT_PATH} with a canvas id \"game\" and a short script that draws a green rectangle. Use the write tool."
tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

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
require_contains "${chat_html}" 'id="tau-ops-chat-agent-canvas"' "agent canvas section"
require_contains "${chat_html}" 'data-preview-status="loaded"' "loaded preview status"
require_contains "${chat_html}" 'id="tau-ops-chat-agent-preview-frame"' "preview frame"
require_contains "${chat_html}" 'sandbox="allow-scripts"' "sandboxed preview frame"
require_contains "${chat_html}" 'data-agent-canvas-runtime="postmessage-v2"' "canvas runtime bridge"
require_contains "${chat_html}" 'data-agent-canvas-artifact-history="true"' "artifact history"
require_contains "${chat_html}" 'data-agent-canvas-controls="postmessage"' "controlled interaction surface"
require_contains "${chat_html}" 'data-agent-canvas-diagnostics="true"' "diagnostics surface"

mkdir -p "$(dirname "${OUTPUT_JSON}")"
cat >"${OUTPUT_JSON}" <<JSON
{
  "schema_version": 1,
  "proof": "ops_chat_canvas_v2",
  "base_url": "$(json_escape "${BASE_URL}")",
  "session_key": "$(json_escape "${SESSION_KEY}")",
  "artifact_path": "$(json_escape "${ARTIFACT_PATH}")",
  "post_status": "$(json_escape "${post_status}")",
  "file_check": "$(json_escape "${file_check}")",
  "result": "passed"
}
JSON

echo "ops-chat-canvas-proof passed"
echo "proof_json=${OUTPUT_JSON}"

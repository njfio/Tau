#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "${script_dir}/common.sh"

init_rc=0
tau_demo_common_init "gateway-auth-session" "Run deterministic gateway password-session auth smoke against /gateway/auth/session and /gateway/status." "$@" || init_rc=$?
if [[ "${init_rc}" -eq 64 ]]; then
  exit 0
fi
if [[ "${init_rc}" -ne 0 ]]; then
  exit "${init_rc}"
fi

tau_demo_common_require_command python3
tau_demo_common_prepare_binary

demo_state_dir=".tau/demo-gateway-auth-session"
session_ttl_seconds=1
gateway_password="demo-gateway-password"
server_pid=""
step_total=0
step_passed=0

mkdir -p "${TAU_DEMO_REPO_ROOT}/${demo_state_dir}"
server_log_path="${TAU_DEMO_REPO_ROOT}/${demo_state_dir}/gateway-openresponses.log"

cleanup() {
  if [[ -n "${server_pid}" ]]; then
    if kill -0 "${server_pid}" >/dev/null 2>&1; then
      kill "${server_pid}" >/dev/null 2>&1 || true
      wait "${server_pid}" >/dev/null 2>&1 || true
    fi
  fi
}
trap cleanup EXIT

demo_log() {
  local message="$1"
  echo "[demo:${TAU_DEMO_NAME}] ${message}"
}

run_step_command() {
  if [[ -z "${TAU_DEMO_TIMEOUT_SECONDS}" ]]; then
    "$@"
    return $?
  fi

  python3 - "${TAU_DEMO_TIMEOUT_SECONDS}" "$@" <<'PY'
import subprocess
import sys

timeout_seconds = int(sys.argv[1])
args = sys.argv[2:]

try:
    completed = subprocess.run(args, timeout=timeout_seconds)
except subprocess.TimeoutExpired:
    sys.exit(124)

sys.exit(completed.returncode)
PY
}

step_begin() {
  local label="$1"
  step_total=$((step_total + 1))
  demo_log "[${step_total}] ${label}"
}

step_pass() {
  local label="$1"
  step_passed=$((step_passed + 1))
  demo_log "PASS ${label}"
}

step_fail() {
  local label="$1"
  local rc="$2"
  if [[ "${rc}" -eq 124 && -n "${TAU_DEMO_TIMEOUT_SECONDS}" ]]; then
    demo_log "TIMEOUT ${label} after ${TAU_DEMO_TIMEOUT_SECONDS}s"
  else
    demo_log "FAIL ${label} exit=${rc}"
  fi
  return "${rc}"
}

gateway_bind="$(python3 - <<'PY'
import socket
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.bind(("127.0.0.1", 0))
    print(f"127.0.0.1:{sock.getsockname()[1]}")
PY
)"
gateway_url="http://${gateway_bind}"

demo_log "starting gateway server: ${gateway_url}"
(
  cd "${TAU_DEMO_REPO_ROOT}"
  "${TAU_DEMO_BINARY}" \
    --model openai/gpt-4o-mini \
    --gateway-openresponses-server \
    --gateway-openresponses-bind "${gateway_bind}" \
    --gateway-openresponses-auth-mode password-session \
    --gateway-openresponses-auth-password "${gateway_password}" \
    --gateway-openresponses-session-ttl-seconds "${session_ttl_seconds}" \
    --gateway-state-dir "${demo_state_dir}" \
    >"${server_log_path}" 2>&1
) &
server_pid=$!

step_begin "wait-for-gateway-startup"
if run_step_command python3 - "${gateway_bind}" <<'PY'
import socket
import sys
import time

host, raw_port = sys.argv[1].split(":", 1)
port = int(raw_port)
deadline = time.time() + 10.0

while time.time() < deadline:
    try:
        with socket.create_connection((host, port), timeout=0.5):
            raise SystemExit(0)
    except OSError:
        time.sleep(0.1)

print(f"gateway server did not start within timeout: {host}:{port}", file=sys.stderr)
raise SystemExit(1)
PY
then
  step_pass "wait-for-gateway-startup"
else
  rc=$?
  step_fail "wait-for-gateway-startup" "${rc}"
fi

step_begin "gateway-auth-session-issue-valid-password"
if session_token="$(run_step_command python3 - "${gateway_url}" "${gateway_password}" <<'PY'
import json
import sys
import urllib.error
import urllib.request

gateway_url = sys.argv[1]
password = sys.argv[2]
url = f"{gateway_url}/gateway/auth/session"
body = json.dumps({"password": password}).encode("utf-8")
request = urllib.request.Request(
    url,
    data=body,
    method="POST",
    headers={"content-type": "application/json"},
)

try:
    with urllib.request.urlopen(request, timeout=5) as response:
        payload = json.loads(response.read().decode("utf-8"))
except urllib.error.HTTPError as error:
    print(f"unexpected HTTP status for valid password: {error.code}", file=sys.stderr)
    raise SystemExit(1)

token = payload.get("access_token")
if not isinstance(token, str) or not token:
    print("missing access_token in auth session response", file=sys.stderr)
    raise SystemExit(1)

print(token)
PY
)"; then
  step_pass "gateway-auth-session-issue-valid-password"
else
  rc=$?
  step_fail "gateway-auth-session-issue-valid-password" "${rc}"
fi

step_begin "gateway-status-authorized-with-issued-token"
if run_step_command python3 - "${gateway_url}" "${session_token}" <<'PY'
import json
import sys
import urllib.error
import urllib.request

gateway_url = sys.argv[1]
token = sys.argv[2]
request = urllib.request.Request(
    f"{gateway_url}/gateway/status",
    method="GET",
    headers={"authorization": f"Bearer {token}"},
)

try:
    with urllib.request.urlopen(request, timeout=5) as response:
        if response.status != 200:
            print(f"unexpected status code: {response.status}", file=sys.stderr)
            raise SystemExit(1)
        payload = json.loads(response.read().decode("utf-8"))
except urllib.error.HTTPError as error:
    print(f"unexpected HTTP status for authorized status call: {error.code}", file=sys.stderr)
    raise SystemExit(1)

if not isinstance(payload, dict):
    print("gateway status response payload is not an object", file=sys.stderr)
    raise SystemExit(1)
if "auth" not in payload:
    print("gateway status response missing auth section", file=sys.stderr)
    raise SystemExit(1)
PY
then
  step_pass "gateway-status-authorized-with-issued-token"
else
  rc=$?
  step_fail "gateway-status-authorized-with-issued-token" "${rc}"
fi

step_begin "gateway-auth-session-invalid-password-fails-closed"
if run_step_command python3 - "${gateway_url}" <<'PY'
import json
import sys
import urllib.error
import urllib.request

gateway_url = sys.argv[1]
url = f"{gateway_url}/gateway/auth/session"
body = json.dumps({"password": "wrong-password"}).encode("utf-8")
request = urllib.request.Request(
    url,
    data=body,
    method="POST",
    headers={"content-type": "application/json"},
)

try:
    urllib.request.urlopen(request, timeout=5)
except urllib.error.HTTPError as error:
    if error.code == 401:
        raise SystemExit(0)
    print(f"unexpected HTTP status for invalid password: {error.code}", file=sys.stderr)
    raise SystemExit(1)

print("invalid password unexpectedly accepted", file=sys.stderr)
raise SystemExit(1)
PY
then
  step_pass "gateway-auth-session-invalid-password-fails-closed"
else
  rc=$?
  step_fail "gateway-auth-session-invalid-password-fails-closed" "${rc}"
fi

step_begin "gateway-status-expired-token-fails-closed"
if run_step_command python3 - "${gateway_url}" "${session_token}" <<'PY'
import sys
import time
import urllib.error
import urllib.request

gateway_url = sys.argv[1]
token = sys.argv[2]
time.sleep(2.2)
request = urllib.request.Request(
    f"{gateway_url}/gateway/status",
    method="GET",
    headers={"authorization": f"Bearer {token}"},
)

try:
    urllib.request.urlopen(request, timeout=5)
except urllib.error.HTTPError as error:
    if error.code == 401:
        raise SystemExit(0)
    print(f"unexpected HTTP status for expired token: {error.code}", file=sys.stderr)
    raise SystemExit(1)

print("expired token unexpectedly accepted", file=sys.stderr)
raise SystemExit(1)
PY
then
  step_pass "gateway-status-expired-token-fails-closed"
else
  rc=$?
  step_fail "gateway-status-expired-token-fails-closed" "${rc}"
fi

failed=$((step_total - step_passed))
demo_log "summary: total=${step_total} passed=${step_passed} failed=${failed}"
if [[ "${failed}" -gt 0 ]]; then
  exit 1
fi

#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust_pi-3803-target}"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tau-openrouter-repair.XXXXXX")"
trap 'if [ -n "${SERVER_PID:-}" ]; then kill "${SERVER_PID}" 2>/dev/null || true; fi; rm -rf "${WORK_DIR}"' EXIT

export CARGO_TARGET_DIR="${TARGET_DIR}"

run_cli() {
  cargo run -q -p tau-coding-agent --bin tau_autonomous_coding_job -- "$@"
}

init_fixture_repo() {
  local repo="$1"
  mkdir -p "${repo}/docs"
  git -C "${repo}" init -b master >/dev/null
  git -C "${repo}" config user.email tau@example.test
  git -C "${repo}" config user.name "Tau Test"
  printf 'fail\n' >"${repo}/status.txt"
  printf 'draft\n' >"${repo}/docs/notes.txt"
  git -C "${repo}" add .
  git -C "${repo}" commit -m "Initial fixture" >/dev/null
}

cd "${REPO_ROOT}"

SERVER="${WORK_DIR}/mock_openrouter.py"
PORT_FILE="${WORK_DIR}/mock_openrouter.port"
REQUEST_FILE="${WORK_DIR}/mock_openrouter_request.json"
cat >"${SERVER}" <<'PY'
import http.server
import json
import sys

port_file = sys.argv[1]
request_file = sys.argv[2]

class Handler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("content-length", "0"))
        raw = self.rfile.read(length).decode("utf-8")
        payload = json.loads(raw)
        with open(request_file, "w", encoding="utf-8") as handle:
            json.dump(
                {
                    "path": self.path,
                    "authorization": self.headers.get("authorization"),
                    "payload": payload,
                },
                handle,
                indent=2,
            )
        content = json.dumps(
            {
                "edits": [
                    {
                        "relative_path": "status.txt",
                        "contents": "pass\n",
                        "reason_code": "openrouter_adapter_status",
                    },
                    {
                        "relative_path": "docs/notes.txt",
                        "contents": "proof\n",
                        "reason_code": "openrouter_adapter_notes",
                    },
                ]
            }
        )
        response = {
            "choices": [
                {
                    "message": {"role": "assistant", "content": content},
                    "finish_reason": "stop",
                }
            ],
            "usage": {
                "prompt_tokens": 41,
                "completion_tokens": 17,
                "total_tokens": 58,
            },
        }
        data = json.dumps(response).encode("utf-8")
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, fmt, *args):
        return

server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
with open(port_file, "w", encoding="utf-8") as handle:
    handle.write(str(server.server_port))
server.serve_forever()
PY

python3 "${SERVER}" "${PORT_FILE}" "${REQUEST_FILE}" &
SERVER_PID="$!"
for _ in $(seq 1 50); do
  if [ -s "${PORT_FILE}" ]; then
    break
  fi
  sleep 0.1
done
if [ ! -s "${PORT_FILE}" ]; then
  echo "mock OpenRouter server did not start" >&2
  exit 1
fi
PORT="$(cat "${PORT_FILE}")"

STATE="${WORK_DIR}/state"
JOBS="${WORK_DIR}/jobs"
REPO="${WORK_DIR}/repo"
mkdir -p "${REPO}"
init_fixture_repo "${REPO}"

OPENROUTER_API_KEY=test-openrouter-key run_cli issue-to-merge \
  --state-dir "${STATE}" \
  --jobs-state-dir "${JOBS}" \
  --repo-path "${REPO}" \
  --intake-id issue-3803-openrouter \
  --mission-id issue-3803-openrouter-mission \
  --session-key script-openrouter-repair \
  --issue-url https://github.com/njfio/Tau/issues/3803 \
  --issue-title "Built-in OpenRouter repair" \
  --issue-body "Use the first-class provider adapter to repair verifier failures." \
  --goal "Make the OpenRouter repair verifier pass" \
  --verifier-command "grep -q pass status.txt" \
  --verifier-command "grep -q proof docs/notes.txt" \
  --provider-repair-openrouter \
  --provider-repair-attempts 1 \
  --provider-repair-model openrouter/test/repair-model \
  --provider-repair-api-base "http://127.0.0.1:${PORT}/api/v1" \
  --provider-repair-max-tokens 333 \
  --commit-message "Make OpenRouter repair verifier green" \
  --pr-mode pr-ready \
  >"${WORK_DIR}/issue-to-merge-openrouter.json"

python3 - \
  "${WORK_DIR}/issue-to-merge-openrouter.json" \
  "${REQUEST_FILE}" \
  "${REPO}/status.txt" \
  "${REPO}/docs/notes.txt" <<'PY'
import json
import os
import sys

with open(sys.argv[1], "r", encoding="utf-8") as handle:
    payload = json.load(handle)

assert payload["status"] == "pr_ready", payload
run = payload["run"]["status"]
assert run["status"] == "pr_ready", run
assert run["provider_repair_status"] == "applied", run
assert run["provider_repair_reason_code"] == "provider_repair_edit_parsed", run
assert run["provider_repair_provider"] == "openrouter", run
assert run["provider_repair_model"] == "openrouter/test/repair-model", run
assert run["provider_repair_attempts"] == 1, run
assert run["provider_repair_max_attempts"] == 1, run

context_path = run["provider_repair_context_path"]
assert context_path and os.path.exists(context_path), context_path
metadata_path = f"{context_path}.openrouter-call.json"
assert os.path.exists(metadata_path), metadata_path
with open(metadata_path, "r", encoding="utf-8") as handle:
    metadata = json.load(handle)
assert metadata["status"] == "validated", metadata
assert metadata["provider"] == "openrouter", metadata
assert metadata["model"] == "openrouter/test/repair-model", metadata
assert metadata["api_model"] == "test/repair-model", metadata
assert metadata["auth_source"] == "OPENROUTER_API_KEY", metadata
assert metadata["edit_count"] == 2, metadata
assert metadata["finish_reason"] == "stop", metadata
assert metadata["usage"]["total_tokens"] == 58, metadata

with open(sys.argv[2], "r", encoding="utf-8") as handle:
    request = json.load(handle)
assert request["path"] == "/api/v1/chat/completions", request
assert request["authorization"] == "Bearer test-openrouter-key", request
body = request["payload"]
assert body["model"] == "test/repair-model", body
assert body["response_format"]["type"] == "json_object", body
assert body["tool_choice"] == "none", body
assert body["max_tokens"] == 333, body

with open(sys.argv[3], "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "pass"
with open(sys.argv[4], "r", encoding="utf-8") as handle:
    assert handle.read().strip() == "proof"
PY

printf 'openrouter_repair_adapter=pass\n'

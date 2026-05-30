#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PROOF_SCRIPT="${REPO_ROOT}/scripts/dev/ops-chat-canvas-proof.sh"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected '${needle}'" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

if [[ ! -x "${PROOF_SCRIPT}" ]]; then
  echo "missing executable proof script: ${PROOF_SCRIPT}" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

fake_curl="${tmp_dir}/curl"
cat >"${fake_curl}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'curl_args=%s\n' "$*" >>"${OPS_CHAT_CANVAS_FAKE_CURL_LOG:?}"
headers_file=""
body_file=""
url=""
previous=""
for arg in "$@"; do
  if [[ "${previous}" == "-D" ]]; then
    headers_file="${arg}"
    previous=""
    continue
  fi
  if [[ "${previous}" == "-o" ]]; then
    body_file="${arg}"
    previous=""
    continue
  fi
  if [[ "${arg}" == "-D" || "${arg}" == "-o" ]]; then
    previous="${arg}"
    continue
  fi
  if [[ "${arg}" == http* ]]; then
    url="${arg}"
  fi
done

write_redirect_headers() {
  local location="$1"
  if [[ -n "${headers_file}" ]]; then
    printf 'HTTP/1.1 303 See Other\r\nlocation: %s\r\n\r\n' "${location}" >"${headers_file}"
  fi
  if [[ -n "${body_file}" ]]; then
    : >"${body_file}"
  fi
}

if [[ "$*" == *"/ops/chat/send"* ]]; then
  if [[ "$*" == *"message=Create an HTML canvas demo"* ]]; then
    write_redirect_headers "/ops/chat?theme=dark&sidebar=expanded&session=proof-session#tau-ops-chat-agent-canvas"
    printf '303'
    exit 0
  fi
  if [[ "$*" == *"--data-urlencode"* ]]; then
    write_redirect_headers "/ops/chat?theme=dark&sidebar=expanded&session=proof-session&chat_status=input-too-large"
    printf '303'
    exit 0
  fi
  write_redirect_headers "/ops/chat?theme=dark&sidebar=expanded&session=proof-session"
  printf '303'
  exit 0
fi
if [[ "${url}" == *"/ops/chat?"* ]]; then
  cat <<'HTML'
<!doctype html>
<section id="tau-ops-chat-agent-canvas" data-preview-status="loaded" data-preview-runtime-status="pending" data-dom-node-count="0" data-dom-snapshot-count="0" data-canvas-count="0" data-console-error-count="0" data-pixel-sample-count="0" data-screenshot-sample-count="0" data-interaction-mode="postmessage">
  <iframe id="tau-ops-chat-agent-preview-frame" sandbox="allow-scripts"></iframe>
  <form id="tau-ops-chat-agent-canvas-controls" data-agent-canvas-controls="postmessage">
    <button id="tau-ops-chat-agent-canvas-probe" type="button" data-agent-canvas-tool="snapshot">Probe</button>
    <button id="tau-ops-chat-agent-canvas-click" type="button" data-agent-canvas-tool="click">Click</button>
    <button id="tau-ops-chat-agent-canvas-type" type="button" data-agent-canvas-tool="type">Type</button>
  </form>
  <section id="tau-ops-chat-agent-canvas-diagnostics" data-agent-canvas-diagnostics="true">
    <ul id="tau-ops-chat-agent-canvas-dom-snapshot" data-agent-canvas-dom-snapshot="true"></ul>
    <ul id="tau-ops-chat-agent-canvas-console" data-agent-canvas-console-events="true"></ul>
    <ul id="tau-ops-chat-agent-canvas-pixels" data-agent-canvas-pixel-samples="true"></ul>
    <ul id="tau-ops-chat-agent-canvas-screenshots" data-agent-canvas-screenshot-samples="true"></ul>
  </section>
  <ol id="tau-ops-chat-agent-canvas-artifacts" data-agent-canvas-artifact-history="true" data-artifact-count="1"></ol>
  <script id="tau-ops-chat-agent-canvas-runtime" data-agent-canvas-runtime="postmessage-v2"></script>
</section>
HTML
  exit 0
fi
printf 'unexpected url: %s\n' "${url}" >&2
exit 9
EOF
chmod +x "${fake_curl}"

artifact="${tmp_dir}/ops-chat-canvas-proof.html"
printf '<!doctype html><canvas id="game"></canvas>\n' >"${artifact}"
output_json="${tmp_dir}/proof.json"
curl_log="${tmp_dir}/curl.log"

output="$(
  PATH="${tmp_dir}:${PATH}" \
  OPS_CHAT_CANVAS_FAKE_CURL_LOG="${curl_log}" \
  TAU_OPS_CHAT_CANVAS_PROOF_RECOVERY_MESSAGE_CHARS=64 \
  "${PROOF_SCRIPT}" \
    --base-url http://fake-gateway.local \
    --session proof-session \
    --artifact-path "${artifact}" \
    --output-json "${output_json}"
)"

assert_contains "${output}" "ops-chat-canvas-proof passed" "success output"
assert_contains "$(cat "${output_json}")" '"proof": "ops_chat_canvas_v2"' "proof kind"
assert_contains "$(cat "${output_json}")" '"result": "passed"' "proof result"
assert_contains "$(cat "${output_json}")" '"file_check": "passed"' "file check result"
assert_contains "$(cat "${output_json}")" '"send_get_recovery": "passed"' "GET recovery result"
assert_contains "$(cat "${output_json}")" '"send_failure_recovery": "passed"' "failure recovery result"
assert_contains "$(cat "${output_json}")" '"runtime_contract_check": "passed"' "runtime contract result"
assert_contains "$(cat "${curl_log}")" "/ops/chat/send" "chat send call"
assert_contains "$(cat "${curl_log}")" "/ops/chat?theme=dark&sidebar=expanded&session=proof-session" "chat render call"

echo "ops-chat-canvas-proof tests passed"

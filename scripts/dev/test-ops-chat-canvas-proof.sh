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
url="${!#}"
if [[ "$*" == *"/ops/chat/send"* ]]; then
  printf '303'
  exit 0
fi
if [[ "${url}" == *"/ops/chat?"* ]]; then
  cat <<'HTML'
<!doctype html>
<section id="tau-ops-chat-agent-canvas" data-preview-status="loaded">
  <iframe id="tau-ops-chat-agent-preview-frame" sandbox="allow-scripts"></iframe>
  <form id="tau-ops-chat-agent-canvas-controls" data-agent-canvas-controls="postmessage"></form>
  <section id="tau-ops-chat-agent-canvas-diagnostics" data-agent-canvas-diagnostics="true"></section>
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
assert_contains "$(cat "${curl_log}")" "/ops/chat/send" "chat send call"
assert_contains "$(cat "${curl_log}")" "/ops/chat?theme=dark&sidebar=expanded&session=proof-session" "chat render call"

echo "ops-chat-canvas-proof tests passed"

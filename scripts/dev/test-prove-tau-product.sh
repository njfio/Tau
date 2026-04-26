#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PROOF_SCRIPT="${REPO_ROOT}/scripts/dev/prove-tau-product.sh"

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to contain '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

count_runner_mode() {
  local mode="$1"
  local log_path="$2"
  grep -c "^runner_mode=${mode}\$" "${log_path}" 2>/dev/null || true
}

assert_runner_mode_seen() {
  local mode="$1"
  local log_path="$2"
  local count
  count="$(count_runner_mode "${mode}" "${log_path}")"
  if [[ "${count}" -lt 1 ]]; then
    echo "assertion failed: expected runner mode '${mode}' to be invoked" >&2
    cat "${log_path}" >&2
    exit 1
  fi
}

assert_runner_mode_count_at_least() {
  local mode="$1"
  local log_path="$2"
  local minimum="$3"
  local count
  count="$(count_runner_mode "${mode}" "${log_path}")"
  if [[ "${count}" -lt "${minimum}" ]]; then
    echo "assertion failed: expected runner mode '${mode}' count >= ${minimum}, got ${count}" >&2
    cat "${log_path}" >&2
    exit 1
  fi
}

assert_fails_with() {
  local output="$1"
  local needle="$2"
  local label="$3"
  assert_contains "${output}" "${needle}" "${label}"
}

if [[ ! -x "${PROOF_SCRIPT}" ]]; then
  echo "missing executable proof script: ${PROOF_SCRIPT}" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

runner="${tmp_dir}/runner.sh"
cat >"${runner}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
mode="$1"
log_path="$2"
pid_path="$3"
shift 3
case "${mode}" in
  up)
    if [[ "${TAU_PRODUCT_PROOF_RUNNER_FAIL_MODE:-}" == "launcher-failure" ]]; then
      printf 'runner_mode=up\nargs=%s\n' "$*" >>"${log_path}"
      exit 21
    fi
    printf 'runner_mode=up\nargs=%s\n' "$*" >>"${log_path}"
    nohup sleep 120 >/dev/null 2>&1 &
    bg_pid=$!
    echo "${bg_pid}" >"${pid_path}"
    ;;
  down)
    printf 'runner_mode=down\nargs=%s\n' "$*" >>"${log_path}"
    if [[ -f "${pid_path}" ]]; then
      kill "$(cat "${pid_path}")" >/dev/null 2>&1 || true
      rm -f "${pid_path}"
    fi
    ;;
  status)
    printf 'runner_mode=status\nargs=%s\n' "$*" >>"${log_path}"
    if [[ "${TAU_PRODUCT_PROOF_RUNNER_FAIL_MODE:-}" == "status-failure" ]]; then
      exit 22
    fi
    ;;
  tui)
    printf 'runner_mode=tui\nargs=%s\n' "$*" >>"${log_path}"
    ;;
  *)
    printf 'runner_mode=unknown\nargs=%s\n' "$*" >>"${log_path}"
    exit 12
    ;;
esac
EOF
chmod +x "${runner}"

fake_curl="${tmp_dir}/curl"
cat >"${fake_curl}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'curl_args=%s\n' "$*" >>"${TAU_PRODUCT_PROOF_CURL_LOG:?}"
url="${!#}"
case "${TAU_PRODUCT_PROOF_CURL_CASE:-success}" in
  success)
    if [[ "${url}" == */webchat ]]; then
      printf '<!doctype html><title>Tau Gateway Webchat</title><button>Dashboard</button><pre id="dashboardStatus"></pre>\n'
    elif [[ "${url}" == */gateway/sessions ]]; then
      printf '{"sessions":[]}\n'
    elif [[ "${url}" == */gateway/memory/* ]]; then
      printf '{"exists":false}\n'
    else
      printf '{"status":"ok","source":"fake-product-proof"}\n'
    fi
    ;;
  invalid-json)
    printf '[]\n'
    ;;
  curl-failure)
    exit 7
    ;;
  webchat-missing-marker)
    if [[ "${url}" == */webchat ]]; then
      printf '<!doctype html><title>Not the webchat shell</title>\n'
    elif [[ "${url}" == */gateway/sessions ]]; then
      printf '{"sessions":[]}\n'
    elif [[ "${url}" == */gateway/memory/* ]]; then
      printf '{"exists":false}\n'
    else
      printf '{"status":"ok","source":"fake-product-proof"}\n'
    fi
    ;;
  webchat-curl-failure)
    if [[ "${url}" == */webchat ]]; then
      exit 7
    fi
    if [[ "${url}" == */gateway/sessions ]]; then
      printf '{"sessions":[]}\n'
      exit 0
    fi
    if [[ "${url}" == */gateway/memory/* ]]; then
      printf '{"exists":false}\n'
      exit 0
    fi
    printf '{"status":"ok","source":"fake-product-proof"}\n'
    ;;
  sessions-invalid-json)
    if [[ "${url}" == */gateway/sessions ]]; then
      printf '[]\n'
      exit 0
    fi
    printf '{"status":"ok","source":"fake-product-proof"}\n'
    ;;
  memory-invalid-json)
    if [[ "${url}" == */gateway/memory/* ]]; then
      printf '[]\n'
      exit 0
    fi
    if [[ "${url}" == */gateway/sessions ]]; then
      printf '{"sessions":[]}\n'
      exit 0
    fi
    printf '{"status":"ok","source":"fake-product-proof"}\n'
    ;;
  memory-curl-failure)
    if [[ "${url}" == */gateway/memory/* ]]; then
      exit 7
    fi
    if [[ "${url}" == */gateway/sessions ]]; then
      printf '{"sessions":[]}\n'
      exit 0
    fi
    printf '{"status":"ok","source":"fake-product-proof"}\n'
    ;;
  sessions-curl-failure)
    if [[ "${url}" == */gateway/sessions ]]; then
      exit 7
    fi
    printf '{"status":"ok","source":"fake-product-proof"}\n'
    ;;
  *)
    printf 'unknown curl case: %s\n' "${TAU_PRODUCT_PROOF_CURL_CASE}" >&2
    exit 8
    ;;
esac
EOF
chmod +x "${fake_curl}"

run_case() {
  local case_name="$1"
  local expected_status="$2"
  local expected_output="$3"
  local curl_case="$4"
  local runner_fail_mode="${5:-}"
  local expect_curl_url="${6:-yes}"
  local webchat_smoke="${7:-no}"
  local sessions_smoke="${8:-no}"
  local memory_smoke="${9:-no}"
  local case_dir="${tmp_dir}/${case_name}"
  local runtime_dir="${case_dir}/runtime"
  local runner_log="${case_dir}/runner.log"
  local runner_pid="${case_dir}/runner.pid"
  local curl_log="${case_dir}/curl.log"
  local report_json="${case_dir}/report.json"
  local output
  local status
  local proof_args=(--run --report "${report_json}")

  if [[ "${webchat_smoke}" == "yes" ]]; then
    proof_args=(--run --webchat-smoke --report "${report_json}")
  fi
  if [[ "${sessions_smoke}" == "yes" ]]; then
    proof_args=("${proof_args[@]:0:1}" --sessions-smoke "${proof_args[@]:1}")
  fi
  if [[ "${memory_smoke}" == "yes" ]]; then
    proof_args=("${proof_args[@]:0:1}" --memory-smoke "${proof_args[@]:1}")
  fi

  mkdir -p "${case_dir}"

  set +e
  output="$(
    TAU_UNIFIED_RUNNER="${runner}" \
    TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
    TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
    TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
    TAU_PRODUCT_PROOF_CURL_BIN="${fake_curl}" \
    TAU_PRODUCT_PROOF_CURL_LOG="${curl_log}" \
    TAU_PRODUCT_PROOF_CURL_CASE="${curl_case}" \
    TAU_PRODUCT_PROOF_RUNNER_FAIL_MODE="${runner_fail_mode}" \
    TAU_PRODUCT_PROOF_STATUS_ATTEMPTS="2" \
    TAU_PRODUCT_PROOF_STATUS_RETRY_DELAY="0" \
    TAU_UNIFIED_BIND="127.0.0.1:8898" \
    "${PROOF_SCRIPT}" "${proof_args[@]}" 2>&1
  )"
  status=$?
  set -e

  if [[ "${expected_status}" == "success" && "${status}" -ne 0 ]]; then
    echo "case ${case_name} failed unexpectedly with exit ${status}" >&2
    echo "${output}" >&2
    exit 1
  fi
  if [[ "${expected_status}" == "failure" && "${status}" -eq 0 ]]; then
    echo "case ${case_name} succeeded unexpectedly" >&2
    echo "${output}" >&2
    exit 1
  fi

  assert_contains "${output}" "${expected_output}" "${case_name} output"
  assert_runner_mode_seen up "${runner_log}"
  assert_runner_mode_seen down "${runner_log}"
  if [[ "${expect_curl_url}" == "yes" ]]; then
    assert_contains "$(cat "${curl_log}" 2>/dev/null || true)" "http://127.0.0.1:8898/gateway/status" "${case_name} gateway status URL"
  fi
  if [[ "${webchat_smoke}" == "yes" ]]; then
    assert_contains "$(cat "${curl_log}" 2>/dev/null || true)" "http://127.0.0.1:8898/webchat" "${case_name} webchat URL"
  fi
  if [[ "${sessions_smoke}" == "yes" ]]; then
    assert_contains "$(cat "${curl_log}" 2>/dev/null || true)" "http://127.0.0.1:8898/gateway/sessions" "${case_name} gateway sessions URL"
  fi
  if [[ "${memory_smoke}" == "yes" ]]; then
    assert_contains "$(cat "${curl_log}" 2>/dev/null || true)" "http://127.0.0.1:8898/gateway/memory/default" "${case_name} gateway memory URL"
  fi

  if [[ "${expected_status}" == "success" ]]; then
    python3 - "${report_json}" "${webchat_smoke}" "${sessions_smoke}" "${memory_smoke}" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)

webchat_smoke = sys.argv[2] == "yes"
sessions_smoke = sys.argv[3] == "yes"
memory_smoke = sys.argv[4] == "yes"
expected_steps = ["up", "status", "gateway_status", "tui", "down"]
expected_middle_steps = []
if webchat_smoke:
    expected_middle_steps.append("webchat")
if sessions_smoke:
    expected_middle_steps.append("sessions_api")
if memory_smoke:
  expected_middle_steps.append("memory_api")
if expected_middle_steps:
    expected_steps = ["up", "status", "gateway_status", *expected_middle_steps, "tui", "down"]

assert payload["mode"] == "run", "product-proof report mode"
assert payload["status"] == "passed", "product-proof report status"
assert payload["gateway_status_url"] == "http://127.0.0.1:8898/gateway/status"
assert payload["completed_steps"] == expected_steps
if webchat_smoke:
    assert payload["webchat_url"] == "http://127.0.0.1:8898/webchat"
else:
    assert "webchat_url" not in payload
if sessions_smoke:
    assert payload["gateway_sessions_url"] == "http://127.0.0.1:8898/gateway/sessions"
else:
    assert "gateway_sessions_url" not in payload
if memory_smoke:
  assert payload["gateway_memory_url"] == "http://127.0.0.1:8898/gateway/memory/default"
else:
  assert "gateway_memory_url" not in payload
PY
  fi

  printf '%s\n' "${runner_log}"
}

success_runner_log="$(run_case success success "Tau product proof passed" success)"
assert_runner_mode_seen status "${success_runner_log}"
assert_runner_mode_seen tui "${success_runner_log}"

webchat_success_runner_log="$(run_case webchat-success success "Tau product proof passed" success "" yes yes)"
assert_runner_mode_seen status "${webchat_success_runner_log}"
assert_runner_mode_seen tui "${webchat_success_runner_log}"

sessions_success_runner_log="$(run_case sessions-success success "Tau product proof passed" success "" yes no yes)"
assert_runner_mode_seen status "${sessions_success_runner_log}"
assert_runner_mode_seen tui "${sessions_success_runner_log}"

webchat_sessions_success_runner_log="$(run_case webchat-sessions-success success "Tau product proof passed" success "" yes yes yes)"
assert_runner_mode_seen status "${webchat_sessions_success_runner_log}"
assert_runner_mode_seen tui "${webchat_sessions_success_runner_log}"

memory_success_runner_log="$(run_case memory-success success "Tau product proof passed" success "" yes no no yes)"
assert_runner_mode_seen status "${memory_success_runner_log}"
assert_runner_mode_seen tui "${memory_success_runner_log}"

all_smokes_success_runner_log="$(run_case all-smokes-success success "Tau product proof passed" success "" yes yes yes yes)"
assert_runner_mode_seen status "${all_smokes_success_runner_log}"
assert_runner_mode_seen tui "${all_smokes_success_runner_log}"

webchat_missing_marker_runner_log="$(run_case webchat-missing-marker failure "webchat response missing expected marker" webchat-missing-marker "" yes yes)"
assert_runner_mode_seen down "${webchat_missing_marker_runner_log}"

webchat_curl_failure_runner_log="$(run_case webchat-curl-failure failure "webchat endpoint did not respond" webchat-curl-failure "" yes yes)"
assert_runner_mode_seen down "${webchat_curl_failure_runner_log}"

sessions_invalid_json_runner_log="$(run_case sessions-invalid-json failure "gateway sessions response is not a JSON object with sessions array" sessions-invalid-json "" yes no yes)"
assert_runner_mode_seen down "${sessions_invalid_json_runner_log}"

sessions_curl_failure_runner_log="$(run_case sessions-curl-failure failure "gateway sessions endpoint did not respond" sessions-curl-failure "" yes no yes)"
assert_runner_mode_seen down "${sessions_curl_failure_runner_log}"

memory_invalid_json_runner_log="$(run_case memory-invalid-json failure "gateway memory response is not a JSON object with exists boolean" memory-invalid-json "" yes no no yes)"
assert_runner_mode_seen down "${memory_invalid_json_runner_log}"

memory_curl_failure_runner_log="$(run_case memory-curl-failure failure "gateway memory endpoint did not respond" memory-curl-failure "" yes no no yes)"
assert_runner_mode_seen down "${memory_curl_failure_runner_log}"

invalid_json_runner_log="$(run_case invalid-json failure "gateway/status response is not a JSON object" invalid-json)"
assert_runner_mode_seen down "${invalid_json_runner_log}"

curl_failure_runner_log="$(run_case curl-failure failure "gateway status endpoint did not become ready" curl-failure)"
assert_runner_mode_seen down "${curl_failure_runner_log}"

status_failure_runner_log="$(run_case status-failure failure "tau-unified: started" success status-failure no-curl)"
assert_runner_mode_seen status "${status_failure_runner_log}"
assert_runner_mode_seen down "${status_failure_runner_log}"

echo "Tau product-proof run contract passed"
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LAUNCHER_SCRIPT="${SCRIPT_DIR}/tau-unified.sh"

assert_equals() {
  local expected="$1"
  local actual="$2"
  local label="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    echo "assertion failed (${label}): expected '${expected}' got '${actual}'" >&2
    exit 1
  fi
}

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

assert_not_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  if [[ "${haystack}" == *"${needle}"* ]]; then
    echo "assertion failed (${label}): expected output to omit '${needle}'" >&2
    echo "actual output:" >&2
    echo "${haystack}" >&2
    exit 1
  fi
}

if [[ ! -x "${LAUNCHER_SCRIPT}" ]]; then
  echo "error: launcher script missing or not executable: ${LAUNCHER_SCRIPT}" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

runtime_dir="${tmp_dir}/runtime"
runner_log="${tmp_dir}/runner.log"
runner_pid="${tmp_dir}/runner.pid"

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
    printf 'runner_mode=up\nargs=%s\n' "$*" >>"${log_path}"
    nohup sleep 120 >/dev/null 2>&1 &
    bg_pid=$!
    echo "${bg_pid}" > "${pid_path}"
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

bash -n "${LAUNCHER_SCRIPT}"

set +e
unknown_output="$("${LAUNCHER_SCRIPT}" nonsense 2>&1)"
unknown_rc=$?
set -e
assert_equals "2" "${unknown_rc}" "unknown command exit"
assert_contains "${unknown_output}" "unknown command: nonsense" "unknown command output"

set +e
up_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" up --profile test-profile --model openai/gpt-5.2 --bind 127.0.0.1:8899 --auth-mode localhost-dev 2>&1
)"
up_rc=$?
set -e
assert_equals "0" "${up_rc}" "up exit"
assert_contains "${up_output}" "tau-unified: started" "up output marker"
assert_contains "${up_output}" "http://127.0.0.1:8899/webchat" "up webchat endpoint"

pid_file="${runtime_dir}/tau-unified.pid"
log_file="${runtime_dir}/tau-unified.log"
cmd_file="${runtime_dir}/tau-unified.last-cmd"

if [[ ! -f "${pid_file}" ]]; then
  echo "expected pid file to exist after up: ${pid_file}" >&2
  exit 1
fi
if [[ ! -f "${log_file}" ]]; then
  echo "expected log file to exist after up: ${log_file}" >&2
  exit 1
fi
if [[ ! -f "${cmd_file}" ]]; then
  echo "expected command file to exist after up: ${cmd_file}" >&2
  exit 1
fi

status_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" status 2>&1
)"
assert_contains "${status_output}" "tau-unified: running" "status running marker"
assert_contains "${status_output}" "pid=" "status pid marker"

down_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" down 2>&1
)"
assert_contains "${down_output}" "tau-unified: stopped" "down marker"

if [[ -f "${pid_file}" ]]; then
  echo "expected pid file to be removed after down: ${pid_file}" >&2
  exit 1
fi

set +e
down_again_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" down 2>&1
)"
down_again_rc=$?
set -e
assert_equals "1" "${down_again_rc}" "down when stopped exit"
assert_contains "${down_again_output}" "tau-unified: not running" "down when stopped output"

up_count_before_tui="$(grep -c '^runner_mode=up$' "${runner_log}" || true)"

tui_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" tui --no-color 2>&1 || true
)"
assert_contains "${tui_output}" "tau-unified: launching tui (agent)" "tui agent marker"
up_count_after_tui="$(grep -c '^runner_mode=up$' "${runner_log}" || true)"
assert_equals "${up_count_before_tui}" "${up_count_after_tui}" "tui default does not bootstrap runtime in runner mode"
assert_contains "$(cat "${runner_log}")" "--request-timeout-ms 45000" "tui default timeout flag"
assert_contains "$(cat "${runner_log}")" "--agent-request-max-retries 0" "tui default retries flag"

up_count_before_bootstrap="$(grep -c '^runner_mode=up$' "${runner_log}" || true)"
tui_bootstrap_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" tui --bootstrap-runtime --no-color 2>&1 || true
)"
assert_contains "${tui_bootstrap_output}" "tau-unified: bootstrapping runtime for tui" "tui bootstrap marker"
assert_contains "${tui_bootstrap_output}" "tau-unified: started" "tui bootstrap started"
up_count_after_bootstrap="$(grep -c '^runner_mode=up$' "${runner_log}" || true)"
if [[ "${up_count_after_bootstrap}" -le "${up_count_before_bootstrap}" ]]; then
  echo "assertion failed (runner up logged for bootstrap path): expected up count to increase" >&2
  echo "before=${up_count_before_bootstrap} after=${up_count_after_bootstrap}" >&2
  echo "runner log:" >&2
  cat "${runner_log}" >&2
  exit 1
fi

tui_live_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" tui --live-shell --iterations 2 --interval-ms 15 --no-color 2>&1 || true
)"
assert_contains "${tui_live_output}" "tau-unified: launching tui (live-shell)" "tui live marker"

tui_override_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" tui --request-timeout-ms 9000 --agent-request-max-retries 2 --no-color 2>&1 || true
)"
assert_contains "${tui_override_output}" "tau-unified: launching tui (agent)" "tui override marker"
assert_contains "$(cat "${runner_log}")" "--request-timeout-ms 9000" "tui override timeout flag"
assert_contains "$(cat "${runner_log}")" "--agent-request-max-retries 2" "tui override retries flag"

assert_contains "$(cat "${runner_log}")" "runner_mode=up" "runner up logged"
assert_contains "$(cat "${runner_log}")" "runner_mode=status" "runner status logged"
assert_contains "$(cat "${runner_log}")" "runner_mode=down" "runner down logged"
assert_contains "$(cat "${runner_log}")" "runner_mode=tui" "runner tui logged"
assert_contains "$(cat "${runner_log}")" "args=agent" "runner tui agent args"
assert_contains "$(cat "${runner_log}")" "args=live-shell" "runner tui live-shell args"

echo "tau-unified launcher tests passed"

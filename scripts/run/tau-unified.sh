#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

RUNTIME_DIR_DEFAULT="${REPO_ROOT}/.tau/unified"
RUNTIME_DIR="${TAU_UNIFIED_RUNTIME_DIR:-${RUNTIME_DIR_DEFAULT}}"
PID_FILE="${RUNTIME_DIR}/tau-unified.pid"
LOG_FILE="${RUNTIME_DIR}/tau-unified.log"
CMD_FILE="${RUNTIME_DIR}/tau-unified.last-cmd"

MODEL_DEFAULT="${TAU_UNIFIED_MODEL:-gpt-5.3-codex}"
BIND_DEFAULT="${TAU_UNIFIED_BIND:-127.0.0.1:8791}"
AUTH_MODE_DEFAULT="${TAU_UNIFIED_AUTH_MODE:-localhost-dev}"
AUTH_TOKEN_DEFAULT="${TAU_UNIFIED_AUTH_TOKEN:-local-dev-token}"
AUTH_PASSWORD_DEFAULT="${TAU_UNIFIED_AUTH_PASSWORD:-local-dev-password}"
PROFILE_DEFAULT="${TAU_UNIFIED_PROFILE:-local-dev}"
GATEWAY_STATE_DIR_DEFAULT="${TAU_UNIFIED_GATEWAY_STATE_DIR:-.tau/gateway}"
DASHBOARD_STATE_DIR_DEFAULT="${TAU_UNIFIED_DASHBOARD_STATE_DIR:-.tau/dashboard}"
REQUEST_TIMEOUT_MS_DEFAULT="${TAU_UNIFIED_REQUEST_TIMEOUT_MS:-180000}"
AGENT_REQUEST_MAX_RETRIES_DEFAULT="${TAU_UNIFIED_AGENT_REQUEST_MAX_RETRIES:-0}"
PROVIDER_MAX_RETRIES_DEFAULT="${TAU_UNIFIED_PROVIDER_MAX_RETRIES:-0}"

RUNNER="${TAU_UNIFIED_RUNNER:-}"
RUNNER_LOG="${TAU_UNIFIED_RUNNER_LOG:-}"
RUNNER_PID="${TAU_UNIFIED_RUNNER_PID:-}"

usage() {
  cat <<'EOF'
Usage: scripts/run/tau-unified.sh <command> [options]

Commands:
  up       Start unified runtime (gateway/dashboard) in background.
  status   Show runtime process status and key artifact paths.
  down     Stop unified runtime and clear pid file.
  tui      Launch live TUI shell view using dashboard artifacts.

Options for `up`:
  --model <model>                 Model id (default: gpt-5.3-codex)
  --bind <host:port>              Gateway bind (default: 127.0.0.1:8791)
  --auth-mode <mode>              Auth mode: localhost-dev|token|password-session
  --auth-token <token>            Token for token mode
  --auth-password <password>      Password for password-session mode
  --profile <name>                Profile marker for output (default: local-dev)
  --gateway-state-dir <path>      Gateway state dir (default: .tau/gateway)
  --dashboard-state-dir <path>    Dashboard state dir (default: .tau/dashboard)
  --request-timeout-ms <n>        Runtime request timeout ms (default: 180000)
  --agent-request-max-retries <n> Runtime agent request retries (default: 0)
  --provider-max-retries <n>      Runtime provider retries (default: 0)

Options for `tui`:
  --agent                         Force interactive agent mode (default)
  --live-shell                    Use read-only dashboard watch shell mode
  --bootstrap-runtime             Start runtime automatically before TUI (default: true)
  --no-bootstrap-runtime          Do not start runtime automatically before TUI
  --state-dir <path>              Dashboard state dir alias (default: .tau/dashboard)
  --dashboard-state-dir <path>    Dashboard state dir (default: .tau/dashboard)
  --gateway-state-dir <path>      Gateway state dir (default: .tau/gateway)
  --model <id>                    Agent model id (default: gpt-5.3-codex)
  --request-timeout-ms <n>        Agent request timeout ms (default: 180000)
  --agent-request-max-retries <n> Agent max request retries (default: 0)
  --profile <name>                TUI profile (default: local-dev)
  --bind <host:port>              Runtime bind for bootstrap path (default: 127.0.0.1:8791)
  --auth-mode <mode>              Runtime auth mode for bootstrap path
  --auth-token <token>            Runtime auth token for bootstrap path
  --auth-password <password>      Runtime auth password for bootstrap path
  --iterations <n>                Live-shell watch iterations (default: 3)
  --interval-ms <n>               Live-shell watch interval ms (default: 1000)
  --no-color                      Disable TUI color output

General:
  --help                          Show usage
EOF
}

log() {
  local message="$1"
  echo "${message}"
}

die() {
  local message="$1"
  echo "${message}" >&2
  exit 2
}

ensure_runtime_dir() {
  mkdir -p "${RUNTIME_DIR}"
}

pid_is_alive() {
  local pid="$1"
  if [[ -z "${pid}" ]]; then
    return 1
  fi
  kill -0 "${pid}" >/dev/null 2>&1
}

get_pid_from_file() {
  if [[ ! -f "${PID_FILE}" ]]; then
    return 1
  fi
  cat "${PID_FILE}"
}

cleanup_stale_pid() {
  if [[ ! -f "${PID_FILE}" ]]; then
    return 0
  fi
  local pid
  pid="$(cat "${PID_FILE}")"
  if [[ -z "${pid}" ]] || ! pid_is_alive "${pid}"; then
    rm -f "${PID_FILE}"
  fi
}

run_runner_mode() {
  local mode="$1"
  shift
  if [[ -z "${RUNNER}" ]]; then
    return 1
  fi
  if [[ -z "${RUNNER_LOG}" || -z "${RUNNER_PID}" ]]; then
    die "runner mode requires TAU_UNIFIED_RUNNER_LOG and TAU_UNIFIED_RUNNER_PID"
  fi
  "${RUNNER}" "${mode}" "${RUNNER_LOG}" "${RUNNER_PID}" "$@"
}

build_up_command() {
  local model="$1"
  local bind="$2"
  local auth_mode="$3"
  local auth_token="$4"
  local auth_password="$5"
  local gateway_state_dir="$6"
  local dashboard_state_dir="$7"
  local request_timeout_ms="$8"
  local agent_request_max_retries="$9"
  local provider_max_retries="${10}"

  local cmd=(
    cargo run -p tau-coding-agent --
    --model "${model}"
    --gateway-state-dir "${gateway_state_dir}"
    --dashboard-state-dir "${dashboard_state_dir}"
    --gateway-openresponses-server
    --gateway-openresponses-bind "${bind}"
    --gateway-openresponses-auth-mode "${auth_mode}"
    --gateway-openresponses-max-input-chars 32000
    --request-timeout-ms "${request_timeout_ms}"
    --agent-request-max-retries "${agent_request_max_retries}"
    --provider-max-retries "${provider_max_retries}"
  )

  if [[ "${auth_mode}" == "token" ]]; then
    cmd+=(--gateway-openresponses-auth-token "${auth_token}")
  elif [[ "${auth_mode}" == "password-session" ]]; then
    cmd+=(--gateway-openresponses-auth-password "${auth_password}")
  fi

  printf '%q ' "${cmd[@]}"
  echo
}

cmd_up() {
  local model="${MODEL_DEFAULT}"
  local bind="${BIND_DEFAULT}"
  local auth_mode="${AUTH_MODE_DEFAULT}"
  local auth_token="${AUTH_TOKEN_DEFAULT}"
  local auth_password="${AUTH_PASSWORD_DEFAULT}"
  local profile="${PROFILE_DEFAULT}"
  local gateway_state_dir="${GATEWAY_STATE_DIR_DEFAULT}"
  local dashboard_state_dir="${DASHBOARD_STATE_DIR_DEFAULT}"
  local request_timeout_ms="${REQUEST_TIMEOUT_MS_DEFAULT}"
  local agent_request_max_retries="${AGENT_REQUEST_MAX_RETRIES_DEFAULT}"
  local provider_max_retries="${PROVIDER_MAX_RETRIES_DEFAULT}"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --model)
        model="$2"
        shift 2
        ;;
      --bind)
        bind="$2"
        shift 2
        ;;
      --auth-mode)
        auth_mode="$2"
        shift 2
        ;;
      --auth-token)
        auth_token="$2"
        shift 2
        ;;
      --auth-password)
        auth_password="$2"
        shift 2
        ;;
      --profile)
        profile="$2"
        shift 2
        ;;
      --gateway-state-dir)
        gateway_state_dir="$2"
        shift 2
        ;;
      --dashboard-state-dir)
        dashboard_state_dir="$2"
        shift 2
        ;;
      --request-timeout-ms)
        request_timeout_ms="$2"
        shift 2
        ;;
      --agent-request-max-retries)
        agent_request_max_retries="$2"
        shift 2
        ;;
      --provider-max-retries)
        provider_max_retries="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown up option: $1"
        ;;
    esac
  done

  case "${auth_mode}" in
    localhost-dev|token|password-session)
      ;;
    *)
      die "invalid --auth-mode: ${auth_mode}"
      ;;
  esac
  if ! [[ "${request_timeout_ms}" =~ ^[0-9]+$ ]] || (( request_timeout_ms < 1 )); then
    die "invalid --request-timeout-ms: ${request_timeout_ms} (expected integer >= 1)"
  fi
  if ! [[ "${agent_request_max_retries}" =~ ^[0-9]+$ ]]; then
    die "invalid --agent-request-max-retries: ${agent_request_max_retries} (expected integer >= 0)"
  fi
  if ! [[ "${provider_max_retries}" =~ ^[0-9]+$ ]]; then
    die "invalid --provider-max-retries: ${provider_max_retries} (expected integer >= 0)"
  fi

  ensure_runtime_dir
  cleanup_stale_pid

  if [[ -f "${PID_FILE}" ]]; then
    local existing_pid
    existing_pid="$(cat "${PID_FILE}")"
    if pid_is_alive "${existing_pid}"; then
      log "tau-unified: already running (pid=${existing_pid})"
      return 0
    fi
    rm -f "${PID_FILE}"
  fi

  local command
  command="$(build_up_command "${model}" "${bind}" "${auth_mode}" "${auth_token}" "${auth_password}" "${gateway_state_dir}" "${dashboard_state_dir}" "${request_timeout_ms}" "${agent_request_max_retries}" "${provider_max_retries}")"
  printf '%s\n' "${command}" > "${CMD_FILE}"
  : > "${LOG_FILE}"

  if [[ -n "${RUNNER}" ]]; then
    run_runner_mode up "${command}" "${profile}" "${bind}" "${dashboard_state_dir}"
    if [[ ! -f "${RUNNER_PID}" ]]; then
      die "runner did not emit pid file: ${RUNNER_PID}"
    fi
    cp "${RUNNER_PID}" "${PID_FILE}"
  else
    (
      cd "${REPO_ROOT}"
      nohup bash -lc "${command}" >>"${LOG_FILE}" 2>&1 &
      echo $! > "${PID_FILE}"
    )
  fi

  local pid
  pid="$(cat "${PID_FILE}")"
  if ! pid_is_alive "${pid}"; then
    rm -f "${PID_FILE}"
    die "tau-unified: failed to start runtime process"
  fi

  log "tau-unified: started (pid=${pid}) profile=${profile}"
  log "tau-unified: webchat=http://${bind}/webchat"
  log "tau-unified: ops=http://${bind}/ops"
  log "tau-unified: dashboard=http://${bind}/dashboard"
  log "tau-unified: log=${LOG_FILE}"
}

cmd_status() {
  cleanup_stale_pid
  if [[ ! -f "${PID_FILE}" ]]; then
    log "tau-unified: not running"
    return 1
  fi

  local pid
  pid="$(cat "${PID_FILE}")"
  if ! pid_is_alive "${pid}"; then
    rm -f "${PID_FILE}"
    log "tau-unified: not running"
    return 1
  fi

  if [[ -n "${RUNNER}" ]]; then
    run_runner_mode status "${pid}"
  fi

  log "tau-unified: running pid=${pid}"
  log "tau-unified: pid_file=${PID_FILE}"
  log "tau-unified: log_file=${LOG_FILE}"
  log "tau-unified: command_file=${CMD_FILE}"
}

cmd_down() {
  cleanup_stale_pid
  if [[ ! -f "${PID_FILE}" ]]; then
    echo "tau-unified: not running" >&2
    return 1
  fi

  local pid
  pid="$(cat "${PID_FILE}")"

  if [[ -n "${RUNNER}" ]]; then
    run_runner_mode down "${pid}"
  else
    kill "${pid}" >/dev/null 2>&1 || true
    for _ in {1..20}; do
      if ! pid_is_alive "${pid}"; then
        break
      fi
      sleep 0.1
    done
    if pid_is_alive "${pid}"; then
      kill -9 "${pid}" >/dev/null 2>&1 || true
    fi
  fi

  rm -f "${PID_FILE}"
  log "tau-unified: stopped"
}

wait_for_dashboard_artifacts() {
  local dashboard_state_dir="$1"
  local timeout_ms="${2:-6000}"
  local elapsed_ms=0
  local step_ms=200

  while (( elapsed_ms < timeout_ms )); do
    if [[ -f "${dashboard_state_dir}/state.json" && -f "${dashboard_state_dir}/control-state.json" && -f "${dashboard_state_dir}/auth-status.json" ]]; then
      return 0
    fi
    sleep 0.2
    elapsed_ms=$((elapsed_ms + step_ms))
  done

  return 1
}

bootstrap_runtime_for_tui() {
  local model="$1"
  local bind="$2"
  local auth_mode="$3"
  local auth_token="$4"
  local auth_password="$5"
  local profile="$6"
  local gateway_state_dir="$7"
  local dashboard_state_dir="$8"
  local request_timeout_ms="$9"
  local agent_request_max_retries="${10}"

  cleanup_stale_pid
  if [[ -f "${PID_FILE}" ]]; then
    local existing_pid
    existing_pid="$(cat "${PID_FILE}")"
    if pid_is_alive "${existing_pid}"; then
      log "tau-unified: runtime already running (pid=${existing_pid})"
      return 0
    fi
    rm -f "${PID_FILE}"
  fi

  log "tau-unified: bootstrapping runtime for tui"
  cmd_up \
    --model "${model}" \
    --bind "${bind}" \
    --auth-mode "${auth_mode}" \
    --auth-token "${auth_token}" \
    --auth-password "${auth_password}" \
    --profile "${profile}" \
    --gateway-state-dir "${gateway_state_dir}" \
    --dashboard-state-dir "${dashboard_state_dir}" \
    --request-timeout-ms "${request_timeout_ms}" \
    --agent-request-max-retries "${agent_request_max_retries}"

  if wait_for_dashboard_artifacts "${dashboard_state_dir}" 6000; then
    log "tau-unified: dashboard artifacts ready (${dashboard_state_dir})"
  else
    log "tau-unified: continuing while dashboard artifacts initialize (${dashboard_state_dir})"
  fi
}

cmd_tui() {
  local dashboard_state_dir="${DASHBOARD_STATE_DIR_DEFAULT}"
  local gateway_state_dir="${GATEWAY_STATE_DIR_DEFAULT}"
  local model="${MODEL_DEFAULT}"
  local bind="${BIND_DEFAULT}"
  local auth_mode="${AUTH_MODE_DEFAULT}"
  local auth_token="${AUTH_TOKEN_DEFAULT}"
  local auth_password="${AUTH_PASSWORD_DEFAULT}"
  local profile="${PROFILE_DEFAULT}"
  local request_timeout_ms="${REQUEST_TIMEOUT_MS_DEFAULT}"
  local agent_request_max_retries="${AGENT_REQUEST_MAX_RETRIES_DEFAULT}"
  local iterations="3"
  local interval_ms="1000"
  local no_color="false"
  local tui_mode="agent"
  local saw_iterations="false"
  local saw_interval="false"
  local bootstrap_runtime="${TAU_UNIFIED_TUI_BOOTSTRAP_RUNTIME:-}"
  if [[ -z "${bootstrap_runtime}" ]]; then
    if [[ -n "${RUNNER}" ]]; then
      bootstrap_runtime="false"
    else
      bootstrap_runtime="true"
    fi
  fi

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --agent)
        tui_mode="agent"
        shift
        ;;
      --live-shell)
        tui_mode="live-shell"
        shift
        ;;
      --bootstrap-runtime)
        bootstrap_runtime="true"
        shift
        ;;
      --no-bootstrap-runtime)
        bootstrap_runtime="false"
        shift
        ;;
      --state-dir|--dashboard-state-dir)
        dashboard_state_dir="$2"
        shift 2
        ;;
      --gateway-state-dir)
        gateway_state_dir="$2"
        shift 2
        ;;
      --model)
        model="$2"
        shift 2
        ;;
      --request-timeout-ms)
        request_timeout_ms="$2"
        shift 2
        ;;
      --agent-request-max-retries)
        agent_request_max_retries="$2"
        shift 2
        ;;
      --bind)
        bind="$2"
        shift 2
        ;;
      --auth-mode)
        auth_mode="$2"
        shift 2
        ;;
      --auth-token)
        auth_token="$2"
        shift 2
        ;;
      --auth-password)
        auth_password="$2"
        shift 2
        ;;
      --profile)
        profile="$2"
        shift 2
        ;;
      --iterations)
        iterations="$2"
        saw_iterations="true"
        shift 2
        ;;
      --interval-ms)
        interval_ms="$2"
        saw_interval="true"
        shift 2
        ;;
      --no-color)
        no_color="true"
        shift
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown tui option: $1"
        ;;
    esac
  done

  if [[ "${tui_mode}" == "agent" && ( "${saw_iterations}" == "true" || "${saw_interval}" == "true" ) ]]; then
    die "--iterations/--interval-ms require --live-shell"
  fi

  case "${auth_mode}" in
    localhost-dev|token|password-session)
      ;;
    *)
      die "invalid --auth-mode: ${auth_mode}"
      ;;
  esac

  case "${bootstrap_runtime}" in
    true|false)
      ;;
    *)
      die "invalid bootstrap runtime setting: ${bootstrap_runtime} (expected true|false)"
      ;;
  esac

  if ! [[ "${request_timeout_ms}" =~ ^[0-9]+$ ]] || (( request_timeout_ms < 1 )); then
    die "invalid --request-timeout-ms: ${request_timeout_ms} (expected integer >= 1)"
  fi
  if ! [[ "${agent_request_max_retries}" =~ ^[0-9]+$ ]]; then
    die "invalid --agent-request-max-retries: ${agent_request_max_retries} (expected integer >= 0)"
  fi

  if [[ "${bootstrap_runtime}" == "true" ]]; then
    bootstrap_runtime_for_tui \
      "${model}" \
      "${bind}" \
      "${auth_mode}" \
      "${auth_token}" \
      "${auth_password}" \
      "${profile}" \
      "${gateway_state_dir}" \
      "${dashboard_state_dir}" \
      "${request_timeout_ms}" \
      "${agent_request_max_retries}"
  fi

  local tui_cmd=()
  if [[ "${tui_mode}" == "live-shell" ]]; then
    tui_cmd=(
      cargo run -p tau-tui -- shell-live
      --state-dir "${dashboard_state_dir}"
      --profile "${profile}"
      --watch
      --iterations "${iterations}"
      --interval-ms "${interval_ms}"
    )
  else
    tui_cmd=(
      cargo run -p tau-tui -- agent
      --dashboard-state-dir "${dashboard_state_dir}"
      --gateway-state-dir "${gateway_state_dir}"
      --profile "${profile}"
      --model "${model}"
      --request-timeout-ms "${request_timeout_ms}"
      --agent-request-max-retries "${agent_request_max_retries}"
    )
  fi
  if [[ "${no_color}" == "true" ]]; then
    tui_cmd+=(--no-color)
  fi

  log "tau-unified: launching tui (${tui_mode})"
  if [[ -n "${RUNNER}" ]]; then
    run_runner_mode tui "${tui_mode}" "${dashboard_state_dir}" "${gateway_state_dir}" "${profile}" "${model}" "${iterations}" "${interval_ms}" "${no_color}" "--request-timeout-ms" "${request_timeout_ms}" "--agent-request-max-retries" "${agent_request_max_retries}"
    return 0
  fi
  (
    cd "${REPO_ROOT}"
    "${tui_cmd[@]}"
  )
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

command="$1"
shift

case "${command}" in
  up)
    cmd_up "$@"
    ;;
  status)
    cmd_status "$@"
    ;;
  down)
    cmd_down "$@"
    ;;
  tui)
    cmd_tui "$@"
    ;;
  --help|-h|help)
    usage
    ;;
  *)
    echo "unknown command: ${command}" >&2
    usage >&2
    exit 2
    ;;
esac

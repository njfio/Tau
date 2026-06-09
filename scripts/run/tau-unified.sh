#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

RUNTIME_DIR_DEFAULT="${REPO_ROOT}/.tau/unified"
RUNTIME_DIR="${TAU_UNIFIED_RUNTIME_DIR:-${RUNTIME_DIR_DEFAULT}}"
PID_FILE="${RUNTIME_DIR}/tau-unified.pid"
LOG_FILE="${RUNTIME_DIR}/tau-unified.log"
CMD_FILE="${RUNTIME_DIR}/tau-unified.last-cmd"
FINGERPRINT_FILE="${RUNTIME_DIR}/tau-unified.runtime-fingerprint"
CONTROL_SNAPSHOT_FILE="${RUNTIME_DIR}/tau-unified.control-plane.env"

MODEL_DEFAULT="${TAU_UNIFIED_MODEL:-gpt-5.3-codex}"
BIND_DEFAULT="${TAU_UNIFIED_BIND:-127.0.0.1:8791}"
AUTH_MODE_DEFAULT="${TAU_UNIFIED_AUTH_MODE:-localhost-dev}"
AUTH_TOKEN_DEFAULT="${TAU_UNIFIED_AUTH_TOKEN:-local-dev-token}"
AUTH_PASSWORD_DEFAULT="${TAU_UNIFIED_AUTH_PASSWORD:-local-dev-password}"
PROFILE_DEFAULT="${TAU_UNIFIED_PROFILE:-local-dev}"
GATEWAY_STATE_DIR_DEFAULT="${TAU_UNIFIED_GATEWAY_STATE_DIR:-.tau/gateway}"
DASHBOARD_STATE_DIR_DEFAULT="${TAU_UNIFIED_DASHBOARD_STATE_DIR:-.tau/dashboard}"
JOBS_STATE_DIR_DEFAULT="${TAU_UNIFIED_JOBS_STATE_DIR:-.tau/jobs}"
AUTONOMOUS_CODING_STATE_DIR_DEFAULT="${TAU_UNIFIED_AUTONOMOUS_CODING_STATE_DIR:-.tau/autonomous-coding}"
REQUEST_TIMEOUT_MS_DEFAULT="${TAU_UNIFIED_REQUEST_TIMEOUT_MS:-180000}"
AGENT_REQUEST_MAX_RETRIES_DEFAULT="${TAU_UNIFIED_AGENT_REQUEST_MAX_RETRIES:-0}"
PROVIDER_MAX_RETRIES_DEFAULT="${TAU_UNIFIED_PROVIDER_MAX_RETRIES:-0}"
TUI_READINESS_TIMEOUT_MS_DEFAULT="${TAU_UNIFIED_TUI_READINESS_TIMEOUT_MS:-6000}"
RUST_MIN_STACK_DEFAULT="${TAU_UNIFIED_RUST_MIN_STACK:-${RUST_MIN_STACK:-16777216}}"

RUNNER="${TAU_UNIFIED_RUNNER:-}"
RUNNER_LOG="${TAU_UNIFIED_RUNNER_LOG:-}"
RUNNER_PID="${TAU_UNIFIED_RUNNER_PID:-}"
AUTONOMOUS_CODING_JOB_CLI="${TAU_UNIFIED_AUTONOMOUS_CODING_JOB_CLI:-}"

usage() {
  cat <<'EOF'
Usage: scripts/run/tau-unified.sh <command> [options]

Commands:
  up       Start unified runtime (gateway/dashboard) in background.
  status   Show runtime process status and key artifact paths.
  jobs     List durable autonomous coding jobs and recovery decisions.
  job      Inspect one autonomous coding job and its evidence.
  intakes  List autonomous coding issue-intake decisions.
  intake   Inspect one autonomous coding issue-intake decision.
  recover  Recover stuck autonomous coding/background jobs.
  replay   Replay one autonomous coding job checkpoint.
  block    Mark one autonomous coding job blocked after operator inspection.
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
  --jobs-state-dir <path>         Background jobs state dir (default: .tau/jobs)
  --autonomous-coding-state-dir <path>
                                  Autonomous coding jobs state dir (default: .tau/autonomous-coding)
  --request-timeout-ms <n>        Runtime request timeout ms (default: 180000)
  --agent-request-max-retries <n> Runtime agent request retries (default: 0)
  --provider-max-retries <n>      Runtime provider retries (default: 0)

Options for `tui`:
  --interactive                   Force graphical interactive TUI mode (default)
  --agent                         Force legacy agent shell mode
  --live-shell                    Use read-only dashboard watch shell mode
  --bootstrap-runtime             Start runtime automatically before TUI (default: true)
  --no-bootstrap-runtime          Do not start runtime automatically before TUI
  --state-dir <path>              Dashboard state dir alias (default: .tau/dashboard)
  --dashboard-state-dir <path>    Dashboard state dir (default: .tau/dashboard)
  --gateway-state-dir <path>      Gateway state dir (default: .tau/gateway)
  --jobs-state-dir <path>         Background jobs state dir (default: .tau/jobs)
  --autonomous-coding-state-dir <path>
                                  Autonomous coding jobs state dir (default: .tau/autonomous-coding)
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
  --autonomous-coding-state-dir <path>
                                  Autonomous coding state dir for job commands
  --jobs-state-dir <path>         Background jobs state dir for recovery command
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

run_autonomous_coding_job_cli() {
  if [[ -n "${AUTONOMOUS_CODING_JOB_CLI}" ]]; then
    "${AUTONOMOUS_CODING_JOB_CLI}" "$@"
    return $?
  fi
  (
    cd "${REPO_ROOT}"
    cargo run -q -p tau-coding-agent --bin tau_autonomous_coding_job -- "$@"
  )
}

require_positive_integer() {
  local value="$1"
  local flag_name="$2"
  if ! [[ "${value}" =~ ^[0-9]+$ ]] || (( value < 1 )); then
    die "invalid ${flag_name}: ${value} (expected integer >= 1)"
  fi
}

require_non_negative_integer() {
  local value="$1"
  local flag_name="$2"
  if ! [[ "${value}" =~ ^[0-9]+$ ]]; then
    die "invalid ${flag_name}: ${value} (expected integer >= 0)"
  fi
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
    rm -f "${FINGERPRINT_FILE}"
    rm -f "${CONTROL_SNAPSHOT_FILE}"
  fi
}

write_control_plane_snapshot() {
  local profile="$1"
  local bind="$2"
  local gateway_state_dir="$3"
  local dashboard_state_dir="$4"
  local jobs_state_dir="$5"
  local autonomous_coding_state_dir="$6"

  {
    printf 'profile=%s\n' "${profile}"
    printf 'bind=%s\n' "${bind}"
    printf 'webchat_url=http://%s/webchat\n' "${bind}"
    printf 'ops_url=http://%s/ops\n' "${bind}"
    printf 'dashboard_url=http://%s/dashboard\n' "${bind}"
    printf 'gateway_status_url=http://%s/gateway/status\n' "${bind}"
    printf 'sessions_endpoint=http://%s/gateway/sessions\n' "${bind}"
    printf 'memory_endpoint=http://%s/gateway/memory/default\n' "${bind}"
    printf 'memory_graph_endpoint=http://%s/gateway/memory-graph/default\n' "${bind}"
    printf 'jobs_endpoint=http://%s/gateway/jobs\n' "${bind}"
    printf 'routines_surface=http://%s/webchat#routines\n' "${bind}"
    printf 'deploy_endpoint=http://%s/ops/deploy\n' "${bind}"
    printf 'gateway_deploy_endpoint=http://%s/gateway/deploy\n' "${bind}"
    printf 'gateway_state_dir=%s\n' "${gateway_state_dir}"
    printf 'dashboard_state_dir=%s\n' "${dashboard_state_dir}"
    printf 'jobs_state_dir=%s\n' "${jobs_state_dir}"
    printf 'autonomous_coding_state_dir=%s\n' "${autonomous_coding_state_dir}"
    printf 'deploy_state_file=%s/deploy-agent-state.json\n' "${gateway_state_dir}"
    printf 'jobs_state=available_via_gateway_jobs_endpoint\n'
    printf 'background_jobs_state_dir=%s\n' "${jobs_state_dir}"
    printf 'background_jobs_manifest_dir=%s/jobs\n' "${jobs_state_dir}"
    printf 'background_jobs_events_file=%s/events.jsonl\n' "${jobs_state_dir}"
    printf 'background_jobs_health_file=%s/state.json\n' "${jobs_state_dir}"
    printf 'background_jobs_restart_recovery=running_manifests_requeued_after_restart\n'
    printf 'background_jobs_ops_guide=docs/guides/background-jobs-ops.md\n'
    printf 'routines_state=visible_via_webchat_routines_panel\n'
    printf 'coding_missions_endpoint=http://%s/gateway/missions\n' "${bind}"
    write_coding_mission_snapshot_fields "${gateway_state_dir}"
    write_autonomous_coding_snapshot_fields "${autonomous_coding_state_dir}"
    printf 'autonomy_boundary=provider_repair_durable_jobs_visible_crash_resume_recovery_in_progress\n'
  } >"${CONTROL_SNAPSHOT_FILE}"
}

control_plane_snapshot_value() {
  local key="$1"
  local default_value="$2"
  if [[ -f "${CONTROL_SNAPSHOT_FILE}" ]]; then
    local line
    line="$(grep -m 1 "^${key}=" "${CONTROL_SNAPSHOT_FILE}" || true)"
    if [[ -n "${line}" ]]; then
      printf '%s' "${line#*=}"
      return 0
    fi
  fi
  printf '%s' "${default_value}"
}

write_default_coding_mission_snapshot_fields() {
  local gateway_state_dir="$1"
  printf 'coding_mission_state_dir=%s/coding-missions\n' "${gateway_state_dir}"
  printf 'coding_mission_id=none\n'
  printf 'coding_mission_phase=none\n'
  printf 'coding_mission_repo=unknown\n'
  printf 'coding_mission_branch=none\n'
  printf 'coding_mission_verifier=none\n'
  printf 'coding_mission_last_failure=none\n'
  printf 'coding_mission_changed_files=none\n'
  printf 'coding_mission_resume_command=none\n'
  printf 'coding_mission_pr_state=none\n'
  printf 'coding_mission_pr_url=none\n'
}

write_coding_mission_snapshot_fields() {
  local gateway_state_dir="$1"
  local missions_dir="${gateway_state_dir}/coding-missions"
  if [[ ! -d "${missions_dir}" ]] || ! command -v python3 >/dev/null 2>&1; then
    write_default_coding_mission_snapshot_fields "${gateway_state_dir}"
    return 0
  fi

  local snapshot
  snapshot="$(python3 - "${missions_dir}" <<'PY' 2>/dev/null || true
import glob
import json
import os
import sys

missions_dir = sys.argv[1]

def clean(value, default="none"):
    if value is None:
        value = default
    value = str(value).replace("\n", " ").replace("\r", " ").replace("\t", " ").strip()
    return value if value else default

states = []
for path in glob.glob(os.path.join(missions_dir, "*.json")):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            payload = json.load(handle)
    except Exception:
        continue
    if payload.get("mission_id"):
        payload["_path"] = path
        states.append(payload)

if not states:
    sys.exit(0)

state = max(states, key=lambda item: int(item.get("updated_unix_ms") or 0))
commands = list(state.get("command_evidence") or [])
git_evidence = list(state.get("git_evidence") or [])
checkpoint = state.get("resume_checkpoint") or {}
pr_bundle = state.get("pr_ready_bundle") or {}

latest_command = commands[-1] if commands else {}
failed_command = next(
    (
        command
        for command in reversed(commands)
        if clean(command.get("status"), "unknown") != "succeeded"
    ),
    {},
)
latest_git = git_evidence[-1] if git_evidence else {}

def command_summary(command):
    if not command:
        return "none"
    return f"{clean(command.get('status'), 'unknown')}:{clean(command.get('reason_code'), 'unknown')}"

changed_files = (
    pr_bundle.get("changed_files")
    or latest_git.get("changed_files")
    or (checkpoint.get("mutation_fingerprint") or {}).get("changed_files")
    or []
)
branch = (
    checkpoint.get("branch_name")
    or pr_bundle.get("branch_name")
    or latest_git.get("branch_name")
)
pr_state = pr_bundle.get("status") or ("pr_ready" if state.get("phase") == "pr_ready" else "none")

fields = {
    "coding_mission_state_dir": missions_dir,
    "coding_mission_id": state.get("mission_id"),
    "coding_mission_phase": state.get("phase"),
    "coding_mission_repo": state.get("repo_path"),
    "coding_mission_branch": branch,
    "coding_mission_verifier": command_summary(latest_command),
    "coding_mission_last_failure": command_summary(failed_command),
    "coding_mission_changed_files": ",".join(map(clean, changed_files)) if changed_files else "none",
    "coding_mission_resume_command": checkpoint.get("operator_resume_command"),
    "coding_mission_pr_state": pr_state,
    "coding_mission_pr_url": pr_bundle.get("pr_url"),
}

for key, value in fields.items():
    print(f"{key}={clean(value)}")
PY
)"

  if [[ -z "${snapshot}" ]]; then
    write_default_coding_mission_snapshot_fields "${gateway_state_dir}"
    return 0
  fi
  printf '%s\n' "${snapshot}"
}

write_default_autonomous_coding_snapshot_fields() {
  local autonomous_coding_state_dir="$1"
  printf 'autonomous_coding_state_dir=%s\n' "${autonomous_coding_state_dir}"
  printf 'autonomous_coding_jobs_dir=%s/autonomous-coding-jobs\n' "${autonomous_coding_state_dir}"
  printf 'autonomous_coding_job_id=none\n'
  printf 'autonomous_coding_mission_id=none\n'
  printf 'autonomous_coding_background_job_id=none\n'
  printf 'autonomous_coding_status=none\n'
  printf 'autonomous_coding_phase=none\n'
  printf 'autonomous_coding_reason_code=none\n'
  printf 'autonomous_coding_repo=unknown\n'
  printf 'autonomous_coding_verifier=none\n'
  printf 'autonomous_coding_changed_files=none\n'
  printf 'autonomous_coding_resume_command=none\n'
  printf 'autonomous_coding_operator_state=none\n'
  printf 'autonomous_coding_operator_next_command=none\n'
  printf 'autonomous_coding_replay_safe=false\n'
  printf 'autonomous_coding_recoverable=false\n'
  printf 'autonomous_coding_needs_authority=false\n'
  printf 'autonomous_coding_stale_lease=false\n'
  printf 'autonomous_coding_mark_blocked_command=none\n'
  printf 'autonomous_coding_pr_state=none\n'
  printf 'autonomous_coding_pr_ready_command=none\n'
  printf 'autonomous_coding_pr_url=none\n'
  printf 'autonomous_coding_pr_publication_reason_code=none\n'
  printf 'autonomous_coding_pr_publication_command=none\n'
  printf 'autonomous_coding_pr_publication_stdout_path=none\n'
  printf 'autonomous_coding_pr_publication_stderr_path=none\n'
  printf 'autonomous_coding_pr_publication_exit_status=none\n'
  printf 'autonomous_coding_auto_merge_status=none\n'
  printf 'autonomous_coding_auto_merge_reason_code=none\n'
  printf 'autonomous_coding_auto_merge_command=none\n'
  printf 'autonomous_coding_auto_merge_pr_url=none\n'
  printf 'autonomous_coding_provider_repair_status=none\n'
  printf 'autonomous_coding_provider_repair_reason_code=none\n'
  printf 'autonomous_coding_provider_repair_attempts=0\n'
  printf 'autonomous_coding_provider_repair_max_attempts=0\n'
  printf 'autonomous_coding_provider_repair_provider=none\n'
  printf 'autonomous_coding_provider_repair_model=none\n'
  printf 'autonomous_coding_provider_repair_context_path=none\n'
  printf 'autonomous_coding_event_log_path=none\n'
  printf 'autonomous_coding_last_heartbeat_unix_ms=none\n'
  printf 'autonomous_coding_lease_expires_unix_ms=none\n'
  printf 'autonomous_coding_recovery_count=0\n'
  printf 'autonomous_coding_replay_count=0\n'
  printf 'autonomous_coding_last_background_reason_code=none\n'
  printf 'autonomous_coding_last_error=none\n'
}

write_autonomous_coding_snapshot_fields() {
  local autonomous_coding_state_dir="$1"
  local jobs_dir="${autonomous_coding_state_dir}/autonomous-coding-jobs"
  if [[ ! -d "${jobs_dir}" ]] || ! command -v python3 >/dev/null 2>&1; then
    write_default_autonomous_coding_snapshot_fields "${autonomous_coding_state_dir}"
    return 0
  fi

  local snapshot
  snapshot="$(python3 - "${autonomous_coding_state_dir}" "${jobs_dir}" <<'PY' 2>/dev/null || true
import glob
import json
import os
import sys

state_dir = sys.argv[1]
jobs_dir = sys.argv[2]

def clean(value, default="none"):
    if value is None:
        value = default
    if isinstance(value, (list, tuple)):
        value = ",".join(clean(item, "") for item in value if clean(item, ""))
    if isinstance(value, bool):
        value = str(value).lower()
    value = str(value).replace("\n", " ").replace("\r", " ").replace("\t", " ").strip()
    return value if value else default

states = []
for path in glob.glob(os.path.join(jobs_dir, "*.status.json")):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            payload = json.load(handle)
    except Exception:
        continue
    if payload.get("job_id"):
        payload["_path"] = path
        payload["_mtime"] = os.path.getmtime(path)
        states.append(payload)

if not states:
    sys.exit(0)

state = max(
    states,
    key=lambda item: (
        int(item.get("last_heartbeat_unix_ms") or 0),
        int(item.get("lease_expires_unix_ms") or 0),
        item.get("_mtime") or 0,
    ),
)

fields = {
    "autonomous_coding_state_dir": state_dir,
    "autonomous_coding_jobs_dir": jobs_dir,
    "autonomous_coding_job_id": state.get("job_id"),
    "autonomous_coding_mission_id": state.get("mission_id"),
    "autonomous_coding_background_job_id": state.get("background_job_id"),
    "autonomous_coding_status": state.get("status"),
    "autonomous_coding_phase": state.get("phase"),
    "autonomous_coding_reason_code": state.get("reason_code"),
    "autonomous_coding_repo": state.get("repo_path"),
    "autonomous_coding_verifier": state.get("verifier_summary"),
    "autonomous_coding_changed_files": state.get("changed_files") or [],
    "autonomous_coding_resume_command": state.get("resume_command"),
    "autonomous_coding_operator_state": state.get("operator_state"),
    "autonomous_coding_operator_next_command": state.get("operator_next_command"),
    "autonomous_coding_replay_safe": state.get("replay_safe", False),
    "autonomous_coding_recoverable": state.get("recoverable", False),
    "autonomous_coding_needs_authority": state.get("needs_authority", False),
    "autonomous_coding_stale_lease": state.get("stale_lease", False),
    "autonomous_coding_mark_blocked_command": state.get("mark_blocked_command"),
    "autonomous_coding_pr_state": state.get("pr_state"),
    "autonomous_coding_pr_ready_command": state.get("pr_ready_command"),
    "autonomous_coding_pr_url": state.get("pr_url"),
    "autonomous_coding_pr_publication_reason_code": state.get("pr_publication_reason_code"),
    "autonomous_coding_pr_publication_command": state.get("pr_publication_command"),
    "autonomous_coding_pr_publication_stdout_path": state.get("pr_publication_stdout_path"),
    "autonomous_coding_pr_publication_stderr_path": state.get("pr_publication_stderr_path"),
    "autonomous_coding_pr_publication_exit_status": state.get("pr_publication_exit_status"),
    "autonomous_coding_auto_merge_status": state.get("auto_merge_status"),
    "autonomous_coding_auto_merge_reason_code": state.get("auto_merge_reason_code"),
    "autonomous_coding_auto_merge_command": state.get("auto_merge_command"),
    "autonomous_coding_auto_merge_pr_url": state.get("auto_merge_pr_url"),
    "autonomous_coding_provider_repair_status": state.get("provider_repair_status"),
    "autonomous_coding_provider_repair_reason_code": state.get("provider_repair_reason_code"),
    "autonomous_coding_provider_repair_attempts": state.get("provider_repair_attempts", 0),
    "autonomous_coding_provider_repair_max_attempts": state.get("provider_repair_max_attempts", 0),
    "autonomous_coding_provider_repair_provider": state.get("provider_repair_provider"),
    "autonomous_coding_provider_repair_model": state.get("provider_repair_model"),
    "autonomous_coding_provider_repair_context_path": state.get("provider_repair_context_path"),
    "autonomous_coding_event_log_path": state.get("event_log_path"),
    "autonomous_coding_last_heartbeat_unix_ms": state.get("last_heartbeat_unix_ms"),
    "autonomous_coding_lease_expires_unix_ms": state.get("lease_expires_unix_ms"),
    "autonomous_coding_recovery_count": state.get("recovery_count", 0),
    "autonomous_coding_replay_count": state.get("replay_count", 0),
    "autonomous_coding_last_background_reason_code": state.get("last_background_reason_code"),
    "autonomous_coding_last_error": state.get("last_error"),
}

for key, value in fields.items():
    print(f"{key}={clean(value)}")
PY
)"

  if [[ -z "${snapshot}" ]]; then
    write_default_autonomous_coding_snapshot_fields "${autonomous_coding_state_dir}"
    return 0
  fi
  printf '%s\n' "${snapshot}"
}

log_control_plane_snapshot() {
  log "tau-unified: control_plane.health=running"
  log "tau-unified: control_plane.runtime_state_dir=${RUNTIME_DIR}"
  log "tau-unified: control_plane.pid_file=${PID_FILE}"
  log "tau-unified: control_plane.log_file=${LOG_FILE}"
  log "tau-unified: control_plane.command_file=${CMD_FILE}"
  log "tau-unified: control_plane.fingerprint_file=${FINGERPRINT_FILE}"

  if [[ ! -f "${CONTROL_SNAPSHOT_FILE}" ]]; then
    log "tau-unified: control_plane.snapshot=missing"
    log "tau-unified: control_plane.coding_mission.id=none"
    log "tau-unified: control_plane.coding_mission.phase=none"
    log "tau-unified: control_plane.coding_mission.repo=unknown"
    log "tau-unified: control_plane.coding_mission.branch=none"
    log "tau-unified: control_plane.coding_mission.verifier=none"
    log "tau-unified: control_plane.coding_mission.last_failure=none"
    log "tau-unified: control_plane.coding_mission.changed_files=none"
    log "tau-unified: control_plane.coding_mission.resume_command=none"
    log "tau-unified: control_plane.coding_mission.pr_state=none"
    log "tau-unified: control_plane.coding_mission.pr_url=none"
    log "tau-unified: control_plane.autonomous_coding.job_id=none"
    log "tau-unified: control_plane.autonomous_coding.status=none"
    log "tau-unified: control_plane.autonomous_coding.verifier=none"
    log "tau-unified: control_plane.autonomous_coding.provider_repair.status=none"
    log "tau-unified: control_plane.autonomous_coding.event_log=none"
    log "tau-unified: control_plane.autonomy_boundary=provider_repair_durable_jobs_visible_crash_resume_recovery_in_progress"
    return 0
  fi

  log "tau-unified: control_plane.snapshot_file=${CONTROL_SNAPSHOT_FILE}"
  log "tau-unified: control_plane.profile=$(control_plane_snapshot_value profile unknown)"
  log "tau-unified: control_plane.webchat_url=$(control_plane_snapshot_value webchat_url unknown)"
  log "tau-unified: control_plane.ops_url=$(control_plane_snapshot_value ops_url unknown)"
  log "tau-unified: control_plane.dashboard_url=$(control_plane_snapshot_value dashboard_url unknown)"
  log "tau-unified: control_plane.gateway_status_url=$(control_plane_snapshot_value gateway_status_url unknown)"
  log "tau-unified: control_plane.sessions_endpoint=$(control_plane_snapshot_value sessions_endpoint unknown)"
  log "tau-unified: control_plane.memory_endpoint=$(control_plane_snapshot_value memory_endpoint unknown)"
  log "tau-unified: control_plane.memory_graph_endpoint=$(control_plane_snapshot_value memory_graph_endpoint unknown)"
  log "tau-unified: control_plane.jobs_endpoint=$(control_plane_snapshot_value jobs_endpoint unknown)"
  log "tau-unified: control_plane.jobs_state=$(control_plane_snapshot_value jobs_state unknown)"
  log "tau-unified: control_plane.background_jobs.state_dir=$(control_plane_snapshot_value background_jobs_state_dir unknown)"
  log "tau-unified: control_plane.background_jobs.manifest_dir=$(control_plane_snapshot_value background_jobs_manifest_dir unknown)"
  log "tau-unified: control_plane.background_jobs.events_file=$(control_plane_snapshot_value background_jobs_events_file unknown)"
  log "tau-unified: control_plane.background_jobs.health_file=$(control_plane_snapshot_value background_jobs_health_file unknown)"
  log "tau-unified: control_plane.background_jobs.restart_recovery=$(control_plane_snapshot_value background_jobs_restart_recovery running_manifests_requeued_after_restart)"
  log "tau-unified: control_plane.background_jobs.ops_guide=$(control_plane_snapshot_value background_jobs_ops_guide docs/guides/background-jobs-ops.md)"
  log "tau-unified: control_plane.routines_surface=$(control_plane_snapshot_value routines_surface unknown)"
  log "tau-unified: control_plane.routines_state=$(control_plane_snapshot_value routines_state unknown)"
  log "tau-unified: control_plane.deploy_endpoint=$(control_plane_snapshot_value deploy_endpoint unknown)"
  log "tau-unified: control_plane.gateway_deploy_endpoint=$(control_plane_snapshot_value gateway_deploy_endpoint unknown)"
  log "tau-unified: control_plane.gateway_state_dir=$(control_plane_snapshot_value gateway_state_dir unknown)"
  log "tau-unified: control_plane.dashboard_state_dir=$(control_plane_snapshot_value dashboard_state_dir unknown)"
  log "tau-unified: control_plane.jobs_state_dir=$(control_plane_snapshot_value jobs_state_dir unknown)"
  log "tau-unified: control_plane.autonomous_coding.state_dir=$(control_plane_snapshot_value autonomous_coding_state_dir unknown)"
  log "tau-unified: control_plane.autonomous_coding.jobs_dir=$(control_plane_snapshot_value autonomous_coding_jobs_dir unknown)"
  log "tau-unified: control_plane.deploy_state_file=$(control_plane_snapshot_value deploy_state_file unknown)"
  log "tau-unified: control_plane.coding_missions_endpoint=$(control_plane_snapshot_value coding_missions_endpoint unknown)"
  log "tau-unified: control_plane.coding_mission.state_dir=$(control_plane_snapshot_value coding_mission_state_dir unknown)"
  log "tau-unified: control_plane.coding_mission.id=$(control_plane_snapshot_value coding_mission_id none)"
  log "tau-unified: control_plane.coding_mission.phase=$(control_plane_snapshot_value coding_mission_phase none)"
  log "tau-unified: control_plane.coding_mission.repo=$(control_plane_snapshot_value coding_mission_repo unknown)"
  log "tau-unified: control_plane.coding_mission.branch=$(control_plane_snapshot_value coding_mission_branch none)"
  log "tau-unified: control_plane.coding_mission.verifier=$(control_plane_snapshot_value coding_mission_verifier none)"
  log "tau-unified: control_plane.coding_mission.last_failure=$(control_plane_snapshot_value coding_mission_last_failure none)"
  log "tau-unified: control_plane.coding_mission.changed_files=$(control_plane_snapshot_value coding_mission_changed_files none)"
  log "tau-unified: control_plane.coding_mission.resume_command=$(control_plane_snapshot_value coding_mission_resume_command none)"
  log "tau-unified: control_plane.coding_mission.pr_state=$(control_plane_snapshot_value coding_mission_pr_state none)"
  log "tau-unified: control_plane.coding_mission.pr_url=$(control_plane_snapshot_value coding_mission_pr_url none)"
  log "tau-unified: control_plane.autonomous_coding.job_id=$(control_plane_snapshot_value autonomous_coding_job_id none)"
  log "tau-unified: control_plane.autonomous_coding.mission_id=$(control_plane_snapshot_value autonomous_coding_mission_id none)"
  log "tau-unified: control_plane.autonomous_coding.background_job_id=$(control_plane_snapshot_value autonomous_coding_background_job_id none)"
  log "tau-unified: control_plane.autonomous_coding.status=$(control_plane_snapshot_value autonomous_coding_status none)"
  log "tau-unified: control_plane.autonomous_coding.phase=$(control_plane_snapshot_value autonomous_coding_phase none)"
  log "tau-unified: control_plane.autonomous_coding.reason_code=$(control_plane_snapshot_value autonomous_coding_reason_code none)"
  log "tau-unified: control_plane.autonomous_coding.repo=$(control_plane_snapshot_value autonomous_coding_repo unknown)"
  log "tau-unified: control_plane.autonomous_coding.verifier=$(control_plane_snapshot_value autonomous_coding_verifier none)"
  log "tau-unified: control_plane.autonomous_coding.changed_files=$(control_plane_snapshot_value autonomous_coding_changed_files none)"
  log "tau-unified: control_plane.autonomous_coding.resume_command=$(control_plane_snapshot_value autonomous_coding_resume_command none)"
  log "tau-unified: control_plane.autonomous_coding.operator_state=$(control_plane_snapshot_value autonomous_coding_operator_state none)"
  log "tau-unified: control_plane.autonomous_coding.operator_next_command=$(control_plane_snapshot_value autonomous_coding_operator_next_command none)"
  log "tau-unified: control_plane.autonomous_coding.replay_safe=$(control_plane_snapshot_value autonomous_coding_replay_safe false)"
  log "tau-unified: control_plane.autonomous_coding.recoverable=$(control_plane_snapshot_value autonomous_coding_recoverable false)"
  log "tau-unified: control_plane.autonomous_coding.needs_authority=$(control_plane_snapshot_value autonomous_coding_needs_authority false)"
  log "tau-unified: control_plane.autonomous_coding.stale_lease=$(control_plane_snapshot_value autonomous_coding_stale_lease false)"
  log "tau-unified: control_plane.autonomous_coding.mark_blocked_command=$(control_plane_snapshot_value autonomous_coding_mark_blocked_command none)"
  log "tau-unified: control_plane.autonomous_coding.pr_state=$(control_plane_snapshot_value autonomous_coding_pr_state none)"
  log "tau-unified: control_plane.autonomous_coding.pr_ready_command=$(control_plane_snapshot_value autonomous_coding_pr_ready_command none)"
  log "tau-unified: control_plane.autonomous_coding.pr_url=$(control_plane_snapshot_value autonomous_coding_pr_url none)"
  log "tau-unified: control_plane.autonomous_coding.pr_publication.reason_code=$(control_plane_snapshot_value autonomous_coding_pr_publication_reason_code none)"
  log "tau-unified: control_plane.autonomous_coding.pr_publication.command=$(control_plane_snapshot_value autonomous_coding_pr_publication_command none)"
  log "tau-unified: control_plane.autonomous_coding.pr_publication.stdout_path=$(control_plane_snapshot_value autonomous_coding_pr_publication_stdout_path none)"
  log "tau-unified: control_plane.autonomous_coding.pr_publication.stderr_path=$(control_plane_snapshot_value autonomous_coding_pr_publication_stderr_path none)"
  log "tau-unified: control_plane.autonomous_coding.pr_publication.exit_status=$(control_plane_snapshot_value autonomous_coding_pr_publication_exit_status none)"
  log "tau-unified: control_plane.autonomous_coding.auto_merge.status=$(control_plane_snapshot_value autonomous_coding_auto_merge_status none)"
  log "tau-unified: control_plane.autonomous_coding.auto_merge.reason_code=$(control_plane_snapshot_value autonomous_coding_auto_merge_reason_code none)"
  log "tau-unified: control_plane.autonomous_coding.auto_merge.command=$(control_plane_snapshot_value autonomous_coding_auto_merge_command none)"
  log "tau-unified: control_plane.autonomous_coding.auto_merge.pr_url=$(control_plane_snapshot_value autonomous_coding_auto_merge_pr_url none)"
  log "tau-unified: control_plane.autonomous_coding.provider_repair.status=$(control_plane_snapshot_value autonomous_coding_provider_repair_status none)"
  log "tau-unified: control_plane.autonomous_coding.provider_repair.reason_code=$(control_plane_snapshot_value autonomous_coding_provider_repair_reason_code none)"
  log "tau-unified: control_plane.autonomous_coding.provider_repair.attempts=$(control_plane_snapshot_value autonomous_coding_provider_repair_attempts 0)"
  log "tau-unified: control_plane.autonomous_coding.provider_repair.max_attempts=$(control_plane_snapshot_value autonomous_coding_provider_repair_max_attempts 0)"
  log "tau-unified: control_plane.autonomous_coding.provider_repair.provider=$(control_plane_snapshot_value autonomous_coding_provider_repair_provider none)"
  log "tau-unified: control_plane.autonomous_coding.provider_repair.model=$(control_plane_snapshot_value autonomous_coding_provider_repair_model none)"
  log "tau-unified: control_plane.autonomous_coding.provider_repair.context=$(control_plane_snapshot_value autonomous_coding_provider_repair_context_path none)"
  log "tau-unified: control_plane.autonomous_coding.event_log=$(control_plane_snapshot_value autonomous_coding_event_log_path none)"
  log "tau-unified: control_plane.autonomous_coding.last_heartbeat_unix_ms=$(control_plane_snapshot_value autonomous_coding_last_heartbeat_unix_ms none)"
  log "tau-unified: control_plane.autonomous_coding.lease_expires_unix_ms=$(control_plane_snapshot_value autonomous_coding_lease_expires_unix_ms none)"
  log "tau-unified: control_plane.autonomous_coding.recovery_count=$(control_plane_snapshot_value autonomous_coding_recovery_count 0)"
  log "tau-unified: control_plane.autonomous_coding.replay_count=$(control_plane_snapshot_value autonomous_coding_replay_count 0)"
  log "tau-unified: control_plane.autonomous_coding.last_background_reason_code=$(control_plane_snapshot_value autonomous_coding_last_background_reason_code none)"
  log "tau-unified: control_plane.autonomous_coding.last_error=$(control_plane_snapshot_value autonomous_coding_last_error none)"
  log "tau-unified: control_plane.autonomy_boundary=$(control_plane_snapshot_value autonomy_boundary provider_repair_durable_jobs_visible_crash_resume_recovery_in_progress)"
}

compute_runtime_fingerprint_checksum() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | awk '{print $1}'
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 -r | awk '{print $1}'
  else
    die "tau-unified: requires sha256sum, shasum, or openssl to compute runtime fingerprint"
  fi
}

build_runtime_fingerprint() {
  local command="$1"
  local git_head="nogit"
  local tracked_changes="clean"

  if git -C "${REPO_ROOT}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git_head="$(git -C "${REPO_ROOT}" rev-parse HEAD 2>/dev/null || echo "unknown")"
    tracked_changes="$(
      git -C "${REPO_ROOT}" status --porcelain --untracked-files=no 2>/dev/null || true
    )"
  fi

  printf 'head=%s\ncommand=%s\ntracked_changes=%s\n' \
    "${git_head}" \
    "${command}" \
    "${tracked_changes}" \
    | compute_runtime_fingerprint_checksum
}

runtime_fingerprint_matches() {
  local expected="$1"
  if [[ ! -f "${FINGERPRINT_FILE}" ]]; then
    return 1
  fi

  local recorded
  recorded="$(tr -d '\n' < "${FINGERPRINT_FILE}")"
  [[ -n "${recorded}" && "${recorded}" == "${expected}" ]]
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

start_detached_runtime_process() {
  local command="$1"

  if command -v perl >/dev/null 2>&1; then
    TAU_UNIFIED_DAEMON_COMMAND="${command}" \
    TAU_UNIFIED_DAEMON_LOG="${LOG_FILE}" \
    TAU_UNIFIED_DAEMON_ROOT="${REPO_ROOT}" \
      perl -MPOSIX=setsid -e '
        use strict;
        use warnings;

        my $command = $ENV{"TAU_UNIFIED_DAEMON_COMMAND"} // die "missing daemon command\n";
        my $log = $ENV{"TAU_UNIFIED_DAEMON_LOG"} // die "missing daemon log\n";
        my $root = $ENV{"TAU_UNIFIED_DAEMON_ROOT"} // die "missing daemon root\n";

        defined(my $pid = fork) or die "fork failed: $!\n";
        if ($pid) {
          print "$pid\n";
          exit 0;
        }

        setsid() or die "setsid failed: $!\n";
        chdir $root or die "chdir $root failed: $!\n";
        open STDIN, "<", "/dev/null" or die "open /dev/null failed: $!\n";
        open STDOUT, ">>", $log or die "open $log failed: $!\n";
        open STDERR, ">&", \*STDOUT or die "redirect stderr failed: $!\n";
        exec "bash", "-lc", $command;
        die "exec failed: $!\n";
      ' >"${PID_FILE}"
  else
    (
      cd "${REPO_ROOT}"
      nohup bash -lc "${command}" </dev/null >>"${LOG_FILE}" 2>&1 &
      echo $! > "${PID_FILE}"
    )
  fi
}

build_up_command() {
  local model="$1"
  local bind="$2"
  local auth_mode="$3"
  local auth_token="$4"
  local auth_password="$5"
  local gateway_state_dir="$6"
  local dashboard_state_dir="$7"
  local jobs_state_dir="$8"
  local autonomous_coding_state_dir="$9"
  local request_timeout_ms="${10}"
  local agent_request_max_retries="${11}"
  local provider_max_retries="${12}"

  local cmd=(
    env "RUST_MIN_STACK=${RUST_MIN_STACK_DEFAULT}"
    "TAU_UNIFIED_AUTONOMOUS_CODING_STATE_DIR=${autonomous_coding_state_dir}"
    cargo run -p tau-coding-agent --bin tau-coding-agent --
    --model "${model}"
    --gateway-state-dir "${gateway_state_dir}"
    --dashboard-state-dir "${dashboard_state_dir}"
    --jobs-state-dir "${jobs_state_dir}"
    --gateway-openresponses-server
    --gateway-openresponses-bind "${bind}"
    --gateway-openresponses-auth-mode "${auth_mode}"
    --gateway-openresponses-max-input-chars 32000
    --request-timeout-ms "${request_timeout_ms}"
    --turn-timeout-ms "${request_timeout_ms}"
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
  local jobs_state_dir="${JOBS_STATE_DIR_DEFAULT}"
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
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
      --jobs-state-dir)
        jobs_state_dir="$2"
        shift 2
        ;;
      --autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
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
  require_positive_integer "${request_timeout_ms}" "--request-timeout-ms"
  require_non_negative_integer "${agent_request_max_retries}" "--agent-request-max-retries"
  require_non_negative_integer "${provider_max_retries}" "--provider-max-retries"

  ensure_runtime_dir
  cleanup_stale_pid

  local command
  command="$(build_up_command "${model}" "${bind}" "${auth_mode}" "${auth_token}" "${auth_password}" "${gateway_state_dir}" "${dashboard_state_dir}" "${jobs_state_dir}" "${autonomous_coding_state_dir}" "${request_timeout_ms}" "${agent_request_max_retries}" "${provider_max_retries}")"
  local runtime_fingerprint
  runtime_fingerprint="$(build_runtime_fingerprint "${command}")"

  if [[ -f "${PID_FILE}" ]]; then
    local existing_pid
    existing_pid="$(cat "${PID_FILE}")"
    if pid_is_alive "${existing_pid}"; then
      if runtime_fingerprint_matches "${runtime_fingerprint}"; then
        log "tau-unified: already running (pid=${existing_pid})"
        return 0
      fi
      log "tau-unified: recycling stale runtime (pid=${existing_pid}) because repo/runtime fingerprint changed"
      cmd_down || true
    else
      rm -f "${PID_FILE}"
      rm -f "${FINGERPRINT_FILE}"
    fi
  fi

  printf '%s\n' "${command}" > "${CMD_FILE}"
  : > "${LOG_FILE}"

  if [[ -n "${RUNNER}" ]]; then
    run_runner_mode up "${command}" "${profile}" "${bind}" "${dashboard_state_dir}"
    if [[ ! -f "${RUNNER_PID}" ]]; then
      die "runner did not emit pid file: ${RUNNER_PID}"
    fi
    cp "${RUNNER_PID}" "${PID_FILE}"
  else
    start_detached_runtime_process "${command}"
  fi

  local pid
  pid="$(cat "${PID_FILE}")"
  if ! pid_is_alive "${pid}"; then
    rm -f "${PID_FILE}"
    rm -f "${CONTROL_SNAPSHOT_FILE}"
    die "tau-unified: failed to start runtime process"
  fi

  write_control_plane_snapshot "${profile}" "${bind}" "${gateway_state_dir}" "${dashboard_state_dir}" "${jobs_state_dir}" "${autonomous_coding_state_dir}"
  printf '%s\n' "${runtime_fingerprint}" > "${FINGERPRINT_FILE}"

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
  log "tau-unified: fingerprint_file=${FINGERPRINT_FILE}"
  log_control_plane_snapshot
}

job_resume_explanation_py='
def resume_explanation(state):
    operator_state = clean(state.get("operator_state"), "unknown")
    next_command = clean(state.get("operator_next_command"), "none")
    if state.get("stale_lease"):
        return f"recoverable stale lease; run {next_command}"
    if state.get("recoverable"):
        return f"recoverable job; run {next_command}"
    if state.get("replay_safe"):
        return f"safe to replay from checkpoint; run {next_command}"
    if state.get("needs_authority"):
        return "blocked on missing verifier/edit/provider authority"
    if operator_state == "complete":
        return "complete; no replay or recovery needed"
    if operator_state == "blocked":
        return "blocked; inspect event log and mark-blocked detail before replay"
    return "unsafe to resume automatically; no safe replay/recover signal present"
'

cmd_jobs() {
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --state-dir|--autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown jobs option: $1"
        ;;
    esac
  done

  python3 - "${autonomous_coding_state_dir}" "${job_resume_explanation_py}" <<'PY'
import glob
import json
import os
import sys

state_dir = sys.argv[1]
resume_code = sys.argv[2]
jobs_dir = os.path.join(state_dir, "autonomous-coding-jobs")

def clean(value, default="none"):
    if value is None:
        value = default
    if isinstance(value, bool):
        value = str(value).lower()
    if isinstance(value, (list, tuple)):
        value = ",".join(clean(item, "") for item in value if clean(item, ""))
    value = str(value).replace("\n", " ").replace("\r", " ").replace("\t", " ").strip()
    return value if value else default

exec(resume_code)

states = []
for path in glob.glob(os.path.join(jobs_dir, "*.status.json")):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            payload = json.load(handle)
    except Exception:
        continue
    if payload.get("job_id"):
        states.append(payload)

states.sort(key=lambda item: clean(item.get("job_id"), ""))
print(f"tau-unified: autonomous_coding.jobs.state_dir={clean(state_dir)}")
print(f"tau-unified: autonomous_coding.jobs.count={len(states)}")
for state in states:
    job_id = clean(state.get("job_id"))
    print(
        "tau-unified: autonomous_coding.job="
        f"{job_id} status={clean(state.get('status'))} "
        f"operator_state={clean(state.get('operator_state'))} "
        f"replay_safe={clean(state.get('replay_safe', False))} "
        f"recoverable={clean(state.get('recoverable', False))} "
        f"needs_authority={clean(state.get('needs_authority', False))} "
        f"pr_state={clean(state.get('pr_state'))} "
        f"reason_code={clean(state.get('reason_code'))}"
    )
    print(f"tau-unified: autonomous_coding.job.{job_id}.next_command={clean(state.get('operator_next_command'))}")
    print(f"tau-unified: autonomous_coding.job.{job_id}.resume_explanation={resume_explanation(state)}")
    print(f"tau-unified: autonomous_coding.job.{job_id}.event_log={clean(state.get('event_log_path'))}")
    print(
        f"tau-unified: autonomous_coding.job.{job_id}.provider_repair_context="
        f"{clean(state.get('provider_repair_context_path'))}"
    )
    print(f"tau-unified: autonomous_coding.job.{job_id}.mark_blocked_command={clean(state.get('mark_blocked_command'))}")
PY
}

cmd_job() {
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
  local job_id=""
  if [[ $# -gt 0 && "$1" != --* ]]; then
    job_id="$1"
    shift
  fi
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --state-dir|--autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
        shift 2
        ;;
      --job-id)
        job_id="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown job option: $1"
        ;;
    esac
  done
  [[ -n "${job_id}" ]] || die "job requires <job-id> or --job-id"

  python3 - "${autonomous_coding_state_dir}" "${job_id}" "${job_resume_explanation_py}" <<'PY'
import json
import os
import sys

state_dir, job_id, resume_code = sys.argv[1:]
path = os.path.join(state_dir, "autonomous-coding-jobs", f"{job_id}.status.json")

def clean(value, default="none"):
    if value is None:
        value = default
    if isinstance(value, bool):
        value = str(value).lower()
    if isinstance(value, (list, tuple)):
        value = ",".join(clean(item, "") for item in value if clean(item, ""))
    value = str(value).replace("\n", " ").replace("\r", " ").replace("\t", " ").strip()
    return value if value else default

exec(resume_code)

if not os.path.exists(path):
    print(f"tau-unified: autonomous_coding.job.error=not_found job_id={clean(job_id)}")
    sys.exit(2)
with open(path, "r", encoding="utf-8") as handle:
    state = json.load(handle)

fields = {
    "id": state.get("job_id"),
    "mission_id": state.get("mission_id"),
    "background_job_id": state.get("background_job_id"),
    "status": state.get("status"),
    "phase": state.get("phase"),
    "reason_code": state.get("reason_code"),
    "repo": state.get("repo_path"),
    "issue_url": state.get("issue_url"),
    "verifier": state.get("verifier_summary"),
    "changed_files": state.get("changed_files") or [],
    "operator_state": state.get("operator_state"),
    "operator_next_command": state.get("operator_next_command"),
    "replay_safe": state.get("replay_safe", False),
    "recoverable": state.get("recoverable", False),
    "needs_authority": state.get("needs_authority", False),
    "stale_lease": state.get("stale_lease", False),
    "mark_blocked_command": state.get("mark_blocked_command"),
    "pr_state": state.get("pr_state"),
    "pr_url": state.get("pr_url"),
    "pr_ready_command": state.get("pr_ready_command"),
    "pr_publication_reason_code": state.get("pr_publication_reason_code"),
    "pr_publication_command": state.get("pr_publication_command"),
    "pr_publication_stdout_path": state.get("pr_publication_stdout_path"),
    "pr_publication_stderr_path": state.get("pr_publication_stderr_path"),
    "pr_publication_exit_status": state.get("pr_publication_exit_status"),
    "auto_merge_status": state.get("auto_merge_status"),
    "auto_merge_reason_code": state.get("auto_merge_reason_code"),
    "auto_merge_command": state.get("auto_merge_command"),
    "auto_merge_pr_url": state.get("auto_merge_pr_url"),
    "provider_repair_status": state.get("provider_repair_status"),
    "provider_repair_reason_code": state.get("provider_repair_reason_code"),
    "provider_repair_attempts": state.get("provider_repair_attempts", 0),
    "provider_repair_max_attempts": state.get("provider_repair_max_attempts", 0),
    "provider_repair_provider": state.get("provider_repair_provider"),
    "provider_repair_model": state.get("provider_repair_model"),
    "provider_repair_context_path": state.get("provider_repair_context_path"),
    "event_log": state.get("event_log_path"),
    "last_background_reason_code": state.get("last_background_reason_code"),
    "last_heartbeat_unix_ms": state.get("last_heartbeat_unix_ms"),
    "lease_expires_unix_ms": state.get("lease_expires_unix_ms"),
    "last_error": state.get("last_error"),
    "resume_explanation": resume_explanation(state),
}
for key, value in fields.items():
    print(f"tau-unified: autonomous_coding.job.{key}={clean(value)}")
PY
}

cmd_intakes() {
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --state-dir|--autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown intakes option: $1"
        ;;
    esac
  done

  python3 - "${autonomous_coding_state_dir}" <<'PY'
import glob
import json
import os
import sys

state_dir = sys.argv[1]
intake_dir = os.path.join(state_dir, "issue-intake")

def clean(value, default="none"):
    if value is None:
        value = default
    if isinstance(value, bool):
        value = str(value).lower()
    if isinstance(value, (list, tuple)):
        value = ",".join(clean(item, "") for item in value if clean(item, ""))
    value = str(value).replace("\n", " ").replace("\r", " ").replace("\t", " ").strip()
    return value if value else default

intakes = []
for path in glob.glob(os.path.join(intake_dir, "*.json")):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            payload = json.load(handle)
    except Exception:
        continue
    if payload.get("intake_id"):
        intakes.append(payload)

intakes.sort(key=lambda item: clean(item.get("intake_id"), ""))
print(f"tau-unified: autonomous_coding.intakes.state_dir={clean(state_dir)}")
print(f"tau-unified: autonomous_coding.intakes.count={len(intakes)}")
for intake in intakes:
    intake_id = clean(intake.get("intake_id"))
    question_count = len(intake.get("clarifying_questions") or [])
    missing_input_count = len(intake.get("missing_inputs") or [])
    print(
        "tau-unified: autonomous_coding.intake="
        f"{intake_id} status={clean(intake.get('status'))} "
        f"classification={clean(intake.get('classification'))} "
        f"decision={clean(intake.get('decision'))} "
        f"reason_code={clean(intake.get('reason_code'))} "
        f"question_count={question_count} "
        f"missing_input_count={missing_input_count}"
    )
    print(f"tau-unified: autonomous_coding.intake.{intake_id}.next_action={clean(intake.get('next_action_summary'))}")
PY
}

cmd_intake() {
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
  local intake_id=""
  if [[ $# -gt 0 && "$1" != --* ]]; then
    intake_id="$1"
    shift
  fi
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --state-dir|--autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
        shift 2
        ;;
      --intake-id)
        intake_id="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown intake option: $1"
        ;;
    esac
  done
  [[ -n "${intake_id}" ]] || die "intake requires <intake-id> or --intake-id"

  python3 - "${autonomous_coding_state_dir}" "${intake_id}" <<'PY'
import json
import os
import sys

state_dir, intake_id = sys.argv[1:]
path = os.path.join(state_dir, "issue-intake", f"{intake_id}.json")

def clean(value, default="none"):
    if value is None:
        value = default
    if isinstance(value, bool):
        value = str(value).lower()
    if isinstance(value, (list, tuple)):
        value = ",".join(clean(item, "") for item in value if clean(item, ""))
    value = str(value).replace("\n", " ").replace("\r", " ").replace("\t", " ").strip()
    return value if value else default

if not os.path.exists(path):
    print(f"tau-unified: autonomous_coding.intake.error=not_found intake_id={clean(intake_id)}")
    sys.exit(2)

with open(path, "r", encoding="utf-8") as handle:
    intake = json.load(handle)

verifier_plan = intake.get("verifier_plan") or {}
fields = {
    "id": intake.get("intake_id"),
    "status": intake.get("status"),
    "reason_code": intake.get("reason_code"),
    "classification": intake.get("classification"),
    "classification_summary": intake.get("classification_summary"),
    "decision": intake.get("decision"),
    "issue_url": intake.get("issue_url"),
    "issue_title": intake.get("issue_title"),
    "repo": intake.get("repo_path"),
    "base_branch": intake.get("base_branch"),
    "next_action": intake.get("next_action_summary"),
    "missing_inputs": intake.get("missing_inputs") or [],
    "verifier_plan.plan_kind": verifier_plan.get("plan_kind"),
    "verifier_plan.summary": verifier_plan.get("summary"),
    "verifier_plan.suggested_verifier_commands": verifier_plan.get("suggested_verifier_commands") or [],
    "verifier_plan.missing_inputs": verifier_plan.get("missing_inputs") or [],
    "verifier_plan.next_action": verifier_plan.get("next_action"),
}
for key, value in fields.items():
    print(f"tau-unified: autonomous_coding.intake.{key}={clean(value)}")

for requirement in intake.get("required_authority") or []:
    reason = clean(requirement.get("reason_code"), "unknown")
    summary = clean(requirement.get("summary"))
    required_input = clean(requirement.get("required_input"))
    print(
        f"tau-unified: autonomous_coding.intake.required_authority.{reason}="
        f"{summary} required_input={required_input}"
    )

for question in intake.get("clarifying_questions") or []:
    reason = clean(question.get("reason_code"), "unknown")
    prompt = clean(question.get("question"))
    required_input = clean(question.get("required_input"))
    print(
        f"tau-unified: autonomous_coding.intake.question.{reason}="
        f"{prompt} required_input={required_input}"
    )
PY
}

cmd_recover_jobs() {
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
  local jobs_state_dir="${JOBS_STATE_DIR_DEFAULT}"
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --state-dir|--autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
        shift 2
        ;;
      --jobs-state-dir)
        jobs_state_dir="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown recover option: $1"
        ;;
    esac
  done
  log "tau-unified: autonomous_coding.recover.state_dir=${autonomous_coding_state_dir}"
  run_autonomous_coding_job_cli recover --state-dir "${autonomous_coding_state_dir}" --jobs-state-dir "${jobs_state_dir}"
}

cmd_replay_job() {
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
  local jobs_state_dir="${JOBS_STATE_DIR_DEFAULT}"
  local job_id=""
  if [[ $# -gt 0 && "$1" != --* ]]; then
    job_id="$1"
    shift
  fi
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --state-dir|--autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
        shift 2
        ;;
      --jobs-state-dir)
        jobs_state_dir="$2"
        shift 2
        ;;
      --job-id)
        job_id="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown replay option: $1"
        ;;
    esac
  done
  [[ -n "${job_id}" ]] || die "replay requires <job-id> or --job-id"
  log "tau-unified: autonomous_coding.replay.job_id=${job_id}"
  run_autonomous_coding_job_cli replay --state-dir "${autonomous_coding_state_dir}" --jobs-state-dir "${jobs_state_dir}" --job-id "${job_id}"
}

cmd_block_job() {
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
  local jobs_state_dir="${JOBS_STATE_DIR_DEFAULT}"
  local reason_code="operator_marked_blocked"
  local detail="operator inspected job and marked it blocked"
  local job_id=""
  if [[ $# -gt 0 && "$1" != --* ]]; then
    job_id="$1"
    shift
  fi
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --state-dir|--autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
        shift 2
        ;;
      --jobs-state-dir)
        jobs_state_dir="$2"
        shift 2
        ;;
      --job-id)
        job_id="$2"
        shift 2
        ;;
      --reason-code)
        reason_code="$2"
        shift 2
        ;;
      --detail)
        detail="$2"
        shift 2
        ;;
      --help)
        usage
        exit 0
        ;;
      *)
        die "unknown block option: $1"
        ;;
    esac
  done
  [[ -n "${job_id}" ]] || die "block requires <job-id> or --job-id"
  log "tau-unified: autonomous_coding.block.job_id=${job_id}"
  run_autonomous_coding_job_cli mark-blocked \
    --state-dir "${autonomous_coding_state_dir}" \
    --jobs-state-dir "${jobs_state_dir}" \
    --job-id "${job_id}" \
    --reason-code "${reason_code}" \
    --detail "${detail}"
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
  rm -f "${FINGERPRINT_FILE}"
  rm -f "${CONTROL_SNAPSHOT_FILE}"
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
  local jobs_state_dir="$9"
  local autonomous_coding_state_dir="${10}"
  local request_timeout_ms="${11}"
  local agent_request_max_retries="${12}"
  local readiness_timeout_ms="${TUI_READINESS_TIMEOUT_MS_DEFAULT}"

  require_positive_integer "${readiness_timeout_ms}" "TAU_UNIFIED_TUI_READINESS_TIMEOUT_MS"

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
    --jobs-state-dir "${jobs_state_dir}" \
    --autonomous-coding-state-dir "${autonomous_coding_state_dir}" \
    --request-timeout-ms "${request_timeout_ms}" \
    --agent-request-max-retries "${agent_request_max_retries}"

  if wait_for_dashboard_artifacts "${dashboard_state_dir}" "${readiness_timeout_ms}"; then
    log "tau-unified: dashboard artifacts ready (${dashboard_state_dir})"
  else
    die "tau-unified: runtime bootstrap not ready after ${readiness_timeout_ms}ms (bind=${bind}, dashboard_state_dir=${dashboard_state_dir}); use --no-bootstrap-runtime to attach manually once the runtime is ready"
  fi
}

cmd_tui() {
  local dashboard_state_dir="${DASHBOARD_STATE_DIR_DEFAULT}"
  local gateway_state_dir="${GATEWAY_STATE_DIR_DEFAULT}"
  local jobs_state_dir="${JOBS_STATE_DIR_DEFAULT}"
  local autonomous_coding_state_dir="${AUTONOMOUS_CODING_STATE_DIR_DEFAULT}"
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
  local tui_mode="interactive"
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
      --interactive)
        tui_mode="interactive"
        shift
        ;;
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
      --jobs-state-dir)
        jobs_state_dir="$2"
        shift 2
        ;;
      --autonomous-coding-state-dir)
        autonomous_coding_state_dir="$2"
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

  require_positive_integer "${request_timeout_ms}" "--request-timeout-ms"
  require_non_negative_integer "${agent_request_max_retries}" "--agent-request-max-retries"

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
      "${jobs_state_dir}" \
      "${autonomous_coding_state_dir}" \
      "${request_timeout_ms}" \
      "${agent_request_max_retries}"
  fi

  local tui_cmd=()
  case "${tui_mode}" in
    live-shell)
      tui_cmd=(
        cargo run -p tau-tui -- shell-live
        --state-dir "${dashboard_state_dir}"
        --profile "${profile}"
        --watch
        --iterations "${iterations}"
        --interval-ms "${interval_ms}"
      )
      ;;
    interactive)
      if [[ "${auth_mode}" == "password-session" ]]; then
        die "interactive tui does not support --auth-mode=password-session"
      fi
      tui_cmd=(
        cargo run -p tau-tui -- interactive
        --profile "${profile}"
        --model "${model}"
        --bind "${bind}"
        --auth-mode "${auth_mode}"
        --request-timeout-ms "${request_timeout_ms}"
      )
      if [[ "${auth_mode}" == "token" ]]; then
        tui_cmd+=(--auth-token "${auth_token}")
      fi
      ;;
    agent)
      tui_cmd=(
        cargo run -p tau-tui -- agent
        --dashboard-state-dir "${dashboard_state_dir}"
        --gateway-state-dir "${gateway_state_dir}"
        --profile "${profile}"
        --model "${model}"
        --request-timeout-ms "${request_timeout_ms}"
        --agent-request-max-retries "${agent_request_max_retries}"
      )
      ;;
    *)
      die "invalid tui mode: ${tui_mode}"
      ;;
  esac
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
  jobs)
    cmd_jobs "$@"
    ;;
  job)
    cmd_job "$@"
    ;;
  intakes)
    cmd_intakes "$@"
    ;;
  intake)
    cmd_intake "$@"
    ;;
  recover)
    cmd_recover_jobs "$@"
    ;;
  replay)
    cmd_replay_job "$@"
    ;;
  block)
    cmd_block_job "$@"
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

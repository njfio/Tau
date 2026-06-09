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

count_runner_mode() {
  local mode="$1"
  local log_path="$2"
  grep -c "^runner_mode=${mode}\$" "${log_path}" 2>/dev/null || true
}

assert_runner_mode_absent() {
  local mode="$1"
  local log_path="$2"
  local label="$3"
  if grep -q "^runner_mode=${mode}\$" "${log_path}" 2>/dev/null; then
    echo "assertion failed (${label}): expected runner mode '${mode}' to be absent" >&2
    echo "runner log:" >&2
    cat "${log_path}" >&2
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
    if [[ "${TAU_UNIFIED_RUNNER_WRITE_DASHBOARD_ARTIFACTS:-false}" == "true" ]]; then
      dashboard_state_dir=""
      for arg in "$@"; do
        dashboard_state_dir="${arg}"
      done
      if [[ -n "${dashboard_state_dir}" ]]; then
        mkdir -p "${dashboard_state_dir}"
        printf '{}\n' >"${dashboard_state_dir}/state.json"
        printf '{}\n' >"${dashboard_state_dir}/control-state.json"
        printf '{}\n' >"${dashboard_state_dir}/auth-status.json"
      fi
    fi
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

test_tui_bootstrap_readiness_fails_closed_without_artifacts() {
  local test_runtime_dir="${tmp_dir}/readiness-runtime"
  local test_runner_log="${tmp_dir}/readiness-runner.log"
  local test_runner_pid="${tmp_dir}/readiness-runner.pid"
  local test_dashboard_state_dir="${tmp_dir}/readiness-dashboard"

  set +e
  local readiness_output
  readiness_output="$(
    TAU_UNIFIED_RUNNER="${runner}" \
    TAU_UNIFIED_RUNNER_LOG="${test_runner_log}" \
    TAU_UNIFIED_RUNNER_PID="${test_runner_pid}" \
    TAU_UNIFIED_RUNTIME_DIR="${test_runtime_dir}" \
    TAU_UNIFIED_TUI_READINESS_TIMEOUT_MS=200 \
    "${LAUNCHER_SCRIPT}" tui --bootstrap-runtime --dashboard-state-dir "${test_dashboard_state_dir}" --no-color 2>&1
  )"
  local readiness_rc=$?
  set -e

  assert_equals "2" "${readiness_rc}" "tui bootstrap readiness failure exit"
  assert_contains "${readiness_output}" "tau-unified: runtime bootstrap not ready" "tui bootstrap readiness diagnostic"
  assert_runner_mode_absent "tui" "${test_runner_log}" "tui launch skipped after readiness failure"
}

test_tui_bootstrap_readiness_failed_bootstrap_does_not_launch_tui() {
  local test_runtime_dir="${tmp_dir}/failed-bootstrap-runtime"
  local test_runner_log="${tmp_dir}/failed-bootstrap-runner.log"
  local test_runner_pid="${tmp_dir}/failed-bootstrap-runner.pid"
  local failed_runner="${tmp_dir}/failed-runner.sh"

  cat >"${failed_runner}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
mode="$1"
log_path="$2"
pid_path="$3"
shift 3
printf 'runner_mode=%s\nargs=%s\n' "${mode}" "$*" >>"${log_path}"
case "${mode}" in
  up)
    echo "tau-unified test: bind conflict on requested port" >&2
    exit 98
    ;;
  tui)
    exit 99
    ;;
  down|status)
    rm -f "${pid_path}"
    ;;
esac
EOF
  chmod +x "${failed_runner}"

  set +e
  local bootstrap_output
  bootstrap_output="$(
    TAU_UNIFIED_RUNNER="${failed_runner}" \
    TAU_UNIFIED_RUNNER_LOG="${test_runner_log}" \
    TAU_UNIFIED_RUNNER_PID="${test_runner_pid}" \
    TAU_UNIFIED_RUNTIME_DIR="${test_runtime_dir}" \
    "${LAUNCHER_SCRIPT}" tui --bootstrap-runtime --bind 127.0.0.1:8899 --no-color 2>&1
  )"
  local bootstrap_rc=$?
  set -e

  assert_equals "98" "${bootstrap_rc}" "tui failed bootstrap exit"
  assert_contains "${bootstrap_output}" "bind conflict" "tui failed bootstrap diagnostic"
  assert_runner_mode_absent "tui" "${test_runner_log}" "tui launch skipped after failed bootstrap"
}

test_status_control_plane_snapshot() {
  local test_runtime_dir="${tmp_dir}/status-runtime"
  local test_runner_log="${tmp_dir}/status-runner.log"
  local test_runner_pid="${tmp_dir}/status-runner.pid"
  local test_gateway_state_dir="${tmp_dir}/status-gateway"
  local test_dashboard_state_dir="${tmp_dir}/status-dashboard"
  local test_jobs_state_dir="${tmp_dir}/status-jobs"
  local test_autonomous_coding_state_dir="${tmp_dir}/status-autonomous-coding"
  local status_bind="127.0.0.1:8911"
  mkdir -p "${test_gateway_state_dir}/coding-missions"
  mkdir -p "${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job"
  mkdir -p "${test_autonomous_coding_state_dir}/issue-intake"
  cat >"${test_gateway_state_dir}/coding-missions/status-coding-alpha.json" <<JSON
{
  "schema_version": 1,
  "state_root": "${test_gateway_state_dir}",
  "mission_id": "status-coding-alpha",
  "session_key": "session-status-alpha",
  "repo_path": "${tmp_dir}/fixture-repo",
  "goal": "surface coding mission status",
  "base_branch": "master",
  "branch_prefix": "codex/issue-3654",
  "verifier_commands": ["cargo test -p tau-agent-core coding_mission"],
  "pr_mode": "pr_ready",
  "allowed_roots": ["${tmp_dir}"],
  "phase": "pr_ready",
  "created_unix_ms": 10,
  "updated_unix_ms": 99,
  "mission": {
    "schema_version": 1,
    "mission_id": "status-coding-alpha",
    "title": "surface coding mission status",
    "status": "checkpointed",
    "created_unix_ms": 10,
    "updated_unix_ms": 99,
    "tool_budget": {"max_tool_calls": 0, "consumed_tool_calls": 0},
    "checkpoints": [],
    "artifacts": [],
    "learning_records": [],
    "verification_gates": [],
    "recovery_state": null
  },
  "events": [],
  "command_evidence": [
    {
      "command_id": "verifier-0",
      "cwd": "${tmp_dir}/fixture-repo",
      "argv": ["cargo", "test", "-p", "tau-agent-core", "coding_mission"],
      "stdout_path": "${tmp_dir}/stdout-old.txt",
      "stderr_path": "${tmp_dir}/stderr-old.txt",
      "exit_status": 101,
      "elapsed_ms": 24,
      "reason_code": "red_verifier_failed",
      "status": "failed"
    },
    {
      "command_id": "verifier-1",
      "cwd": "${tmp_dir}/fixture-repo",
      "argv": ["cargo", "test", "-p", "tau-agent-core", "coding_mission"],
      "stdout_path": "${tmp_dir}/stdout.txt",
      "stderr_path": "${tmp_dir}/stderr.txt",
      "exit_status": 0,
      "elapsed_ms": 42,
      "reason_code": "verification_passed",
      "status": "succeeded"
    }
  ],
  "git_evidence": [
    {
      "kind": "branch_prepared",
      "branch_name": "codex/issue-3654-operator",
      "base_branch": "master",
      "created_branch": true,
      "reused_branch": false,
      "commit_hash": "abc1234",
      "changed_files": ["crates/tau-agent-core/src/coding_mission.rs"],
      "reason_code": "branch_prepared",
      "created_unix_ms": 40
    }
  ],
  "resume_checkpoint": {
    "next_action": "run_verifier",
    "branch_name": "codex/issue-3654-operator",
    "pending_verifier_command": "cargo test -p tau-agent-core coding_mission",
    "latest_verifier_command_id": "verifier-1",
    "mutation_fingerprint": {
      "changed_files": ["crates/tau-agent-core/src/coding_mission.rs"],
      "diff_hash": "diffhash"
    },
    "latest_learning_summary": "operator status checkpoint",
    "operator_resume_command": "tau coding resume status-coding-alpha",
    "updated_unix_ms": 99
  },
  "pr_ready_bundle": {
    "status": "manual_ready",
    "branch_name": "codex/issue-3654-operator",
    "commit_hash": "abc1234",
    "title": "Surface coding operator state",
    "body": "ready",
    "body_path": "${tmp_dir}/pr-body.md",
    "manual_gh_pr_create_command": "gh pr create --draft",
    "changed_files": ["crates/tau-agent-core/src/coding_mission.rs"],
    "verifier_evidence_ids": ["verifier-1"],
    "risk_notes": [],
    "rollback_notes": [],
    "pr_url": "https://github.com/example/tau/pull/3654",
    "created_unix_ms": 100
  }
}
JSON
  cat >"${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job.status.json" <<JSON
{
  "schema_version": 1,
  "job_id": "status-job",
  "mission_id": "status-mission",
  "background_job_id": "background-status-job",
  "status": "pr_ready",
  "phase": "pr_ready",
  "reason_code": "autonomous_coding_job_pr_ready",
  "repo_path": "${tmp_dir}/fixture-repo",
  "issue_url": "https://github.com/njfio/Tau/issues/3802",
  "verifier_summary": "succeeded:coding_verifier_green",
  "changed_files": ["status.txt", "docs/notes.txt"],
  "resume_command": "tau-autonomous-coding-job run --job-id status-job",
  "operator_state": "complete",
  "operator_next_command": "none",
  "replay_safe": false,
  "recoverable": false,
  "needs_authority": false,
  "stale_lease": false,
  "mark_blocked_command": "tau-autonomous-coding-job mark-blocked --state-dir ${test_autonomous_coding_state_dir} --job-id status-job --reason-code operator_marked_blocked",
  "pr_state": "draft_created",
  "pr_ready_command": "gh pr create --draft",
  "pr_url": "https://github.com/example/tau/pull/3802",
  "recovery_count": 1,
  "replay_count": 2,
  "last_background_reason_code": "autonomous_coding_job_background_recovered",
  "last_error": null,
  "auto_merge_status": "requested",
  "auto_merge_reason_code": "auto_merge_requested",
  "auto_merge_command": "gh pr merge --auto --squash",
  "auto_merge_pr_url": "https://github.com/example/tau/pull/3802",
  "provider_repair_status": "applied",
  "provider_repair_reason_code": "provider_repair_edit_parsed",
  "provider_repair_attempts": 1,
  "provider_repair_max_attempts": 3,
  "provider_repair_provider": "openrouter",
  "provider_repair_model": "qwen/qwen3-235b-a22b",
  "provider_repair_context_path": "${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/provider-repair-context-1.json",
  "event_log_path": "${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/events.jsonl",
  "last_heartbeat_unix_ms": 200,
  "lease_expires_unix_ms": 900200,
  "pr_publication_reason_code": "draft_pr_created",
  "pr_publication_command": "gh pr create --draft --head codex/status-job",
  "pr_publication_stdout_path": "${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/gh-pr-create.stdout",
  "pr_publication_stderr_path": "${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/gh-pr-create.stderr",
  "pr_publication_exit_status": 0,
  "metadata": {}
}
JSON
  cat >"${test_autonomous_coding_state_dir}/issue-intake/issue-3808-vague.json" <<JSON
{
  "schema_version": 1,
  "intake_id": "issue-3808-vague",
  "status": "blocked",
  "reason_code": "issue_intake_underspecified",
  "classification": "underspecified",
  "classification_summary": "Issue is missing expected behavior and verifier detail.",
  "decision": "needs_clarification",
  "clarifying_questions": [
    {
      "reason_code": "expected_behavior",
      "question": "What should happen after the fix?",
      "required_input": "expected behavior"
    },
    {
      "reason_code": "verifier_command",
      "question": "Which verifier command proves the fix?",
      "required_input": "verifier command"
    }
  ],
  "issue_url": "https://github.com/njfio/Tau/issues/3808",
  "issue_title": "Fix it",
  "issue_body_summary": "Broken",
  "repo_path": "${tmp_dir}/fixture-repo",
  "base_branch": "master",
  "required_authority": [
    {
      "reason_code": "verifier_authority_required",
      "summary": "A verifier command is required before mutation.",
      "required_input": "verifier command"
    }
  ],
  "verifier_plan": {
    "plan_kind": "generic_coding",
    "summary": "Need a focused verifier before editing.",
    "suggested_verifier_commands": ["cargo test -p tau-runtime spec_3808"],
    "missing_inputs": ["expected behavior", "verifier command"],
    "next_action": "Ask for expected behavior and verifier command."
  },
  "missing_inputs": ["expected behavior", "verifier command"],
  "next_action_summary": "Ask for expected behavior and verifier command.",
  "created_unix_ms": 11,
  "updated_unix_ms": 12
}
JSON
  cat >"${test_autonomous_coding_state_dir}/autonomous-coding-jobs/stale-job.status.json" <<JSON
{
  "schema_version": 1,
  "job_id": "stale-job",
  "mission_id": "stale-mission",
  "status": "running",
  "phase": "executing",
  "reason_code": "autonomous_coding_job_running",
  "repo_path": "${tmp_dir}/fixture-repo",
  "verifier_summary": "failed:coding_verifier_red",
  "changed_files": [],
  "resume_command": "mission resume stale-mission",
  "operator_state": "stale_lease",
  "operator_next_command": "tau-autonomous-coding-job recover --state-dir ${test_autonomous_coding_state_dir}",
  "replay_safe": true,
  "recoverable": true,
  "needs_authority": false,
  "stale_lease": true,
  "mark_blocked_command": "tau-autonomous-coding-job mark-blocked --state-dir ${test_autonomous_coding_state_dir} --job-id stale-job --reason-code operator_marked_blocked",
  "pr_state": "none",
  "recovery_count": 0,
  "replay_count": 0,
  "event_log_path": "${test_autonomous_coding_state_dir}/autonomous-coding-jobs/stale-job/events.jsonl",
  "last_heartbeat_unix_ms": 100,
  "lease_expires_unix_ms": 1,
  "last_error": "simulated stale lease",
  "metadata": {}
}
JSON

  local up_status_output
  up_status_output="$(
    TAU_UNIFIED_RUNNER="${runner}" \
    TAU_UNIFIED_RUNNER_LOG="${test_runner_log}" \
    TAU_UNIFIED_RUNNER_PID="${test_runner_pid}" \
    TAU_UNIFIED_RUNTIME_DIR="${test_runtime_dir}" \
    "${LAUNCHER_SCRIPT}" up \
      --profile status-profile \
      --bind "${status_bind}" \
      --gateway-state-dir "${test_gateway_state_dir}" \
      --dashboard-state-dir "${test_dashboard_state_dir}" \
      --jobs-state-dir "${test_jobs_state_dir}" \
      --autonomous-coding-state-dir "${test_autonomous_coding_state_dir}" 2>&1
  )"
  assert_contains "${up_status_output}" "tau-unified: started" "status contract up marker"
  assert_contains "$(cat "${test_runtime_dir}/tau-unified.last-cmd")" "--jobs-state-dir ${test_jobs_state_dir}" "status jobs state command propagation"

  local status_output
  status_output="$(
    TAU_UNIFIED_RUNNER="${runner}" \
    TAU_UNIFIED_RUNNER_LOG="${test_runner_log}" \
    TAU_UNIFIED_RUNNER_PID="${test_runner_pid}" \
    TAU_UNIFIED_RUNTIME_DIR="${test_runtime_dir}" \
    "${LAUNCHER_SCRIPT}" status 2>&1
  )"

  assert_contains "${status_output}" "tau-unified: control_plane.health=running" "status control-plane health"
  assert_contains "${status_output}" "tau-unified: control_plane.runtime_state_dir=${test_runtime_dir}" "status runtime dir"
  assert_contains "${status_output}" "tau-unified: control_plane.log_file=${test_runtime_dir}/tau-unified.log" "status log path"
  assert_contains "${status_output}" "tau-unified: control_plane.command_file=${test_runtime_dir}/tau-unified.last-cmd" "status command path"
  assert_contains "${status_output}" "tau-unified: control_plane.fingerprint_file=${test_runtime_dir}/tau-unified.runtime-fingerprint" "status fingerprint path"
  assert_contains "${status_output}" "tau-unified: control_plane.profile=status-profile" "status profile marker"
  assert_contains "${status_output}" "tau-unified: control_plane.webchat_url=http://${status_bind}/webchat" "status webchat endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.ops_url=http://${status_bind}/ops" "status ops endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.dashboard_url=http://${status_bind}/dashboard" "status dashboard endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.sessions_endpoint=http://${status_bind}/gateway/sessions" "status sessions endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.memory_endpoint=http://${status_bind}/gateway/memory/default" "status memory endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.memory_graph_endpoint=http://${status_bind}/gateway/memory-graph/default" "status memory graph endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.jobs_endpoint=http://${status_bind}/gateway/jobs" "status jobs endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.background_jobs.state_dir=${test_jobs_state_dir}" "status background jobs state dir"
  assert_contains "${status_output}" "tau-unified: control_plane.background_jobs.manifest_dir=${test_jobs_state_dir}/jobs" "status background jobs manifest dir"
  assert_contains "${status_output}" "tau-unified: control_plane.background_jobs.events_file=${test_jobs_state_dir}/events.jsonl" "status background jobs events file"
  assert_contains "${status_output}" "tau-unified: control_plane.background_jobs.health_file=${test_jobs_state_dir}/state.json" "status background jobs health file"
  assert_contains "${status_output}" "tau-unified: control_plane.background_jobs.restart_recovery=running_manifests_requeued_after_restart" "status background jobs restart recovery"
  assert_contains "${status_output}" "tau-unified: control_plane.background_jobs.ops_guide=docs/guides/background-jobs-ops.md" "status background jobs ops guide"
  assert_contains "${status_output}" "tau-unified: control_plane.routines_surface=http://${status_bind}/webchat#routines" "status routines surface"
  assert_contains "${status_output}" "tau-unified: control_plane.deploy_endpoint=http://${status_bind}/ops/deploy" "status ops deploy endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.gateway_deploy_endpoint=http://${status_bind}/gateway/deploy" "status gateway deploy endpoint"
  assert_contains "${status_output}" "tau-unified: control_plane.gateway_state_dir=${test_gateway_state_dir}" "status gateway state dir"
  assert_contains "${status_output}" "tau-unified: control_plane.dashboard_state_dir=${test_dashboard_state_dir}" "status dashboard state dir"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.state_dir=${test_autonomous_coding_state_dir}" "status autonomous coding state dir"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.jobs_dir=${test_autonomous_coding_state_dir}/autonomous-coding-jobs" "status autonomous coding jobs dir"
  assert_contains "${status_output}" "tau-unified: control_plane.deploy_state_file=${test_gateway_state_dir}/deploy-agent-state.json" "status deploy state file"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.id=status-coding-alpha" "status coding mission id"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.phase=pr_ready" "status coding mission phase"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.repo=${tmp_dir}/fixture-repo" "status coding mission repo"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.branch=codex/issue-3654-operator" "status coding mission branch"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.verifier=succeeded:verification_passed" "status coding mission verifier"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.last_failure=failed:red_verifier_failed" "status coding mission last failure"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.changed_files=crates/tau-agent-core/src/coding_mission.rs" "status coding mission changed files"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.resume_command=tau coding resume status-coding-alpha" "status coding mission resume command"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.pr_state=manual_ready" "status coding mission pr state"
  assert_contains "${status_output}" "tau-unified: control_plane.coding_mission.pr_url=https://github.com/example/tau/pull/3654" "status coding mission pr url"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.job_id=status-job" "status autonomous coding job id"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.mission_id=status-mission" "status autonomous coding mission id"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.background_job_id=background-status-job" "status autonomous coding background job id"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.status=pr_ready" "status autonomous coding status"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.phase=pr_ready" "status autonomous coding phase"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.reason_code=autonomous_coding_job_pr_ready" "status autonomous coding reason"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.repo=${tmp_dir}/fixture-repo" "status autonomous coding repo"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.verifier=succeeded:coding_verifier_green" "status autonomous coding verifier"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.changed_files=status.txt,docs/notes.txt" "status autonomous coding changed files"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.resume_command=tau-autonomous-coding-job run --job-id status-job" "status autonomous coding resume"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.operator_state=complete" "status autonomous coding operator state"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.operator_next_command=none" "status autonomous coding operator next"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.replay_safe=false" "status autonomous coding replay safe"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.recoverable=false" "status autonomous coding recoverable"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.needs_authority=false" "status autonomous coding needs authority"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.stale_lease=false" "status autonomous coding stale lease"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.mark_blocked_command=tau-autonomous-coding-job mark-blocked --state-dir ${test_autonomous_coding_state_dir} --job-id status-job --reason-code operator_marked_blocked" "status autonomous coding mark blocked"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.pr_state=draft_created" "status autonomous coding pr state"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.pr_ready_command=gh pr create --draft" "status autonomous coding pr ready command"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.pr_url=https://github.com/example/tau/pull/3802" "status autonomous coding pr url"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.pr_publication.stdout_path=${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/gh-pr-create.stdout" "status autonomous coding pr stdout"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.pr_publication.stderr_path=${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/gh-pr-create.stderr" "status autonomous coding pr stderr"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.pr_publication.exit_status=0" "status autonomous coding pr exit"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.auto_merge.status=requested" "status autonomous coding auto merge"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.auto_merge.reason_code=auto_merge_requested" "status autonomous coding auto merge reason"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.auto_merge.command=gh pr merge --auto --squash" "status autonomous coding auto merge command"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.auto_merge.pr_url=https://github.com/example/tau/pull/3802" "status autonomous coding auto merge pr"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.provider_repair.status=applied" "status autonomous coding provider repair"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.provider_repair.reason_code=provider_repair_edit_parsed" "status autonomous coding provider repair reason"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.provider_repair.attempts=1" "status autonomous coding provider repair attempts"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.provider_repair.max_attempts=3" "status autonomous coding provider repair max"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.provider_repair.provider=openrouter" "status autonomous coding provider"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.provider_repair.model=qwen/qwen3-235b-a22b" "status autonomous coding model"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.provider_repair.context=${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/provider-repair-context-1.json" "status autonomous coding repair context"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.event_log=${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/events.jsonl" "status autonomous coding event log"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.last_heartbeat_unix_ms=200" "status autonomous coding heartbeat"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.lease_expires_unix_ms=900200" "status autonomous coding lease"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.recovery_count=1" "status autonomous coding recovery"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.replay_count=2" "status autonomous coding replay"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomous_coding.last_background_reason_code=autonomous_coding_job_background_recovered" "status autonomous coding background reason"
  assert_contains "${status_output}" "tau-unified: control_plane.autonomy_boundary=provider_repair_durable_jobs_visible_crash_resume_recovery_in_progress" "status autonomy boundary"

  local jobs_output
  jobs_output="$("${LAUNCHER_SCRIPT}" jobs --autonomous-coding-state-dir "${test_autonomous_coding_state_dir}" 2>&1)"
  assert_contains "${jobs_output}" "tau-unified: autonomous_coding.jobs.count=2" "jobs list count"
  assert_contains "${jobs_output}" "tau-unified: autonomous_coding.job=stale-job status=running operator_state=stale_lease replay_safe=true recoverable=true" "jobs stale summary"
  assert_contains "${jobs_output}" "tau-unified: autonomous_coding.job.stale-job.resume_explanation=recoverable stale lease; run tau-autonomous-coding-job recover --state-dir ${test_autonomous_coding_state_dir}" "jobs stale explanation"
  assert_contains "${jobs_output}" "tau-unified: autonomous_coding.job=status-job status=pr_ready operator_state=complete" "jobs complete summary"
  assert_contains "${jobs_output}" "tau-unified: autonomous_coding.job.status-job.event_log=${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/events.jsonl" "jobs complete event log"
  assert_contains "${jobs_output}" "tau-unified: autonomous_coding.job.status-job.provider_repair_context=${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/provider-repair-context-1.json" "jobs complete provider context"
  assert_contains "${jobs_output}" "tau-unified: autonomous_coding.job.status-job.mark_blocked_command=tau-autonomous-coding-job mark-blocked --state-dir ${test_autonomous_coding_state_dir} --job-id status-job --reason-code operator_marked_blocked" "jobs complete mark blocked command"

  local job_output
  job_output="$("${LAUNCHER_SCRIPT}" job status-job --autonomous-coding-state-dir "${test_autonomous_coding_state_dir}" 2>&1)"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.id=status-job" "job inspect id"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.background_job_id=background-status-job" "job inspect background job id"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.verifier=succeeded:coding_verifier_green" "job inspect verifier"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.pr_publication_reason_code=draft_pr_created" "job inspect pr publication reason"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.pr_publication_command=gh pr create --draft --head codex/status-job" "job inspect pr publication command"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.pr_publication_stdout_path=${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/gh-pr-create.stdout" "job inspect pr publication stdout"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.pr_publication_stderr_path=${test_autonomous_coding_state_dir}/autonomous-coding-jobs/status-job/gh-pr-create.stderr" "job inspect pr publication stderr"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.pr_publication_exit_status=0" "job inspect pr publication exit"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.auto_merge_command=gh pr merge --auto --squash" "job inspect auto merge command"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.last_background_reason_code=autonomous_coding_job_background_recovered" "job inspect background reason"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.provider_repair_model=qwen/qwen3-235b-a22b" "job inspect provider model"
  assert_contains "${job_output}" "tau-unified: autonomous_coding.job.resume_explanation=complete; no replay or recovery needed" "job inspect complete explanation"

  local intakes_output
  intakes_output="$("${LAUNCHER_SCRIPT}" intakes --autonomous-coding-state-dir "${test_autonomous_coding_state_dir}" 2>&1)"
  assert_contains "${intakes_output}" "tau-unified: autonomous_coding.intakes.count=1" "intakes list count"
  assert_contains "${intakes_output}" "tau-unified: autonomous_coding.intake=issue-3808-vague status=blocked classification=underspecified decision=needs_clarification reason_code=issue_intake_underspecified question_count=2 missing_input_count=2" "intakes list summary"
  assert_contains "${intakes_output}" "tau-unified: autonomous_coding.intake.issue-3808-vague.next_action=Ask for expected behavior and verifier command." "intakes list next action"

  local intake_output
  intake_output="$("${LAUNCHER_SCRIPT}" intake issue-3808-vague --autonomous-coding-state-dir "${test_autonomous_coding_state_dir}" 2>&1)"
  assert_contains "${intake_output}" "tau-unified: autonomous_coding.intake.id=issue-3808-vague" "intake inspect id"
  assert_contains "${intake_output}" "tau-unified: autonomous_coding.intake.decision=needs_clarification" "intake inspect decision"
  assert_contains "${intake_output}" "tau-unified: autonomous_coding.intake.verifier_plan.plan_kind=generic_coding" "intake inspect verifier plan kind"
  assert_contains "${intake_output}" "tau-unified: autonomous_coding.intake.verifier_plan.suggested_verifier_commands=cargo test -p tau-runtime spec_3808" "intake inspect suggested verifier"
  assert_contains "${intake_output}" "tau-unified: autonomous_coding.intake.required_authority.verifier_authority_required=A verifier command is required before mutation. required_input=verifier command" "intake inspect authority"
  assert_contains "${intake_output}" "tau-unified: autonomous_coding.intake.question.expected_behavior=What should happen after the fix? required_input=expected behavior" "intake inspect question"

  local fake_cli="${tmp_dir}/fake-autonomous-coding-job.sh"
  local fake_cli_args="${tmp_dir}/fake-autonomous-coding-job.args"
  cat >"${fake_cli}" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$@" >"${TAU_FAKE_AUTONOMOUS_CODING_JOB_ARGS}"
printf '{"fake_cli":true,"command":"%s"}\n' "${1:-none}"
EOF
  chmod +x "${fake_cli}"

  local recover_output
  recover_output="$(
    TAU_UNIFIED_AUTONOMOUS_CODING_JOB_CLI="${fake_cli}" \
    TAU_FAKE_AUTONOMOUS_CODING_JOB_ARGS="${fake_cli_args}" \
    "${LAUNCHER_SCRIPT}" recover --autonomous-coding-state-dir "${test_autonomous_coding_state_dir}" --jobs-state-dir "${test_jobs_state_dir}" 2>&1
  )"
  assert_contains "${recover_output}" "tau-unified: autonomous_coding.recover.state_dir=${test_autonomous_coding_state_dir}" "recover marker"
  assert_contains "${recover_output}" '"command":"recover"' "recover json"
  assert_contains "$(cat "${fake_cli_args}")" "recover" "recover delegates command"
  assert_contains "$(cat "${fake_cli_args}")" "--jobs-state-dir" "recover delegates jobs state"

  local replay_output
  replay_output="$(
    TAU_UNIFIED_AUTONOMOUS_CODING_JOB_CLI="${fake_cli}" \
    TAU_FAKE_AUTONOMOUS_CODING_JOB_ARGS="${fake_cli_args}" \
    "${LAUNCHER_SCRIPT}" replay stale-job --autonomous-coding-state-dir "${test_autonomous_coding_state_dir}" --jobs-state-dir "${test_jobs_state_dir}" 2>&1
  )"
  assert_contains "${replay_output}" "tau-unified: autonomous_coding.replay.job_id=stale-job" "replay marker"
  assert_contains "${replay_output}" '"command":"replay"' "replay json"
  assert_contains "$(cat "${fake_cli_args}")" "stale-job" "replay delegates job id"

  local block_output
  block_output="$(
    TAU_UNIFIED_AUTONOMOUS_CODING_JOB_CLI="${fake_cli}" \
    TAU_FAKE_AUTONOMOUS_CODING_JOB_ARGS="${fake_cli_args}" \
    "${LAUNCHER_SCRIPT}" block stale-job --autonomous-coding-state-dir "${test_autonomous_coding_state_dir}" --jobs-state-dir "${test_jobs_state_dir}" --reason-code inspected_block --detail "inspected and blocked" 2>&1
  )"
  assert_contains "${block_output}" "tau-unified: autonomous_coding.block.job_id=stale-job" "block marker"
  assert_contains "${block_output}" '"command":"mark-blocked"' "block json"
  assert_contains "$(cat "${fake_cli_args}")" "inspected_block" "block delegates reason"

  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${test_runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${test_runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${test_runtime_dir}" \
  "${LAUNCHER_SCRIPT}" down >/dev/null 2>&1 || true
}

case "${1:-all}" in
  all)
    ;;
  tui_bootstrap_readiness)
    test_tui_bootstrap_readiness_fails_closed_without_artifacts
    test_tui_bootstrap_readiness_failed_bootstrap_does_not_launch_tui
    echo "tau-unified tui bootstrap readiness tests passed"
    exit 0
    ;;
  status_contract)
    test_status_control_plane_snapshot
    echo "tau-unified status control-plane tests passed"
    exit 0
    ;;
  *)
    echo "unknown test selector: $1" >&2
    exit 2
    ;;
esac

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
  "${LAUNCHER_SCRIPT}" up --profile test-profile --bind 127.0.0.1:8899 --auth-mode localhost-dev 2>&1
)"
up_rc=$?
set -e
assert_equals "0" "${up_rc}" "up exit"
assert_contains "${up_output}" "tau-unified: started" "up output marker"
assert_contains "${up_output}" "http://127.0.0.1:8899/webchat" "up webchat endpoint"

pid_file="${runtime_dir}/tau-unified.pid"
log_file="${runtime_dir}/tau-unified.log"
cmd_file="${runtime_dir}/tau-unified.last-cmd"
fingerprint_file="${runtime_dir}/tau-unified.runtime-fingerprint"

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
if [[ ! -f "${fingerprint_file}" ]]; then
  echo "expected fingerprint file to exist after up: ${fingerprint_file}" >&2
  exit 1
fi
assert_contains "$(cat "${cmd_file}")" "--model gpt-5.3-codex" "up default model flag"
assert_contains "$(cat "${cmd_file}")" "cargo run -p tau-coding-agent --bin tau-coding-agent --" "up explicit cargo binary"
assert_contains "$(cat "${cmd_file}")" "--request-timeout-ms 180000" "up default timeout flag"
assert_contains "$(cat "${cmd_file}")" "--turn-timeout-ms 180000" "up default turn timeout flag"
assert_contains "$(cat "${cmd_file}")" "--agent-request-max-retries 0" "up default agent retries flag"
assert_contains "$(cat "${cmd_file}")" "--provider-max-retries 0" "up default provider retries flag"
assert_contains "$(cat "${cmd_file}")" "--jobs-state-dir .tau/jobs" "up default jobs state dir flag"
if [[ -z "$(cat "${fingerprint_file}")" ]]; then
  echo "expected fingerprint file to contain a non-empty fingerprint" >&2
  exit 1
fi

same_up_count_before="$(count_runner_mode up "${runner_log}")"
same_down_count_before="$(count_runner_mode down "${runner_log}")"
same_up_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" up --profile test-profile --bind 127.0.0.1:8899 --auth-mode localhost-dev 2>&1
)"
assert_contains "${same_up_output}" "tau-unified: already running" "same fingerprint reuse marker"
same_up_count_after="$(count_runner_mode up "${runner_log}")"
same_down_count_after="$(count_runner_mode down "${runner_log}")"
assert_equals "${same_up_count_before}" "${same_up_count_after}" "same fingerprint up reuse"
assert_equals "${same_down_count_before}" "${same_down_count_after}" "same fingerprint down reuse"

printf 'stale-fingerprint\n' > "${fingerprint_file}"
stale_up_count_before="$(count_runner_mode up "${runner_log}")"
stale_down_count_before="$(count_runner_mode down "${runner_log}")"
stale_up_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" up --profile test-profile --bind 127.0.0.1:8899 --auth-mode localhost-dev 2>&1
)"
assert_contains "${stale_up_output}" "tau-unified: recycling stale runtime" "stale up recycle marker"
assert_contains "${stale_up_output}" "tau-unified: started" "stale up restart marker"
stale_up_count_after="$(count_runner_mode up "${runner_log}")"
stale_down_count_after="$(count_runner_mode down "${runner_log}")"
if [[ "${stale_up_count_after}" -le "${stale_up_count_before}" ]]; then
  echo "assertion failed (stale up restarts runtime): expected runner up count to increase" >&2
  cat "${runner_log}" >&2
  exit 1
fi
if [[ "${stale_down_count_after}" -le "${stale_down_count_before}" ]]; then
  echo "assertion failed (stale up recycles runtime): expected runner down count to increase" >&2
  cat "${runner_log}" >&2
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
assert_contains "${status_output}" "tau-unified: control_plane.health=running" "status control-plane running marker"
assert_contains "${status_output}" "tau-unified: control_plane.sessions_endpoint=http://127.0.0.1:8899/gateway/sessions" "status sessions marker"
assert_contains "${status_output}" "tau-unified: control_plane.memory_endpoint=http://127.0.0.1:8899/gateway/memory/default" "status memory marker"
assert_contains "${status_output}" "tau-unified: control_plane.jobs_endpoint=http://127.0.0.1:8899/gateway/jobs" "status jobs marker"
assert_contains "${status_output}" "tau-unified: control_plane.background_jobs.state_dir=.tau/jobs" "status background jobs default state marker"
assert_contains "${status_output}" "tau-unified: control_plane.background_jobs.restart_recovery=running_manifests_requeued_after_restart" "status background jobs recovery marker"
assert_contains "${status_output}" "tau-unified: control_plane.deploy_endpoint=http://127.0.0.1:8899/ops/deploy" "status deploy marker"
assert_contains "${status_output}" "tau-unified: control_plane.autonomy_boundary=provider_repair_durable_jobs_visible_crash_resume_recovery_in_progress" "status autonomy boundary marker"

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
if [[ -f "${fingerprint_file}" ]]; then
  echo "expected fingerprint file to be removed after down: ${fingerprint_file}" >&2
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

up_count_before_tui="$(count_runner_mode up "${runner_log}")"

tui_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  "${LAUNCHER_SCRIPT}" tui --no-color 2>&1 || true
)"
assert_contains "${tui_output}" "tau-unified: launching tui (interactive)" "tui interactive marker"
up_count_after_tui="$(count_runner_mode up "${runner_log}")"
assert_equals "${up_count_before_tui}" "${up_count_after_tui}" "tui default does not bootstrap runtime in runner mode"
assert_contains "$(cat "${runner_log}")" "--request-timeout-ms 180000" "tui default timeout flag"
assert_contains "$(cat "${runner_log}")" "--turn-timeout-ms 180000" "tui default turn timeout flag"
assert_contains "$(cat "${runner_log}")" "--agent-request-max-retries 0" "tui default retries flag"

up_count_before_bootstrap="$(count_runner_mode up "${runner_log}")"
tui_bootstrap_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  TAU_UNIFIED_RUNNER_WRITE_DASHBOARD_ARTIFACTS=true \
  "${LAUNCHER_SCRIPT}" tui --bootstrap-runtime --no-color 2>&1 || true
)"
assert_contains "${tui_bootstrap_output}" "tau-unified: bootstrapping runtime for tui" "tui bootstrap marker"
assert_contains "${tui_bootstrap_output}" "tau-unified: started" "tui bootstrap started"
up_count_after_bootstrap="$(count_runner_mode up "${runner_log}")"
if [[ "${up_count_after_bootstrap}" -le "${up_count_before_bootstrap}" ]]; then
  echo "assertion failed (runner up logged for bootstrap path): expected up count to increase" >&2
  echo "before=${up_count_before_bootstrap} after=${up_count_after_bootstrap}" >&2
  echo "runner log:" >&2
  cat "${runner_log}" >&2
  exit 1
fi

tui_same_up_count_before="$(count_runner_mode up "${runner_log}")"
tui_same_down_count_before="$(count_runner_mode down "${runner_log}")"
tui_same_bootstrap_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  TAU_UNIFIED_RUNNER_WRITE_DASHBOARD_ARTIFACTS=true \
  "${LAUNCHER_SCRIPT}" tui --bootstrap-runtime --no-color 2>&1 || true
)"
assert_not_contains "${tui_same_bootstrap_output}" "tau-unified: recycling stale runtime" "tui bootstrap same fingerprint reuse"
tui_same_up_count_after="$(count_runner_mode up "${runner_log}")"
tui_same_down_count_after="$(count_runner_mode down "${runner_log}")"
assert_equals "${tui_same_up_count_before}" "${tui_same_up_count_after}" "tui bootstrap same fingerprint up reuse"
assert_equals "${tui_same_down_count_before}" "${tui_same_down_count_after}" "tui bootstrap same fingerprint down reuse"

printf 'stale-fingerprint\n' > "${fingerprint_file}"
tui_stale_up_count_before="$(count_runner_mode up "${runner_log}")"
tui_stale_down_count_before="$(count_runner_mode down "${runner_log}")"
tui_stale_bootstrap_output="$(
  TAU_UNIFIED_RUNNER="${runner}" \
  TAU_UNIFIED_RUNNER_LOG="${runner_log}" \
  TAU_UNIFIED_RUNNER_PID="${runner_pid}" \
  TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}" \
  TAU_UNIFIED_RUNNER_WRITE_DASHBOARD_ARTIFACTS=true \
  "${LAUNCHER_SCRIPT}" tui --bootstrap-runtime --no-color 2>&1 || true
)"
assert_contains "${tui_stale_bootstrap_output}" "tau-unified: recycling stale runtime" "tui bootstrap stale recycle marker"
tui_stale_up_count_after="$(count_runner_mode up "${runner_log}")"
tui_stale_down_count_after="$(count_runner_mode down "${runner_log}")"
if [[ "${tui_stale_up_count_after}" -le "${tui_stale_up_count_before}" ]]; then
  echo "assertion failed (tui bootstrap stale runtime restarts): expected runner up count to increase" >&2
  cat "${runner_log}" >&2
  exit 1
fi
if [[ "${tui_stale_down_count_after}" -le "${tui_stale_down_count_before}" ]]; then
  echo "assertion failed (tui bootstrap stale runtime recycles): expected runner down count to increase" >&2
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
assert_contains "${tui_override_output}" "tau-unified: launching tui (interactive)" "tui override marker"

up_override_count_before="$(count_runner_mode up "${runner_log}")"
tui_override_bootstrap_output="$(
  TAU_UNIFIED_RUNNER="${runner}"   TAU_UNIFIED_RUNNER_LOG="${runner_log}"   TAU_UNIFIED_RUNNER_PID="${runner_pid}"   TAU_UNIFIED_RUNTIME_DIR="${runtime_dir}"   TAU_UNIFIED_RUNNER_WRITE_DASHBOARD_ARTIFACTS=true   "${LAUNCHER_SCRIPT}" tui --bootstrap-runtime --request-timeout-ms 9000 --agent-request-max-retries 2 --no-color 2>&1 || true
)"
assert_contains "${tui_override_bootstrap_output}" "tau-unified: bootstrapping runtime for tui" "tui override bootstrap marker"
up_override_count_after="$(count_runner_mode up "${runner_log}")"
if [[ "${up_override_count_after}" -le "${up_override_count_before}" ]]; then
  echo "assertion failed (tui override bootstrap restarts runtime): expected runner up count to increase" >&2
  cat "${runner_log}" >&2
  exit 1
fi
assert_contains "$(cat "${runner_log}")" "--request-timeout-ms 9000" "tui override timeout flag"
assert_contains "$(cat "${runner_log}")" "--turn-timeout-ms 9000" "tui override turn timeout flag"
assert_contains "$(cat "${runner_log}")" "--agent-request-max-retries 2" "tui override retries flag"

assert_contains "$(cat "${runner_log}")" "runner_mode=up" "runner up logged"
assert_contains "$(cat "${runner_log}")" "runner_mode=status" "runner status logged"
assert_contains "$(cat "${runner_log}")" "runner_mode=down" "runner down logged"
assert_contains "$(cat "${runner_log}")" "runner_mode=tui" "runner tui logged"
assert_contains "$(cat "${runner_log}")" "args=interactive" "runner tui interactive args"
assert_contains "$(cat "${runner_log}")" "args=live-shell" "runner tui live-shell args"

echo "tau-unified launcher tests passed"

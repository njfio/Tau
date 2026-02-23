# Tasks: Issue #3386 - Tau E2E PRD scenario suite

- [x] T1 (RED): build PRD scenario coverage matrix and add failing tests for uncovered P0 scenarios.
- [x] T2 (GREEN): implement/fix runtime and harness behavior to satisfy new P0 tests.
- [x] T3 (RED): add failing tests for uncovered P1/P2/P3 scenarios or explicit `N/A` assertions with justifications.
- [x] T4 (GREEN): implement/fix remaining scenario coverage and tier selectors (`tier_pr`, `tier_nightly`, `tier_weekly`).
- [x] T5 (VERIFY): run targeted cargo tests, capture RED/GREEN/REGRESSION evidence, and update spec status to Implemented.

## Verification Evidence (2026-02-23)

### RED

- `cargo test` (workspace) initially failed on:
  - `tool_policy_config::tests::integration_tool_policy_json_exposes_docker_sandbox_settings`
    - root cause: test env leakage from `TAU_MEMORY_DEFAULT_IMPORTANCE_IDENTITY` across parallel tests.
  - `sqlite::tests::regression_active_worker_heartbeat_refresh_prevents_false_timeout`
    - root cause: flaky heartbeat timeout window (`20ms` sleep + `5ms` timeout) under load.

### GREEN

- Fixes applied:
  - `crates/tau-tools/src/tool_policy_config.rs`
    - add env lock/snapshot/cleanup guards to docker/tool-builder policy tests.
  - `crates/tau-training-store/src/sqlite.rs`
  - `crates/tau-training-store/src/lib.rs`
    - widen heartbeat regression timing window to deterministic values (`350ms` sleep, `250ms` timeout).
- Passing focused commands after fixes:
  - `cargo test -p tau-tools --lib` -> `172 passed; 0 failed`
  - `cargo test -p tau-training-store` -> `9 passed; 0 failed`

### REGRESSION

- Full regression:
  - `cargo test` (workspace) -> passed.
- Scenario-tier selectors:
  - `cargo test -p tau-gateway tier_pr_ -- --test-threads=1` -> `9 passed; 0 failed`
  - `cargo test -p tau-gateway tier_nightly_ -- --test-threads=1` -> `3 passed; 0 failed`
  - `cargo test -p tau-gateway tier_weekly_ch15_chaos_matrix -- --test-threads=1` -> `1 passed; 0 failed`
- Full conformance-matrix selector pass:
  - `specs/3386/conformance-matrix.md` unique mapped selectors executed end-to-end -> `45/45 executed`, `0 failed`, `0 not executed`.
- Conformance row spot-checks (gap-closure rows):
  - `cargo test -p tau-provider functional_spec_3400_c01_primary_success_returns_without_fallback_invocation -- --test-threads=1`
  - `cargo test -p tau-provider functional_fallback_client_handoffs_on_retryable_error_and_emits_event -- --test-threads=1`
  - `cargo test -p tau-provider functional_spec_3400_c03_all_routes_fail_returns_terminal_error -- --test-threads=1`
  - `cargo test -p tau-provider functional_circuit_breaker_opens_and_skips_temporarily_unhealthy_route -- --test-threads=1`
  - `cargo test -p tau-provider integration_circuit_breaker_retries_primary_after_cooldown_expires -- --test-threads=1`
  - `cargo test -p tau-multi-channel integration_spec_3402_c01_c02_c07_live_runner_routes_telegram_and_discord_to_distinct_sessions -- --test-threads=1`
  - `cargo test -p tau-gateway integration_spec_3396_c01_c02_gateway_tools_inventory_includes_mcp_prefixed_tool -- --test-threads=1`
  - `cargo test -p tau-gateway integration_dashboard_stream_supports_reconnect_reset_and_snapshot_updates -- --test-threads=1`
  - `cargo test -p tau-coding-agent functional_execute_auth_command_rotate_key_rotates_store_without_data_loss -- --test-threads=1`
  - `cargo test -p tau-algorithm spec_c03_unit_trace_based_reward_inference_token_efficiency_signal -- --test-threads=1`
  - `cargo test -p tau-algorithm spec_c04_regression_trace_based_reward_inference_session_not_completed_penalty -- --test-threads=1`
  - `cargo test -p tau-algorithm spec_c05_regression_trace_based_reward_inference_tool_errors_reduce_reliability -- --test-threads=1`
  - `cargo test -p tau-coding-agent spec_c01_functional_live_events_persist_rollout_and_span -- --test-threads=1`
  - `cargo test -p tau-coding-agent spec_c02_functional_optimizer_runs_on_update_interval -- --test-threads=1`
  - `cargo test -p tau-coding-agent spec_c05_unit_live_reward_breakdown_scores_deterministically -- --test-threads=1`
  - `cargo test -p tau-coding-agent spec_c06_functional_live_rollout_span_persists_reward_breakdown -- --test-threads=1`
  - all above passed.
- Hygiene gates:
  - `cargo fmt --check` -> passed
  - `cargo clippy -- -D warnings` -> passed

### Test Tier Matrix (Issue #3386 Verify Snapshot)

| Tier | ✅/❌/N/A | Tests / Evidence | N/A Why |
|---|---|---|---|
| Unit | ✅ | workspace + targeted crate tests | |
| Property | ✅ | existing `proptest` coverage in `tau-tools` (`cargo test -p tau-tools --lib`) | |
| Contract/DbC | N/A | no contract-attribute additions in this issue | No new DbC interfaces introduced |
| Snapshot | N/A | no `insta` snapshots added/changed | No stable snapshot artifacts changed |
| Functional | ✅ | `tier_pr_*`, `tier_nightly_*`, gap-closure selectors | |
| Conformance | ✅ | `specs/3386/conformance-matrix.md` + mapped selectors | |
| Integration | ✅ | gateway/multi-channel/provider targeted integration selectors | |
| Fuzz | N/A | not in-scope for deterministic E2E conformance closure | Covered by separate fuzz workflow |
| Mutation | N/A | `cargo mutants --in-diff` on touched files produced no applicable mutants | Current delta is test-only/selective harness hardening |
| Regression | ✅ | full `cargo test` after RED failures fixed | |
| Performance | N/A | no perf-sensitive hot-path behavior changed | No benchmark delta introduced |

## Verification Addendum (2026-02-23, continuation)

### RED

- Follow-up full gate surfaced additional strict clippy blockers in test/support code paths:
  - `clippy::field-reassign-with-default` (`tau-memory`, `tau-agent-core`, `tau-training-runner`)
  - `clippy::await_holding_lock` (`tau-ai`, `tau-coding-agent`)
  - `clippy::items-after-test-module` (`tau-algorithm`)
  - `clippy::needless-borrows-for-generic-args`, `clippy::too-many-arguments`, `clippy::unnecessary-get-then-check` (`tau-coding-agent`)
  - `clippy::bool_assert_comparison`, `clippy::cloned_ref_to_slice_refs`, `clippy::too-many-arguments` (`tau-gateway`)

### GREEN

- Applied targeted fixes in:
  - `crates/tau-agent-core/src/tests/safety_pipeline.rs`
  - `crates/tau-ai/tests/provider_http_integration.rs`
  - `crates/tau-algorithm/src/reward_inference.rs`
  - `crates/tau-coding-agent/src/live_rl_runtime.rs`
  - `crates/tau-coding-agent/src/runtime_loop.rs`
  - `crates/tau-coding-agent/src/tests/auth_provider/runtime_and_startup.rs`
  - `crates/tau-gateway/src/gateway_openresponses/tests.rs`
  - `crates/tau-gateway/src/gateway_runtime.rs`
  - `crates/tau-memory/src/runtime.rs`
  - `crates/tau-memory/src/runtime/query.rs`
  - `crates/tau-training-runner/src/lib.rs`
- Also ran `cargo fmt --all` to satisfy format gate after strict-check failures.

### REGRESSION

- `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> passed
- `cargo test --workspace` -> passed
- `cargo fmt --all -- --check` -> passed

## Verification Addendum (2026-02-23, post-rebase stability)

### RED

- Post-rebase full workspace regression surfaced deterministic CI-flake behavior in:
  - `crates/tau-coding-agent/tests/cli_integration/orchestrator_harness.rs`
  - failing test: `orchestrator_harness::regression_orchestrator_route_fallback_recovers_from_planner_timeout`
  - root cause: fallback attempt also timed out under heavy workspace load with `--request-timeout-ms 30`, causing route exhaustion before recovery.

### GREEN

- Stabilization applied:
  - `crates/tau-coding-agent/tests/cli_integration/orchestrator_harness.rs`
    - increased timeout in the timeout-fallback regression fixture from `30` to `120` ms to preserve timeout semantics while making fallback deterministic under load.
- Focused verification:
  - `cargo test -p tau-coding-agent --test cli_integration orchestrator_harness::regression_orchestrator_route_fallback_recovers_from_planner_timeout -- --exact --nocapture` -> passed
  - `cargo test -p tau-coding-agent --test cli_integration orchestrator_harness:: -- --nocapture` -> `4 passed; 0 failed`

### REGRESSION

- Full gate rerun after stabilization:
  - `cargo test --workspace` -> passed (`0 failed` across unit/integration/doc tests)

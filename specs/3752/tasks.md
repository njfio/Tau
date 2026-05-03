# Tasks: Issue #3752 - Define shared Tau Agent Harness mission contract

Status: Completed

- [x] T1 (SPEC): create issue-bound spec, plan, and task artifacts.
- [x] T2 (ADR): document the mission ownership inversion.
- [x] T3 (RED): add failing shared mission lifecycle and gateway projection tests.
- [x] T4 (GREEN): implement shared mission primitives in `tau-agent-core`.
- [x] T5 (GREEN): add gateway adapter projection into shared mission snapshots.
- [x] T6 (VERIFY): run scoped core/gateway tests and static checks.
- [x] T7 (CLOSEOUT): update spec/tasks status and report AC evidence.
- [x] T8 (SPEC): extend #3752 for Slice 3 mission DAG/checkpoint runtime.
- [x] T9 (RED): add failing core tests for DAG readiness, checkpoint resume,
      and completion blockers.
- [x] T10 (GREEN): implement mission DAG validation/readiness helpers.
- [x] T11 (GREEN): implement checkpoint/recovery and completion readiness
      helpers.
- [x] T12 (VERIFY): run scoped core tests, fmt, Clippy, and PR checks.

## Verification Evidence

- RED: `cargo test -p tau-agent-core mission --lib` failed before
  implementation with unresolved shared mission symbols.
- GREEN: `cargo test -p tau-agent-core mission --lib` passed, 2 tests.
- GREEN: `cargo test -p tau-gateway gateway_mission_state_projects_to_shared_snapshot --lib -- --test-threads=1` passed, 1 test.
- Regression: `cargo test -p tau-gateway mission_supervisor_runtime --lib -- --test-threads=1` passed, 4 tests.
- Regression: `cargo test -p tau-gateway regression_gateway_mission_detail_exposes_verifier_and_completion_state --lib -- --test-threads=1` passed, 1 test.
- Regression: `cargo test -p tau-gateway regression_gateway_missions_list_exposes_persisted_checkpointed_and_blocked_missions --lib -- --test-threads=1` passed, 1 test.
- Anchor: `cargo test -p tau-orchestrator validate_ok_for_valid_dag --lib` passed, 1 test.
- Static: `cargo fmt --check -p tau-agent-core -p tau-gateway` passed.
- Static: `cargo check -p tau-agent-core -p tau-gateway` passed.
- Static: `cargo clippy -p tau-agent-core -p tau-gateway -- -D warnings` passed.
- RED: `cargo test -p tau-agent-core mission --lib` failed before Slice 3
  implementation with missing `validate_plan_dag`, `ready_plan_node_ids`,
  `record_checkpoint`, `block_for_recovery`, `completion_blockers`, and related
  error/blocker types.
- GREEN: `cargo test -p tau-agent-core mission --lib` passed, 6 tests.
- Static: `cargo fmt --check -p tau-agent-core` passed.
- Static: `cargo clippy -p tau-agent-core --all-targets --all-features -- -D warnings` passed.

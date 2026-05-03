# Plan: Issue #3752 - Define shared Tau Agent Harness mission contract

## Goal

Implement the first concrete slice of
`docs/plans/2026-05-03-001-feat-tau-agent-harness-lane-plan.md` by adding a
shared mission contract and gateway adapter projection while preserving current
gateway behavior.

## Approach

1. Record the ownership inversion in an ADR.
2. Add RED tests for mission lifecycle invariants and gateway projection.
3. Implement shared mission types in `tau-agent-core`.
4. Export the shared mission surface from `tau-agent-core`.
5. Add gateway projection helpers that convert `GatewayMissionState` into shared
   mission snapshots without changing gateway persistence.
6. Surface additive shared mission projections from gateway list/detail handlers
   while preserving the existing `mission` payload.
7. Run focused core and gateway tests, then cargo formatting/checking for the
   touched crates.

## Affected Modules

- `docs/architecture/adr-009-tau-agent-harness-mission-ownership.md`
- `docs/plans/2026-05-03-001-feat-tau-agent-harness-lane-plan.md`
- `specs/3752/spec.md`
- `specs/3752/plan.md`
- `specs/3752/tasks.md`
- `crates/tau-agent-core/src/lib.rs`
- `crates/tau-agent-core/src/mission.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_api_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/mission_supervisor_runtime.rs`
- `crates/tau-gateway/src/gateway_openresponses/tests.rs`

## Risks / Mitigations

- Risk: gateway JSON payloads drift.
  Mitigation: keep existing `mission` fields unchanged and expose the shared
  snapshot through additive `harness_mission` / `harness_missions` fields.
- Risk: shared mission model becomes too large too early.
  Mitigation: include required harness fields, but leave plan/tool/memory
  details as typed lightweight records for later crate integration.
- Risk: mission lifecycle semantics conflict with current gateway `Running`
  status.
  Mitigation: map gateway `Running` to shared `Executing` and keep gateway enum
  names unchanged.

## Verification

- RED: `cargo test -p tau-agent-core mission --lib`
- GREEN: `cargo test -p tau-agent-core mission --lib`
- GREEN: `cargo test -p tau-gateway gateway_mission_state_projects_to_shared_snapshot --lib -- --test-threads=1`
- Regression: `cargo test -p tau-gateway mission_supervisor_runtime --lib -- --test-threads=1`
- Regression: `cargo test -p tau-gateway regression_gateway_mission_detail_exposes_verifier_and_completion_state --lib -- --test-threads=1`
- Regression: `cargo test -p tau-gateway regression_gateway_missions_list_exposes_persisted_checkpointed_and_blocked_missions --lib -- --test-threads=1`
- Static: `cargo fmt --check`
- Static: `cargo check -p tau-agent-core -p tau-gateway`
- Static: `cargo clippy -p tau-agent-core -p tau-gateway -- -D warnings`

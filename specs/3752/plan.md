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

## Slice 3 Addendum: Plan DAG and Checkpoint Runtime

### Goal

Make the shared mission contract executable enough for harness runtime code to
ask core questions without gateway ownership: which plan nodes are ready, which
DAG defects block execution, what checkpoint should resume, and what evidence
still blocks completion.

### Approach

1. Add RED unit tests in `tau-agent-core` for plan readiness, DAG validation,
   checkpoint pending-node capture, and completion blockers.
2. Keep the public mission schema lightweight and backward-compatible by
   preserving the existing string plan-node status field.
3. Add core helper methods on `MissionSnapshot` for DAG validation, ready-node
   selection, checkpoint recording, recovery blocking, and completion readiness.
4. Export any new core error/blocker types from `tau-agent-core`.
5. Re-run focused mission tests and static checks for `tau-agent-core`.

### Additional Affected Modules

- `crates/tau-agent-core/src/mission.rs`
- `crates/tau-agent-core/src/lib.rs`

### Additional Risks / Mitigations

- Risk: introducing a richer plan-node enum breaks adapter compatibility.
  Mitigation: keep the serialized `status` field as a string in this slice and
  centralize status interpretation in core helper methods.
- Risk: completion semantics become too strict for existing gateway snapshots.
  Mitigation: expose completion readiness as an explicit helper instead of
  changing gateway completion behavior in this slice.

### Additional Verification

- RED: `cargo test -p tau-agent-core mission --lib`
- GREEN: `cargo test -p tau-agent-core mission --lib`
- Static: `cargo fmt --check -p tau-agent-core`
- Static: `cargo clippy -p tau-agent-core --all-targets --all-features -- -D warnings`

## Slice 4 Addendum: Tool Budget and Evidence Ledger

### Goal

Make mission tool proof explicit in the shared harness contract. Core mission
state should be able to record attributable tool-call evidence, enforce
configured budgets, and block completion when budget consumption is not backed
by ledger evidence.

### Approach

1. Add RED unit tests in `tau-agent-core` for tool-call attribution, budget
   exhaustion, and missing tool-evidence completion blockers.
2. Add a serialized `tool_evidence` ledger to `MissionSnapshot` with
   mission/plan-node/tool/status/timing/artifact/gate fields.
3. Add core budget checks for allowed tools, max calls, runtime, and cost.
4. Add a recording helper that enforces budget before mutating consumed budget
   counters or the evidence ledger.
5. Keep gateway and `tau-tools` wiring deferred until the shared contract is
   stable; `tau-tools` currently does not expose a reusable mission ledger type.

### Additional Affected Modules

- `crates/tau-agent-core/src/mission.rs`
- `crates/tau-agent-core/src/lib.rs`

### Additional Risks / Mitigations

- Risk: budget accounting semantics diverge between adapters.
  Mitigation: keep accounting helpers in `tau-agent-core` and make adapters
  call the shared methods later.
- Risk: completion readiness becomes too strict for legacy snapshots.
  Mitigation: only require ledger evidence for consumed tool calls in this
  slice.

### Additional Verification

- RED: `cargo test -p tau-agent-core mission --lib`
- GREEN: `cargo test -p tau-agent-core mission --lib`
- Static: `cargo fmt --check -p tau-agent-core`
- Static: `cargo clippy -p tau-agent-core --all-targets --all-features -- -D warnings`

## Slice 5 Addendum: Memory and Learning Records

### Goal

Make mission memory proof and mission learning records first-class in the
shared harness contract. Core mission state should record whether planning used
memory or explicitly found no relevant memory, and final/failure learning should
write through `tau-memory` with curator review status.

### Approach

1. Add RED unit tests in `tau-agent-core` for memory-hit/no-memory recall proof,
   final learning writes, and failure learning writes.
2. Extend `MissionMemoryHit` with source-event, plan rationale, plan-node link,
   and metadata fields while preserving serde defaults.
3. Add `MissionMemoryRecallEvidence` so a mission can prove either used hits or
   an explicit no-memory result.
4. Add `MissionLearningRecord` with kind, curator status, root cause, evidence,
   artifacts, verification gates, rollback plan, and metadata.
5. Add helpers that write final/failure learning records through
   `tau_memory::runtime::FileMemoryStore` using the existing public
   `tau-memory` API.
6. Require memory recall evidence as part of mission completion readiness.

### Additional Affected Modules

- `crates/tau-agent-core/src/mission.rs`
- `crates/tau-agent-core/src/lib.rs`

### Additional Risks / Mitigations

- Risk: adding memory proof breaks legacy snapshots that predate Slice 5.
  Mitigation: keep new fields serde-defaulted and expose completion readiness as
  an explicit helper; adapters can add no-memory proof before relying on the
  stricter readiness gate.
- Risk: curator queues become gateway-owned again.
  Mitigation: store curator status directly on mission learning records and
  persist it through `tau-memory` tags/facts before any gateway adapter wiring.
- Risk: touching dirty `tau-memory` runtime files causes unrelated churn.
  Mitigation: use only the existing public `FileMemoryStore` API from
  `tau-agent-core`.

### Additional Verification

- RED: `cargo test -p tau-agent-core mission --lib`
- GREEN: `cargo test -p tau-agent-core mission --lib`
- Static: `cargo fmt --check -p tau-agent-core`
- Static: `cargo clippy -p tau-agent-core --all-targets --all-features -- -D warnings`

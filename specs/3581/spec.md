# Spec: Issue #3581 - Define shared operator turn/task state protocol for TUI and webchat

Status: Reviewed

## Problem Statement
TUI, webchat, dashboard, and gateway mission surfaces currently describe an operator turn through related but different shapes. TUI parses streaming gateway events into local `GatewayTurnEvent` values and local app state, webchat parses `response.*` SSE frames in embedded JavaScript, dashboard streams snapshot-oriented runtime state, and mission APIs expose persisted mission snapshots. Without a shared operator turn/task state protocol, the same runtime outcome can be rendered differently across clients and failure modes such as timeout, blocked mission, or partial output are easy to lose.

## Scope
In scope:
- Define an additive shared `OperatorTurnState` / operator event v1 contract.
- Define runtime semantics for mapping provider/gateway events into state phases and statuses.
- Define client consumption rules for TUI and webchat.
- Define backwards-compatibility boundaries for existing gateway SSE, mission, and dashboard endpoints.
- Add executable contract tests and fixtures for the shared schema.

Out of scope:
- Replacing existing `response.*` SSE events in this stage.
- Rewriting the TUI or webchat UI.
- Adding external dependencies.
- Migrating every dashboard/runtime consumer in one pass.
- Changing provider request/timeout behavior.

## Acceptance Criteria
### AC-1 Shared operator turn/task state contract is defined
Given an operator-visible runtime turn,
when the shared contract represents it,
then it includes stable fields for `schema_version`, `turn_id`, `task_id`, `session_key`, `mission_id`, `phase`, `status`, `events`, and current assistant/tool/error context.

### AC-2 Runtime semantics are defined
Given existing AgentEvent, gateway SSE, mission snapshot, timeout, blocked-mission, and tool execution events,
when the runtime maps them into the shared contract,
then each maps to a documented phase/status/event vocabulary without losing provider or tool identifiers.

### AC-3 TUI and webchat consumption rules are defined
Given the same contract fixture,
when TUI and webchat consume it,
then both clients can render the same turn outcome, including partial output, tool start/finish, timeout, blocked mission, and final success/failure.

### AC-4 Backwards-compatibility boundaries are explicit
Given existing clients that read `response.output_text.delta`, `response.tool_execution.started`, `response.tool_execution.completed`, `response.completed`, `response.failed`, `/gateway/missions`, or dashboard stream endpoints,
when the new shared contract is introduced,
then those existing surfaces continue to work during migration.

### AC-5 Test plan is executable
Given the shared contract is defined,
when contract tests run,
then they verify serialization, fixture parsing, phase/status vocabulary, stable tool identifiers, timeout/error contexts, and blocked mission states.

## Conformance Cases
- C-01 / AC-1 / Contract:
  A success turn fixture serializes and deserializes with stable id, phase, status, assistant text, and tool event fields.
- C-02 / AC-2 / Runtime mapping:
  Tool start/completion event vocabulary can represent existing `response.tool_execution.started` and `response.tool_execution.completed` semantics.
- C-03 / AC-3 / Client parity:
  A partial-output plus tool-failure fixture contains enough context for TUI and webchat to render the same final status.
- C-04 / AC-4 / Compatibility:
  Documentation names every legacy SSE/mission/dashboard surface preserved during migration.
- C-05 / AC-5 / Failure vocabulary:
  Timeout and blocked-mission fixtures include actionable reason codes and human-readable summaries.

## Success Metrics / Observable Signals
- `cargo test -p tau-contract operator_state -- --test-threads=1` proves the shared schema and fixtures.
- `docs/architecture/shared-operator-state-v1.md` documents mappings and compatibility boundaries.
- `docs/adrs/0005-shared-operator-turn-task-state.md` records the schema home and migration decision.
- Root `Cargo.toml` and `Cargo.lock` remain unchanged.

## Files To Touch
- `specs/3581/spec.md`
- `specs/3581/plan.md`
- `specs/3581/tasks.md`
- `docs/adrs/0005-shared-operator-turn-task-state.md`
- `docs/architecture/shared-operator-state-v1.md`
- `crates/tau-contract/src/lib.rs`
- `crates/tau-contract/tests/operator_state.rs` or an equivalent tau-contract test module

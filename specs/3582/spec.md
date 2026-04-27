# Spec: Issue #3582 - Redesign Tau TUI as a transcript-first operator terminal

Status: Reviewed

## Problem Statement
Tau TUI currently updates the transcript and tools panel directly from gateway SSE events. That path works, but it keeps TUI state coupled to gateway event details and makes the transcript-first operator model harder to share with webchat. Issue #3581 introduced `OperatorTurnState` as the shared contract. This #3582 slice uses that contract as the TUI consumption seam while preserving the existing streaming gateway path.

## Scope
In scope for this stage:
- Define the transcript-first TUI consumption model for `OperatorTurnState`.
- Add RED tests for assistant output, tool lifecycle, timeout state, and blocked mission state.
- Add an additive adapter from `OperatorTurnState` into existing TUI chat/status/tool models.
- Preserve current gateway SSE parsing and JSON fallback behavior.
- Document the TUI consumption boundary and backwards compatibility rules.

Out of scope for this stage:
- Full visual redesign of every TUI panel.
- Removing `GatewayTurnEvent` or the existing SSE parser.
- Webchat implementation changes.
- Gateway-side `OperatorTurnState` emission.
- New external dependencies.

## Acceptance Criteria
### AC-1 Transcript-first interaction model is defined
Given a shared `OperatorTurnState` fixture,
when TUI consumes it,
then the assistant transcript, status bar, and tool panel derive from the shared turn state rather than from duplicated ad hoc client vocabulary.

### AC-2 Layout/system behavior is preserved
Given the current interactive TUI shell,
when the new adapter is added,
then existing chat, input, tool-panel, gateway streaming, mission list/detail, and command palette behavior continue to work.

### AC-3 Progress and failure UX are explicit
Given a running, timed-out, failed, succeeded, or blocked operator turn,
when TUI renders the shared state,
then it shows an operator-readable transcript/status outcome and preserves stable tool-call identifiers.

### AC-4 Shared-state dependency is honored
Given `OperatorTurnState` lives in `tau-contract`,
when TUI consumes the contract,
then `tau-tui` depends on the internal contract crate instead of re-declaring the schema.

### AC-5 Test plan is executable
Given the adapter is implemented,
when `cargo test -p tau-tui operator_state -- --test-threads=1` runs,
then it validates assistant text, tool lifecycle rows, timeout errors, and blocked mission status.

## Conformance Cases
- C-01 / AC-1: success fixture appends assistant text to the transcript and leaves status idle/succeeded.
- C-02 / AC-3: tool started/completed events create and reconcile tool-panel rows by `tool_call_id`.
- C-03 / AC-3: timed-out fixture surfaces a system message with the timeout reason code.
- C-04 / AC-3: blocked mission fixture surfaces waiting-for-verifier/blocked status in the transcript.
- C-05 / AC-2: existing gateway SSE tests remain green.

## Success Metrics / Observable Signals
- New `operator_state` TUI tests pass.
- Existing `app_gateway_tests` remain compatible through scoped tau-tui clippy/test gates.
- `docs/architecture/tui-operator-state-consumption-v1.md` names `OperatorTurnState`, transcript-first mapping, and backwards compatibility.
- Root `Cargo.toml` remains unchanged; any dependency change is limited to `crates/tau-tui/Cargo.toml` and the workspace lock metadata required for the internal dependency.

## Files To Touch
- `specs/3582/spec.md`
- `specs/3582/plan.md`
- `specs/3582/tasks.md`
- `crates/tau-tui/Cargo.toml`
- `crates/tau-tui/src/interactive/mod.rs`
- `crates/tau-tui/src/interactive/app.rs`
- `crates/tau-tui/src/interactive/operator_state.rs`
- `crates/tau-tui/src/interactive/operator_state_tests.rs`
- `docs/architecture/tui-operator-state-consumption-v1.md`

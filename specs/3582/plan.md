# Plan: Issue #3582 - Redesign Tau TUI as a transcript-first operator terminal

## Approach
1. Capture the transcript-first TUI slice in spec artifacts and map current state ownership.
2. Add RED tests for consuming `OperatorTurnState` fixtures into TUI chat/status/tool state.
3. Add a small adapter module under `crates/tau-tui/src/interactive/operator_state.rs`.
4. Add the internal `tau-contract` dependency to `tau-tui` only when implementation begins.
5. Keep existing `GatewayTurnEvent` streaming behavior intact; this stage adds a shared-state consumption seam rather than replacing gateway SSE.
6. Document the consumption boundary and verification evidence.

## Current Touchpoint Map
- `crates/tau-contract/src/operator_state.rs`: shared `OperatorTurnState`, phase/status/event/tool/error vocabulary introduced by #3581.
- `crates/tau-tui/src/interactive/gateway_client.rs`: current gateway SSE parser producing `GatewayTurnEvent::TextDelta`, `ToolStarted`, `ToolCompleted`, and `Finished`.
- `crates/tau-tui/src/interactive/app.rs`: applies gateway turn events to chat transcript, status bar, and tool panel.
- `crates/tau-tui/src/interactive/chat.rs`: owns `ChatMessage`, `MessageRole`, and transcript text behavior.
- `crates/tau-tui/src/interactive/tools.rs`: owns `ToolEntry`, `ToolStatus`, active tool counts, and tool-call-id reconciliation.
- `crates/tau-tui/src/interactive/status.rs`: owns `AgentStateDisplay`, including idle/thinking/tool/stream/error states.
- `crates/tau-tui/src/interactive/app_gateway_tests.rs`: existing SSE streaming and tool lifecycle compatibility coverage.

## Proposed Adapter Boundary
Add `interactive::operator_state` as a pure adapter layer:

- Input: borrowed `OperatorTurnState` from `tau-contract`.
- Output: updates to an existing `App` via small methods, or a local adapter result that `App` can apply.
- Behavior: append assistant transcript text, reconcile tool rows by `tool_call_id`, set operator-readable status/error messages, and preserve status transitions.
- Compatibility: no removal of `GatewayTurnEvent`; the SSE parser remains the live gateway path until a later gateway adapter emits full `OperatorTurnState` snapshots.

## Dependency Decision
ADR 0005 already accepted `tau-contract` as the shared `OperatorTurnState` schema home and stated future clients can opt in. This stage uses that ADR instead of creating a new ADR for the internal `tau-tui -> tau-contract` dependency. If implementation requires a broader runtime or gateway dependency change, write a new ADR before proceeding.

## Backwards Compatibility
Preserve:
- existing `/v1/responses` streaming requests with `stream: true`;
- `response.output_text.delta` assistant streaming;
- `response.tool_execution.started` and `response.tool_execution.completed` tool-panel behavior;
- mission list/detail commands;
- command palette and input handling;
- current gateway error fallback behavior.

## Risks / Mitigations
- Risk: adapter duplicates gateway event logic.
  Mitigation: keep it pure and focused on state application; do not replace SSE parsing in this stage.
- Risk: internal dependency changes the workspace lock unexpectedly.
  Mitigation: add only the local `tau-contract` dependency and verify root `Cargo.toml` remains unchanged.
- Risk: transcript-first behavior becomes visual redesign creep.
  Mitigation: limit this stage to data/state consumption and documentation; leave full layout redesign as a later slice.
- Risk: blocked/timeout states are rendered as generic errors.
  Mitigation: RED tests require explicit reason-code visibility.

## Verification
- `bash -c '! cargo test -p tau-tui operator_state -- --test-threads=1'` for RED.
- `cargo test -p tau-tui operator_state -- --test-threads=1` for GREEN.
- `cargo clippy -p tau-tui --tests --no-deps -- -D warnings`.
- `cargo fmt --check`.
- `git diff --quiet -- Cargo.toml` to prove the root manifest is untouched.

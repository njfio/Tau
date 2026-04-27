# TUI Operator State Consumption v1

## Summary
The Tau TUI now exposes an additive `interactive::operator_state` adapter that consumes the shared `OperatorTurnState` contract from `tau-contract`. The adapter maps a shared turn snapshot into the existing TUI transcript, status bar, mission binding, and tool panel state without changing the gateway streaming client.

## Consumption Boundary
`OperatorTurnState` remains owned by `tau-contract`, as accepted in ADR 0005. `tau-tui` depends on that crate and does not redeclare the schema. The first TUI consumption boundary is deliberately small:

- bind `session_key` and `mission_id` into the active gateway/status context;
- render non-empty `assistant_text` into the assistant transcript;
- reconcile `OperatorToolState` rows by stable `tool_call_id`;
- map running tools to the tool panel and completed/failed/cancelled tools to final rows;
- surface timed-out, failed, cancelled, and blocked turns as operator-readable system messages with reason codes.

This is a transcript-first bridge, not a full UI redesign. It gives future gateway and webchat work a shared state target while preserving the current TUI event loop.

## Status Mapping
- `succeeded` + `completed` maps to idle after writing assistant text.
- `tool_running` or `waiting_for_tool` maps to tool execution.
- `streaming` maps to streaming.
- `pending`, `queued`, and `running` map to thinking.
- `blocked`, `timed_out`, `failed`, and `cancelled` map to error and emit a system transcript message.

## Tool Mapping
The adapter treats `tool_call_id` as the reconciliation key. A running tool creates a row only when no row with the same id exists. A completed, failed, or cancelled tool updates the matching running row when present; otherwise it creates a final row. Tool summaries become the row detail so existing tool-panel rendering remains unchanged.

## Backwards compatibility
The existing gateway path remains intact:

- `/v1/responses` requests still use the current streaming payload and metadata;
- `GatewayTurnEvent::TextDelta` still updates assistant text incrementally;
- `response.tool_execution.started` and `response.tool_execution.completed` still drive live tool-panel updates;
- mission list, mission detail, and mission resume commands are unchanged;
- gateway JSON and SSE fallback/error handling remains in `gateway_client.rs`.

A later runtime/gateway stage can emit full `OperatorTurnState` snapshots and call the adapter directly. Until then, the TUI supports both the legacy SSE event path and the shared-state consumption path side by side.

## Verification
- `cargo test -p tau-tui operator_state -- --test-threads=1`
- `cargo clippy -p tau-tui --tests --no-deps -- -D warnings`
- `cargo fmt --check`
- `git diff --quiet -- Cargo.toml`

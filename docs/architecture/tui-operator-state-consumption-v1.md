# TUI Operator State Consumption v1

## Summary
The Tau TUI now exposes an additive `interactive::operator_state` adapter that consumes the shared `OperatorTurnState` contract from `tau-contract`. The adapter maps a shared turn snapshot into the existing TUI transcript, status bar, mission binding, and tool panel state without changing the gateway streaming client.

The live gateway client also accepts an optional `response.operator_turn_state.snapshot` SSE frame. Its `data` payload is parsed as `OperatorTurnState` and applied through the same adapter used by direct tests.

## Consumption Boundary
`OperatorTurnState` remains owned by `tau-contract`, as accepted in ADR 0005. `tau-tui` depends on that crate and does not redeclare the schema. The first TUI consumption boundary is deliberately small:

- bind `session_key` and `mission_id` into the active gateway/status context;
- render non-empty `assistant_text` into the assistant transcript;
- reconcile `OperatorToolState` rows by stable `tool_call_id`;
- map running tools to the tool panel and completed/failed/cancelled tools to final rows;
- surface timed-out, failed, cancelled, and blocked turns as operator-readable system messages with reason codes.

This is a transcript-first bridge, not a full UI redesign. It gives future gateway and webchat work a shared state target while preserving the current TUI event loop.

## Live stream event

`response.operator_turn_state.snapshot` is an additive SSE event. TUI treats it as a richer state projection that may appear beside legacy response frames:

- `response.output_text.delta` can still stream incremental text;
- `response.tool_execution.started` and `response.tool_execution.completed` can still drive live tool rows;
- `response.completed` still finishes the turn and carries usage metadata;
- `response.failed` still fails the turn.

When a snapshot and legacy text delta describe the same current assistant turn, the adapter reconciles the assistant message instead of adding a duplicate. This keeps transcript-first rendering stable while the gateway and clients transition from event-level deltas toward shared state snapshots.

Gateway emission boundary: the producer side should emit `response.operator_turn_state.snapshot` through the existing gateway `SseFrame::Json` path. The emission should be additive and ordered before `response.completed` for successful turns so clients can apply the shared state before final turn completion. Legacy `response.output_text.done`, `response.completed`, `response.failed`, and tool execution frames remain required compatibility frames.

Expected gateway touchpoints:

- `gateway_openresponses/types.rs` owns `SseFrame` and already serializes named JSON SSE frames.
- `gateway_openresponses/stream_response_handler.rs` owns final success/failure stream frames.
- `gateway_openresponses/openresponses_execution_handler.rs` owns execution state, response ids, text deltas, and tool lifecycle frames.
- `tau-contract::operator_state` owns the snapshot schema and must remain the only schema source.

Transcript-first layout boundary: the TUI render layer should keep the conversation transcript as the primary surface while preserving a stable status bar, input editor, help line, and secondary tool-progress panel. The layout slice should refine the existing `ui_layout`, `ui_chat`, `ui_tools`, and `ui_status` modules rather than replacing the application shell. Transcript-first means current assistant output and operator-readable turn state remain visible in the main panel, while tool summaries augment the transcript and the side panel provides scannable execution detail.

Expected layout touchpoints:

- `interactive/ui_layout.rs` owns the status/body/input/help rectangles and chat/tools split.
- `interactive/ui_chat.rs` owns transcript rendering and tool summary lines in the main panel.
- `interactive/ui_chat_tool_lines.rs` owns transcript-visible tool progress rows.
- `interactive/ui_tools.rs` owns the secondary tools panel.
- `interactive/ui_status.rs` owns compact status chips for model, mission, tokens, and agent state.
- Existing keyboard, mouse, command palette, and input behavior must remain unchanged.

Layout verification:

- Focused render tests cover the transcript-first label, assistant transcript visibility, current turn state, active tool progress, status mission chip, input editor, and secondary tools panel.
- Existing operator-state and gateway snapshot tests remain the compatibility guard for the shared-state transport path.

Live snapshot tool/failure boundary: gateway snapshots should evolve from terminal success summaries into richer turn projections while remaining additive to legacy SSE frames. A live `OperatorTurnState` snapshot may carry tool rows, failure context, and partial assistant text, but it must not remove or reorder required compatibility events such as `response.output_text.delta`, `response.tool_execution.*`, `response.completed`, or `response.failed`. The TUI should reconcile snapshot rows by `tool_call_id` and apply assistant text only to the intended active turn.

Turn-keyed reconciliation expectations:

- `turn_id` is the snapshot identity boundary; mixed or stale snapshots must not overwrite an assistant message for a newer turn.
- legacy text deltas and snapshots for the same turn should update one assistant transcript message rather than creating duplicates.
- tool rows from snapshots and legacy tool lifecycle frames should converge on one tool entry per `tool_call_id`.
- failed, blocked, timed-out, and cancelled turn snapshots should produce one operator-readable system message with reason context.
- current gateway success snapshots include observed tool rows and tool-completion/failure events while still preserving the legacy `response.tool_execution.*` frames for existing clients.

Recovery-policy failure snapshots: gateway failure paths that end in a verifier-controlled blocked mission should emit an additive `response.operator_turn_state.snapshot` before the legacy `response.failed` frame. The snapshot carries `status: "blocked"`, a `mission.blocked` event, and `error.reason_code` so clients can show domain-specific recovery policy failures such as `required_tool_evidence_missing_exhausted`. The TUI treats that operator snapshot as the richer active-turn error and suppresses the duplicate generic gateway error message from the following compatibility `response.failed` frame.

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

Runtime/gateway code can now emit full `OperatorTurnState` snapshots and call the adapter directly. During transition, the TUI supports both the legacy SSE event path and the shared-state consumption path side by side.

## Verification
- `cargo test -p tau-tui operator_state -- --test-threads=1`
- `cargo test -p tau-tui operator_turn_state_snapshot_turn_keyed -- --test-threads=1`
- `cargo test -p tau-gateway operator_turn_state_tool_failure_snapshot -- --test-threads=1`
- `cargo test -p tau-gateway operator_turn_state_recovery_policy_snapshot -- --test-threads=1`
- `cargo test -p tau-tui operator_turn_state_recovery_policy -- --test-threads=1`
- `cargo clippy -p tau-tui --tests --no-deps -- -D warnings`
- `cargo fmt --check`
- `git diff --quiet -- Cargo.toml`

# Shared Operator State v1

`OperatorTurnState` is the additive contract for representing one operator-visible turn or task across gateway, TUI, webchat, dashboard, and future runtime clients. It lives in `tau-contract` so contract fixtures can be validated without making UI clients depend on gateway internals.

## Contract

The v1 schema is fixture-first and serde-serializable:

- `OperatorTurnFixture`: schema version, fixture name, and example cases.
- `OperatorTurnState`: `schema_version`, `turn_id`, `task_id`, `session_key`, `mission_id`, `phase`, `status`, `assistant_text`, `tools`, `events`, and optional `error`.
- `OperatorTurnPhase`: `created`, `queued`, `running`, `waiting_for_tool`, `waiting_for_verifier`, `completed`.
- `OperatorTurnStatus`: `pending`, `streaming`, `tool_running`, `blocked`, `timed_out`, `failed`, `succeeded`, `cancelled`.
- `OperatorTurnEvent`: stable event id, event kind, summary, optional text delta, optional tool identity, optional reason code, and optional timestamp.
- `OperatorToolState`: stable tool call id, tool name, status, summary, start timestamp, and completion timestamp.
- `OperatorErrorContext`: reason code, human-readable message, and retryability.

## Runtime Mapping

Existing gateway/runtime surfaces can map into v1 without changing their current wire behavior:

| Existing surface | Operator event/status mapping |
| --- | --- |
| `response.output_text.delta` | `OperatorTurnEventKind::ResponseOutputTextDelta`; status remains `streaming` unless a terminal event arrives. |
| `response.tool_execution.started` | `OperatorTurnEventKind::ResponseToolExecutionStarted`; phase becomes `waiting_for_tool`, status becomes `tool_running`, tool status becomes `running`. |
| `response.tool_execution.completed` | `OperatorTurnEventKind::ResponseToolExecutionCompleted`; tool status becomes `completed`; the turn may remain `streaming` or become `succeeded` depending on following terminal events. |
| `response.failed` | `OperatorTurnEventKind::ResponseFailed`; phase becomes `completed`, status becomes `failed`, and `OperatorErrorContext` carries the reason. |
| `response.completed` | `OperatorTurnEventKind::ResponseCompleted`; phase becomes `completed`, status becomes `succeeded`. |
| provider read timeout | `OperatorTurnEventKind::Timeout`; phase becomes `completed`, status becomes `timed_out`, and `error.reason_code` identifies the timeout class. |
| mission checkpoint | `OperatorTurnEventKind::MissionCheckpointed`; status remains non-terminal unless the mission also emits a blocked/final state. |
| mission blocked | `OperatorTurnEventKind::MissionBlocked`; phase becomes `waiting_for_verifier`, status becomes `blocked`. |

## TUI consumption

TUI consumption should treat `OperatorTurnState` as the canonical presentation snapshot once a gateway adapter is added. Existing local `GatewayTurnEvent` parsing can continue during migration, but it should preserve these rules:

- Use `turn_id` as the stable turn key and `task_id` / `mission_id` as optional cross-links.
- Render `assistant_text` as the reconciled output, using text delta events only for progressive streaming.
- Render tool rows from `tools`, not by inferring completion from message text.
- Surface `OperatorErrorContext.reason_code` for timeout, blocked mission, and failed turn status.
- Treat `blocked`, `timed_out`, `failed`, `succeeded`, and `cancelled` as terminal display states.

## Webchat consumption

Webchat consumption should use the same state vocabulary as TUI once a gateway adapter is added. The embedded webchat SSE parser can keep consuming legacy events while the adapter builds `OperatorTurnState` snapshots.

- Text deltas append to the active assistant message and also become `response.output_text.delta` operator events.
- Tool lifecycle events use `tool_call_id` and `tool_name` to avoid duplicate or mismatched rows.
- Timeout and blocked mission states render as explicit operator statuses instead of generic stream failure.
- Final success and failure states come from terminal status, not from the presence or absence of assistant text.

## Backwards compatibility

Backwards compatibility is a hard boundary for v1. This contract is additive and does not remove, rename, or reinterpret existing gateway surfaces:

- `response.output_text.delta`
- `response.tool_execution.started`
- `response.tool_execution.completed`
- `response.completed`
- `response.failed`
- `/gateway/missions`
- `/gateway/missions/{mission_id}`
- dashboard stream/status endpoints

Later stages may add gateway adapters or client consumption paths, but existing clients must continue to work until a separate ADR or migration guide explicitly changes that contract.

## Test Fixtures

The tau-contract `operator_state` tests cover:

- a successful turn with output text and tool lifecycle events;
- unsupported schema-version rejection;
- timeout state with retryable error context;
- blocked mission state with verifier-required context.

These fixtures are the compatibility anchor for future TUI and webchat migration work.
